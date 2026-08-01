//! Parser for Fulfillment requests.

use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Request {
    #[serde(rename = "requestId")]
    request_id: String,
    inputs: Vec<Value>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct Command {
    devices: Vec<Value>,
    execution: Vec<Value>,
}

impl Command {
    pub(super) fn get_devices(&self) -> Vec<String> {
        self.devices
            .iter()
            .map(|d| {
                let Some(id) = d.as_object() else {
                    return String::new();
                };
                let Some(id) = id.get("id") else {
                    return String::new();
                };
                let Some(id) = id.as_str() else {
                    return String::new();
                };
                id.to_string()
            })
            .collect()
    }

    pub(super) fn get_execution(&self) -> &Vec<Value> {
        &self.execution
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
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
        let intent = intent.as_object()?;

        let intent_type = intent.get("intent")?;

        match intent_type.as_str() {
            Some("action.devices.SYNC") => Some(Intent::Sync),
            Some("action.devices.QUERY") => {
                let payload = intent.get("payload")?;
                let devices = payload.as_object()?;
                let devices = devices.get("devices")?;
                let devices = devices.as_array()?;
                Some(Intent::Query(devices.clone()))
            }
            Some("action.devices.EXECUTE") => {
                let payload = intent.get("payload")?;
                let commands = payload.as_object()?;
                let commands = commands.get("commands")?;
                let commands = commands.as_array()?;

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
