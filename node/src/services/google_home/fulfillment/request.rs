//! Parser for Fulfillment requests.

use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Request {
    #[serde(rename = "requestId")]
    request_id: String,
    inputs: Vec<Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Intent {
    Sync,
    Query(Vec<Value>),
    Execute,
    Disconnect,
}

impl Request {
    pub(crate) fn get_request_id(&self) -> String {
        self.request_id.clone()
    }

    pub(crate) fn get_inputs(&self) -> &Vec<Value> {
        &self.inputs
    }

    pub(crate) fn parse_intent(intent: &Value) -> Option<Intent> {
        let Some(intent) = intent.as_object() else {
            return None;
        };

        let Some(intent_type) = intent.get("intent") else {
            return None;
        };

        match intent_type.as_str() {
            Some("action.devices.SYNC") => Some(Intent::Sync),
            Some("action.devices.QUERY") => {
                let Some(payload) = intent.get("payload") else {
                    return None;
                };
                let Some(devices) = payload.as_object() else {
                    return None;
                };
                let Some(devices) = devices.get("devices") else {
                    return None;
                };
                let Some(devices) = devices.as_array() else {
                    return None;
                };
                Some(Intent::Query(devices.clone()))
            }
            Some("action.devices.EXECUTE") => Some(Intent::Execute),
            Some("action.devices.DISCONNECT") => Some(Intent::Disconnect),
            _ => None,
        }
    }
}
