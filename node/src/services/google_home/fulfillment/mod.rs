//! Handler for Google Home fulfillment requests.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::response::IntoResponse;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::jwt::decode_jwt_unverified;
use futures_util::TryFutureExt;
use gcp_auth::TokenProvider;
use http::HeaderMap;
use http::StatusCode;
use http::header::AUTHORIZATION;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::Map;
use serde_json::Value;
use url::Url;

use crate::services::google_home::GoogleHome;
use crate::services::google_home::fulfillment::request::Intent;
use crate::services::google_home::fulfillment::request::Request;
use crate::services::google_home::fulfillment::response::Response;

pub(crate) mod report_state;
pub(crate) mod request;
pub(crate) mod response;

impl GoogleHome {
    pub(super) async fn fulfillment_handler(
        headers: HeaderMap,
        Json(json): Json<Request>,
        db: Arc<dyn AuthCache + Send + Sync>,
        gcp_auth_token: Arc<dyn TokenProvider>,
        node_uri: Url,
    ) -> impl IntoResponse {
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

        let sub = id.sub;
        let request_id = json.get_request_id();

        let mut response = Response::new(request_id.clone(), sub.clone());
        let Some(intent) = json.get_inputs().first() else {
            tracing::error!("No intents to process. {json:#?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let node_client = ClientBuilder::new(Client::new()).build();
        loop {
            match Request::parse_intent(intent) {
                Some(Intent::Sync) => {
                    response = response.set_intent(Intent::Sync);

                    let mut node_uri = node_uri.clone();
                    let path = node_uri.path().to_string();
                    let path = path.strip_suffix("/").unwrap();
                    node_uri.set_path(path);

                    let request = node_client.get(node_uri);
                    let Ok(resp) = request.send().await.map_err(|e| {
                        tracing::error!("Resceive Sync from Node: {e}");
                        e
                    }) else {
                        response = response.error_payload("hardError".to_string());
                        break;
                    };
                    let status = resp.status();

                    if status != StatusCode::OK {
                        tracing::error!("Sync error: {}", resp.text().await.unwrap());
                        response = response.error_payload("hardError".to_string());
                        break;
                    }

                    let Ok(devices): Result<Vec<Value>, _> = resp.json().await else {
                        response = response.error_payload("hardError".to_string());
                        break;
                    };

                    for device in devices {
                        let Some(id) = device.as_object().clone() else {
                            continue;
                        };
                        let Some(id) = id.get("id") else {
                            continue;
                        };
                        let Some(id) = id.as_str() else {
                            continue;
                        };
                        response = response.add_device(id.to_string(), device);
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

                        let Ok(node_uri) = node_uri.join(id) else {
                            continue;
                        };

                        let request = node_client.get(node_uri.clone());
                        let Ok(resp) = request.send().await.map_err(|e| {
                            tracing::error!("Resceive Sync from Node: {e}");
                            e
                        }) else {
                            response = response.error_payload("hardError".to_string());
                            break;
                        };
                        let status = resp.status();

                        if status == StatusCode::NOT_FOUND {
                            tracing::error!("NOT FOUND: {id}");
                            response = response.error_payload("deviceNotFound".to_string());
                            break;
                        }

                        if status != StatusCode::OK {
                            tracing::error!("Query error: {}", resp.text().await.unwrap());
                            response = response.error_payload("hardError".to_string());
                            break;
                        }

                        let Ok(state): Result<Value, _> = resp.json().await else {
                            response = response.error_payload("hardError".to_string());
                            break;
                        };

                        response = response.add_device(id.to_string(), state);
                    }
                }
                Some(Intent::Execute(commands)) => {
                    response = response.set_intent(Intent::Execute(vec![]));

                    let mut result: HashMap<Value, Vec<String>> = HashMap::new();

                    for command in commands {
                        for id in command.get_devices() {
                            let Ok(node_uri) = node_uri.join(&id) else {
                                continue;
                            };

                            let request = node_client
                                .post(node_uri.clone())
                                .json(command.get_execution());

                            let Ok(resp) = request.send().await.map_err(|e| {
                                tracing::error!("Resceive Sync from Node: {e}");
                                e
                            }) else {
                                response = response.error_payload("hardError".to_string());
                                break;
                            };
                            let status = resp.status();

                            if status == StatusCode::NOT_FOUND {
                                tracing::error!("NOT FOUND: {id}");
                                response = response.error_payload("deviceNotFound".to_string());
                                break;
                            }

                            if status != StatusCode::OK {
                                tracing::error!("Query error: {}", resp.text().await.unwrap());
                                response = response.error_payload("hardError".to_string());
                                break;
                            }

                            let Ok(state): Result<Value, _> = resp.json().await else {
                                response = response.error_payload("hardError".to_string());
                                break;
                            };
                            let ids = match result.get_mut(&state) {
                                Some(ids) => ids,
                                None => {
                                    result.insert(state.clone(), vec![]);
                                    result.get_mut(&state).unwrap()
                                }
                            };
                            ids.push(id);
                        }
                    }
                    response = response.add_command_status(result);
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
            break;
        }

        let _ = Self::report_state(
            gcp_auth_token,
            request_id,
            sub,
            response.get_intent(),
            &response,
        )
        .map_err(|e| {
            tracing::warn!("{intent} - Issue with reporting state: {e}");
        })
        .await;

        response
            .build()
            .map_err(|e| tracing::error!("Response build failed: {e}"))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}
