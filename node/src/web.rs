//! Sets up the web services.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::routing;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::reqwest::Client;
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
    node_client: Arc<RwLock<Option<(ClientWithMiddleware, String)>>>,
    kasa_devices: Option<Kasa>,
    topic_routes: Arc<RwLock<Vec<String>>>,
}

impl Web {
    pub(crate) async fn new(
        scheduler: Arc<RwLock<JobScheduler>>,
        kasa_devices: Option<Kasa>,
    ) -> Self {
        Self {
            router: None,
            listener: None,
            node_client: Arc::new(RwLock::const_new(None)),
            scheduler,
            kasa_devices,
            topic_routes: Arc::new(RwLock::const_new(Vec::new())),
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
                self.topic_routes
                    .write()
                    .await
                    .push(format!("/kasa/{}", device_config.0.clone()));
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

    fn setup_routes_route(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router.take().unwrap();
        let topic_routes = self.topic_routes.clone();

        router = router.route(
            "/topics",
            routing::get(|| async move {
                let topic_routes = topic_routes.clone();
                let routes = topic_routes.read().await;
                format!("{:?}", routes)
            }),
        );

        self.router = Some(router);
        Ok(self)
    }

    async fn setup_node_client(
        self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let endpoint = if config.get_node_config().is_none()
                && let Some(node_ip) = config.get_server_config().unwrap().node_ip
            {
                node_ip
            } else {
                config.get_ip()
            };

            let mut node_client = self.node_client.write().await;
            let client = ClientBuilder::new(Client::new()).build();
            node_client.replace((client, endpoint));
        }

        Ok(self)
    }

    async fn setup_server_polling(self) -> Result<Self, Box<dyn std::error::Error>> {
        {
            let node = self.node_client.read().await;
            let (node_client, ip) = node.as_ref().unwrap();
            let data = node_client
                .get(format!("http://{}/topics", ip))
                .send()
                .await
                .unwrap();
            tracing::warn!("{:?}", data.text().await.unwrap());
        }

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
            if !self.topic_routes.read().await.is_empty() {
                self = self.setup_routes_route()?;
            }
        }

        if let Some(server) = config.get_server_config() {
            if server.frontend {
                self = self.setup_frontend_route()?;
            }

            self = self
                .setup_node_client(config)
                .await?
                .setup_server_polling()
                .await?;
        }

        Ok(self)
    }

    pub(crate) async fn setup_listener(
        mut self,
        config: &SysConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(config.get_ip())
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
