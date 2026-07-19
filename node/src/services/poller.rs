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
                let route = polling.clone().get_api().unwrap_or_default();

                poller = poller
                    .add_kasa_job(route, topic, *device_config, polling)
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

        let job = Job::new_async(polling.get_schedule(), move |uuid, _l| {
            Box::pin({
                let topic = topic.clone();
                let device_config = device_config.clone();
                let mut db = db.clone();
                let node_client = node_client.clone();
                let route = route.clone();

                async move {
                    let mut sample_count = 0;
                    let mut url = device_config.get_uri().join(route.as_str()).unwrap();

                    if let Some(size) = device_config.get_batch_size() {
                        url.set_query(Some(format!("size={size}").as_str()));
                    }

                    tracing::debug!("{uuid} - Kasa Polling - {url}");

                    if let Err(e) = db.try_write_lock().await {
                        tracing::warn!(
                            "{} - Kasa Polling action already happening: {:#?}",
                            uuid,
                            e
                        );
                        return;
                    }

                    loop {
                        sample_count += 1;

                        match node_client.get(url.clone()).send().await {
                            Ok(mut data) => {
                                data = match data.error_for_status() {
                                    Err(err) => {
                                        tracing::warn!("Received status code: {:#?}", err);
                                        break;
                                    }
                                    Ok(data) => data,
                                };

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
                                    device_config.get_uri(),
                                    route,
                                    e
                                );
                                break;
                            }
                        }
                    }

                    tracing::debug!("{} - Kasa Requested {}x {}", uuid, sample_count, url);
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
