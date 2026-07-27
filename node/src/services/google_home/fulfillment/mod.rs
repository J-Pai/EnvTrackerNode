//! Handler for Google Home fulfillment requests.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::jwt::decode_jwt_unverified;
use http::HeaderMap;
use http::StatusCode;
use http::header::AUTHORIZATION;
use serde_json::Map;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::services::google_home::Device;
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
        devices: HashMap<String, Arc<RwLock<impl Device>>>,
    ) -> impl IntoResponse {
        tracing::info!("HEADERS : {headers:#?}");

        let (id, auth_session) = if let Some(bearer_token) = headers.get(AUTHORIZATION)
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
            (id.1, auth_session)
        } else {
            tracing::error!("No authorization field.");
            return StatusCode::UNAUTHORIZED.into_response();
        };

        tracing::info!("[{}] JSON: {json:#?}", id.email.unwrap());

        let sub = id.sub;
        let request_id = json.get_request_id();

        let mut response = Response::new(request_id, sub);
        let Some(intent) = json.get_inputs().first() else {
            tracing::error!("No intents to process. {json:#?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        match Request::parse_intent(intent) {
            Some(Intent::Sync) => {
                response = response.set_intent(Intent::Sync);
                for (id, device) in devices {
                    response = response.add_device(id, device.read().await.get_sync_value());
                }
            }
            Some(Intent::Query(query_devices)) => {
                response = response.set_intent(Intent::Query(vec![]));

                for device in query_devices {
                    let Some(id) = device.get("id") else {
                        continue;
                    };

                    let Some(id) = id.as_str() else {
                        continue;
                    };

                    let Some(device) = devices.get(id) else {
                        response = response.add_device(id.to_string(), Value::Null);
                        continue;
                    };

                    response =
                        response.add_device(id.to_string(), device.read().await.get_query_value());
                }
            }
            Some(Intent::Execute(commands)) => {
                response = response.set_intent(Intent::Execute(vec![]));

                for command in commands {}
            }
            Some(Intent::Disconnect) => {
                if let Err(e) = db
                    .invalidate_auth_session(&format!(
                        "google_home_auth_token|{}",
                        auth_session.access_token
                    ))
                    .await
                {
                    tracing::error!("Failed to invalidate auth token: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
                if let Err(e) = db
                    .invalidate_auth_session(&format!(
                        "google_home_refresh_token|{}",
                        auth_session.refresh_token.unwrap()
                    ))
                    .await
                {
                    tracing::error!("Failed to invalidate auth token: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
                return Json(Value::Object(Map::new())).into_response();
            }
            _ => {
                response = response.error_payload(String::new());
            }
        }

        response
            .build()
            .map_err(|e| tracing::error!("Response build failed: {e}"))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}
