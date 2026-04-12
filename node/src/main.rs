//! Entrypoint for services.

use tokio::sync::Mutex;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

mod config;
mod kasa;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    static MQ: Mutex<Option<MessageQueue>> = Mutex::const_new(None);
    {
        let mut mq_lock = MQ.lock().await;
        mq_lock.replace(MessageQueue::new());
    }
    let config = config::SysConfig::new();

    let mq = kasa::handler(&config, &MQ).await?;

    static SUB: Mutex<Option<Subscriber>> = Mutex::const_new(None);
    {
        let mq_lock = mq.lock().await;
        let mq = mq_lock.as_ref().unwrap();
        let mut sub_lock = SUB.lock().await;
        sub_lock.replace(
            mq.subscriber_with_options_and_mode(
                "kasa".to_string(),
                TopicOptions::default(),
                ConsumptionMode::Earliest,
            )
            .await
            .unwrap(),
        );
    }

    web::server(&config, &SUB).await?;

    Ok(())
}
