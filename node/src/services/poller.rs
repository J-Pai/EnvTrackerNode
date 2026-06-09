//! Logic for polling node endpoints.

use std::collections::HashMap;
use std::sync::Arc;

use reqwest_middleware::reqwest::Client;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;

use crate::config::ApiServerConfig;
use crate::services::db::Db;

pub(crate) struct Poller {
    scheduler: Arc<RwLock<JobScheduler>>,
    db: Option<Db>,
    clients: HashMap<String, Client>,
}

impl Poller {
    pub(crate) fn new(scheduler: Arc<RwLock<JobScheduler>>, db: Option<Db>) -> Self {
        Self {
            scheduler,
            db,
            clients: HashMap::new(),
        }
    }

    pub(crate) fn setup_node_client(
        self,
        config: &ApiServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self)
    }

    pub(crate) async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let scheduler = self.scheduler.write().await;
        scheduler.start().await?;
        Ok(())
    }
}
