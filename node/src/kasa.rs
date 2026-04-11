/// Sets up and manages interactions with Kasa devices.
/// - Kasa Smart Power Strip HS300
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Utc;
use kasa_core::Credentials;
use kasa_core::DeviceConfig;
use kasa_core::connect;
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
        let response = transport.send(r#"{"system":{"get_sysinfo":{}}}"#).await?;
        println!("{}", response);
    }

    let sched = JobScheduler::new().await?;

    sched
        .add(Job::new("1/10 * * * * *", |_uuid, _l| {
            let system_time = SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            println!(
                "[{}] I run every 10 seconds",
                datetime.format("%d/%m/%Y %T")
            );
        })?)
        .await?;

    sched.start().await?;

    Ok(())
}
