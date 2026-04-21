/// Parses and handles the system configuration from a config.toml file.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process::exit;
use std::str::FromStr;

use chrono::Local;
use config::Config;
use cron::Schedule;
use rand::RngExt;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct KasaDeviceConfig {
    pub(crate) ip: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) polling_schedule: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) jwt_secret: Option<String>,
    pub(crate) authorized_api_keys: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Server {
    pub(crate) node_ip: String,
    pub(crate) db: String,
    pub(crate) frontend: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) kasa: HashMap<String, KasaDeviceConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Web {
    server: Option<Server>,
    node: Option<Node>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct SysConfig {
    pub(crate) settings: Settings,
    pub(crate) web: Web,
}

macro_rules! tagged_fmt {
    ($e: expr) => {
        &format!("[SysConfig] {}", $e).to_string()
    };
}

macro_rules! general_println {
    ($e: expr) => {
        println!("[general] {}", $e)
    };
}

macro_rules! server_println {
    ($e: expr) => {
        println!("[server] {}", $e)
    };
}

macro_rules! node_println {
    ($e: expr) => {
        println!("[node] {}", $e)
    };
}

impl SysConfig {
    pub(crate) fn new() -> Self {
        let home_dir = env::home_dir().expect(tagged_fmt!("HOME dir not specified."));
        let config_dir = home_dir.join(".config/envtrackernode");
        let config_file = config_dir.join("config.toml");
        let settings = match Config::builder()
            .add_source(config::File::with_name(
                config_file
                    .to_str()
                    .expect(tagged_fmt!("Malformed path for config.toml.")),
            ))
            .build()
        {
            Ok(built_config) => built_config.try_deserialize::<SysConfig>().unwrap(),
            Err(e) => {
                general_println!(format!("Error obtaining config file: {:?}", e));
                general_println!("Create configuration file? ([Y]es / [n]o)");
                let mut response = String::new();
                io::stdin()
                    .read_line(&mut response)
                    .expect(tagged_fmt!("Unknown response provided."));
                if response.to_uppercase().trim() == "Y" {
                    println!("Creating {:?}", config_file);
                    fs::create_dir_all(config_file.parent().expect(tagged_fmt!(""))).unwrap();
                    let config = Self::config_generator();
                    let text = toml::to_string(&config).unwrap();
                    fs::write(&config_file, text).unwrap();
                    println!("Created {:?}. Relaunch the server.", config_file);
                    exit(0);
                } else if response.to_uppercase().trim() != "N" {
                    println!("Please specify either [Y]es or [N]o.");
                }
                exit(0);
            }
        };
        settings
    }

    pub(crate) fn get_node_config(&self) -> Option<Node> {
        if let Some(node) = &self.web.node {
            Some(node.clone())
        } else {
            None
        }
    }

    pub(crate) fn get_server_config(&self) -> Option<Server> {
        if let Some(server) = &self.web.server {
            Some(server.clone())
        } else {
            None
        }
    }

    fn config_generator() -> Self {
        let config = SysConfig::default();
        config.configure_servers().configure_settings()
    }

    pub fn configure_servers(mut self) -> Self {
        general_println!("List out deseired services, separated by commas.");
        general_println!("Supported services:");
        general_println!("- server");
        general_println!("  - Serves the frontend application.");
        general_println!("  - Queries the node (or nodes) for data.");
        general_println!("  - Provides an endpoint for the frontend application to request data.");
        general_println!("  - Saves/Manages data in database.");
        general_println!("- node");
        general_println!("  - Polls a device for data.");
        general_println!("  - Maintains a queue of data.");
        general_println!("  - Provides an endpoint for server to request data.");
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        response = response.trim().to_string();

        if response.is_empty() {
            println!("No services provided.");
            exit(0);
        }

        for service in response.split(",") {
            match service.trim() {
                "server" => {
                    if self.web.server.is_some() {
                        continue;
                    }
                    server_println!("Configuring");
                    self = self.configure_server();
                    server_println!("Done");
                }
                "node" => {
                    if self.web.node.is_some() {
                        continue;
                    }
                    node_println!("Configuring");
                    self = self.configure_node();
                    node_println!("Done");
                }
                s => println!("Unknown service: [{}]", s),
            }
        }

        self
    }

    fn handle_response() -> Option<String> {
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        response = response.trim().to_string();
        if response.is_empty() {
            return None;
        }
        return Some(response);
    }

    fn configure_settings(mut self) -> Self {
        general_println!("Provide jwt_secret (leave empty to generate a random value): ");
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        response = response.trim().to_string();
        if response.is_empty() {
            let random_jwt: [u8; 64] = rand::rng().random();
            let hex = hex::encode(&random_jwt);
            general_println!(format!("Generated jwt_secret: {}", hex));
            self.settings.jwt_secret = Some(hex);
        } else {
            self.settings.jwt_secret = Some(response.clone());
        }
        general_println!(
            "Generate authorized API key (provide the name of the authorized application, leave empty when done):"
        );
        loop {
            io::stdin().read_line(&mut response).unwrap();
            response = response.trim().to_string();
            if response.is_empty() {
                break;
            }
            let random_api_key: [u8; 32] = rand::rng().random();
            let hex = hex::encode(&random_api_key);
            println!("Generated api_key for [{}]: {}", response.trim(), hex);
            self.settings.authorized_api_keys = match self.settings.authorized_api_keys {
                None => Some(HashMap::from([(response.to_string(), hex)])),
                Some(mut api_keys) => {
                    api_keys.insert(response.to_string(), hex);
                    Some(api_keys)
                }
            };
            response.clear();
        }
        self
    }

    fn configure_server(mut self) -> Self {
        self.web.server = Some(Server {
            node_ip: {
                server_println!("Provide node ip address (default - 0.0.0.0:3000): ");
                if let Some(resp) = Self::handle_response() {
                    resp
                } else {
                    "0.0.0.0:3000".to_string()
                }
            },
            db: {
                server_println!("Provide the path to the db file. (default - sqlite.db): ");
                if let Some(resp) = Self::handle_response() {
                    resp
                } else {
                    "sqlite.db".to_string()
                }
            },
            frontend: {
                server_println!("Serve frontend ([Y]es / [N]o, default - [Y]es)?: ");
                if let Some(resp) = Self::handle_response() {
                    if resp.to_uppercase().trim() == "N" {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            },
        });
        self
    }

    fn configure_node(mut self) -> Self {
        self.web.node = Some(Node {
            kasa: Self::configure_kasa(),
        });
        self
    }

    fn configure_kasa() -> HashMap<String, KasaDeviceConfig> {
        let mut kasa_device_map: HashMap<String, KasaDeviceConfig> = HashMap::new();

        loop {
            let mut device = KasaDeviceConfig::default();

            node_println!("Provide kasa device name (leave empty to skip): ");
            let device_name = if let Some(resp) = Self::handle_response() {
                resp
            } else {
                break;
            };

            node_println!("Provide kasa device ip address: ");
            device.ip = if let Some(resp) = Self::handle_response() {
                resp
            } else {
                break;
            };

            node_println!("Provide kasa device username: ");
            device.username = if let Some(resp) = Self::handle_response() {
                resp
            } else {
                break;
            };

            node_println!("Provide kasa device password: ");
            device.password = if let Some(resp) = Self::handle_response() {
                resp
            } else {
                break;
            };

            node_println!(
                "Provide kasa device polling schedule (CRON-like, leave empty for poll every 2 second): "
            );
            device.polling_schedule = if let Some(resp) = Self::handle_response() {
                let schedule = Schedule::from_str(resp.as_str()).unwrap();
                println!("Upcoming fire times:");
                for datetime in schedule.upcoming(Local).take(10) {
                    println!("-> {}", datetime)
                }
                resp
            } else {
                "1/1 * * * * *".to_string()
            };

            node_println!("Provide kasa device description: ");
            device.description = Self::handle_response();
            kasa_device_map.insert(device_name, device);
        }

        kasa_device_map
    }
}
