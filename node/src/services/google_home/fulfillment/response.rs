//! Generator for Fulfillment responses.

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
    pub(crate) fn new(request_id: String, agent_user_id: String) -> Self {
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

    pub(crate) fn add_device(mut self, id: String, value: Value) -> Self {
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
            Some(Intent::Query(_)) => {
                if self.payload.is_null() {
                    let mut payload = Map::new();
                    let devices = Map::new();
                    payload.insert("devices".to_string(), Value::Object(devices));
                    self.payload = Value::Object(payload);
                }

                let devices = self
                    .payload
                    .as_object_mut()
                    .unwrap()
                    .get_mut("devices")
                    .unwrap()
                    .as_object_mut()
                    .unwrap();

                if value.is_null() {
                    let mut error = Map::new();
                    error.insert(
                        "errorCode".to_string(),
                        Value::String("deviceOffline".to_string()),
                    );
                    error.insert("status".to_string(), Value::String("ERROR".to_string()));
                    devices.insert(id, Value::Object(error));
                } else {
                    devices.insert(id, value);
                }
            }
            _ => {}
        }
        self
    }

    pub(crate) fn add_command_status(
        mut self,
        ids: Vec<String>,
        status: String,
        states: Value,
        error_code: String,
    ) -> Self {
        match self.handling_intent {
            Some(Intent::Execute(_)) => {}
            _ => {}
        }
        self
    }

    pub(crate) fn error_payload(mut self, error_code: String) -> Self {
        let mut error = Map::new();
        error.insert(
            "errorCode".to_string(),
            Value::String(if error_code.is_empty() {
                "notSupported".to_string()
            } else {
                error_code
            }),
        );
        error.insert("status".to_string(), Value::String("ERROR".to_string()));
        self.payload = Value::Object(error);
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
