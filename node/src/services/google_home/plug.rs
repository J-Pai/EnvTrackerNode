//! Structure representing a smart lights.

use http::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::Map;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::services::google_home::Device;

pub(crate) struct Wemo {
    id: String,
    name: String,
    device_id: String,
    on: bool,
    uri: Option<Url>,
}

impl Default for Wemo {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            id: format!("WEMO_BASE_{uuid}"),
            name: "WEMO_BASE".to_string(),
            device_id: uuid.to_string(),
            on: false,
            uri: None,
        }
    }
}

impl Wemo {
    pub(crate) async fn new(uri: Url) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            uri: Some(uri),
            ..Default::default()
        })
    }

    pub(crate) async fn query_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn execution(&mut self, state: bool) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn send_request(&self, state: &bool) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Device for Wemo {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn get_sync_value(&mut self) -> serde_json::Value {
        let _ = self.query_state().await;
        let mut fields = Map::new();

        fields.insert("id".to_string(), Value::String(self.id.clone()));
        fields.insert(
            "type".to_string(),
            Value::String("action.devices.types.OUTLET".to_string()),
        );
        fields.insert(
            "traits".to_string(),
            Value::Array(vec![Value::String(
                "action.devices.traits.OnOff".to_string(),
            )]),
        );
        fields.insert(
            "traits".to_string(),
            Value::Array(vec![Value::String(
                "action.devices.traits.OnOff".to_string(),
            )]),
        );
        let mut name = Map::new();
        name.insert("name".to_string(), Value::String(self.name.clone()));
        fields.insert("name".to_string(), Value::Object(name));
        fields.insert("willReportState".to_string(), Value::Bool(true));
        let attributes = Map::new();
        fields.insert("attributes".to_string(), Value::Object(attributes));
        let mut device_info = Map::new();
        device_info.insert(
            "manufacturer".to_string(),
            Value::String("envtrackernode".to_string()),
        );
        device_info.insert("model".to_string(), Value::String("wemo".to_string()));
        device_info.insert("hwVersion".to_string(), Value::String("1.0".to_string()));
        device_info.insert("swVersion".to_string(), Value::String("1.0".to_string()));
        fields.insert("deviceInfo".to_string(), Value::Object(device_info));

        serde_json::Value::Object(fields)
    }

    async fn get_query_value(&mut self) -> Value {
        let _ = self.query_state().await;
        let mut fields = Map::new();
        fields.insert("status".to_string(), Value::String("SUCCESS".to_string()));
        fields.insert("online".to_string(), Value::Bool(true));
        fields.insert("on".to_string(), Value::Bool(self.on));
        serde_json::Value::Object(fields)
    }

    async fn execute_actions(&mut self, execution: &[Value]) -> Value {
        let mut fields = Map::new();

        fields.insert("status".to_string(), Value::String("ERROR".to_string()));
        fields.insert(
            "errorCode".to_string(),
            Value::String("transientError".to_string()),
        );

        let mut success = true;

        let mut on_set = false;

        for action in execution {
            let Some(action) = action.as_object() else {
                success = false;
                break;
            };
            let Some(command) = action.get("command") else {
                success = false;
                break;
            };
            let Some(params) = action.get("params") else {
                success = false;
                break;
            };
            let Some(params) = params.as_object() else {
                success = false;
                break;
            };
            match command.as_str() {
                Some("action.devices.commands.OnOff") => {
                    let Some(on) = params.get("on") else {
                        success = false;
                        break;
                    };

                    let Some(on) = on.as_bool() else {
                        success = false;
                        break;
                    };

                    self.on = on;
                    on_set = true;
                }
                _ => {}
            }
        }

        if success {
            fields.insert("status".to_string(), Value::String("SUCCESS".to_string()));
            fields.remove(&"errorCode".to_string());

            let mut state = Map::new();
            state.insert("online".to_string(), Value::Bool(true));

            let mut command = false;

            if on_set {
                state.insert("on".to_string(), Value::Bool(self.on));
                command = self.on;
            }

            if let Err(e) = self.execution(command).await {
                tracing::error!("UPDATE Issue: {command:#?} => {e}");
            }

            fields.insert("states".to_string(), Value::Object(state));
        }

        Value::Object(fields)
    }
}
