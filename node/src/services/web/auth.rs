//! Authentication handlers.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing;
use axum_oidc_client::auth::{AuthenticationLayer, CodeChallengeMethod};
use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::cache::TwoTierAuthCache;
use axum_oidc_client::cache::config::TwoTierCacheConfig;
use axum_oidc_client::logout::handle_default_logout::DefaultLogoutHandler;

use crate::config::OAuth2Config;
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
    async fn userinfo_handler(session: AuthSession) -> impl IntoResponse {
        let expires = session
            .expires
            .map(|e| e.to_string())
            .unwrap_or_else(|| "(no expiry)".to_string());
        format!("Hello World: expires {}", expires)
    }

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

        let cache: Arc<dyn AuthCache + Send + Sync> =
            Arc::new(TwoTierAuthCache::new(None, TwoTierCacheConfig::default())?);

        let logout_handler = Arc::new(DefaultLogoutHandler);

        router = router
            .route("/userinfo", routing::get(Self::userinfo_handler))
            .layer(AuthenticationLayer::new(
                Arc::new(config),
                cache,
                logout_handler,
            ));

        self.router = router;
        Ok(self)
    }
}
