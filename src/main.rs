use config::Config;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::vec;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct KasaDevice {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    jwt_secret: String,
    authorized_api_keys: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigToml {
    settings: Settings,
    kasa: Option<HashMap<String, KasaDevice>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut venv_setup = Command::new("virtualenv")
        .arg("venv")
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = venv_setup.wait().await?;

    println!("==> virtualenv status: {status}");

    let mut venv_requirements_setup = Command::new("venv/bin/pip")
        .arg("install")
        .arg("-r")
        .arg("requirements.txt")
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = venv_requirements_setup.wait().await?;

    println!("==> virtualenv requirements setup status: {status}");

    let home_dir = env::home_dir().expect("HOME dir not specified.");
    let config_dir = format!("{}/.config/envtrackernode", home_dir.to_str().unwrap());
    let config_file = format!("{}/config.toml", config_dir);
    let settings = match Config::builder()
        .add_source(config::File::with_name(&config_file))
        .build()
    {
        Ok(built_config) => {
            built_config
        }
        Err(e) => {
            println!("Error obtaining config file: {:?}", e);
            println!("Create configuration file? ([Y]es / [n]o)");
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            if response.to_uppercase().trim() == "Y" {
                println!("Creating {}", config_file);
                fs::create_dir_all(config_dir)?;

                let config = ConfigToml {
                    settings: Settings {
                        jwt_secret: "placeholder".to_string(),
                        authorized_api_keys: Some(vec!["placeholder".to_string(), "placeholder".to_string()]),
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
                let text = toml::to_string(&config)?;

                fs::write(config_file, text)?;
            } else if response.to_uppercase().trim() != "N" {
                println!("Please specify either [Y]es or [N]o.")
            }
            return Ok(());
        }
    };

    let settings = settings.try_deserialize::<ConfigToml>()?;

    println!("Config: {:#?}", settings);

    let kasa_map = settings.kasa.unwrap();
    let kasa_device = kasa_map.get("smart_strip").unwrap();

    let mut kasa_data = Command::new("venv/bin/kasa")
        .arg("--host")
        .arg(kasa_device.ip.as_str())
        .arg("--username")
        .arg(kasa_device.username.as_str())
        .arg("--password")
        .arg(kasa_device.password.as_str())
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = kasa_data.wait().await?;

    println!("==> kasa data status: {status}");

    Ok(())
}
