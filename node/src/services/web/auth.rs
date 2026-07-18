//! Authentication handlers.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::response::Html;
use axum::routing;
use axum_oidc_client::auth::AuthenticationLayer;
use axum_oidc_client::auth::CodeChallengeMethod;
use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::extractors::OptionalAuthSession;
use axum_oidc_client::logout::handle_default_logout::DefaultLogoutHandler;

use crate::config::{FrontendServerConfig, OAuth2Config};
use crate::services::web::Web;

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

impl Web {
    fn parse_client_json(
        oauth2_client_json: &PathBuf,
    ) -> Result<ClientJsonWeb, Box<dyn std::error::Error>> {
        let json_str = fs::read_to_string(oauth2_client_json)?;
        let json = serde_json::from_str(&json_str)?;
        let json = serde_json::from_value::<ClientJson>(json)?;
        Ok(json.web)
    }

    pub(crate) async fn setup_auth(
        mut self,
        oauth2_config: &OAuth2Config,
        frontend_config: Option<FrontendServerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client_secret = Self::parse_client_json(&oauth2_config.get_client_json())?;
        let mut router = self.router;

        let mut key: [u8; 64] = [0u8; 64];
        rand::fill(&mut key);

        // Example target:
        // http://localhost:3000/auth?redirect=/userinfo

        let config = OAuthConfigurationBuilder::default()
            .with_authorization_endpoint(&client_secret.auth_uri)
            .with_token_endpoint(&client_secret.token_uri)
            .with_client_id(&client_secret.client_id)
            .with_client_secret(&client_secret.client_secret)
            .with_redirect_uri(
                &format!("{}/auth/callback", &oauth2_config.get_redirect_uri_base()).to_string(),
            )
            .with_private_cookie_key(&String::from_utf8_lossy(&key))
            .with_scopes(vec!["openid", "email", "profile"])
            .with_code_challenge_method(CodeChallengeMethod::S256)
            .with_post_logout_redirect_uri("/")
            .with_session_max_age(30)
            .with_token_max_age(300)
            .with_base_path("/auth")
            .build()?;

        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.as_ref().unwrap().clone());

        let logout_handler = Arc::new(DefaultLogoutHandler);
        let base_redirect = oauth2_config.get_redirect_uri_base();

        let base = if let Some(frontend_config) = frontend_config
            && let Some(base) = frontend_config.get_base()
        {
            base
        } else {
            String::new()
        };

        let google_home_link = move |_session: AuthSession| async move {};

        let google_home_login = move |OptionalAuthSession(session): OptionalAuthSession| async move {
            match session {
                Some(session) => {
                    let expires = session
                        .expires
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "(no expiry)".to_string());
                    Html(format!(
                        r#"
                            Hello World for Google Home: expires {expires}
                            <br />
                            <a href='{}/google_home/link'>Authorize Link</a>
                        "#,
                        base
                    ))
                }
                None => Html(format!(
                    "<a href='{base_redirect}/auth?redirect={}/google_home/login'>GOOGLE LOGIN</a>",
                    base
                )),
            }
        };

        let google_home_fulfillment = move |_session: AuthSession| async move {
            tracing::debug!("Handling fulfillment");
            "{}"
        };

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

        self.router = router;
        Ok(self)
    }
}
