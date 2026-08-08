//! Handler for Google Home devices.
//!
//! This is supposed to run on the node endpoint.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Form;
use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use gcp_auth::TokenProvider;
use http::StatusCode;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::services::google_home::Device;
use crate::services::google_home::GoogleHome;
use crate::services::google_home::ReportStateParams;
use crate::services::google_home::SupportedDevices;

impl GoogleHome {
    pub(super) async fn device_get_handler(
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
        device: Option<Path<String>>,
    ) -> impl IntoResponse {
        tracing::info!("{device:#?}");
        let Some(Path(device)) = device else {
            let mut device_sync: Vec<Value> = vec![];

            for device in devices.values() {
                device_sync.push(device.lock().await.get_sync_value().await);
            }

            return Json(device_sync).into_response();
        };

        let Some(device) = devices.get(&device) else {
            tracing::error!("Device {device} not found.");
            return StatusCode::NOT_FOUND.into_response();
        };

        Json(device.lock().await.get_query_value().await).into_response()
    }

    pub(super) async fn device_post_handler(
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
        Path(device): Path<String>,
        Json(actions): Json<Vec<Value>>,
    ) -> impl IntoResponse {
        let Some(device) = devices.get(&device) else {
            tracing::error!("Device {device} not found.");
            return StatusCode::NOT_FOUND.into_response();
        };

        Json(device.lock().await.execute_actions(&actions).await).into_response()
    }

    pub(super) async fn device_report_state_handler(
        devices: HashMap<String, Arc<Mutex<SupportedDevices>>>,
        gcp_auth_token: Arc<dyn TokenProvider>,
        Form(params): Form<ReportStateParams>,
    ) -> impl IntoResponse {
        tracing::info!("{params:#?}");

        let Ok(status) = Self::report_state(
            gcp_auth_token,
            params.request_id,
            params.agent_user_id,
            devices,
        )
        .await
        .map_err(|e| {
            tracing::warn!("Issue with reporting state: {e}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        Json(status).into_response()
    }
}
