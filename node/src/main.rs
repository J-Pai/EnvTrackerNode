//! Entrypoint for services.

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::kasa::Kasa;

mod config;
mod error;
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

    let mq: Arc<RwLock<MessageQueue>> = Arc::new(RwLock::const_new(MessageQueue::new()));
    let scheduler: Arc<RwLock<JobScheduler>> = Arc::new(RwLock::new(JobScheduler::new().await?));

    let mut kasa = if let Some(kasa_devices) = config.get_kasa_devices() {
        let mut kasa = Kasa::new(&kasa_devices, mq.clone(), scheduler.clone()).await;
        kasa.add_polling().await?;
        Some(kasa)
    } else {
        None
    };

    {
        let scheduler_lock = scheduler.write().await;
        scheduler_lock.start().await?;
    }

    web::server(&config, &mut kasa, mq, scheduler).await?;

    Ok(())
}
