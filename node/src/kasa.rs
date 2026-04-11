//! Kasa handler

use tokio::process::Command;

use crate::config::SysConfig;

pub(crate) async fn handlers(config: &SysConfig)  -> Result<(), Box<dyn std::error::Error>> {
    if let Some(kasa_devices) = config.get_kasa_devices() {
        let kasa_device = kasa_devices.get("smart_strip").unwrap();
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
    }

    Ok(())
}
