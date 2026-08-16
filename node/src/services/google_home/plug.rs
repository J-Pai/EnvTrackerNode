//! Structure representing a smart lights.

use std::time::Duration;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use http::StatusCode;
use http::header::CONTENT_LENGTH;
use http::header::CONTENT_TYPE;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde_json::Map;
use serde_json::Value;
use url::Url;

use crate::error::NodeError;
use crate::services::google_home::Device;
use crate::services::kasa::KasaDevice;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct GetDeviceInfo {
    #[serde(rename = "@xmlns:u")]
    namespace: Option<String>,
    #[serde(rename = "#text")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Payload")]
    payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DeviceInformation")]
    device_information: Option<String>,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct BinaryState {
    #[serde(rename = "@xmlns:u")]
    namespace: Option<String>,
    #[serde(rename = "BinaryState")]
    binary_state: u8,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "u:GetDeviceInformation")]
    get_device_info: Option<GetDeviceInfo>,
    #[serde(rename = "u:GetDeviceInformationResponse")]
    get_device_info_resp: Option<GetDeviceInfo>,
    #[serde(rename = "u:GetBinaryState")]
    get_binary_state: Option<BinaryState>,
    #[serde(rename = "u:GetBinaryStateResponse")]
    get_binary_state_resp: Option<BinaryState>,
    #[serde(rename = "u:SetBinaryState")]
    set_binary_state: Option<BinaryState>,
    #[serde(rename = "u:SetBinaryStateResponse")]
    set_binary_state_resp: Option<BinaryState>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "s:Envelope")]
struct DeviceInfo {
    #[serde(rename = "@s:encodingStyle")]
    encoding_style: Option<String>,
    #[serde(rename = "s:Body")]
    body: Body,
}

pub(crate) struct Wemo {
    id: String,
    name: String,
    on: bool,
    pub(super) uri: Option<Url>,
    start_unreachable: Option<DateTime<Utc>>,
    state_changed: bool,
}

impl Default for Wemo {
    fn default() -> Self {
        Self {
            id: "WEMO_BASE".to_string(),
            name: "WEMO_BASE".to_string(),
            on: false,
            uri: None,
            start_unreachable: None,
            state_changed: false,
        }
    }
}

impl Wemo {
    const RETRY: i32 = 3;

    pub(crate) async fn new(uri: Url, name: String) -> Result<Self, Box<dyn std::error::Error>> {
        Self::discover(uri, name).await
    }

    pub(crate) async fn query_state(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let prev_on = self.on;

        self.on = self.send_request("GetBinaryState", false).await?;

        if self.on == prev_on {
            return Ok(false);
        }

        Ok(true)
    }

    async fn execution(&mut self, state: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.on = self.send_request("SetBinaryState", state).await?;
        Ok(())
    }

    async fn send_request(
        &mut self,
        request_type: &str,
        state: bool,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let device_client = ClientBuilder::new(Client::new()).build();
        let mut count = Self::RETRY;

        loop {
            let uri = self
                .uri
                .as_ref()
                .unwrap()
                .join("/upnp/control/basicevent1")
                .unwrap();
            count -= 1;
            let config = serde_xml_rs::SerdeXml::new()
                .namespace("s", "http://schemas.xmlsoap.org/soap/envelope/");

            let body = if request_type == "SetBinaryState" {
                Body {
                    set_binary_state: Some(BinaryState {
                        namespace: Some("urn:Belkin:service:basicevent:1".to_string()),
                        binary_state: u8::from(state),
                    }),
                    ..Default::default()
                }
            } else {
                Body {
                    get_binary_state: Some(BinaryState {
                        namespace: Some("urn:Belkin:service:basicevent:1".to_string()),
                        binary_state: u8::from(state),
                    }),
                    ..Default::default()
                }
            };

            let request_body = config
                .clone()
                .to_string(&DeviceInfo {
                    encoding_style: None,
                    body,
                })
                .unwrap();

            let request = device_client.post(uri.clone());
            let request = request
                .header(
                    "SOAPACTION",
                    format!("\"urn:Belkin:service:basicevent:1#{request_type}\""),
                )
                .header(CONTENT_TYPE, "text/xml; charset=\"utf-8\"")
                .header(CONTENT_LENGTH, request_body.len())
                .body(request_body);

            let state = if let Ok(resp) = request
                .timeout(Duration::from_millis(200))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Received response from Node: {e}");
                    let port = uri.port().unwrap();
                    let mut uri = self.uri.clone().unwrap();
                    if port.is_multiple_of(2) {
                        if uri.set_port(Some(port - 1)).is_err() {
                            return;
                        }
                    } else {
                        if uri.set_port(Some(port + 1)).is_err() {
                            return;
                        }
                    }
                    self.uri.replace(uri);
                }) {
                let status = resp.status();

                if status != StatusCode::OK {
                    if count >= 0 {
                        continue;
                    }
                    return Err(NodeError::new(&format!("Error Code {status}")));
                }

                let xml = resp.text().await?;

                let device_info: DeviceInfo = config.from_str(xml.as_str())?;

                if let Some(state) = device_info.body.get_binary_state_resp {
                    state.binary_state == 1
                } else if let Some(state) = device_info.body.set_binary_state_resp {
                    state.binary_state == 1
                } else {
                    if count >= 0 {
                        continue;
                    }
                    return Err(NodeError::new(&format!("Malformed Response\n{xml}")));
                }
            } else {
                if count >= 0 {
                    continue;
                }
                return Err(NodeError::new("Request Sending Issue."));
            };

