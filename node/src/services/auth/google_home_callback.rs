//! Handler for Google Home OAuth2 token request callback.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::jwt::DecodingKey;
use http::StatusCode;
use tokio::sync::RwLock;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;
use crate::services::db::Db;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct OAuth2CallbackRequest {
    code: String,
    iss: String,
    state: String,
    scope: String,
    prompt: String,
}

impl Auth {
    pub(super) async fn google_home_callback_handler(
        session: AuthSession,
        Query(query): Query<OAuth2CallbackRequest>,
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        db: Db,
        client_json: ClientJsonWeb,
        google_home_client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        let Ok(_) = Self::validate_session(certs, &session, &[client_json.client_id])
            .await
            .map_err(|e| tracing::error!("JWT validation failed: {e} - {}", session.id_token))
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        tracing::info!("Callback Received: {query:#?}");

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
                if code_verifier.starts_with("CALLBACK_RECEIVED") {
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

                if let Err(e) = db
                    .set_code_verifier(&query.state, &format!("CALLBACK_RECEIVED:{}", query.code))
                    .await
                {
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

        tracing::info!("Redirecting: {redirect_uri}");

        Redirect::to(redirect_uri.as_str()).into_response()
    }
}
