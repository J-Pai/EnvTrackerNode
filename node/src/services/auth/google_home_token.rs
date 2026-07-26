//! Handler for Google Home token requesters.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Form;
use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::jwt::DecodingKey;
use chrono::DateTime;
use chrono::Local;
use http::HeaderMap;
use http::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use tokio::sync::RwLock;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;
use crate::services::db::Db;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct OAuth2TokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: Option<String>,
    refresh_token: Option<String>,
    redirect_uri: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct OAuth2Token {
    token_type: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

impl Auth {
    async fn oauth2_token_request(
        google_home_client_json: &ClientJsonWeb,
        mut form: HashMap<&str, String>,
    ) -> Result<OAuth2Token, ()> {
        form.insert("client_id", google_home_client_json.client_id.clone());
        form.insert(
            "client_secret",
            google_home_client_json.client_secret.clone(),
        );

        tracing::info!("OAuth2 Request Form: {form:#?}");

        let oauth2_client = ClientBuilder::new(Client::new()).build();

        let Ok(data) = oauth2_client
            .post(google_home_client_json.token_uri.clone())
            .form(&form)
            .send()
            .await
            .map_err(|e| tracing::error!("OAuth2 request failed: {e}"))
        else {
            return Err(());
        };

        tracing::info!("OAuth2 Response: {data:#?}");
        let status = data.status();

        let Ok(body) = &data.text().await else {
            tracing::error!("Unable to convert body.");
            return Err(());
        };

        if status != StatusCode::OK {
            let body = serde_json::from_str::<HashMap<String, String>>(body);
            tracing::error!("OAuth2 Request Error: {:#?}", body);
            return Err(());
        }

        tracing::info!("OAuth2 Raw Response: {body:#?}");

        let body = serde_json::from_str::<OAuth2Token>(body).unwrap();

        tracing::info!("OAuth2 Response: {body:#?}");

        Ok(body)
    }

    pub(super) async fn google_home_token_handler(
        headers: HeaderMap,
        Form(form): Form<OAuth2TokenRequest>,
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        base: Url,
        db: Arc<dyn AuthCache + Send + Sync>,
        client_json: ClientJsonWeb,
        google_home_client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        tracing::info!("HEADERS : {headers:#?}");

        tracing::info!("TOKEN ENDPOINT\n{form:#?}");
        let invalid_response = (
            StatusCode::BAD_REQUEST,
            Json::from(HashMap::from([("error", "invalid_grant")])),
        )
            .into_response();

        if form.client_id != google_home_client_json.client_id
            || form.client_secret != google_home_client_json.client_secret
        {
            tracing::error!("Invalid request: {form:#?}");
            return invalid_response;
        }

        if &form.grant_type == "authorization_code"
            && let Some(code) = &form.code
            && let Ok(Some(code_verifier)) = db.get_code_verifier(code).await
        {
            let mut parts = code_verifier.split("|");
            let session_id = parts.next();
            let redirect_uri = parts.next();

            if session_id.is_none() || redirect_uri.is_none() {
                if let Err(e) = db.invalidate_code_verifier(&code).await {
                    tracing::error!("Failed to update state: {e}");
                }
                tracing::error!("Bad state in DB: {form:#?}");
                return invalid_response;
            }

            if let Err(e) = db.invalidate_code_verifier(&code).await {
                tracing::error!("Failed to update state: {e}");
                return invalid_response;
            }

            let Some(session_id) = session_id else {
                tracing::error!("No state session_key: {form:#?}");
                return invalid_response;
            };

            tracing::info!("SESSION ID: {session_id}");

            let Ok(Some(auth_session)) = db.get_auth_session(session_id).await else {
                tracing::error!("Stale session_key: {form:#?}");
                return invalid_response;
            };

            match Self::validate_session(certs, &auth_session, &[client_json.client_id]).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Invalid session id token: {e:#?}");
                    return invalid_response;
                }
            }

            if let Some(redirect_uri) = redirect_uri
                && redirect_uri != &form.redirect_uri
            {
                if let Err(e) = db.invalidate_code_verifier(&code).await {
                    tracing::error!("Failed to update state: {e}");
                }

                tracing::error!("Incorrect Redirect URI: {redirect_uri} {form:#?}");
                return invalid_response;
            }

            let form = HashMap::from([
                ("grant_type", "authorization_code".to_string()),
                ("code", code.clone()),
                (
                    "redirect_uri",
                    base.join("google_home/callback").unwrap().to_string(),
                ),
            ]);

            let Ok(mut body) = Self::oauth2_token_request(&google_home_client_json, form).await else {
                tracing::error!("OAuth2 token request failed");
                return invalid_response;
            };

            let Some(refresh_token) = &body.refresh_token else {
                tracing::error!("Failed to obtain Google Home refresh token.");
                return invalid_response;
            };

            let utc = DateTime::from_timestamp(Db::expires_at(body.expires_in), 0).unwrap();
            let updated_auth_session = AuthSession {
                access_token: body.access_token.clone(),
                refresh_token: Some(refresh_token.clone()),
                expires: Some(DateTime::<Local>::from(utc)),
                ..auth_session
            };

            if let Err(e) = db
                .set_auth_session(
                    &format!("google_home_auth_token|{}", body.access_token),
                    updated_auth_session.clone(),
                )
                .await
            {
                tracing::error!("Failed to save Google Home token: {e}");
                return invalid_response;
            }

            if let Err(e) = db
                .set_auth_session(
                    &format!("google_home_refresh_token|{}", refresh_token),
                    updated_auth_session.clone(),
                )
                .await
            {
                tracing::error!("Failed to save Google Home token: {e}");
                return invalid_response;
            }

            // Set refresh token to expiration of 1 year.
            if let Err(e) = db
                .extend_auth_session(
                    &format!("google_home_refresh_token|{}", refresh_token),
                    60 * 24 * 365,
                )
                .await
            {
                tracing::error!("Failed to save Google Home refresh token: {e}");
                return invalid_response;
            }

            if let Err(e) = db.invalidate_auth_session(session_id).await {
                tracing::error!("Failed to invalidate Google Home link session. {e}");
                return invalid_response;
            };

            body.expires_in = 60;
            return Json::from(&body).into_response();
        } else if &form.grant_type == "refresh_token"
            && let Some(refresh_token) = &form.refresh_token
            && let Ok(Some(auth_session)) = db
                .get_auth_session(&format!("google_home_refresh_token|{}", refresh_token))
                .await
        {
            let Some(stored_refresh_token) = &auth_session.refresh_token else {
                tracing::error!("Google Home session has no refresh token.");
                return invalid_response;
            };

            if &refresh_token != &stored_refresh_token {
                tracing::error!(
                    "Mismatched refresh token. {refresh_token} != {stored_refresh_token}"
                );
                return invalid_response;
            }

            let form = HashMap::from([
                ("grant_type", "refresh_token".to_string()),
                ("refresh_token", stored_refresh_token.clone()),
            ]);

            let Ok(mut body) = Self::oauth2_token_request(&google_home_client_json, form).await else {
                return invalid_response;
            };

            let utc = DateTime::from_timestamp(Db::expires_at(body.expires_in), 0).unwrap();
            let updated_auth_session = AuthSession {
                access_token: body.access_token.clone(),
                expires: Some(DateTime::<Local>::from(utc)),
                ..auth_session
            };

            if let Err(e) = db
                .set_auth_session(
                    &format!("google_home_auth_token|{}", body.access_token),
                    updated_auth_session,
                )
                .await
            {
                tracing::error!("Failed to save Google Home token: {e}");
                return invalid_response;
            }

            // Set refresh token to expiration of 1 year.
            if let Err(e) = db
                .extend_auth_session(
                    &format!("google_home_refresh_token|{}", refresh_token),
                    60 * 24 * 365,
                )
                .await
            {
                tracing::error!("Failed to save Google Home refresh token: {e}");
                return invalid_response;
            }

            body.expires_in = 60;
            return Json::from(&body).into_response();
        }

        tracing::error!("Unknown grant type. {form:#?}");
        invalid_response
    }
}
