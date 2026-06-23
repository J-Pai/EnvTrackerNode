//! Sets up the web services.

use axum::Router;

use crate::services::db::Db;
use crate::services::poller::Poller;

mod api;
mod frontend;
mod kasa_node;

pub(crate) struct Web {
    router: Router,
    db: Option<Db>,
    #[cfg(debug_assertions)]
    watcher: Option<notify::RecommendedWatcher>,
}

impl Web {
    pub(crate) fn new(db: Option<Db>) -> Self {
        Self {
            router: Router::new(),
            db,
            #[cfg(debug_assertions)]
            watcher: None,
        }
    }

    pub(crate) async fn start(self, poller: Poller) -> Result<(), Box<dyn std::error::Error>> {
        poller.start().await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
