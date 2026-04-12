//! Sets up the web services.

use std::time::Duration;

use axum::Router;
use axum::routing;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;

use crate::config::SysConfig;

pub(crate) async fn server(
    _config: &SysConfig,
    subscriber: &'static Mutex<Option<Subscriber>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route(
        "/",
        routing::get(|| async {
            let sub_lock = subscriber.lock().await;
            let sub = sub_lock.as_ref().unwrap();
            let current_offset = sub.current_offset().await.unwrap();

            let msg = match timeout(Duration::from_millis(100), sub.recv_batch(100)).await {
                Ok(result) => result.unwrap(),
                Err(_) => {
                    return format!("Hello, World! [{}]", current_offset);
                }
            };
            let current_offset = sub.current_offset().await.unwrap();
            let mut output: String = "".to_owned();
            for (i, m) in msg.iter().enumerate() {
                output
                    .push_str(format!("{}. {}\n", i, m.deserialize::<String>().unwrap()).as_str());
            }
            format!("Hello, World! [{}] \n{}", current_offset, output)
        }),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
