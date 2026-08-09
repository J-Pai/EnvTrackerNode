//! Handler for Google Home devices.
//!
//! This is supposed to run on the node endpoint.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::response::IntoResponse;
use gcp_auth::TokenProvider;
use http::StatusCode;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::services::google_home::Device;
use crate::services::google_home::GoogleHome;
use crate::services::google_home::ReportStateParams;
use crate::services::google_home::SupportedDevices;

impl GoogleHome {
    pub(super) async fn device_get_handler(
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
        device_ids: Option<Query<Vec<(String, String)>>>,
        device_id: Option<Path<String>>,
    ) -> impl IntoResponse {
        let multi_query = async |device_ids: &[String], sync: bool| -> Vec<(String, Value)> {
            let mut device_sync: Vec<(String, Value)> = vec![];

            let mut device_handles: Vec<JoinHandle<(String, Value)>> =
                Vec::with_capacity(devices.len());

            for device_id in device_ids {
                let device_id = device_id.clone();
                if let Some(device) = devices.get(&device_id) {
                    let device = device.clone();
                    device_handles.push(tokio::spawn(async move {
                        if sync {
                            (device_id, device.lock().await.get_sync_value().await)
                        } else {
                            (device_id, device.lock().await.get_query_value().await.1)
                        }
                    }));
                }
            }

            for handle in device_handles {
                if let Ok(result) = handle.await.map_err(|e| {
                    tracing::warn!("Query State Failed: {e}");
                }) {
                    device_sync.push(result);
                    continue;
                }
            }

            device_sync
        };

        if let Some(Query(device_ids)) = device_ids
            && !device_ids.is_empty()
        {
            let device_ids: Vec<String> = device_ids
                .iter()
                .filter(|(k, _)| k == "device_ids")
                .map(|(_, v)| v.clone())
                .collect();

            return Json(multi_query(&device_ids, false).await).into_response();
        }

        let Some(Path(device)) = device_id else {
            let device_ids: Vec<String> = devices.keys().cloned().collect();
            return Json(multi_query(&device_ids, true).await).into_response();
        };

        let Some(device) = devices.get(&device) else {
            tracing::error!("Device {device} not found.");
            return StatusCode::NOT_FOUND.into_response();
        };

        Json(device.lock().await.get_query_value().await).into_response()
    }

    pub(super) async fn device_post_handler(
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
        device_id: Path<String>,
        Json(actions): Json<Vec<Value>>,
    ) -> impl IntoResponse {
        let device_id = device_id.clone();
        let Some(device) = devices.get(&device_id) else {
            tracing::error!("Device {device_id} not found.");
            return StatusCode::NOT_FOUND.into_response();
        };

        return Json(device.lock().await.execute_actions(&actions).await).into_response();
    }

    pub(super) async fn device_report_state_handler(
        Json(params): Json<ReportStateParams>,
        agent_user_id: String,
        gcp_auth_token: Arc<dyn TokenProvider>,
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
    ) -> impl IntoResponse {
        let Ok(status) = Self::report_state(agent_user_id, gcp_auth_token, params, devices)
            .await
            .map_err(|e| {
                tracing::warn!("Issue with reporting state: {e}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        Json(status).into_response()
    }
}
