//! Logic for serving the frontend.

use axum::body::Body;
use axum::extract::Path;
use axum::extract::Request;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing;
use notify::Watcher;
use tokio::io::AsyncReadExt;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

use crate::config::FrontendServerConfig;

use super::Web;

impl Web {
    pub(crate) async fn setup_frontend_route(
        mut self,
        config: &FrontendServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let router = self.router;
        let base = config.get_base().unwrap_or("".to_string());

        let mut index_file = String::new();
        tokio::fs::File::open("dist/index.html")
            .await?
            .read_to_string(&mut index_file)
            .await?;
        let serve_dir = ServeDir::new("dist");

        let update_index_file = async move |_request: Request| -> Response {
            let index_file = index_file
                .clone()
                .replace("href=\"/", format!("href=\"{}/", base).as_str())
                .replace("'/", format!("'{}/", base).as_str());

            let body = Body::new(index_file);

            let headers = [
                (header::CONTENT_TYPE, "text/html, charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "inline; filename=\"index.html\"",
                ),
            ];

            (headers, body).into_response()
        };

        self.router = router
            .route("/", routing::get(update_index_file))
            .fallback_service(serve_dir);

        #[cfg(debug_assertions)]
        {
            let router = self.router;
            let livereload = LiveReloadLayer::new();
            let reloader = livereload.reloader();
            self.router = router.layer(livereload);
            let mut watcher = notify::recommended_watcher(move |event: Result<_, _>| {
                if event.is_ok_and(|evt: notify::Event| !evt.kind.is_access()) {
                    tracing::debug!("Detected site update.");
                    reloader.reload();
                }
            })?;
            watcher.watch(
                std::path::Path::new("dist"),
                notify::RecursiveMode::Recursive,
            )?;
            self.watcher.replace(watcher);
        }

        Ok(self)
    }
}
