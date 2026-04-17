//! Sets up the web services.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::routing;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

use crate::config::SysConfig;
use crate::kasa::Kasa;

pub(crate) async fn server(
    config: &SysConfig,
    devices: &mut Option<Kasa>,
    mq: Arc<RwLock<MessageQueue>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Router::new();

    let kasa_subscribers: Arc<RwLock<HashMap<String, Arc<RwLock<Subscriber>>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    for device in config.get_kasa_devices().unwrap_or(HashMap::new()) {
        let device = device.clone();
        kasa_subscribers.write().await.insert(
            device.0.clone(),
            devices
                .as_mut()
                .unwrap()
                .allocate_subscriber(
                    device.0.clone(),
                    TopicOptions::default(),
                    ConsumptionMode::Earliest,
                )
                .await?,
        );
    }

    if !kasa_subscribers.read().await.is_empty() {
        app = app.route(
            "/kasa/{topic}",
            routing::get(move |Path(topic): Path<String>| {
                let kasa_subscribers = kasa_subscribers.clone();

                async move {
                    let kasa_subscribers = kasa_subscribers.read().await;
                    let subscriber = kasa_subscribers.get(&topic).unwrap().read().await;
                    let current_offset = subscriber.current_offset().await.unwrap();

                    let mq_lock = mq.read().await;
                    let mq_stats = mq_lock.get_topic_stats(topic).await.unwrap();

                    let msg = match timeout(Duration::from_millis(100), subscriber.recv_batch(100))
                        .await
                    {
                        Ok(result) => result.unwrap(),
                        Err(_) => {
                            return format!(
                                "Hello, World! [{}, {}, {}]",
                                current_offset, mq_stats.total_payload_size, mq_stats.message_count
                            );
                        }
                    };

                    let current_offset = subscriber.current_offset().await.unwrap();
                    let mut output: String = "".to_owned();
                    for (i, m) in msg.iter().enumerate() {
                        let json = m.deserialize::<Value>().unwrap();
                        output.push_str(format!("{}. {}\n", i, json).as_str());
                    }

                    format!(
                        "Hello, World! [{}, {}, {}] \n{}",
                        current_offset, mq_stats.total_payload_size, mq_stats.message_count, output
                    )
                }
            }),
        );
    }

    let index_service = ServeFile::new("dist/index.html");
    let serve_dir = ServeDir::new("dist").not_found_service(index_service.clone());
    app = app
        .route(
            "/",
            routing::get(|request: Request| async {
                let service = index_service;
                let result = service.oneshot(request).await;
                result
            }),
        )
        .fallback_service(serve_dir);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}