            return Ok(state);
        }
    }

    async fn discover(uri: Url, name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut uri = uri.join("/upnp/control/deviceinfo1").unwrap();
        let device_client = ClientBuilder::new(Client::new()).build();
        let mut count = Self::RETRY;
        loop {
            count -= 1;

            let config = serde_xml_rs::SerdeXml::new()
                .namespace("s", "http://schemas.xmlsoap.org/soap/envelope/");
            let request_body = config
                .clone()
                .to_string(&DeviceInfo {
                    encoding_style: Some("http://schemas.xmlsoap.org/soap/encoding/".to_string()),
                    body: Body {
                        get_device_info: Some(GetDeviceInfo {
                            namespace: Some("urn:Belkin:service:deviceinfo:1".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                })
                .unwrap();

            let request = device_client.post(uri.clone());
            let request = request
                .header(
                    "SOAPACTION",
                    "\"urn:Belkin:service:deviceinfo:1#GetDeviceInformation\"",
                )
                .header(CONTENT_TYPE, "text/xml; charset=\"utf-8\"")
                .header(CONTENT_LENGTH, request_body.len())
                .body(request_body);

            let id = if let Ok(resp) = request
                .timeout(Duration::from_millis(200))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Receive Sync Error from Node: {e}");
                    let port = uri.port().unwrap();
                    if port.is_multiple_of(2) {
                        let _ = uri.set_port(Some(port - 1));
                    } else {
                        let _ = uri.set_port(Some(port + 1));
                    }
                }) {
                let status = resp.status();

                if status != StatusCode::OK {
                    if count >= 0 {
                        continue;
                    }
                    return Err(NodeError::new(&format!("Error Code {status}")));
                }

                let xml = resp.text().await?;

                let device_info: DeviceInfo = config.from_str(xml.as_str())?;

                if let Some(resp) = device_info.body.get_device_info_resp
                    && let Some(id) = resp.payload
                {
                    let mut split = id.split("//");
                    split.next();
                    split.next().unwrap().to_string()
                } else {
                    if count >= 0 {
                        continue;
                    }
                    return Err(NodeError::new(&format!("Malformed Payload\n{xml}")));
                }
            } else {
                if count >= 0 {
                    continue;
                }
                return Err(NodeError::new("Request Sending Issue."));
            };

            return Ok(Self {
                id: format!("{name}_{id}"),
                name,
                uri: Some(uri),
                state_changed: true,
                ..Default::default()
            });
        }
    }
}

impl Device for Wemo {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn get_sync_value(&mut self) -> serde_json::Value {
        self.state_changed = true;
        if let Err(e) = self.query_state().await {
            let dt = self.start_unreachable.get_or_insert(Utc::now());
            tracing::warn!(
                "[{}] Device Unreachable [starting from: {}]: {e}",
                self.id,
                dt.with_timezone(&Local)
            );
            let mut fields = Map::new();
            fields.insert("status".to_string(), Value::String("ERROR".to_string()));
            fields.insert(
                "errorCode".to_string(),
                Value::String("deviceOffline".to_string()),
            );
            return Value::Object(fields);
        }
        self.start_unreachable = None;
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

    async fn get_query_value(&mut self) -> (bool, Value) {
        let changed = match self.query_state().await {
            Err(e) => {
                let prev_reachable = self.start_unreachable.is_none();
                let dt = self.start_unreachable.get_or_insert(Utc::now());
                tracing::warn!(
                    "[{}] Device Unreachable [starting from: {}]: {e}",
                    self.id,
                    dt.with_timezone(&Local)
                );
                let mut fields = Map::new();
                fields.insert("status".to_string(), Value::String("ERROR".to_string()));
                fields.insert(
                    "errorCode".to_string(),
                    Value::String("deviceOffline".to_string()),
                );
                return (prev_reachable, Value::Object(fields));
            }
            Ok(state) => state || self.start_unreachable.is_some() || self.state_changed,
        };
        self.start_unreachable = None;
        self.state_changed = false;
        let mut fields = Map::new();
        fields.insert("status".to_string(), Value::String("SUCCESS".to_string()));
        fields.insert("online".to_string(), Value::Bool(true));
        fields.insert("on".to_string(), Value::Bool(self.on));
        (changed, serde_json::Value::Object(fields))
    }

    async fn execute_actions(&mut self, execution: &[Value]) -> Value {
        self.state_changed = true;
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
            if let Some("action.devices.commands.OnOff") = command.as_str() {
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
                let dt = self.start_unreachable.get_or_insert(Utc::now());
                tracing::warn!(
                    "[{}] Device Unreachable [starting from: {}]: {e}",
                    self.id,
                    dt.with_timezone(&Local)
                );
                let mut fields = Map::new();
                fields.insert("status".to_string(), Value::String("ERROR".to_string()));
                fields.insert(
                    "errorCode".to_string(),
                    Value::String("deviceOffline".to_string()),
                );
                return Value::Object(fields);
            }

            self.start_unreachable = None;

            fields.insert("states".to_string(), Value::Object(state));
        }

        Value::Object(fields)
    }
}

pub(crate) struct KasaDeviceId {
    id: String,
    kasa_device: KasaDevice,
}

impl KasaDeviceId {
    pub(crate) fn new(
        id: String,
        kasa_device: KasaDevice,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { id, kasa_device })
    }
}

impl Device for KasaDeviceId {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn get_sync_value(&mut self) -> serde_json::Value {
        Value::Null
    }

    async fn get_query_value(&mut self) -> (bool, Value) {
        (false, Value::Null)
    }

    async fn execute_actions(&mut self, _execution: &[Value]) -> Value {
        Value::Null
    }
}
