//! Handler for Google Home devices.
//!
//! This is supposed to run on the node endpoint.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use http::StatusCode;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::services::google_home::Device;
use crate::services::google_home::GoogleHome;

impl GoogleHome {
    pub(super) async fn device_get_handler(
        devices: HashMap<String, Arc<Mutex<impl Device>>>,
        device: Option<Path<String>>,
    ) -> impl IntoResponse {
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
        devices: HashMap<String, Arc<Mutex<impl Device>>>,
        Path(device): Path<String>,
        Json(actions): Json<Vec<Value>>,
    ) -> impl IntoResponse {
        let Some(device) = devices.get(&device) else {
            tracing::error!("Device {device} not found.");
            return StatusCode::NOT_FOUND.into_response();
        };

        Json(device.lock().await.execute_actions(&actions).await).into_response()
    }
}
