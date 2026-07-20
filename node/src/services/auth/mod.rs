//! Configuration and set up for OAuth2 based authentication.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::OriginalUri;
use axum::extract::Query;
use axum::response::Html;
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
use http::StatusCode;
use http::Uri;
use http::header;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::reqwest::redirect;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tower_sessions::cookie::Key;
use url::Url;

use crate::config::OAuth2Config;
use crate::error::NodeError;
use crate::services::db::Db;

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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct OAuth2AuthRequest {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    state: Option<String>,
    scope: Option<String>,
    response_type: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct OAuth2TokenRequest {
    client_id: Option<String>,
    client_secret: Option<String>,
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct OAuth2CallbackRequest {
    code: String,
    iss: String,
    state: String,
    scope: String,
    prompt: String,
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

        let base = self.redirect_uri_base.clone();
        let certs = self.certs.clone();
        let client_id = self.client_json.client_id.clone();
        let google_home_client_json = self.google_home_client_json.clone();
        let db = self.db.clone();
        let google_home_link = move |session: AuthSession,
                                     Query(query): Query<OAuth2AuthRequest>,
                                     OriginalUri(uri): OriginalUri| async move {
            let Ok(_) = Self::validate_session(certs, &session, &[client_id])
                .await
                .map_err(|e| tracing::error!("JWT validation failed: {e} - {}", session.id_token))
            else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            let (decoded_query, encoded_query) = Self::stringify_query(&uri);

            tracing::info!("{decoded_query}\n{encoded_query}\n{query:#?}");

            let client_id = if let Some(client_id) = query.client_id.clone()
                && client_id == google_home_client_json.client_id
            {
                client_id
            } else {
                return (
                    StatusCode::OK,
                    Html(format!(
                        r#"
                        <pre><code>{query:#?}</code></pre>
                        "#,
                    )),
                )
                    .into_response();
            };

            let redirect_uri = if let Some(redirect_uri) = query.redirect_uri
                && let Ok(redirect_uri) = Url::parse(&redirect_uri)
                && redirect_uri.path() == format!("/r/{}", google_home_client_json.project_id)
                && let Some(host) = redirect_uri.host_str()
                && (host == "oauth-redirect.googleusercontent.com"
                    || host == "oauth-redirect-sandbox.googleusercontent.com")
            {
                redirect_uri
            } else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            let state = if let Some(state) = query.state {
                state
            } else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            let Ok(mut auth_uri) = Url::parse(&google_home_client_json.auth_uri) else {
                return StatusCode::UNAUTHORIZED.into_response();
            };
            auth_uri
                .query_pairs_mut()
                .append_pair("client_id", &client_id);
            auth_uri.query_pairs_mut().append_pair(
                "redirect_uri",
                base.join("google_home/callback").unwrap().as_str(),
            );
            auth_uri.query_pairs_mut().append_pair("state", &state);
            auth_uri
                .query_pairs_mut()
                .append_pair("response_type", "code");
            auth_uri
                .query_pairs_mut()
                .append_pair("scope", "openid email profile");

            if let Err(e) = db
                .set_code_verifier(
                    &state,
                    &format!(
                        "{}|{redirect_uri}|{}",
                        &session.access_token, google_home_client_json.project_id
                    ),
                )
                .await
            {
                tracing::error!("Issue storing Google Home code verifier {e}");
                return StatusCode::UNAUTHORIZED.into_response();
            }

            Redirect::to(auth_uri.as_str()).into_response()
        };

        let base = self.redirect_uri_base.clone();
        let google_home_token = move |OriginalUri(uri): OriginalUri| async move {
            let (decoded_query, encoded_query) = Self::stringify_query(&uri);

            tracing::info!("TOKEN ENDPOINT {decoded_query}\n{encoded_query}");

            Redirect::to(format!("{base}google_home").as_str()).into_response()
        };

        let base = self.redirect_uri_base.clone();
        let certs = self.certs.clone();
        let client_id = self.client_json.client_id.clone();
        let google_home_client_json = self.google_home_client_json.clone();
        let db = self.db.clone();
        let google_home_callback =
            move |session: AuthSession,
                  Query(query): Query<OAuth2CallbackRequest>,
                  OriginalUri(uri): OriginalUri| async move {
                let (decoded_query, encoded_query) = Self::stringify_query(&uri);

                let Ok(_) = Self::validate_session(certs, &session, &[client_id])
                    .await
                    .map_err(|e| {
                        tracing::error!("JWT validation failed: {e} - {}", session.id_token)
                    })
                else {
                    return StatusCode::UNAUTHORIZED.into_response();
                };

                if query.iss != "https://accounts.google.com" {
                    tracing::error!("Incorrect issuer: {query:#?}");
                    return StatusCode::UNAUTHORIZED.into_response();
                }

                if query.prompt != "none" {
                    tracing::error!("Incorrect prompt: {query:#?}");
                    return StatusCode::UNAUTHORIZED.into_response();
                }

                let mut redirect_uri =
                    if let Ok(Some(code_verifier)) = db.get_code_verifier(&query.state).await {
                        if code_verifier == "REDIRECTED" {
                            tracing::error!("Code is already redirected: {query:#?}");
                            return StatusCode::UNAUTHORIZED.into_response();
                        }

                        let mut parts = code_verifier.split("|");
                        let access_token = parts.next();
                        let redirect_uri = parts.next();
                        let project_id = parts.next();

                        if let Some(access_token) = access_token
                            && access_token != session.access_token
                        {
                            tracing::error!("Unmatched state / access_token: {query:#?}");
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                        if let Some(project_id) = project_id
                            && project_id != google_home_client_json.project_id
                        {
                            tracing::error!("Unmatched state / project_id: {query:#?}");
                            return StatusCode::UNAUTHORIZED.into_response();
                        }

                        if let Err(e) = db.set_code_verifier(&query.state, "REDIRECTED").await {
                            tracing::error!("Failed to update state: {query:#?} {e}");
                            return StatusCode::UNAUTHORIZED.into_response();
                        }

                        if let Some(redirect_uri) = redirect_uri {
                            Url::parse(redirect_uri).unwrap()
                        } else {
                            tracing::error!("Unmatched state / redirect_uri: {query:#?}");
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                    } else {
                        tracing::error!("Unknown state: {query:#?}");
                        return StatusCode::UNAUTHORIZED.into_response();
                    };

                redirect_uri
                    .query_pairs_mut()
                    .append_pair("code", &query.code);
                redirect_uri
                    .query_pairs_mut()
                    .append_pair("state", &query.state);

                Redirect::to(redirect_uri.as_str()).into_response()
            };

        let base = self.redirect_uri_base.clone();
        let base_path = base.path().to_string().clone();
        let certs = self.certs.clone();
        let client_id = self.client_json.client_id.clone();
        let google_home_login =
            move |OptionalAuthSession(session): OptionalAuthSession,
                  Query(query): Query<OAuth2AuthRequest>,
                  OriginalUri(uri): OriginalUri| async move {
                let (decoded_query, encoded_query) = Self::stringify_query(&uri);
                match session {
                    Some(session) => {
                        let expires = session
                            .expires
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "(no expiry)".to_string());

                        let Ok(email) = Self::validate_session(certs, &session, &[client_id])
                            .await
                            .map_err(|e| {
                                tracing::error!("JWT validation failed: {e} - {}", session.id_token)
                            })
                        else {
                            return (StatusCode::UNAUTHORIZED, Html("Unauthorized".to_string()))
                                .into_response();
                        };

                        tracing::info!("JWT data verified {email:#?}");

                        (
                            StatusCode::OK,
                            Html(format!(
                                r#"
                                Hello World for Google Home ({email}): expires {expires}
                                <br />
                                <a href='{base}google_home/link{decoded_query}'>Authorize Link</a>
                                <br />
                                <a href='{base}auth/logout'>Logout</a>
                                <br />
                                <pre><code>{query:#?}</code></pre>
                                "#,
                            )),
                        )
                            .into_response()
                    }
                    None => {
                        (
                        StatusCode::OK,
                        Html(format!(
                            "<a href='{base}auth?redirect={base_path}google_home{encoded_query}'>GOOGLE LOGIN</a>",
                        )),
                    )
                    }
                        .into_response(),
                }
            };
        let google_home_fulfillment = move |_session: AuthSession| async move {
            tracing::debug!("Handling fulfillment");
            "{}"
        };

        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.clone());

        router = router
            .route("/google_home/link", routing::get(google_home_link))
            .route("/google_home/token", routing::get(google_home_token))
            .route("/google_home/callback", routing::get(google_home_callback))
            .route("/google_home", routing::get(google_home_login))
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
