//! Authentication handlers.

use std::collections::HashMap;
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
use axum_oidc_client::jwt::Algorithm;
use axum_oidc_client::jwt::DecodingKey;
use axum_oidc_client::jwt::Validation;
use axum_oidc_client::jwt::decode_jwt;
use axum_oidc_client::jwt::decode_jwt_unverified;
use axum_oidc_client::logout::handle_default_logout::DefaultLogoutHandler;
use tokio::sync::RwLock;
use tower_sessions::cookie::Key;

use crate::config::FrontendServerConfig;
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
    fn parse_client_json(
        oauth2_client_json: &PathBuf,
    ) -> Result<ClientJsonWeb, Box<dyn std::error::Error>> {
        let json_str = fs::read_to_string(oauth2_client_json)?;
        let json = serde_json::from_str(&json_str)?;
        let json = serde_json::from_value::<ClientJson>(json)?;
        Ok(json.web)
    }

    fn request_google_oauth2_certs(cers: RwLock<HashMap<String, DecodingKey>>) {}

    pub(crate) async fn setup_auth(
        mut self,
        oauth2_config: &OAuth2Config,
        frontend_config: Option<FrontendServerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client_secret = Self::parse_client_json(&oauth2_config.get_client_json())?;
        let mut router = self.router;
        let key = oauth2_config.get_cookie_secret_key();
        let base = if let Some(frontend_config) = frontend_config
            && let Some(base) = frontend_config.get_base()
        {
            base
        } else {
            String::new()
        };

        // Example target:
        // http://localhost:3000/auth?redirect=/userinfo

        let mut config = OAuthConfigurationBuilder::default()
            .with_authorization_endpoint(&client_secret.auth_uri)
            .with_token_endpoint(&client_secret.token_uri)
            .with_client_id(&client_secret.client_id)
            .with_client_secret(&client_secret.client_secret)
            .with_redirect_uri(
                &format!("{}/auth/callback", &oauth2_config.get_redirect_uri_base()).to_string(),
            )
            .with_scopes(vec!["openid", "email", "profile"])
            .with_code_challenge_method(CodeChallengeMethod::S256)
            .with_post_logout_redirect_uri(&format!("{base}/google_home/login"))
            .with_session_max_age(60 * 25 * 365)
            .with_base_path("/auth");
        config.private_cookie_key = Some(Key::from(&key[..64]));
        let config = config.build()?;

        let logout_handler = Arc::new(DefaultLogoutHandler);
        let base_redirect = oauth2_config.get_redirect_uri_base();

        let google_home_link = move |_session: AuthSession| async move {};
        let google_home_login = move |OptionalAuthSession(session): OptionalAuthSession| async move {
            match session {
                Some(session) => {
                    let expires = session
                        .expires
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "(no expiry)".to_string());
                    tracing::info!("token {:#?}", session.id_token);

                    let token_data = decode_jwt_unverified(&session.id_token)
                        .map_err(|e| tracing::error!("JWT Error: {e}"));
                    tracing::info!("JWT data unverified {:#?}", token_data);

                    let key = DecodingKey::from_rsa_pem(
                        "-----BEGIN CERTIFICATE-----\nMIIDJzCCAg+gAwIBAgIJAKInBBbLHDHYMA0GCSqGSIb3DQEBBQUAMDYxNDAyBgNV\nBAMMK2ZlZGVyYXRlZC1zaWdub24uc3lzdGVtLmdzZXJ2aWNlYWNjb3VudC5jb20w\nHhcNMjYwNzEwMTc1OTE4WhcNMjYwNzI3MDYxNDE4WjA2MTQwMgYDVQQDDCtmZWRl\ncmF0ZWQtc2lnbm9uLnN5c3RlbS5nc2VydmljZWFjY291bnQuY29tMIIBIjANBgkq\nhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsjGSud3Gx+92yeucu7BIAhvzkybkL21e\nOCejL9t2JMpqy6cMThhS2Dtr+ByKdjtoD0GLP8LCT2yJfJH5YAbHVbvBU88eUsbG\nd7ZlaYqicTD5Pc6B/BO8LjSr3YH1kVoOcn8Lct31+EhloVAIxBLROsS2489N3bwW\nOLaOhnYCLvMWqVFqV5TJPMbIBzADXeJmAyF/K2uP5P7KDWYlGz2V6AH7aS6n3/K0\nvdb8SVeqCu8N0M5SifpSUMQidVp5Ku+wd0Yu6P9mZcAzS9GuzePJMNsbKjDkITlc\n1k+KZMO2RH23zAbMCNqVABQRFLCQhulYEAbd+sYbulsrHaw4MYo3YwIDAQABozgw\nNjAMBgNVHRMBAf8EAjAAMA4GA1UdDwEB/wQEAwIHgDAWBgNVHSUBAf8EDDAKBggr\nBgEFBQcDAjANBgkqhkiG9w0BAQUFAAOCAQEAEww8cWooeHcykMf6g5WEf/CXPdlO\nkqrebFA9RLjf3rLB/ausHSgYkvHg8wq3rWICDPA3VmR6YjgLpQ+EmO8ShyFn75Cp\nPq3SwUJt3AEuNUiAYgF6xp2JUkMAjCpFfBYcyWJIia2+WCsKYB83dMmhOxAsKdn2\n0GGn4uGnrltA+3QYtTW7cy4IplPVT4XrCIVKFj9rnqJujpr1zHTN9eqML8ovtiyO\nKniDV7SHS5I8tfZ39EKOEZzOCrhyy/2ZDgqqMwjl8YmTCiHJk+AGDmDPGcZZGdH3\nRanMPhXXF59N7hXusIHuX2BZ/Ypg4VIp2VCEheZboLdwfufybdn0mZ52zQ==\n-----END CERTIFICATE-----\n".as_bytes(),
                    ).unwrap();
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[
                        "42441590702-te8vdhfdd8s3ft960ct6ksle7hp25jtj.apps.googleusercontent.com",
                    ]);

                    let token_data = decode_jwt(&session.id_token, &key, &validation)
                        .map_err(|e| tracing::error!("JWT Error: {e}"));
                    tracing::info!("JWT data verified {:#?}", token_data);

                    Html(format!(
                        r#"
                            Hello World for Google Home: expires {expires}
                            <br />
                            <a href='{base}/google_home/link'>Authorize Link</a>
                            <br />
                            <a href='{base}/auth/logout'>Logout</a>
                        "#,
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

        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.as_ref().unwrap().clone());

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
