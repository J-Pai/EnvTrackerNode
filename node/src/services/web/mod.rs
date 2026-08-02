//! Sets up the web services.

use axum::Router;

use crate::services::auth::Auth;
use crate::services::db::Db;
use crate::services::google_home::GoogleHome;
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

    pub(crate) async fn start(self, poller: Poller) -> Result<(), Box<dyn std::error::Error>> {
        poller.start().await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.router).await?;
        Ok(())
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let google_home = GoogleHome::new(self.db.clone()).await?;
        self.router = google_home.setup_route(self.router).await?;
        self.google_home = Some(google_home);
        Ok(self)
    }
}
