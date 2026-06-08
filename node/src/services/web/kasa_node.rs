//! Logic for setting up the API for a Kasa Node.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Path;
use axum::routing;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

use crate::services::kasa::Kasa;

use super::Web;

impl Web {
    pub(crate) async fn setup_kasa_route(
        mut self,
        devices: &mut Kasa,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let kasa_subscribers: Arc<RwLock<HashMap<String, Arc<RwLock<Subscriber>>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let router = self.router;

        let mut children_ids: HashMap<String, Vec<String>> = HashMap::new();

        for device_id in devices.get_devices() {
            kasa_subscribers.write().await.insert(
                device_id.clone(),
                devices
                    .allocate_subscriber(
                        device_id.clone(),
                        TopicOptions::default(),
                        ConsumptionMode::Earliest,
                    )
                    .await?,
            );

            children_ids.insert(
                device_id.clone(),
                devices.get_children_ids(device_id.clone()).await?,
            );
        }

        self.router = router
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
            );

        Ok(self)
    }
}
