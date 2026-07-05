//! Manages db interactions for specific devices.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OwnedMutexGuard;
use tokio::sync::RwLock;
use turso::Builder;
use turso::Connection;
use turso::Database;

use crate::config::ApiServerConfig;
use crate::config::NodeClass;
use crate::error::NodeError;
use crate::services::kasa::EMeter;
use crate::services::kasa::KasaChildInfo;
use crate::services::kasa::KasaDeviceChild;

#[derive(serde::Deserialize, Clone, Debug)]
#[allow(nonstandard_style)]
enum Column {
    utc_ns,
    alias,
    id,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[allow(nonstandard_style)]
enum Order {
    asc,
    desc,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct DeviceQuery {
    start_time_ns: Option<i64>,
    end_time_ns: Option<i64>,
    alias: Option<String>,
    id: Option<String>,
    distinct: Option<String>,
    order_by: Option<Order>,
    column: Option<Column>,
    limit: Option<usize>,
}

pub(crate) enum QueryResult {
    KasaDeviceInfo(Vec<KasaChildInfo>),
    Distinct(Vec<(String, String)>),
}

#[derive(Debug)]
struct DeviceQueryArgs(String, String, i64, i64);

impl DeviceQuery {
    const DEFAULT_LIMIT: usize = 100;

    fn generate_query(&self, table: &String) -> (String, DeviceQueryArgs) {
        let DeviceQuery {
            start_time_ns,
            end_time_ns,
            alias,
            id,
            distinct,
            order_by,
            column,
            limit,
        } = self;

        let mut base_query = format!("SELECT * FROM {table}");
        let mut args = DeviceQueryArgs(String::new(), String::new(), 0, 0);

        if let Some(_) = distinct
            && let Some(column) = column
        {
            match column {
                Column::alias => base_query = format!("SELECT DISTINCT(alias), id FROM {table}"),
                Column::id => base_query = format!("SELECT DISTINCT(id), alias FROM {table}"),
                _ => {}
            }

            return (
                base_query,
                DeviceQueryArgs(String::new(), String::new(), 0, 0),
            );
        }

        if start_time_ns.is_some() || end_time_ns.is_some() || alias.is_some() || id.is_some() {
            base_query = format!("{base_query} WHERE ");
        }

        let mut append = "";
        if let Some(alias) = alias {
            args.0 = alias.clone();
            base_query = format!("{base_query} alias = ?1");
            append = "and";
        }

        if let Some(id) = id {
            args.1 = id.clone();
            base_query = format!("{base_query} {append} id = ?2");
            append = "and";
        }

        if let Some(start_time_ns) = start_time_ns {
            args.2 = *start_time_ns;
            base_query = format!("{base_query} {append} utc_ns >= ?3");
            append = "and";
        }

        if let Some(end_time_ns) = end_time_ns {
            args.3 = *end_time_ns;
            base_query = format!("{base_query} {append} utc_ns <= ?4");
        }

        if let Some(order_by) = order_by
            && let Some(column) = column
            && distinct.is_none()
        {
            let order = "ORDER BY";
            match column {
                Column::utc_ns => base_query = format!("{base_query} {order} utc_ns"),
                Column::alias => base_query = format!("{base_query} {order} alias"),
                Column::id => base_query = format!("{base_query} {order} id"),
            }
            match order_by {
                Order::asc => base_query = format!("{base_query} ASC"),
                Order::desc => base_query = format!("{base_query} DESC"),
            }
        }

        match limit {
            Some(0) => {}
            Some(limit) => base_query = format!("{base_query} LIMIT {limit}"),
            None => base_query = format!("{base_query} LIMIT {}", Self::DEFAULT_LIMIT),
        };

        return (base_query, args);
    }
}

pub(crate) struct Db {
    db: Arc<RwLock<Database>>,
    write_conn: Arc<Mutex<Connection>>,
    write_conn_guard: Option<OwnedMutexGuard<Connection>>,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        let db = self.db.clone();

