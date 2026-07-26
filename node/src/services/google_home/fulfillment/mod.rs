//! Handler for Google Home fulfillment requests.

use std::sync::Arc;

use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::jwt::decode_jwt_unverified;
use http::HeaderMap;
use http::StatusCode;
use http::header::AUTHORIZATION;

use crate::services::google_home::GoogleHome;
use crate::services::google_home::fulfillment::request::Intent;
use crate::services::google_home::fulfillment::request::Request;
use crate::services::google_home::fulfillment::response::Response;

pub(crate) mod request;
pub(crate) mod response;

impl GoogleHome {
    pub(super) async fn google_home_fulfillment_handler(
        headers: HeaderMap,
        Json(json): Json<Request>,
        db: Arc<dyn AuthCache + Send + Sync>,
    ) -> impl IntoResponse {
        tracing::info!("HEADERS : {headers:#?}");

        let id = if let Some(bearer_token) = headers.get(AUTHORIZATION)
            && let Ok(bearer_token) = bearer_token.to_str()
        {
            let split = bearer_token.split(" ");
            let Some(bearer_token) = split.last() else {
                tracing::error!("No bearer token in authorization field.");
                return StatusCode::UNAUTHORIZED.into_response();
            };

            let Ok(Some(auth_session)) = db
                .get_auth_session(&format!("google_home_auth_token|{bearer_token}"))
                .await
            else {
                tracing::error!("Bearer/Access Token not found. {bearer_token}");
                return StatusCode::UNAUTHORIZED.into_response();
            };
            let Ok(id) = decode_jwt_unverified(&auth_session.id_token)
                .map_err(|e| tracing::error!("Token ID Failure: {e}"))
            else {
                tracing::error!("Invalid Auth Session. {auth_session:#?}");
                return StatusCode::UNAUTHORIZED.into_response();
            };
            id.1
        } else {
            tracing::error!("No authorization field.");
            return StatusCode::UNAUTHORIZED.into_response();
        };

        tracing::info!("[{}] JSON: {json:#?}", id.email.unwrap());

        let sub = id.sub;
        let request_id = json.get_request_id();
        let response = Response::new(request_id, sub);

        for intent in json.get_inputs() {
            match Request::parse_intent(intent) {
                Some(Intent::Sync) => {
                    response = response
                        .set_intent(Intent::Sync)
                        .add_device(value);
                }
                _ => {}
            }
        }

        response
            .build()
            .map_err(|e| tracing::error!("Response build failed: {e}"))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}
