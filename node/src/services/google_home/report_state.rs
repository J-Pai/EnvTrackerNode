//! Handles reporting state for Googe Home QUERY and EXECUTE.

use std::collections::HashMap;
use std::sync::Arc;

use gcp_auth::TokenProvider;
use http::HeaderMap;
use http::StatusCode;
use http::header::AUTHORIZATION;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::Map;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::NodeError;
use crate::services::google_home::Device;
use crate::services::google_home::GoogleHome;
use crate::services::google_home::SupportedDevices;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct ReportStateRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "agentUserId")]
    agent_user_id: String,
    payload: Value,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct ReportStateParams {
    pub(super) request_id: String,
    pub(super) agent_user_id: String,
    pub(super) device_ids: Vec<String>,
}

impl GoogleHome {
    pub(super) async fn report_state(
        agent_user_id: String,
        google_home_service_account: Arc<dyn TokenProvider>,
        params: ReportStateParams,
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
    ) -> Result<ReportStateRequest, Box<dyn std::error::Error>> {
        let token = google_home_service_account
            .token(&["https://www.googleapis.com/auth/homegraph"])
            .await
            .map_err(|e| {
                tracing::error!("Google Home API Authorization Token Failure: {e}");
                e
            })?;

        let mut states = Map::new();

        let mut device_handles: Vec<JoinHandle<(String, Value)>> =
            Vec::with_capacity(devices.len());

        for (id, device) in devices.clone() {
            if !params.device_ids.is_empty() && !params.device_ids.contains(&id) {
                continue;
            }

            device_handles.push(tokio::spawn(async move {
                (id, device.lock().await.get_query_value().await)
            }));
        }

        let mut device_handle_result: Vec<(String, Value)> = Vec::with_capacity(devices.len());

        for handle in device_handles {
            if let Ok(result) = handle.await.map_err(|e| {
                tracing::warn!("Query State Failed: {e}");
            }) {
                device_handle_result.push(result);
                continue;
            }
        }

        for (id, mut query) in device_handle_result {
            if !params.device_ids.is_empty() && !params.device_ids.contains(&id) {
                continue;
            }
            let Some(object) = query.as_object_mut() else {
                continue;
            };
            object.remove("status");
            states.insert(id, query);
        }

        let mut devices = Map::new();
        let mut device_states = Map::new();
        device_states.insert("states".to_string(), Value::Object(states));
        device_states.insert("notifications".to_string(), Value::Object(Map::new()));
        devices.insert("devices".to_string(), Value::Object(device_states));
        let report_state = ReportStateRequest {
            request_id: params.request_id,
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

        let resp = request.send().await.map_err(|e| {
            tracing::error!("Failed to report state: {e}");
            e
        })?;
        let status = resp.status();
        if status != StatusCode::OK {
            tracing::error!("{report_state:#?} => {}", resp.text().await?);
            return Err(NodeError::new(
                format!("Report State Received: {status}").as_str(),
            ));
        }

        Ok(report_state)
    }
}
