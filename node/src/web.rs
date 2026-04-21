//! Sets up the web services.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::routing;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_cron_scheduler::JobScheduler;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

use crate::config::KasaDeviceConfig;
use crate::config::SysConfig;
use crate::error::NodeError;
use crate::kasa::Kasa;
use crate::kasa::KasaChildInfo;

pub(crate) struct Web {
    router: Option<Router>,
    listener: Option<TcpListener>,
    scheduler: Arc<RwLock<JobScheduler>>,
    kasa_devices: Option<Kasa>,
}

impl Web {
    pub(crate) async fn new(
        scheduler: Arc<RwLock<JobScheduler>>,
        kasa_devices: Option<Kasa>,
    ) -> Self {
        Self {
            router: None,
            listener: None,
            scheduler,
            kasa_devices,
        }
    }

    async fn setup_kasa_routes(
        mut self,
        device_configs: &HashMap<String, KasaDeviceConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let kasa_subscribers: Arc<RwLock<HashMap<String, Arc<RwLock<Subscriber>>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let mut router = self.router.take().unwrap();

        for device_config in device_configs {
            if let Some(kasa) = &mut self.kasa_devices {
                kasa_subscribers.write().await.insert(
                    device_config.0.clone(),
                    kasa.allocate_subscriber(
                        device_config.0.clone(),
                        TopicOptions::default(),
                        ConsumptionMode::Earliest,
                    )
                    .await?,
                );
            }
        }

        router = router.route(
            "/kasa/{topic}",
            routing::get(move |Path(topic): Path<String>| {
                let kasa_subscribers = kasa_subscribers.clone();

                async move {
                    let kasa_subscribers = kasa_subscribers.read().await;
                    let subscriber = kasa_subscribers.get(&topic).unwrap().read().await;
                    let msg = match timeout(Duration::from_millis(100), subscriber.recv_batch(100))
                        .await
                    {
                        Ok(result) => result.unwrap(),
                        Err(_) => {
                            return "[]".to_string();
                        }
                    };

                    let mut output: String = String::new();
                    for m in msg.iter() {
                        let json = m.deserialize::<Value>().unwrap();
                        let child_info: Vec<KasaChildInfo> =
                            serde_json::from_value(json.clone()).unwrap();
                        output.push_str(format!("{:#?}\n", child_info).as_str());
                    }

                    output
                }
            }),
        );

        self.router = Some(router);

        Ok(self)
    }

    fn setup_frontend_route(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router.take().unwrap();

        let index_service = ServeFile::new("dist/index.html");
        let serve_dir = ServeDir::new("dist").not_found_service(index_service.clone());
        router = router
            .route(
                "/",
                routing::get(|request: Request| async {
                    let service = index_service;
                    let result = service.oneshot(request).await;
                    result
                }),
            )
            .fallback_service(serve_dir);

        self.router = Some(router);
        Ok(self)
    }

    pub(crate) async fn setup_router(
        mut self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.router = Some(Router::new());

        if let Some(node) = config.get_node_config() {
            if !node.kasa.is_empty() {
                self = self.setup_kasa_routes(&node.kasa).await?;
            }
        }

        if let Some(server) = config.get_server_config()
            && server.frontend
        {
            self = self.setup_frontend_route()?;
        }

        Ok(self)
    }

    pub(crate) async fn setup_listener(
        mut self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(config.get_server_config().unwrap().node_ip)
            .await
            .unwrap();

        self.listener = Some(listener);

        Ok(self)
    }

    pub(crate) async fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(listener) = self.listener.take()
            && let Some(router) = self.router.take()
        {
            let scheduler = self.scheduler.write().await;
            scheduler.start().await?;
            tracing::info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, router).await?;
            Ok(())
        } else {
            Err(NodeError::new("No server is configured."))
        }
    }
}
