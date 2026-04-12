//! Sets up the web services.

use std::time::SystemTime;

use axum::Router;
use axum::routing;
use chrono::DateTime;
use chrono::Local;
use tokio::sync::Mutex;
use tokio_memq::MessageSubscriber;
use tokio_memq::Subscriber;

use crate::config::SysConfig;

pub(crate) async fn server(
    _config: &SysConfig,
    subscriber: &'static Mutex<Option<Subscriber>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", routing::get(|| async {
        let system_time = SystemTime::now();
        let datetime: DateTime<Local> = system_time.into();
        println!("[{}] get recv", datetime.format("%d/%m/%Y %T"));
        println!("[{}] subbed", datetime.format("%d/%m/%Y %T"));

        let sub_lock = subscriber.lock().await;
        let sub = sub_lock.as_ref().unwrap();
        let msg = sub.recv_batch(10).await.unwrap();
        println!("[{}] received", datetime.format("%d/%m/%Y %T"));
        let mut output: String = "".to_owned();
        for (i, m) in msg.iter().enumerate() {
            output.push_str(format!("{}. {}\n", i, m.deserialize::<String>().unwrap()).as_str());
        }
        println!("[{}] sending", datetime.format("%d/%m/%Y %T"));
        format!("Hello, World!\n{}", output)
    }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
