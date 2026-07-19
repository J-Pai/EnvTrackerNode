//! Configuration and set up for OAuth2 based authentication.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::response::Html;
use axum::routing;
use axum_oidc_client::auth::AuthenticationLayer;
use axum_oidc_client::auth::CodeChallengeMethod;
use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::extractors::OptionalAuthSession;
use axum_oidc_client::jwt::Algorithm;
use axum_oidc_client::jwt::DecodingKey;
use axum_oidc_client::jwt::Validation;
use axum_oidc_client::jwt::decode_jwt;
use axum_oidc_client::jwt::decode_jwt_unverified;
use axum_oidc_client::logout::handle_default_logout::DefaultLogoutHandler;
use http::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tower_sessions::cookie::Key;
use url::Url;

use crate::config::OAuth2Config;
use crate::services::db::Db;

#[derive(Default, serde::Deserialize)]
#[allow(unused)]
pub(crate) struct ClientJsonWeb {
    client_id: String,
    project_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_secret: String,
}

#[derive(Default, serde::Deserialize)]
struct ClientJson {
    web: ClientJsonWeb,
}

pub(crate) struct Auth {
    db: Db,
    certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
    client_json: ClientJsonWeb,
    redirect_uri_base: Url,
    cookie_secret_key: Vec<u8>,
}

impl Auth {
    pub(crate) async fn new(
        oauth2_config: &OAuth2Config,
        db: Db,
        scheduler: Arc<RwLock<JobScheduler>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client_json = Self::parse_client_json(&oauth2_config.get_client_json())?;
        let certs = Arc::new(RwLock::new(HashMap::new()));
        db.create_auth_table().await?;
        db.add_auth_table_cleanup_job(scheduler).await?;
        Ok(Self {
            db,
            certs: certs,
            client_json,
            redirect_uri_base: oauth2_config.get_redirect_uri_base(),
            cookie_secret_key: oauth2_config.get_cookie_secret_key(),
        })
    }

    fn parse_client_json(
        oauth2_client_json: &PathBuf,
    ) -> Result<ClientJsonWeb, Box<dyn std::error::Error>> {
        let json_str = fs::read_to_string(oauth2_client_json)?;
        let json = serde_json::from_str(&json_str)?;
        let json = serde_json::from_value::<ClientJson>(json)?;
        Ok(json.web)
    }

    async fn update_google_certs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cert_client = ClientBuilder::new(Client::new()).build();
        let data = cert_client
            .get(&self.client_json.auth_provider_x509_cert_url)
            .send()
            .await?;
        let data = data.error_for_status()?;
        let json = data.text().await.unwrap_or("{}".to_string()).to_string();

        let google_certs = serde_json::from_str::<HashMap<String, String>>(&json)?;

        let mut certs = self.certs.write().await;

        for (kid, pem) in google_certs.iter() {
            certs.insert(kid.clone(), DecodingKey::from_rsa_pem(pem.as_bytes())?);
        }

        Ok(())
    }

    pub(crate) async fn setup_auth_router(
        &mut self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        self.update_google_certs().await?;
        let base = self.redirect_uri_base.clone();
        let base_path = base.path().to_string().clone();
        let logout_uri = base.join("google_home/login").unwrap();
        let logout_redirect = logout_uri.path();
        let mut config = OAuthConfigurationBuilder::default()
            .with_authorization_endpoint(&self.client_json.auth_uri)
            .with_token_endpoint(&self.client_json.token_uri)
            .with_client_id(&self.client_json.client_id)
            .with_client_secret(&self.client_json.client_secret)
            .with_redirect_uri(base.join("auth/callback").unwrap().as_str())
            .with_scopes(vec!["openid", "email", "profile"])
            .with_code_challenge_method(CodeChallengeMethod::S256)
            .with_post_logout_redirect_uri(logout_redirect)
            .with_session_max_age(60 * 24 * 365)
            .with_token_max_age(300)
            .with_base_path("/auth");
        config.private_cookie_key = Some(Key::from(&self.cookie_secret_key));
        let config = config.build()?;
        let certs = self.certs.clone();

        let logout_handler = Arc::new(DefaultLogoutHandler);

        let google_home_link = move |_session: AuthSession| async move {};
        let google_home_login = move |OptionalAuthSession(session): OptionalAuthSession| async move {
            match session {
                Some(session) => {
                    let expires = session
                        .expires
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "(no expiry)".to_string());

                    let token_data = decode_jwt_unverified(&session.id_token)
                        .map_err(|e| tracing::error!("JWT Error: {e} - {}", session.id_token))
                        .unwrap();

                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[
                        "42441590702-te8vdhfdd8s3ft960ct6ksle7hp25jtj.apps.googleusercontent.com",
                    ]);

                    let token_data = decode_jwt(
                        &session.id_token,
                        certs.read().await.get(&token_data.0.kid.unwrap()).unwrap(),
                        &validation,
                    )
                    .map_err(|e| tracing::error!("JWT Error: {e} - {}", session.id_token));

                    if token_data.is_err() {
                        return (StatusCode::UNAUTHORIZED, Html("Unauthorized".to_string()));
                    }

                    tracing::info!("JWT data verified {:#?}", token_data);

                    (
                        StatusCode::OK,
                        Html(format!(
                            r#"
                            Hello World for Google Home: expires {expires}
                            <br />
                            <a href='{base}google_home/link'>Authorize Link</a>
                            <br />
                            <a href='{base}auth/logout'>Logout</a>
                        "#,
                        )),
                    )
                }
                None => (
                    StatusCode::OK,
                    Html(format!(
                        "<a href='{base}auth?redirect={base_path}google_home/login'>GOOGLE LOGIN</a>",
                    )),
                ),
            }
        };
        let google_home_fulfillment = move |_session: AuthSession| async move {
            tracing::debug!("Handling fulfillment");
            "{}"
        };

        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.clone());

        router = router
            .route("/google_home/link", routing::get(google_home_link))
            .route("/google_home/login", routing::get(google_home_login))
            .route(
                "/google_home/fulfillment",
                routing::post(google_home_fulfillment),
            )
            .layer(AuthenticationLayer::new(
                Arc::new(config),
                cache,
                logout_handler,
            ));

        Ok(router)
    }
}
