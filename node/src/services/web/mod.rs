//! Sets up the web services.

use std::sync::Arc;

use axum::Router;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;

use crate::config::SysConfig;
use crate::config2::ApiServerConfig;
use crate::services::db::Db;
use crate::services::poller::Poller;

mod api;
mod frontend;
mod kasa_node;

pub(crate) struct Web {
    router: Router,
    db: Option<Db>,
}

impl Web {
    pub(crate) fn new(db: Option<Db>) -> Self {
        Self {
            router: Router::new(),
            db,
        }
    }

    async fn setup_node_client(
        mut self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let endpoint = if config.get_node_config().is_none()
            && let Some(node_ip) = config.get_server_config().unwrap().node_ip
        {
            node_ip
        } else {
            config.get_ip()
        };

        // {
        //     let mut node_client = self.node_client.lock().await;
        //     let client = ClientBuilder::new(Client::new()).build();
        //     node_client.replace((client, endpoint));
        // }

        // self.node_polling_schedule.replace(
        //     config
        //         .get_server_config()
        //         .unwrap()
        //         .node_polling_schedule
        //         .clone(),
        // );

        Ok(self)
    }

    async fn setup_server_polling(self) -> Result<Self, Box<dyn std::error::Error>> {
        // let scheduler = self.scheduler.read().await;
        // let node_client = self.node_client.clone();
        // let db = self.db.as_ref().unwrap().clone();
        // scheduler
        //     .add(Job::new_async(
        //         self.node_polling_schedule.clone().unwrap(),
        //         move |_uuid, _l| {
        //             Box::pin({
        //                 let route = "/kasa/smart_strip";
        //                 let mut db = db.clone();
        //                 let node_client = node_client.clone();
        //                 async move {
        //                     let node = if let Ok(node) = node_client.try_lock() {
        //                         node
        //                     } else {
        //                         tracing::debug!("Skipping polling event: {}", route);
        //                         return;
        //                     };
        //                     let (client, ip) = node.as_ref().unwrap();
        //                     db.create_connection().await.unwrap();
        //                     let mut count = 0;
        //                     loop {
        //                         count += 1;
        //                         if let Ok(data) =
        //                             client.get(format!("http://{}{}", ip, route)).send().await
        //                         {
        //                             match &route {
        //                                 r if r.starts_with("/kasa") => {
        //                                     let json =
        //                                         data.text().await.unwrap_or("[]".to_string());

        //                                     let kasa_data: Result<
        //                                         Vec<Vec<KasaChildInfo>>,
        //                                         serde_json::Error,
        //                                     > = serde_json::from_str(&json);

        //                                     match kasa_data {
        //                                         Ok(k) => {
        //                                             if k.is_empty() {
        //                                                 break;
        //                                             }
        //                                             db.push_kasa_data(&k).await.unwrap();
        //                                         }
        //                                         Err(e) => {
        //                                             tracing::warn!("Error parsing data {:#?}", e);
        //                                         }
        //                                     }
        //                                 }
        //                                 _ => {
        //                                     tracing::warn!("Unhandled {}{}", ip, route);
        //                                     break;
        //                                 }
        //                             }
        //                         } else {
        //                             tracing::warn!("Issue with {}{}", ip, route);
        //                             break;
        //                         }
        //                     }
        //                     tracing::debug!("Reqeusted {}x {}", count, route);
        //                 }
        //             })
        //         },
        //     )?)
        //     .await?;

        // drop(scheduler);

        Ok(self)
    }

    pub(crate) async fn setup_router(
        mut self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // if let Some(server) = config.get_server_config() {
        //     if server.frontend {
        //         self = self.setup_frontend_route().await?;
        //     }
        //     if server.node_ip.is_some() {
        //         self = self
        //             .setup_node_client(config)
        //             .await?
        //             .setup_server_polling()
        //             .await?;
        //     }
        // }

        Ok(self)
    }

    pub(crate) async fn setup_api_route(
        mut self,
        config: &ApiServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self)
    }

    pub(crate) async fn start(self, poller: Poller) -> Result<(), Box<dyn std::error::Error>> {
        poller.start().await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
