//! EnvTrackerNode
//!
//! Sets up and launches services for interacting with IoT devices.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config2::NodeClass;
use crate::services::db::Db;
use crate::services::kasa::Kasa;
use crate::services::web::Web;

mod config;
mod config2;
mod error;
mod services;

/// Commandline arguments for Server.
#[derive(Parser, Debug)]
struct Args {
    /// Path to configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

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

    let args = Args::parse();

    let config2 = config2::ServerConfig::new(match args.config {
        Some(path) => path,
        None => {
            let home_dir = env::home_dir().expect("HOME dir not specified.");
            let config_dir = home_dir.join(".config/envtrackernode");
            config_dir.join("config2.toml")
        }
    });

    let home_dir = env::home_dir().expect("HOME dir not specified.");
    let config_file = home_dir.join(".config/envtrackernode/config.toml");
    let config = config::SysConfig::new(Some(config_file.to_str().unwrap().to_string()));

    let mq: Arc<RwLock<MessageQueue>> = Arc::new(RwLock::const_new(MessageQueue::new()));
    let scheduler: Arc<RwLock<JobScheduler>> = Arc::new(RwLock::new(JobScheduler::new().await?));
    let mut kasa_devices: Option<Kasa> = None;

    if let Some(node) = config2.get_node_config() {
        for n in node.get_nodes() {
            let NodeClass::KasaDevice(id, cfg, sch) = n;
            let mut kasa = Kasa::new(mq.clone(), scheduler.clone()).await;
            kasa.add_device(&id, &cfg).await?;
            kasa.add_polling(&id, &sch).await?;
            kasa_devices.replace(kasa);
        }
    }

    let db = if let Some(server) = config.get_server_config() {
        Some(Db::new(&server).await?.create_kasa_table().await?)
    } else {
        None
    };

    Web::new(scheduler, kasa_devices, db)
        .await
        .setup_router(&config)
        .await?
        .setup_listener(&config)
        .await?
        .start()
        .await?;

    Ok(())
}