        Self {
            db,
            write_conn: self.write_conn.clone(),
            write_conn_guard: None,
        }
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        self.write_lock_release();
        tracing::debug!("db write lock dropped");
    }
}

impl Db {
    pub(crate) async fn new(config: &ApiServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Builder::new_local(config.get_db().as_str()).build().await?;
        let write_conn = db.connect()?;
        write_conn
            .pragma_update("wal_checkpoint", "TRUNCATE")
            .await?;

        let mut db = Self {
            db: Arc::new(RwLock::new(db)),
            write_conn: Arc::new(Mutex::new(write_conn)),
            write_conn_guard: None,
        };

        for node in config.get_nodes() {
            match node {
                NodeClass::KasaDevice(topic, _, _) => {
                    db = db.create_kasa_table(topic).await?;
                }
                NodeClass::Unknown => {}
            }
        }

        Ok(db)
    }

    pub(crate) async fn try_write_lock(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.write_conn_guard
            .replace(self.write_conn.clone().try_lock_owned()?);
        Ok(())
    }

    fn write_lock_release(&mut self) {
        self.write_conn_guard.take();
    }

    pub(crate) async fn create_kasa_table(
        self,
        topic: &String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let db = self.db.read().await;
            let conn = db.connect()?;
            conn.execute(
                format!(
                    r#"CREATE TABLE IF NOT EXISTS kasa_device_{} (
                            utc_ns INTEGER PRIMARY KEY NOT NULL UNIQUE,
                            alias TEXT NOT NULL,
                            id TEXT NOT NULL,
                            current_ma INTEGER NOT NULL,
                            power_mw INTEGER NOT NULL,
                            voltage_mv INTEGER NOT NULL,
                            total_wh INTEGER NOT NULL
                    )"#,
                    topic
                ),
                (),
            )
            .await?;
        }
        Ok(self)
    }

    pub(crate) async fn push_kasa_data(
        &mut self,
        topic: &String,
        kasa_data: &Vec<Vec<KasaChildInfo>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(write_conn) = &self.write_conn_guard else {
            return Err(NodeError::new("db write guard not secured."));
        };

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
            "INSERT INTO kasa_device_{} (utc_ns, alias, id, current_ma, power_mw, voltage_mv, total_wh) VALUES {};",
            topic,
            data.join(",").as_str()
        );

        tracing::debug!("SQL insertion");
        write_conn.execute(statement, ()).await?;
        tracing::debug!("SQL insertion completed");

        Ok(())
    }

    pub(crate) async fn query_kasa_data(
        &self,
        topic: &String,
        query: &DeviceQuery,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let mut data: Vec<KasaChildInfo> = vec![];
        let mut distinct: Vec<(String, String)> = vec![];
        let db = self.db.read().await;
        let conn = db.connect()?;

        let table = format!("kasa_device_{}", topic);
        let sql_query = query.generate_query(&table);

        tracing::debug!("SQL query [{:?}]", sql_query);
        let DeviceQueryArgs(alias, id, start_time_ns, end_time_ns) = sql_query.1;
        let mut rows = conn
            .query(sql_query.0, (alias, id, start_time_ns, end_time_ns))
            .await?;

        while let Some(row) = rows.next().await? {
            if row.column_count() == 2 {
                distinct.push((row.get(0)?, row.get(1)?));
            } else {
                data.push(KasaChildInfo {
                    utc_ns: row.get(0)?,
                    info: KasaDeviceChild {
                        alias: row.get(1)?,
                        id: row.get(2)?,
                        state: true,
                    },
                    emeter: EMeter {
                        current_ma: row.get(3)?,
                        power_mw: row.get(4)?,
                        voltage_mv: row.get(5)?,
                        total_wh: row.get(6)?,
                    },
                });
            }
        }

        tracing::debug!("SQL query completed");

        if !distinct.is_empty() {
            Ok(QueryResult::Distinct(distinct))
        } else {
            Ok(QueryResult::KasaDeviceInfo(data))
        }
    }
}
