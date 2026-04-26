//! Manages db interactions for specific devices.

use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::RwLock;
use turso::Builder;
use turso::Connection;
use turso::Database;

use crate::config::Server;
use crate::error::NodeError;

pub(crate) struct Db {
    db: RwLock<Database>,
}

impl Db {
    pub(crate) async fn new(config: &Server) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Builder::new_local(config.db.as_str()).build().await?;

        let db = Self {
            db: RwLock::new(db),
        };

        Ok(db)
    }

    pub(crate) async fn create_kasa_table(self) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let db = self.db.read().await;
            let conn = db.connect()?;
            conn.execute(
                r#"CREATE TABLE IF NOT EXISTS kasa (utc_ns INTEGER,
                                                    alias TEXT, id TEXT,
                                                    current_ma INTEGER,
                                                    power_mw INTEGER,
                                                    voltage_mv INTEGER,
                                                    total_wh INTEGER)"#,
                (),
            )
            .await?;
        }
        Ok(self)
    }

    pub(crate) async fn create_connection(
        &self,
    ) -> Result<Arc<Mutex<Connection>>, Box<dyn std::error::Error>> {
        let db = self.db.read().await;
        let conn = db.connect()?;
        Ok(Arc::new(Mutex::new(conn)))
    }
}
