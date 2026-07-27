//! Parser for Fulfillment requests.

use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Request {
    #[serde(rename = "requestId")]
    request_id: String,
    inputs: Vec<Value>,
}

pub(crate) struct Command {
    devices: Vec<Value>,
    execution: Vec<Value>,
}

pub(crate) enum Intent {
    Sync,
    Query(Vec<Value>),
    Execute(Vec<Command>),
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
            Some("action.devices.EXECUTE") => {
                let Some(payload) = intent.get("payload") else {
                    return None;
                };
                let Some(commands) = payload.as_object() else {
                    return None;
                };
                let Some(commands) = commands.get("commands") else {
                    return None;
                };
                let Some(commands) = commands.as_array() else {
                    return None;
                };

                let mut parsed_commands: Vec<Command> = vec![];

                for command in commands {
                    let Some(devices) = command.get("devices") else {
                        continue;
                    };
                    let Some(devices) = devices.as_array() else {
                        continue;
                    };
                    let Some(execution) = command.get("execution") else {
                        continue;
                    };
                    let Some(execution) = execution.as_array() else {
                        continue;
                    };
                    parsed_commands.push(Command {
                        devices: devices.clone(),
                        execution: execution.clone(),
                    });
                }

                Some(Intent::Execute(parsed_commands))
            }
            Some("action.devices.DISCONNECT") => Some(Intent::Disconnect),
            _ => None,
        }
    }
}
