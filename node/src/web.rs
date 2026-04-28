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
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_cron_scheduler::Job;
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
use crate::db::Db;
use crate::error::NodeError;
use crate::kasa::Kasa;
use crate::kasa::KasaChildInfo;

pub(crate) struct Web {
    router: Option<Router>,
    listener: Option<TcpListener>,
    scheduler: Arc<RwLock<JobScheduler>>,
    db: Option<Db>,
    node_client: Arc<Mutex<Option<(ClientWithMiddleware, String)>>>,
    node_polling_schedule: Option<String>,
    kasa_devices: Option<Kasa>,
    topic_routes: Arc<RwLock<Vec<String>>>,
}

impl Web {
    pub(crate) async fn new(
        scheduler: Arc<RwLock<JobScheduler>>,
        kasa_devices: Option<Kasa>,
        db: Option<Db>,
    ) -> Self {
        Self {
            router: None,
            listener: None,
            db,
            node_client: Arc::new(Mutex::const_new(None)),
            node_polling_schedule: None,
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

        let mut children_ids: HashMap<String, Vec<String>> = HashMap::new();

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

                children_ids.insert(
                    device_config.0.clone(),
                    self.kasa_devices
                        .as_ref()
                        .unwrap()
                        .get_children_ids(device_config.0.clone())
                        .await?,
                );
            }
        }

        router = router
            .route(
                "/kasa/{topic}",
                routing::get(move |Path(topic): Path<String>| {
                    let kasa_subscribers = kasa_subscribers.clone();

                    async move {
                        let kasa_subscribers = kasa_subscribers.read().await;
                        let subscriber = if let Some(subscriber) = kasa_subscribers.get(&topic) {
                            subscriber.read().await
                        } else {
                            return "[]".to_string();
                        };
                        let msg =
                            match timeout(Duration::from_millis(100), subscriber.recv_batch(100))
                                .await
                            {
                                Ok(result) => result.unwrap(),
                                Err(_) => {
                                    return "[]".to_string();
                                }
                            };

                        let mut output: Vec<String> = Vec::new();

                        for m in msg.iter() {
                            let json = m.deserialize::<Value>().unwrap();
                            output.push(serde_json::to_string(&json).unwrap());
                        }

                        format!("[{}]", output.join(","))
                    }
                }),
            )
            .route(
                "/kasa/{topic}/children",
                routing::get(move |Path(topic): Path<String>| async move {
                    format!("{:#?}", children_ids.get(&topic).unwrap_or(&Vec::new()))
                }),
            );

        self.router.replace(router);

        Ok(self)
    }

    fn setup_frontend_route(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router.take().unwrap();

        let index_service = ServeFile::new("dist/index.html");
        let serve_dir = ServeDir::new("dist");
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

        self.router.replace(router);
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

        self.router.replace(router);

        Ok(self)
    }

    async fn setup_node_client(
        mut self,
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

            let mut node_client = self.node_client.lock().await;
            let client = ClientBuilder::new(Client::new()).build();
            node_client.replace((client, endpoint));
            self.node_polling_schedule.replace(
                config
                    .get_server_config()
                    .unwrap()
                    .node_polling_schedule
                    .clone(),
            );
        }

        Ok(self)
    }

    async fn setup_server_polling(self) -> Result<Self, Box<dyn std::error::Error>> {
        let topic_routes = {
            let topic_routes = self.topic_routes.read().await;

            if !topic_routes.is_empty() {
                topic_routes.clone()
            } else {
                let node = self.node_client.lock().await;
                let (node_client, ip) = node.as_ref().unwrap();
                if let Ok(data) = node_client
                    .get(format!("http://{}/topics", ip))
                    .send()
                    .await
                {
                    if let Ok(data) = data.json::<Value>().await {
                        let mut topic_routes: Vec<String> = Vec::new();

                        for route in data.as_array().unwrap() {
                            topic_routes.push(route.as_str().unwrap().to_string());
                        }

                        topic_routes
                    } else {
                        topic_routes.clone()
                    }
                } else {
                    topic_routes.clone()
                }
            }
        };

        tracing::debug!("Topics: {:#?}", topic_routes);

        for route in topic_routes {
            let scheduler = self.scheduler.read().await;
            let node_client = self.node_client.clone();
            let db = self.db.as_ref().unwrap().clone();
            scheduler
                .add(Job::new_async(
                    self.node_polling_schedule.clone().unwrap(),
                    move |_uuid, _l| {
                        Box::pin({
                            let route = route.clone();
                            let mut db = db.clone();
                            let node_client = node_client.clone();
                            async move {
                                let node = if let Ok(node) = node_client.try_lock() {
                                    node
                                } else {
                                    tracing::debug!("Skipping polling event: {}", route);
                                    return;
                                };
                                let (client, ip) = node.as_ref().unwrap();
                                db.create_connection().await.unwrap();
                                let mut count = 0;
                                loop {
                                    count += 1;
                                    if let Ok(data) =
                                        client.get(format!("http://{}{}", ip, route)).send().await
                                    {
                                        match &route {
                                            r if r.starts_with("/kasa") => {
                                                let json =
                                                    data.text().await.unwrap_or("[]".to_string());

                                                let kasa_data: Result<
                                                    Vec<Vec<KasaChildInfo>>,
                                                    serde_json::Error,
                                                > = serde_json::from_str(&json);

                                                match kasa_data {
                                                    Ok(k) => {
                                                        if k.is_empty() {
                                                            break;
                                                        }
                                                        db.push_kasa_data(&k).await.unwrap();
                                                    }
                                                    Err(e) => {
                                                        tracing::warn!(
                                                            "Error parsing data {:#?}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                            _ => {
                                                tracing::warn!("Unhandled {}{}", ip, route);
                                                break;
                                            }
                                        }
                                    } else {
                                        tracing::warn!("Issue with {}{}", ip, route);
                                        break;
                                    }
                                }
                                tracing::debug!("Reqeusted {}x {}", count, route);
                            }
                        })
                    },
                )?)
                .await?;
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
