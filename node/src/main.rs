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

use crate::config::NodeClass;
use crate::services::db::Db;
use crate::services::kasa::Kasa;
use crate::services::poller::Poller;
use crate::services::web::Web;

mod config;
mod error;
mod services;

/// Commandline arguments for Server.
#[derive(Parser, Debug)]
struct Args {
    /// Path to configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Edit server configuration.
    #[arg(short, long)]
    edit_config: bool,
    /// Override the base defined in config.toml.
    #[arg(short, long)]
    no_base: bool,
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

    let mut config = config::ServerConfig::new(
        match args.config {
            Some(path) => path,
            None => {
                let home_dir = env::home_dir().expect("HOME dir not specified.");
                let config_dir = home_dir.join(".config/envtrackernode");
                config_dir.join("config.toml")
            }
        },
        args.edit_config,
    );

    if args.no_base {
        config.override_frontend_base("");
    }

    let mq: Arc<RwLock<MessageQueue>> = Arc::new(RwLock::const_new(MessageQueue::new()));
    let scheduler: Arc<RwLock<JobScheduler>> = Arc::new(RwLock::new(JobScheduler::new().await?));
    let mut kasa: Option<Kasa> = None;

    let db = if let Some(config) = config.get_api_config() {
        Some(Db::new(&config).await?)
    } else {
        None
    };

    if let Some(node) = config.get_node_config() {
        for n in node.get_nodes() {
            let NodeClass::KasaDevice(id, cfg, sch) = n else {
                continue;
            };
            let kasa = kasa.get_or_insert(Kasa::new(mq.clone(), scheduler.clone()).await);
            kasa.add_device(&id, &cfg).await?;
            kasa.add_polling(&id, &sch).await?;
        }
    }

    let mut web = Web::new(db.clone());
    let mut poller = Poller::new(scheduler, db);

    if let Some(mut kasa) = kasa {
        tracing::info!("[Service] Kasa Node");
        web = web.setup_kasa_route(&mut kasa).await?;
    }

    if let Some(config) = config.get_frontend_config() {
        tracing::info!("[Service] Frontend");
        web = web.setup_frontend_route(&config).await?;
    }

    if let Some(api_config) = config.get_api_config() {
        tracing::info!("[Service] API Backend");
        poller = poller.setup_node_polling(&api_config).await?;
        web = web.setup_api_route(&api_config).await?;

        if let Some(oauth2) = api_config.get_oauth2_config() {
            tracing::info!("[Service] Authed API Backend");
            web = web
                .setup_auth(&oauth2, config.get_frontend_config())
                .await?;
        }
    }

    web.start(poller).await?;

    Ok(())
}
