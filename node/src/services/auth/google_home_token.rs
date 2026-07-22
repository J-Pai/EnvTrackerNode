//! Handler for Google Home token requests.

use std::collections::HashMap;

use axum::Form;
use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use http::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
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

impl Auth {
    pub(super) async fn google_home_token_handler(
        Form(form): Form<OAuth2TokenRequest>,
        base: Url,
        db: Db,
        google_home_client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        tracing::info!("TOKEN ENDPOINT\n{form:#?}");
        let invalid_response_json = HashMap::from([("error", "invalid_grant")]);

        if form.client_id != google_home_client_json.client_id
            || form.client_secret != google_home_client_json.client_secret
        {
            tracing::error!("Invalid request: {form:#?}");
            return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json)).into_response();
        }

        let oauth2_client = ClientBuilder::new(Client::new()).build();

        if &form.grant_type == "authorization_code"
            && let Some(code) = &form.code
            && let Ok(Some(code_verifier)) = db.get_code_verifier(code).await
        {
            let mut parts = code_verifier.split("|");
            let action = parts.next();
            let redirect_uri = parts.next();

            if action.is_none() || redirect_uri.is_none() {
                if let Err(e) = db.invalidate_code_verifier(&code).await {
                    tracing::error!("Failed to update state: {e}");
                }
                tracing::error!("Bad state in DB: {form:#?}");
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            }

            if let Some(action) = action
                && action != "CALLBACK_RECEIVED"
            {
                if let Err(e) = db.invalidate_code_verifier(&code).await {
                    tracing::error!("Failed to update state: {e}");
                }
                tracing::error!("Incorrect state in DB: {form:#?}");
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            }

            if let Some(redirect_uri) = redirect_uri
                && redirect_uri != &form.redirect_uri
            {
                if let Err(e) = db.invalidate_code_verifier(&code).await {
                    tracing::error!("Failed to update state: {e}");
                }

                tracing::error!("Incorrect Redirect URI: {redirect_uri} {form:#?}");
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            }

            let form = HashMap::from([
                ("client_id", google_home_client_json.client_id),
                ("client_secret", google_home_client_json.client_secret),
                ("grant_type", "authorization_code".to_string()),
                ("code", code.clone()),
                (
                    "redirect_uri",
                    base.join("google_home/callback").unwrap().to_string(),
                ),
            ]);

            tracing::info!("OAuth2 Request Form: {form:#?}");

            let Ok(data) = oauth2_client
                .post(google_home_client_json.token_uri)
                .form(&form)
                .send()
                .await
                .map_err(|e| tracing::error!("OAuth2 request failed: {e}"))
            else {
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            };

            tracing::info!("OAuth2 Response: {data:#?}");
            let status = data.status();

            let Ok(body) = &data.text().await else {
                tracing::error!("Unable to convert body.");
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            };

            if status != StatusCode::OK {
                let body = Json::from(body);
                tracing::error!("OAuth2 Request Error: {:#?}", body);
                return (StatusCode::BAD_REQUEST, Json::from(invalid_response_json))
                    .into_response();
            }

            let body = Json::from(body);

            tracing::info!("OAuth2 Response Body: {body:#?}");
        } else if &form.grant_type == "refresh_token"
            && let Some(refresh_token) = &form.code
            && let Ok(Some(refresh_state)) = db.get_auth_session(refresh_token).await
        {
        }

        (StatusCode::BAD_REQUEST, Json::from(invalid_response_json)).into_response()
    }
}
