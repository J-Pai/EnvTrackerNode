//! Entrypoint for services.

use kasa_core::Transport;
use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tokio_memq::Publisher;

use crate::kasa::Kasa;

mod config;
mod kasa;
mod traits;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::SysConfig::new();

    static MQ: Mutex<Option<MessageQueue>> = Mutex::const_new(None);
    {
        let mut mq_lock = MQ.lock().await;
        mq_lock.replace(MessageQueue::new());
    }

    static SCHEDULER: Mutex<Option<JobScheduler>> = Mutex::const_new(None);
    {
        let mut scheduler_lock = SCHEDULER.lock().await;
        scheduler_lock.replace(JobScheduler::new().await?);
    }

    static TRANSPORTS: Mutex<Vec<Box<dyn Transport>>> = Mutex::const_new(Vec::new());

    static PUBLISHERS: Mutex<Vec<Mutex<Option<Publisher>>>> = Mutex::const_new(Vec::new());

    let kasa_devices = config.get_kasa_devices().unwrap();
    let mut kasa = Kasa::new(&kasa_devices, &MQ, &TRANSPORTS).await;
    kasa.add_polling(&MQ, &PUBLISHERS, &SCHEDULER, &TRANSPORTS).await?;

    let scheduler_lock = SCHEDULER.lock().await;
    scheduler_lock.as_ref().unwrap().start().await?;

    web::server().await?;

    Ok(())
}
