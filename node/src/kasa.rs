/// Sets up and manages interactions with Kasa devices.
/// - Kasa Smart Power Strip HS300
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Local;
use kasa_core::Credentials;
use kasa_core::DeviceConfig;
use kasa_core::commands::INFO;
use kasa_core::connect;
use serde_json::Value;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

use crate::config::SysConfig;

pub(crate) async fn handler(config: &SysConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(kasa_devices) = config.get_kasa_devices() {
        let kasa_device = kasa_devices.get("smart_strip").unwrap();

        let kasa_config = DeviceConfig::new(kasa_device.ip.as_str()).with_credentials(
            Credentials::new(kasa_device.username.as_str(), kasa_device.password.as_str()),
        );
        let transport = connect(kasa_config).await?;
        let response = transport.send(INFO).await?;

        let data: Value = serde_json::from_str(&response.as_str())?;

        println!("{:#?}", data);
    }

    let sched = JobScheduler::new().await?;

    for i in 0..10 {
        sched
            .add(Job::new("1/10 * * * * *", move |_uuid, _l| {
                let system_time = SystemTime::now();
                let datetime: DateTime<Local> = system_time.into();
                println!(
                    "[{}] {:05} I run every 10 seconds",
                    i,
                    datetime.format("%d/%m/%Y %T")
                );
            })?)
            .await?;
    }

    sched.start().await?;

    Ok(())
}
