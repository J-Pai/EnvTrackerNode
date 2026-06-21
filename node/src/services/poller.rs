//! Logic for polling node endpoints.

use std::sync::Arc;

use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use tokio::sync::RwLock;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

use crate::config::ApiServerConfig;
use crate::config::KasaDeviceConfig;
use crate::config::NodeClass;
use crate::config::PollingConfig;
use crate::services::db::Db;
use crate::services::kasa::KasaChildInfo;

pub(crate) struct Poller {
    scheduler: Arc<RwLock<JobScheduler>>,
    db: Option<Db>,
}

impl Poller {
    pub(crate) fn new(scheduler: Arc<RwLock<JobScheduler>>, db: Option<Db>) -> Self {
        Self { scheduler, db }
    }

    pub(crate) async fn setup_node_polling(
        self,
        config: &ApiServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut poller = self;
        let nodes = config.get_nodes();

        for node in nodes {
            let node = node.clone();

            if let NodeClass::KasaDevice(topic, device_config, polling) = node {
                let route = match polling.clone().get_api() {
                    Some(endpoint) => endpoint,
                    None => String::new(),
                };

                poller = poller
                    .add_kasa_job(route, topic, device_config, polling)
                    .await?;
            }
        }

        Ok(poller)
    }

    async fn add_kasa_job(
        self,
        route: String,
        topic: String,
        device_config: KasaDeviceConfig,
        polling: PollingConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let node_client = ClientBuilder::new(Client::new()).build();
        let db = self.db.as_ref().unwrap().clone();

        let job = Job::new_async(polling.get_schedule(), move |_uuid, _l| {
            Box::pin({
                let topic = topic.clone();
                let device_config = device_config.clone();
                let route = route.clone();
                let mut db = db.clone();
                let node_client = node_client.clone();
                async move {
                    if let Err(e) = db.create_connection().await {
                        tracing::warn!("Unable to create connection: {:#?}", e);
                        return;
                    };

                    let mut sample_count = 0;
                    let url = format!("http://{}{}", device_config.get_ip(), route);

                    loop {
                        sample_count += 1;

                        match node_client.get(&url).send().await {
                            Ok(data) => {
                                let json = data.text().await.unwrap_or("[]".to_string());

                                match serde_json::from_str::<Vec<Vec<KasaChildInfo>>>(&json) {
                                    Ok(data) => {
                                        if data.is_empty() {
                                            break;
                                        }
                                        if let Err(e) = db.push_kasa_data(&topic, &data).await {
                                            tracing::warn!("Failed to write data: {:#?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Error parsing data {:#?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Issue with: {}{} - {:#?}",
                                    device_config.get_ip(),
                                    route,
                                    e
                                );
                                break;
                            }
                        }
                    }

                    tracing::debug!(
                        "Requested {}x {}{}",
                        sample_count,
                        device_config.get_ip(),
                        route
                    );
                }
            })
        })?;

        self.scheduler.read().await.add(job).await?;

        Ok(self)
    }

    pub(crate) async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let scheduler = self.scheduler.write().await;
        scheduler.start().await?;
        Ok(())
    }
}
