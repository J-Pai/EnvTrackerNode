//! Handler for Google Home OAuth2 token request callback.

use std::sync::Arc;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum_oidc_client::auth_cache::AuthCache;
use http::StatusCode;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;

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
        Query(query): Query<OAuth2CallbackRequest>,
        db: Arc<dyn AuthCache + Send + Sync>,
        google_home_client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        tracing::info!("Callback Received: {query:#?}");

        if query.iss != "https://accounts.google.com" {
            tracing::error!("Incorrect issuer: {query:#?}");
            return StatusCode::BAD_REQUEST.into_response();
        }

        if query.prompt != "consent" {
            tracing::error!("Incorrect prompt: {query:#?}");
            return StatusCode::BAD_REQUEST.into_response();
        }

        let mut redirect_uri =
            if let Ok(Some(code_verifier)) = db.get_code_verifier(&query.state).await {
                let mut parts = code_verifier.split("|");
                let session_id = parts.next();
                let redirect_uri = parts.next();
                let project_id = parts.next();

                if let Some(project_id) = project_id
                    && project_id != google_home_client_json.project_id
                {
                    tracing::error!("Unmatched state / project_id: {query:#?}");
                    return StatusCode::BAD_REQUEST.into_response();
                }

                let Some(session_id) = session_id else {
                    tracing::error!("No session_id: {query:#?}");
                    return StatusCode::BAD_REQUEST.into_response();
                };

                if let Err(e) = db.invalidate_code_verifier(&query.state).await {
                    tracing::error!("Failed to update state: {query:#?} {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                if let Some(redirect_uri) = redirect_uri {
                    if let Err(e) = db
                        .set_code_verifier(&query.code, &format!("{}|{}", session_id, redirect_uri))
                        .await
                    {
                        tracing::error!("Failed to add code: {query:#?} {e}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                    Url::parse(redirect_uri).unwrap()
                } else {
                    tracing::error!("Unmatched state / redirect_uri: {query:#?}");
                    return StatusCode::BAD_REQUEST.into_response();
                }
            } else {
                tracing::error!("Unknown state: {query:#?}");
                return StatusCode::BAD_REQUEST.into_response();
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
