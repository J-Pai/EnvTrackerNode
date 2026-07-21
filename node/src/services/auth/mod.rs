//! Configuration and set up for OAuth2 based authentication.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Form;
use axum::Router;
use axum::extract::OriginalUri;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Redirect;
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
use http::Uri;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tower_sessions::cookie::Key;
use url::Url;

use crate::config::OAuth2Config;
use crate::error::NodeError;
use crate::services::auth::google_home_callback::OAuth2CallbackRequest;
use crate::services::auth::google_home_link::OAuth2AuthRequest;
use crate::services::auth::google_home_token::OAuth2TokenRequest;
use crate::services::db::Db;

mod google_home_callback;
mod google_home_link;
mod google_home_login_handler;
mod google_home_token;

#[derive(Clone, Default, serde::Deserialize)]
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
    google_home_client_json: ClientJsonWeb,
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
        let google_home_client_json = Self::parse_client_json(
            &oauth2_config
                .get_google_home_client_json()
                .expect("Need google home client json."),
        )?;
        let certs = Arc::new(RwLock::new(HashMap::new()));
        db.create_auth_table().await?;
        db.add_auth_table_cleanup_job(scheduler).await?;
        Ok(Self {
            db,
            certs,
            client_json,
            google_home_client_json,
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

    async fn validate_session(
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        session: &AuthSession,
        audience: &[String],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let token_data = decode_jwt_unverified(&session.id_token)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(audience);

        let token_data = decode_jwt(
            &session.id_token,
            certs.read().await.get(&token_data.0.kid.unwrap()).unwrap(),
            &validation,
        )?;

        Ok(token_data.claims.email.ok_or(NodeError::new("No email."))?)
    }

    fn stringify_query(uri: &Uri) -> (String, String) {
        if let Some(query) = uri.query() {
            let decoded_query = format!("?{query}");
            (
                decoded_query.clone(),
                urlencoding::encode(&decoded_query).to_string(),
            )
        } else {
            (String::new(), String::new())
        }
    }

    pub(crate) async fn setup_auth_router(
        &mut self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        self.update_google_certs().await?;
        let base = self.redirect_uri_base.clone();
        let logout_uri = base.join("google_home").unwrap();
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
        let logout_handler = Arc::new(DefaultLogoutHandler);

        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.clone());

        router = router
            .route(
                "/google_home/link",
                routing::get({
                    let db = self.db.clone();
                    let client_json = self.client_json.clone();
                    let google_home_client_json = self.google_home_client_json.clone();
                    let certs = self.certs.clone();
                    let base = self.redirect_uri_base.clone();

                    |session: AuthSession, query: Query<OAuth2AuthRequest>, uri: OriginalUri| {
                        Self::google_home_link_handler(
                            session,
                            query,
                            uri,
                            base,
                            certs,
                            db,
                            client_json,
                            google_home_client_json,
                        )
                    }
                }),
            )
            .route(
                "/google_home/token",
                routing::post({
                    let base = self.redirect_uri_base.clone();

                    |params: Form<OAuth2TokenRequest>| Self::google_home_token_handler(params, base)
                }),
            )
            .route(
                "/google_home/callback",
                routing::get({
                    let db = self.db.clone();
                    let client_json = self.client_json.clone();
                    let google_home_client_json = self.google_home_client_json.clone();
                    let certs = self.certs.clone();
                    |session: AuthSession, query: Query<OAuth2CallbackRequest>| {
                        Self::google_home_callback_handler(
                            session,
                            query,
                            certs,
                            db,
                            client_json,
                            google_home_client_json,
                        )
                    }
                }),
            )
            .route(
                "/google_home",
                routing::get({
                    let base = self.redirect_uri_base.clone();
                    let certs = self.certs.clone();
                    let client_json = self.client_json.clone();
                    |session: OptionalAuthSession,
                     query: Query<OAuth2AuthRequest>,
                     uri: OriginalUri| {
                        Self::google_home_login_handler(
                            session,
                            query,
                            uri,
                            base,
                            certs,
                            client_json,
                        )
                    }
                }),
            )
            .route(
                "/google_home/fulfillment",
                routing::post({
                    |_session: AuthSession| async move {
                        tracing::debug!("Handling fulfillment");
                        "{}"
                    }
                }),
            )
            .layer(AuthenticationLayer::new(
                Arc::new(config),
                cache,
                logout_handler,
            ));

        Ok(router)
    }
}
