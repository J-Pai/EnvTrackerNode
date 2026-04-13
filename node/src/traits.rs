//! Traits used across the project.

use tokio::sync::Mutex;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

pub trait Subscribale {
    async fn allocate_subscriber(
        &mut self,
        mq: &MessageQueue,
        subscribers: &'static Mutex<Vec<Mutex<Option<Subscriber>>>>,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<usize, Box<dyn std::error::Error>>;
}
