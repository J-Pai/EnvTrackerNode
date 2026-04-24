//! Manages db interactions for specific devices.

use tokio::sync::RwLock;
use turso::Builder;
use turso::Database;

use crate::{config::SysConfig, error::NodeError};

pub(crate) struct Db {
    db: RwLock<Database>,
}

impl Db {
    pub(crate) async fn new(config: &SysConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let server_config = config.get_server_config().ok_or(NodeError::new("No server config."))?;

        let db = Builder::new_local(server_config.db.as_str()).build().await?;

        let db = Self {
            db: RwLock::new(db),
        };

        Ok(db)
    }

    pub(crate) async fn create_kasa_tables(self) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let db = self.db.read().await;
            let conn = db.connect()?;

            let tables: Vec<&str> = vec![
                "begin",
                "end",
            ];
            let tables = tables.join(";");

            let _ = conn.execute_batch(&tables).await?;
        }
        Ok(self)
    }
}
