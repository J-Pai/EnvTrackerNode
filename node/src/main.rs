//! Entrypoint for services.

mod config;
mod kasa;
mod setup;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup::venv().await?;
    let config = config::SysConfig::new();
    kasa::handler(&config).await?;
    web::server(&config).await?;
    Ok(())
}
