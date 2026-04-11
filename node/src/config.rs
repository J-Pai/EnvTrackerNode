use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process::exit;

use config::Config;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct KasaDevice {
    pub(crate) ip: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) jwt_secret: String,
    pub(crate) authorized_api_keys: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SysConfig {
    pub(crate) settings: Settings,
    pub(crate) kasa: Option<HashMap<String, KasaDevice>>,
}

impl SysConfig {
    pub(crate) fn new() -> Self {
        let home_dir = env::home_dir().expect("HOME dir not specified.");
        let config_dir = format!("{}/.config/envtrackernode", home_dir.to_str().unwrap());
        let config_file = format!("{}/config.toml", config_dir);
        let settings = match Config::builder()
            .add_source(config::File::with_name(&config_file))
            .build()
        {
            Ok(built_config) => built_config,
            Err(e) => {
                println!("Error obtaining config file: {:?}", e);
                println!("Create configuration file? ([Y]es / [n]o)");
                let mut response = String::new();
                io::stdin().read_line(&mut response).unwrap();
                if response.to_uppercase().trim() == "Y" {
                    println!("Creating {}", config_file);
                    fs::create_dir_all(config_dir).unwrap();

                    let config = SysConfig {
                        settings: Settings {
                            jwt_secret: "placeholder".to_string(),
                            authorized_api_keys: Some(vec![
                                "placeholder".to_string(),
                                "placeholder".to_string(),
                            ]),
                        },
                        kasa: Some(HashMap::from([(
                            "smart_strip".to_string(),
                            KasaDevice {
                                ip: "placeholder".to_string(),
                                username: "placeholder".to_string(),
                                password: "placeholder".to_string(),
                                description: Some("placeholder".to_string()),
                            },
                        )])),
                    };
                    let text = toml::to_string(&config).unwrap();

                    fs::write(&config_file, text).unwrap();
                } else if response.to_uppercase().trim() != "N" {
                    println!("Please specify either [Y]es or [N]o.")
                }
                println!("Created {}. Restart application.", config_file);
                exit(0);
            }
        };

        settings.try_deserialize::<SysConfig>().unwrap()
    }

    pub(crate) fn get_kasa_devices(&self) -> Option<HashMap<String, KasaDevice>> {
        self.kasa.clone()
    }
}
