//! Logic for setting up the API for a Kasa Node.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;
use tower::BoxError;
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;

use crate::services::kasa::Kasa;

use super::Web;

#[derive(serde::Deserialize, Clone, Debug)]
struct KasaRouteQuery {
    size: Option<usize>,
}

impl Web {
    const DEFAULT_KASA_BATCH_SIZE: usize = 100;
    const PER_BATCH_MILLISECONDS: usize = 2;
    const DEFAULT_KASA_NODE_TIMEOUT_SECONDS: u64 = 10;

    pub(crate) async fn setup_kasa_route(
        mut self,
        devices: &mut Kasa,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let kasa_subscribers: Arc<RwLock<HashMap<String, Arc<RwLock<Subscriber>>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let mut router = self.router;

        let mut children_ids: HashMap<String, Vec<String>> = HashMap::new();

        for device_id in devices.get_devices() {
            kasa_subscribers.write().await.insert(
                device_id.clone(),
                devices
                    .allocate_subscriber(
                        device_id.clone(),
                        TopicOptions {
                            max_messages: Some(Kasa::MAX_MESSAGES),
                            ..Default::default()
                        },
                        ConsumptionMode::Earliest,
                    )
                    .await?,
            );

            children_ids.insert(
                device_id.clone(),
                devices.get_children_ids(device_id.clone()).await?,
            );

            let endpoint = if let Some(endpoint) = devices.get_api(&device_id)? {
                endpoint
            } else {
                String::from("/")
            };

            let topic = devices.get_topic(&device_id)?.clone();

            let subscribers = kasa_subscribers.clone();
            router = router
                .route(
                    &endpoint,
                    routing::get(move |Query(query): Query<KasaRouteQuery>| {
                        let kasa_subscribers = subscribers.clone();

                        async move {
                            let kasa_subscribers = kasa_subscribers.read().await;
                            let subscriber = if let Some(subscriber) = kasa_subscribers.get(&topic)
                            {
                                subscriber.read().await
                            } else {
                                return "[]".to_string();
                            };

                            tracing::debug!("Query: {:?}", query);

                            let msg = match timeout(
                                Duration::from_millis(
                                    query.size.unwrap_or(Self::DEFAULT_KASA_BATCH_SIZE + 1) as u64
                                        * Self::PER_BATCH_MILLISECONDS as u64,
                                ),
                                subscriber
                                    .recv_batch(query.size.unwrap_or(Self::DEFAULT_KASA_BATCH_SIZE)),
                            )
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
                .layer(
                    ServiceBuilder::new()
                        .layer(HandleErrorLayer::new(|_: BoxError| async {
                            StatusCode::REQUEST_TIMEOUT
                        }))
                        .layer(TimeoutLayer::new(Duration::from_secs(
                            Web::DEFAULT_KASA_NODE_TIMEOUT_SECONDS,
                        ))),
                );
        }

        self.router = router;

        Ok(self)
    }
}
