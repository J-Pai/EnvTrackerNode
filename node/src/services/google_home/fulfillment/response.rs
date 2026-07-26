//! Generator for Fulfillment responses.
//!
//! let response = FulfillmentResponse {
//!     request_id,
//!     payload: Payload {
//!         agent_user_id: Some(sub.clone()),
//!         devices: Some(vec![Device {
//!             id: format!("{}_dlight", sub.clone()),
//!             device_type: "action.devices.types.LIGHT".to_string(),
//!             traits: [
//!                 "action.devices.traits.ColorSetting".to_string(),
//!                 "action.devices.traits.Brightness".to_string(),
//!                 "action.devices.traits.OnOff".to_string(),
//!             ]
//!             .to_vec(),
//!             name: Name {
//!                 name: "glamp".to_string(),
//!             },
//!             will_report_state: false,
//!             attributes: Attributes {
//!                 color_temperature_range: Some(LightColorTemperatureRange {
//!                     temperature_min_k: 2600,
//!                     temperature_max_k: 6000,
//!                 }),
//!             },
//!             device_info: DeviceInfo {
//!                 manufacturer: "Me".to_string(),
//!                 model: "dLight".to_string(),
//!                 hw_version: "1".to_string(),
//!                 sw_version: "1".to_string(),
//!             },
//!         }]),
//!     },
//! };

use axum::Json;
use axum::response::IntoResponse;
use serde_json::Map;
use serde_json::Value;

use crate::{error::NodeError, services::google_home::fulfillment::request::Intent};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Response {
    #[serde(rename = "requestId")]
    request_id: String,
    payload: Value,
    #[serde(skip_serializing)]
    agent_user_id: String,
    #[serde(skip_serializing)]
    handling_intent: Option<Intent>,
}

impl Response {
    pub(crate) fn new(handling_intent: Intent, request_id: String, agent_user_id: String) -> Self {
        Response {
            request_id,
            payload: Value::Null,
            agent_user_id,
            handling_intent: None,
        }
    }

    pub(crate) fn set_intent(mut self, intent: Intent) -> Self {
        if self.handling_intent.is_some() {
            return self;
        }
        self.handling_intent = Some(intent);
        self
    }

    pub(crate) fn add_device(mut self, value: Value) -> Self {
        match self.handling_intent {
            Some(Intent::Sync) => {
                if self.payload.is_null() {
                    let mut payload = Map::new();
                    payload.insert(
                        "agentUserId".to_string(),
                        Value::String(self.agent_user_id.clone()),
                    );
                    let devices: Vec<Value> = vec![];
                    payload.insert(
                        "agentUserId".to_string(),
                        Value::String(self.agent_user_id.clone()),
                    );
                    payload.insert("devices".to_string(), Value::Array(devices));
                    self.payload = Value::Object(payload);
                }

                let devices = self
                    .payload
                    .as_object_mut()
                    .unwrap()
                    .get_mut("devices")
                    .unwrap()
                    .as_array_mut()
                    .unwrap();

                devices.push(value);
            }
            _ => {}
        }
        self
    }

    pub(crate) fn build(
        self,
    ) -> Result<axum::http::Response<axum::body::Body>, Box<dyn std::error::Error>> {
        if self.payload.is_null() {
            return Err(NodeError::new("Incomplete Payload"));
        }

        Ok(Json(self).into_response())
    }
}
