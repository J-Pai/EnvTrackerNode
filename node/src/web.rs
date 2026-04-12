//! Sets up the web services.

use axum::Router;
use axum::routing;
use tokio::sync::Mutex;
use tokio_memq::MessageQueue;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;

use crate::config::SysConfig;

pub(crate) async fn server(
    _config: &SysConfig,
    mq: &'static Mutex<Option<MessageQueue>>,
    subscriber: &'static Mutex<Option<Subscriber>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", routing::get(|| async {
        let message_count = {
            let mq_lock = mq.lock().await;
            if let  Some(stats) = mq_lock.as_ref().unwrap().get_topic_stats("kasa".to_string()).await {
                stats.message_count
            } else {
                return "Hello, World! No messages! [mq_check]".to_string();
            }
        };
        let sub_lock = subscriber.lock().await;
        let sub = sub_lock.as_ref().unwrap();

        if sub.current_offset().await.unwrap() == message_count {
            return "Hello, World! No messages! [matched path]".to_string();
        }

        let msg = sub.recv_batch(10).await.expect("It's wrong");
        let mut output: String = "".to_owned();
        for (i, m) in msg.iter().enumerate() {
            output.push_str(format!("{}. {}\n", i, m.deserialize::<String>().unwrap()).as_str());
        }
        format!("Hello, World! \n{}", output)
    }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
