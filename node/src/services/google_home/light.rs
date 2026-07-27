//! Structure representing a smart lights.
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

use serde_json::Map;
use serde_json::Number;
use serde_json::Value;

use crate::services::google_home::Device;

pub(crate) struct DLight {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) brightness: u64,
    pub(crate) temperature: u64,
    pub(crate) on: bool,
}

impl Device for DLight {
    fn get_sync_value(&self) -> serde_json::Value {
        let mut fields = Map::new();

        fields.insert("id".to_string(), Value::String(self.id.clone()));
        fields.insert(
            "type".to_string(),
            Value::String("action.devices.types.LIGHT".to_string()),
        );
        fields.insert(
            "traits".to_string(),
            Value::Array(vec![
                Value::String("action.devices.traits.ColorSetting".to_string()),
                Value::String("action.devices.traits.Brightness".to_string()),
                Value::String("action.devices.traits.OnOff".to_string()),
            ]),
        );
        fields.insert(
            "traits".to_string(),
            Value::Array(vec![
                Value::String("action.devices.traits.ColorSetting".to_string()),
                Value::String("action.devices.traits.Brightness".to_string()),
                Value::String("action.devices.traits.OnOff".to_string()),
            ]),
        );
        let mut name = Map::new();
        name.insert("name".to_string(), Value::String(self.name.clone()));
        fields.insert("name".to_string(), Value::Object(name));
        fields.insert("willReportState".to_string(), Value::Bool(false));
        let mut attributes = Map::new();
        let mut color_temperature_range = Map::new();
        color_temperature_range.insert(
            "temperatureMinK".to_string(),
            Value::Number(Number::from(2000)),
        );
        color_temperature_range.insert(
            "temperatureMaxK".to_string(),
            Value::Number(Number::from(6500)),
        );
        attributes.insert(
            "colorTemperatureRange".to_string(),
            Value::Object(color_temperature_range),
        );
        fields.insert("attributes".to_string(), Value::Object(attributes));
        let mut device_info = Map::new();
        device_info.insert(
            "manufacturer".to_string(),
            Value::String("envtrackernode".to_string()),
        );
        device_info.insert("model".to_string(), Value::String("glamp".to_string()));
        device_info.insert("hwVersion".to_string(), Value::String("1.0".to_string()));
        device_info.insert("swVersion".to_string(), Value::String("1.0".to_string()));
        fields.insert("deviceInfo".to_string(), Value::Object(device_info));

        serde_json::Value::Object(fields)
    }

    fn get_query_value(&self) -> Value {
        let mut fields = Map::new();
        fields.insert("status".to_string(), Value::String("SUCCESS".to_string()));
        fields.insert("online".to_string(), Value::Bool(true));
        fields.insert("on".to_string(), Value::Bool(self.on));
        fields.insert(
            "brightness".to_string(),
            Value::Number(Number::from(self.brightness)),
        );
        let mut color = Map::new();
        color.insert(
            "temperatureK".to_string(),
            Value::Number(Number::from(self.temperature)),
        );
        fields.insert("color".to_string(), Value::Object(color));
        serde_json::Value::Object(fields)
    }

    fn execute_actions(&mut self, execution: Vec<Value>) -> Value {

    }
}
