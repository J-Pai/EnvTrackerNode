//! Manages db interactions for specific devices.

use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::RwLock;
use turso::Builder;
use turso::Connection;
use turso::Database;

use crate::config::Server;
use crate::error::NodeError;
use crate::kasa::KasaChildInfo;

pub(crate) struct Db {
    db: Arc<RwLock<Database>>,
    conn: Option<Mutex<Connection>>,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            conn: None,
        }
    }
}

impl Db {
    pub(crate) async fn new(config: &Server) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Builder::new_local(config.db.as_str()).build().await?;

        let db = Self {
            db: Arc::new(RwLock::new(db)),
            conn: None,
        };

        // Truncate the wal file before things get heavy.
        {
            let db = db.db.read().await;
            let conn = db.connect()?;
            conn.pragma_update("wal_checkpoint", "TRUNCATE").await?;
        }

        Ok(db)
    }

    pub(crate) async fn create_connection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.conn.is_none() {
            let db = self.db.read().await;
            self.conn.replace(Mutex::new(db.connect()?));
        }
        Ok(())
    }

    pub(crate) async fn create_kasa_table(self) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let db = self.db.read().await;
            let conn = db.connect()?;
            conn.execute(
                r#"CREATE TABLE IF NOT EXISTS kasa (utc_ns INTEGER NOT NULL,
                                                    alias TEXT NOT NULL,
                                                    id TEXT NOT NULL,
                                                    current_ma INTEGER NOT NULL,
                                                    power_mw INTEGER NOT NULL,
                                                    voltage_mv INTEGER NOT NULL,
                                                    total_wh INTEGER NOT NULL)"#,
                (),
            )
            .await?;
        }
        Ok(self)
    }

    pub(crate) async fn push_kasa_data(
        &mut self,
        kasa_data: &Vec<Vec<KasaChildInfo>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data: Vec<String> = vec![];

        for single_poll in kasa_data {
            for child_data in single_poll {
                data.push(format!(
                    "({utc_ns}, \"{alias}\", \"{id}\", {current_ma}, {power_mw}, {voltage_mv}, {total_wh})",
                    utc_ns = child_data.utc_ns,
                    alias = child_data.info.alias,
                    id = child_data.info.id,
                    current_ma = child_data.emeter.current_ma,
                    power_mw = child_data.emeter.power_mw,
                    voltage_mv = child_data.emeter.voltage_mv,
                    total_wh = child_data.emeter.total_wh,
                ));
            }
        }

        if data.is_empty() {
            return Ok(());
        }

        let statement = format!(
            "INSERT INTO kasa (utc_ns, alias, id, current_ma, power_mw, voltage_mv, total_wh) VALUES {};",
            data.join(",").as_str()
        );

        tracing::debug!("SQL query [{}]", statement);
        let conn = self
            .conn
            .as_ref()
            .ok_or(NodeError::new("Connector not setup!"))?
            .lock()
            .await;
        conn.execute(statement, ()).await?;
        tracing::debug!("SQL query completed");

        Ok(())
    }
}
