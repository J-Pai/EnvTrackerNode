//! General web server.

mod config;
mod kasa;
mod setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup::venv().await?;
    let config = config::SysConfig::new();
    kasa::handlers(&config).await?;
    Ok(())
}
