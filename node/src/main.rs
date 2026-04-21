//! Entrypoint for services.

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::kasa::Kasa;
use crate::web::Web;

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
    let mut kasa_devices: Option<Kasa> = None;

    if let Some(node) = config.get_node_config() {
        let mut kasa = Kasa::new(&node.kasa, mq.clone(), scheduler.clone()).await;
        kasa.add_polling().await?;
        kasa_devices.replace(kasa);
    }

    Web::new(scheduler, kasa_devices)
        .await
        .setup_router(&config)
        .await?
        .setup_listener(&config)
        .await?
        .start()
        .await?;

    Ok(())
}
