//! Structure representing a smart lights.

use std::time::Duration;

use serde_json::Map;
use serde_json::Number;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use url::Url;
use uuid::Uuid;

use crate::error::NodeError;
use crate::services::google_home::Device;

pub(crate) struct DLight {
    id: String,
    name: String,
    device_id: String,
    /// Percent: 0 - 100
    brightness: u64,
    /// Kelvin: 2600 - 6000
    temperature: u64,
    on: bool,
    uri: Option<Url>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum DLightCommand {
    #[serde(rename = "EXECUTE")]
    Execute,
    #[serde(rename = "QUERY_DEVICE_STATES")]
    QueryDeviceStates,
    #[serde(rename = "QUERY_DEVICE_INFO")]
    QueryDeviceInfo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DLightColor {
    temperature: u64,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DLightState {
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brightness: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<DLightColor>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DLightRequest {
    #[serde(rename = "commandId")]
    command_id: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "commandType")]
    command_type: DLightCommand,
    #[serde(skip_serializing_if = "Option::is_none")]
    commands: Option<Vec<DLightState>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DLightResponse {
    #[serde(rename = "commandId")]
    command_id: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    status: String,
    states: Option<DLightState>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DLightDiscovery {
    #[serde(rename = "deviceModel")]
    device_model: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "swVersion")]
    sw_version: String,
    #[serde(rename = "hwVersion")]
    hw_version: String,
}

impl Default for DLight {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            id: format!("GLAMP_BASE_{uuid}"),
            name: "GLAMP_BASE".to_string(),
            device_id: uuid.to_string(),
            brightness: 0,
            temperature: 0,
            on: false,
            uri: None,
        }
    }
}

impl DLight {
    pub(crate) async fn new(uri: Url) -> Result<Self, Box<dyn std::error::Error>> {
        Self::discover(uri).await
    }

    pub(crate) async fn query_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let uuid = Uuid::new_v4();
        let resp = self
            .send_request(&DLightRequest {
                command_id: uuid.to_string(),
                device_id: self.device_id.clone(),
                command_type: DLightCommand::QueryDeviceStates,
                commands: None,
            })
            .await?;
        if resp.status != "SUCCESS"
            || resp.command_id != uuid.to_string()
            || resp.device_id != self.device_id
        {
            return Err(NodeError::new(
                format!("DLight Response Error: {resp:#?}").as_str(),
            ));
        }
        self.on = resp.states.clone().unwrap().on.unwrap();
        self.brightness = resp.states.clone().unwrap().brightness.unwrap();
        self.temperature = resp.states.clone().unwrap().color.unwrap().temperature;
        Ok(())
    }

    async fn execution(&mut self, state: &DLightState) -> Result<(), Box<dyn std::error::Error>> {
        let uuid = Uuid::new_v4();
        let resp = self
            .send_request(&DLightRequest {
                command_id: uuid.to_string(),
                device_id: self.device_id.clone(),
                command_type: DLightCommand::Execute,
                commands: Some(vec![state.clone()]),
            })
            .await?;
        if resp.status != "SUCCESS"
            || resp.command_id != uuid.to_string()
            || resp.device_id != self.device_id
        {
            return Err(NodeError::new(
                format!("DLight Response Error: {resp:#?}").as_str(),
            ));
        }
        self.query_state().await?;
        Ok(())
    }

    async fn send_request(
        &self,
        request: &DLightRequest,
    ) -> Result<DLightResponse, Box<dyn std::error::Error>> {
        let host = self.uri.as_ref().unwrap().host().unwrap();
        let port = self.uri.as_ref().unwrap().port().unwrap();

        let mut tcp_stream = TcpStream::connect((host.to_string(), port)).await?;
        let src = serde_json::to_string(request)?;

        tcp_stream.write_all(src.as_bytes()).await?;
        tcp_stream.write_all(b"\n").await?;

        timeout(Duration::from_secs(5), async {
            tcp_stream.readable().await?;
            let mut header = [0; 4];
            tcp_stream.try_read(&mut header)?;
            let size = u32::from_be_bytes(header) as usize;
            let mut data = [0; 1024];
            let _ = tcp_stream.try_read(&mut data)?;
            let response: DLightResponse = serde_json::from_slice(&data[..size])?;
            Ok(response)
        })
        .await
        .map_err(|e| NodeError::new(e.to_string().as_str()))?
    }

    async fn discover(uri: Url) -> Result<Self, Box<dyn std::error::Error>> {
        const COMMAND_PAYLOAD: &[u8; 40] = b"476f6f676c654e50455f457269635f5761796e65";
        let host = uri.host().unwrap();
        let udp = UdpSocket::bind(("0.0.0.0", 9487)).await?;
        udp.set_broadcast(true)?;
        let json: Result<DLightDiscovery, Box<dyn std::error::Error>> = {
            timeout(Duration::from_secs(5), async {
                udp.send_to(COMMAND_PAYLOAD, (host.to_string(), 9478))
                    .await?;
                let mut buf = [0u8; 256];
                let (size, _) = udp.recv_from(&mut buf).await?;
                let resp: DLightDiscovery = serde_json::from_slice(&buf[..size])?;
                Ok(resp)
            })
            .await
            .map_err(|e| NodeError::new(e.to_string().as_str()))?
        };

        let json = json?;

        let mut new = Self {
            id: format!("{}_{}", json.device_model, json.device_id),
            name: json.device_model,
            device_id: json.device_id,
            brightness: 0,
            temperature: 0,
            on: false,
            uri: Some(uri),
        };

        new.query_state().await?;

        Ok(new)
    }
}

impl Device for DLight {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn get_sync_value(&mut self) -> serde_json::Value {
        let _ = self.query_state().await;
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
        fields.insert("willReportState".to_string(), Value::Bool(true));
        let mut attributes = Map::new();
        let mut color_temperature_range = Map::new();
        color_temperature_range.insert(
            "temperatureMinK".to_string(),
            Value::Number(Number::from(2600)),
        );
        color_temperature_range.insert(
            "temperatureMaxK".to_string(),
            Value::Number(Number::from(6000)),
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
        device_info.insert("model".to_string(), Value::String("dlight".to_string()));
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

    async fn execute_actions(&mut self, execution: &[Value]) -> Value {
        let mut fields = Map::new();

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

            let mut command = DLightState::default();

            if color_set {
                let mut color = Map::new();
                color.insert(
                    "temperatureK".to_string(),
                    Value::Number(Number::from(self.temperature)),
                );
                state.insert("color".to_string(), Value::Object(color));
                command.color = Some(DLightColor {
                    temperature: self.temperature,
                });
            }

            if brightness_set {
                state.insert(
                    "brightness".to_string(),
                    Value::Number(Number::from(self.brightness)),
                );
                command.brightness = Some(self.brightness);
            }

            if on_set {
                state.insert("on".to_string(), Value::Bool(self.on));
                command.on = Some(self.on);
            }

            if let Err(e) = self.execution(&command).await {
                tracing::error!("UPDATE Issue: {command:#?} => {e}");
            }

            fields.insert("states".to_string(), Value::Object(state));
        }

        Value::Object(fields)
    }
}
