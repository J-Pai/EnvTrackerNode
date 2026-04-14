//! Traits used across the project.

use tokio::sync::RwLock;
use tokio_memq::ConsumptionMode;
use tokio_memq::MessageQueue;
use tokio_memq::Subscriber;
use tokio_memq::TopicOptions;

pub trait Subscribale {
    async fn allocate_subscriber(
        &mut self,
        mq: &'static RwLock<Option<MessageQueue>>,
        options: TopicOptions,
        mode: ConsumptionMode,
    ) -> Result<(&'static RwLock<Vec<RwLock<Subscriber>>>, usize), Box<dyn std::error::Error>>;
}
