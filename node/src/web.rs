//! Sets up the web services.

use axum::Router;
use axum::routing;

use crate::config::SysConfig;

pub(crate) async fn server(_config: &SysConfig) -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", routing::get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
