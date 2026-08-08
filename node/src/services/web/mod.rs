//! Sets up the web services.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use url::Url;

use crate::services::auth::Auth;
use crate::services::db::Db;
use crate::services::google_home::{GoogleHome, SupportedDevices};
use crate::services::poller::Poller;

mod api;
mod frontend;
mod kasa_node;

pub(crate) struct Web {
    router: Router,
    db: Option<Db>,
    auth: Option<Auth>,
    google_home: Option<GoogleHome>,
    #[cfg(debug_assertions)]
    watcher: Option<notify::RecommendedWatcher>,
}

impl Web {
    pub(crate) fn new(db: Option<Db>) -> Self {
        Self {
            router: Router::new(),
            db,
            auth: None,
            google_home: None,
            #[cfg(debug_assertions)]
            watcher: None,
        }
    }

    pub(crate) async fn setup_auth_route(
        mut self,
        mut auth: Auth,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.router = auth.setup_route(self.router).await?;
        self.auth = Some(auth);
        Ok(self)
    }

    pub(crate) async fn setup_google_home_route(
        mut self,
        node_api_uri: Option<Url>,
        agent_user_id: Option<String>,
        service_account: Option<PathBuf>,
        devices: Option<HashMap<String, Arc<Mutex<SupportedDevices>>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let google_home = GoogleHome::new(
            self.db.clone(),
            agent_user_id,
            service_account,
            node_api_uri,
        )
        .await?;
        self.router = google_home.setup_route(self.router, devices).await?;
        self.google_home = Some(google_home);
        Ok(self)
    }

    pub(crate) async fn start(self, poller: Poller) -> Result<(), Box<dyn std::error::Error>> {
        poller.start().await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        let mut router = self.router;
        router = router.layer(TraceLayer::new_for_http());
        axum::serve(listener, router).await?;
        Ok(())
    }
}
