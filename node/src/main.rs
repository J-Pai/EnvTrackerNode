//! EnvTrackerNode
//!
//! Sets up and launches services for interacting with IoT devices.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use gcp_auth::CustomServiceAccount;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::MessageQueue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::NodeClass;
use crate::services::auth::Auth;
use crate::services::db::Db;
use crate::services::google_home::light::DLight;
use crate::services::google_home::plug::Wemo;
use crate::services::google_home::{Device, SupportedDevices};
use crate::services::kasa::Kasa;
use crate::services::poller::Poller;
use crate::services::web::Web;

mod config;
mod error;
mod services;
mod timer;

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
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Can't set crypto provider");

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
    let mut poller = Poller::new(scheduler.clone(), db.clone());

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
        web = web.setup_api_route(&api_config)?;

        if let Some(oauth2) = api_config.get_oauth2_config() {
            tracing::info!("[Service] Authed API Backend");
            web = web
                .setup_auth_route(
                    Auth::new(
                        &oauth2,
                        db.clone().expect("Auth requires a DB."),
                        scheduler.clone(),
                    )
                    .await?,
                )
                .await?;

            web = web
                .setup_google_home_route(api_config.get_google_home_node_api(), None, None, None)
                .await?;
        }
    }

    let mut devices: HashMap<String, Arc<Mutex<SupportedDevices>>> = HashMap::new();

    if let Some(node) = config.get_node_config()
        && let Some(uri) = node.get_dlight_uri()
    {
        tracing::info!("[Service] dlight");
        let dlight = DLight::new(uri).await?;
        devices.insert(
            dlight.get_id(),
            Arc::new(Mutex::new(SupportedDevices::DLight(dlight))),
        );
    }

    if let Some(node) = config.get_node_config()
        && let Some(uri) = node.get_wemo0_uri()
    {
        tracing::info!("[Service] wemo0");
        let wemo0 = Wemo::new(uri, "wemo0".to_string()).await?;
        devices.insert(
            wemo0.get_id(),
            Arc::new(Mutex::new(SupportedDevices::Wemo(wemo0))),
        );
    }

    if !devices.is_empty()
        && let Some(node) = config.get_node_config()
    {
        tracing::info!("[Service] Google Home Devices Node");

        let service_account = if let Some(path) = node.get_google_home_service_account_json() {
            Some(Arc::new(CustomServiceAccount::from_file(path)?))
        } else {
            None
        };

        web = web
            .setup_google_home_route(
                None,
                node.get_agent_user_id(),
                service_account.clone(),
                Some(devices.clone()),
            )
            .await?;

        poller = poller
            .add_devices_job(node.get_agent_user_id(), service_account, devices)
            .await?;
    }

    web.start(poller).await?;

    Ok(())
}
