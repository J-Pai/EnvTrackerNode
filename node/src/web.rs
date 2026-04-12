//! Sets up the web services.

use axum::Router;
use axum::routing;
use tokio_memq::MessageQueue;
use tokio_memq::MessageSubscriber;

use crate::config::SysConfig;

pub(crate) async fn server(
    _config: &SysConfig,
    mq: &'static MessageQueue,
) -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", routing::get(|| async {
        let sub = mq.subscriber("kasa".to_string()).await.unwrap();
        let msg = sub.recv_batch(10).await.unwrap();

        let mut output: String = "".to_owned();

        for (i, m) in msg.iter().enumerate() {
            output.push_str(format!("{}. {}\n", i, m.deserialize::<String>().unwrap()).as_str());
        }

        format!("Hello, World!\n{}", output)
    }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
