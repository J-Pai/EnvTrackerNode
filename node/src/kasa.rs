//! Sets up and manages interactions with Kasa devices.
//! - Kasa Smart Power Strip HS300

use std::time::Duration;
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Local;
use kasa_core::Credentials;
use kasa_core::DeviceConfig;
use kasa_core::Transport;
use kasa_core::commands::INFO;
use kasa_core::connect;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::AsyncMessagePublisher;
use tokio_memq::MessageQueue;
use tokio_memq::TopicOptions;

use crate::config::SysConfig;

pub(crate) async fn handler(
    config: &SysConfig,
    mq: &'static Mutex<Option<MessageQueue>>,
) -> Result<&'static Mutex<Option<MessageQueue>>, Box<dyn std::error::Error>> {
    static KASA_TRANSPORT: Mutex<Option<Box<dyn Transport>>> = Mutex::const_new(None);

    if let Some(kasa_devices) = config.get_kasa_devices() {
        let kasa_device = kasa_devices.get("smart_strip").unwrap();

        let kasa_config = DeviceConfig::new(kasa_device.ip.as_str()).with_credentials(
            Credentials::new(kasa_device.username.as_str(), kasa_device.password.as_str()),
        );

        let mut kasa_lock = KASA_TRANSPORT.lock().await;
        kasa_lock.replace(connect(kasa_config).await?);
    } else {
        return Ok(mq);
    };

    let sched = JobScheduler::new().await?;

    let pub_instance = {
        let mq_lock = mq.lock().await;
        let mq = mq_lock.as_ref().unwrap();
        mq.create_topic(
            "kasa".to_string(),
            TopicOptions {
                // Should track data for up to 3 months.
                max_messages: Some(Duration::as_secs(&Duration::from_hours(24 * 90)) as usize),
                ..Default::default()
            },
        )
        .await?;
        mq.publisher("kasa".to_string())
    };
    sched
        .add(Job::new_async("1/1 * * * * *", move |_uuid, _l| {
            let pub_instance = pub_instance.clone();
            Box::pin(async move {
                let system_time = SystemTime::now();
                let datetime: DateTime<Local> = system_time.into();
                println!("[{}] Sampling...", datetime.format("%d/%m/%Y %T"));
                let response = KASA_TRANSPORT
                    .lock()
                    .await
                    .as_ref()
                    .unwrap()
                    .send(INFO)
                    .await
                    .unwrap();
                let data: Value = serde_json::from_str(&response.as_str()).unwrap();
                pub_instance.publish(data).await.unwrap();
            })
        })?)
        .await?;

    sched.start().await?;

    Ok(mq)
}
