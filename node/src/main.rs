//! Entrypoint for services.

use std::sync::LazyLock;

use tokio_memq::MessageQueue;

mod config;
mod kasa;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    static MQ: LazyLock<MessageQueue> = LazyLock::new(|| {
        MessageQueue::new()
    });
    let config = config::SysConfig::new();
    kasa::handler(&config, &MQ).await?;
    web::server(&config, &MQ).await?;
    Ok(())
}
