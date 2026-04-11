//! General web server.

use axum::{Router, routing};

mod config;
mod kasa;
mod setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup::venv().await?;
    let config = config::SysConfig::new();
    kasa::handlers(&config).await?;

    // build our application with a single route
    let app = Router::new().route("/", routing::get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
