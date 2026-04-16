//! Entrypoint for services.

use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::kasa::Kasa;

mod config;
mod kasa;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    let mut kasa = if let Some(kasa_devices) = config.get_kasa_devices() {
        let mut kasa = Kasa::new(&kasa_devices, &MQ, &SCHEDULER).await;
        kasa.add_polling().await?;
        Some(kasa)
    } else {
        None
    };

    let scheduler_lock = SCHEDULER.write().await;
    scheduler_lock.as_ref().unwrap().start().await?;

    web::server(&config, &mut kasa, &MQ).await?;

    Ok(())
}
