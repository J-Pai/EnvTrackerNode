//! Handles reporting state for Googe Home QUERY and EXECUTE.

use std::collections::HashMap;
use std::sync::Arc;

use gcp_auth::TokenProvider;
use http::header::AUTHORIZATION;
use http::{HeaderMap, StatusCode};
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::{Map, Value};
use tokio::sync::Mutex;

use crate::error::NodeError;
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
}

impl GoogleHome {
    pub(super) async fn report_state(
        google_home_service_account: Arc<dyn TokenProvider>,
        request_id: String,
        agent_user_id: String,
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
