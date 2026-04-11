use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use tokio::process::Command;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = env::home_dir().expect("HOME dir not specified.");
    let config_dir = format!("{}/.config/envtrackernode", home_dir.to_str().unwrap());
    let config_file = format!("{}/config.toml", config_dir);
    let settings = match Config::builder()
        .add_source(config::File::with_name(&config_file))
        .build() {
        Ok(built_config) => {
            println!("{:?}", built_config);
            built_config
        },
        Err(e) => {
            println!("Error obtaining config file: {:?}", e);
            println!("Set up configuration? ([Y]es / [n]o)");
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            if response.to_uppercase().trim()  == "Y" {
                println!("Creating {}", config_file);
                fs::create_dir_all(config_dir)?;
                fs::write(config_file, "")?;
            } else if response.to_uppercase().trim() != "N" {
                println!("Please specify either [Y]es or [N]o.")
            }
            return Ok(());
        }
    };

    let _settings = match settings.try_deserialize::<HashMap<String, String>>() {
        Ok(deserialized) => {
            println!("{:?}", deserialized);
        },
        Err(e) => {
            println!("Error parsing config file: {:?}", e);
            return Ok(());
        }
    };

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

    let mut venv_requirements_setup = Command::new("venv/bin/kasa")
        .arg("install")
        .arg("-r")
        .arg("requirements.txt")
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = venv_requirements_setup.wait().await?;

    Ok(())
}
