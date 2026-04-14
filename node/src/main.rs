//! Entrypoint for services.

use std::collections::HashMap;

use kasa_core::Transport;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::Publisher;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

use crate::kasa::Kasa;

mod config;
mod kasa;
mod traits;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::SysConfig::new();

    static MQ: RwLock<Option<MessageQueue>> = RwLock::const_new(None);
    {
        let mut mq_lock = MQ.write().await;
        mq_lock.replace(MessageQueue::new());
    }

    static SCHEDULER: RwLock<Option<JobScheduler>> = RwLock::const_new(None);
    {
        let mut scheduler_lock = SCHEDULER.write().await;
        scheduler_lock.replace(JobScheduler::new().await?);
    }

    let kasa_devices = config.get_kasa_devices().unwrap();
    let mut kasa = Kasa::new(&kasa_devices, &MQ).await;
    kasa.add_polling(&MQ, &SCHEDULER)
        .await?;

    let scheduler_lock = SCHEDULER.write().await;
    scheduler_lock.as_ref().unwrap().start().await?;

    web::server(&config).await?;

    Ok(())
}
