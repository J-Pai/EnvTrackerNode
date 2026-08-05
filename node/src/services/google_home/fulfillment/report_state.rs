//! Handles reporting state for Googe Home QUERY and EXECUTE.

use std::sync::Arc;

use futures_util::TryFutureExt;
use gcp_auth::TokenProvider;
use http::header::AUTHORIZATION;
use http::{HeaderMap, StatusCode};
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::{Map, Value};

use crate::error::NodeError;
use crate::services::google_home::GoogleHome;
use crate::services::google_home::fulfillment::request::Intent;
use crate::services::google_home::fulfillment::response::Response;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ReportStateRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "agentUserId")]
    agent_user_id: String,
    payload: Value,
}

impl GoogleHome {
    pub(super) async fn report_state(
        google_home_service_account: Arc<dyn TokenProvider>,
        request_id: String,
        agent_user_id: String,
        intent: Option<Intent>,
        response: &Response,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let token = google_home_service_account
            .token(&["https://www.googleapis.com/auth/homegraph"])
            .await
            .map_err(|e| {
                tracing::error!("Google Home API Authorization Token Failure: {e}");
                e
            })?;

        let mut states = Map::new();

        let payload = response.payload.clone();

        match intent {
            Some(Intent::Sync) => {
                return Ok(());
            }
            Some(Intent::Query(_)) => {
                let Some(devices) = payload.as_object() else {
                    return Err(NodeError::new("Query - Mismatched payload type"));
                };
                let Some(devices) = devices.get("devices") else {
                    return Err(NodeError::new("Query - Mismatched payload type"));
                };
                let Some(devices) = devices.as_object() else {
                    return Err(NodeError::new("Query - Mismatched payload type"));
                };
                for (id, data) in devices {
                    let Some(state) = data.as_object() else {
                        continue;
                    };
                    let mut new_state = state.clone();
                    new_state.remove_entry("status");
                    states.insert(id.to_string(), Value::Object(new_state));
                }
            }
            Some(Intent::Execute(_)) => {
                let Some(commands) = payload.as_object() else {
                    return Err(NodeError::new("Execute - Mismatched payload type"));
                };
                let Some(commands) = commands.get("commands") else {
                    return Err(NodeError::new("Execute - Mismatched payload type"));
                };
                let Some(commands) = commands.as_array() else {
                    return Err(NodeError::new("Execute - Mismatched payload type"));
                };

                for command in commands {
                    let Some(command) = command.as_object() else {
                        continue;
                    };
                    let Some(ids) = command.get("ids") else {
                        continue;
                    };
                    let Some(ids) = ids.as_array() else {
                        continue;
                    };
                    let Some(device_state) = command.get("states") else {
                        continue;
                    };
                    for id in ids {
                        let Some(id) = id.as_str() else {
                            continue;
                        };

                        let Some(state) = device_state.as_object() else {
                            continue;
                        };
                        let mut new_state = state.clone();
                        new_state.remove_entry("status");
                        states.insert(id.to_string(), Value::Object(new_state));
                    }
                }
            }
            Some(Intent::Disconnect) => return Ok(()),
            _ => return Ok(()),
        }

        let mut devices = Map::new();
        let mut device_states = Map::new();
        device_states.insert("states".to_string(), Value::Object(states));
        device_states.insert("notifications".to_string(), Value::Object(Map::new()));
        devices.insert("devices".to_string(), Value::Object(device_states));
        let report_state = ReportStateRequest {
            request_id,
            agent_user_id,
            payload: Value::Object(devices),
        };

        let google_home_api_client = ClientBuilder::new(Client::new()).build();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", token.as_str()).parse().unwrap(),
        );

        let request = google_home_api_client
            .post("https://homegraph.googleapis.com/v1/devices:reportStateAndNotification")
            .headers(headers)
            .json(&report_state);

        let resp = request
            .send()
            .map_err(|e| {
                tracing::error!("Failed to report state: {e}");
                e
            })
            .await?;
        let status = resp.status();
        if status != StatusCode::OK {
            tracing::error!("{report_state:#?} => {}", resp.text().await?);
            return Err(NodeError::new(
                format!("Report State Received: {status}").as_str(),
            ));
        }

        Ok(())
    }
}
