//! Configuration and set up for OAuth2 based authentication.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::response::Html;
use axum::{Router, routing};
use axum_oidc_client::auth::{AuthenticationLayer, CodeChallengeMethod};
use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::extractors::OptionalAuthSession;
use axum_oidc_client::jwt::{
    Algorithm, DecodingKey, Validation, decode_jwt, decode_jwt_unverified,
};
use axum_oidc_client::logout::handle_default_logout::DefaultLogoutHandler;
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
    _certs: RwLock<HashMap<String, DecodingKey>>,
    client_json: ClientJsonWeb,
    redirect_uri_base: Url,
    cookie_secret_key: Vec<u8>,
}

impl Auth {
    pub(crate) async fn new(
        oauth2_config: &OAuth2Config,
        db: Db,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client_json = Self::parse_client_json(&oauth2_config.get_client_json())?;
        let certs = RwLock::new(HashMap::new());
        db.create_auth_table().await?;
        Ok(Self {
            db,
            _certs: certs,
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

    pub(crate) fn setup_auth_router(
        &mut self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        let base = self.redirect_uri_base.clone();

        let base_path = if base.path().ends_with("/") {
            base.path().to_string()
        } else {
            let base = base.clone();
            let mut base_path = String::from(base.clone().path());
            base_path.push('/');
            base_path
        };
        let base = base.join(&base_path).unwrap();

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
            .with_session_max_age(60 * 25 * 365)
            .with_base_path("/auth");
        config.private_cookie_key = Some(Key::from(&self.cookie_secret_key));
        let config = config.build()?;

        let logout_handler = Arc::new(DefaultLogoutHandler);

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
                            <a href='{base}google_home/link'>Authorize Link</a>
                            <br />
                            <a href='{base}auth/logout'>Logout</a>
                        "#,
                    ))
                }
                None => Html(format!(
                    "<a href='{base}auth?redirect={base_path}google_home/login'>GOOGLE LOGIN</a>",
                )),
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
