/// Parses and handles the system configuration from a config.toml file.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process::exit;

use config::Config;
use rand::RngExt;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct KasaDevice {
    pub(crate) ip: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) jwt_secret: Option<String>,
    pub(crate) authorized_api_keys: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct SysConfig {
    pub(crate) settings: Settings,
    pub(crate) kasa: Option<HashMap<String, KasaDevice>>,
}

macro_rules! tagged_fmt {
    ($e: expr) => {
        &format!("[SysConfig] {}", $e).to_string()
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
                println!("Error obtaining config file: {:?}", e);
                println!("Create configuration file? ([Y]es / [n]o)");
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

    pub(crate) fn get_kasa_devices(&self) -> Option<HashMap<String, KasaDevice>> {
        self.kasa.clone()
    }

    fn config_generator() -> Self {
        let config = SysConfig::default();
        config
            .update_settings()
            .update_kasa()
    }

    fn update_settings(mut self) -> Self {
        println!("Provide jwt_secret (leave empty to generate a random value): ");
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        response = response.trim().to_string();
        if response.is_empty() {
            let random_jwt: [u8; 64] = rand::rng().random();
            let hex = hex::encode(&random_jwt);
            println!("Generated jwt_secret: {}", hex);
            self.settings.jwt_secret = Some(hex);
        } else {
            self.settings.jwt_secret = Some(response.clone());
        }
        println!("Generate authorized API key (provide the name of the authorized application, leave empty when done):");
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
                None => {
                    Some(HashMap::from([(response.to_string(), hex)]))
                }
                Some(mut api_keys) => {
                    api_keys.insert(response.to_string(), hex);
                    Some(api_keys)
                }
            };
            response.clear();
        }
        self
    }

    fn update_kasa(mut self) -> Self {
        loop {
            fn handle_response() -> Option<String> {
                let mut response = String::new();
                io::stdin().read_line(&mut response).unwrap();
                response = response.trim().to_string();
                if response.is_empty() {
                    return None;
                }
                return Some(response);
            }

            let mut device = KasaDevice::default();

            println!("Provide kasa device name (leave empty to skip): ");
            let device_name = if let Some(resp) = handle_response() {
                resp
            } else {
                break;
            };

            println!("Provide kasa device ip address: ");
            device.ip = if let Some(resp) = handle_response() {
                resp
            } else {
                break;
            };

            println!("Provide kasa device username: ");
            device.username = if let Some(resp) = handle_response() {
                resp
            } else {
                break;
            };

            println!("Provide kasa device password: ");
            device.password = if let Some(resp) = handle_response() {
                resp
            } else {
                break;
            };

            println!("Provide kasa device description: ");
            device.description = handle_response();

            self.kasa = match self.kasa {
                None => {
                    Some(HashMap::from([(device_name, device)]))
                }
                Some(mut kasa_device_map) => {
                    kasa_device_map.insert(device_name, device);
                    Some(kasa_device_map)
                }
            }
        }
        self
    }
}
