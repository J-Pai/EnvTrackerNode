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
            let ids = Value::Array(
                devices
                    .keys()
                    .map(|f| Value::String(f.to_string()))
                    .collect(),
            );
            return Json(ids).into_response();
        };

        let Some(device) = devices.get(&device) else {
            tracing::error!("Device {device} not found.");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
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
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        Json(device.lock().await.execute_actions(&actions).await).into_response()
    }
}
