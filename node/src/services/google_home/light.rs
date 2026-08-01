//! Structure representing a smart lights.

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

    fn execute_actions(&mut self, execution: &[Value]) -> Value {
        let mut fields = Map::new();

        tracing::info!("EXECUTION: {execution:#?}");

        fields.insert("status".to_string(), Value::String("ERROR".to_string()));
        fields.insert(
            "errorCode".to_string(),
            Value::String("transientError".to_string()),
        );

        let mut success = true;

        let mut on_set = false;
        let mut brightness_set = false;
        let mut color_set = false;

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
                Some("action.devices.commands.BrightnessAbsolute") => {
                    let Some(brightness) = params.get("brightness") else {
                        success = false;
                        break;
                    };

                    let Some(brightness) = brightness.as_u64() else {
                        success = false;
                        break;
                    };

                    self.brightness = brightness;
                    brightness_set = true;
                }
                Some("action.devices.commands.ColorAbsolute") => {
                    let Some(color) = params.get("color") else {
                        success = false;
                        break;
                    };
                    let Some(color) = color.as_object() else {
                        success = false;
                        break;
                    };
                    let Some(temperature) = color.get("temperature") else {
                        success = false;
                        break;
                    };

                    let Some(temperature) = temperature.as_u64() else {
                        success = false;
                        break;
                    };

                    self.temperature = temperature;
                    color_set = true;
                }
                _ => {}
            }
        }

        if success {
            fields.insert("status".to_string(), Value::String("SUCCESS".to_string()));
            fields.remove(&"errorCode".to_string());

            let mut state = Map::new();
            state.insert("online".to_string(), Value::Bool(true));

            if color_set {
                let mut color = Map::new();
                color.insert(
                    "temperatureK".to_string(),
                    Value::Number(Number::from(self.temperature)),
                );
                state.insert("color".to_string(), Value::Object(color));
            }

            if brightness_set {
                state.insert(
                    "brightness".to_string(),
                    Value::Number(Number::from(self.brightness)),
                );
            }

            if on_set {
                state.insert("on".to_string(), Value::Bool(self.on));
            }

            fields.insert("states".to_string(), Value::Object(state));
        }

        Value::Object(fields)
    }
}
