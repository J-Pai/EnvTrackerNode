//! Logic for polling node endpoints.

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;

use crate::services::db::Db;

pub(crate) struct Poller {
    scheduler: Arc<RwLock<JobScheduler>>,
    db: Option<Db>,
}

impl Poller {
    pub(crate) fn new(scheduler: Arc<RwLock<JobScheduler>>, db: Option<Db>) -> Self {
        Self { scheduler, db }
    }

    pub(crate) async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let scheduler = self.scheduler.write().await;
        scheduler.start().await?;
        Ok(())
    }
}
