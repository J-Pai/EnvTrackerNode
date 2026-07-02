//! Logic for serving the frontend.

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use crate::config::FrontendServerConfig;
use axum::body::Bytes;
use axum::http::Request;
use axum::http::Response;
use axum::http::StatusCode;
use axum::http::header;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::combinators::UnsyncBoxBody;
use tokio::io::AsyncReadExt;
use tower::Layer;
use tower::Service;
use tower_http::services::ServeDir;

use super::Web;

#[derive(Clone)]
pub struct UpdateBaseUriHtmlService<S> {
    inner: S,
    base: String,
    api_server_ip: String,
    kasa_api: String,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for UpdateBaseUriHtmlService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ResBody: BodyExt<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<UnsyncBoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let response_future = self.inner.call(request);
        let base = self.base.clone();
        let api_server_ip = self.api_server_ip.clone();
        let kasa_api = self.kasa_api.clone();

        Box::pin(async move {
            let response = response_future.await?;

            let is_html = response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("text/html"))
                .unwrap_or(false);

            if !is_html {
                return Ok(response.map(|b| b.map_err(|e| e.into()).boxed_unsync()));
            }

            let (mut parts, body) = response.into_parts();

            parts.headers.remove(header::CONTENT_LENGTH);

            let bytes = match body.collect().await {
                Ok(c) => c.to_bytes(),
                Err(_e) => {
                    tracing::error!("Failed to collect response body for base uri replacement.");
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::from("").map_err(|e| e.into()).boxed_unsync())
                        .unwrap());
                }
            };

            let content = match String::from_utf8(bytes.to_vec()) {
                Ok(content) => content,
                Err(_e) => {
                    tracing::error!("HTML content is not utf8.");
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::from("").map_err(|e| e.into()).boxed_unsync())
                        .unwrap());
                }
            };

            // Replaces
            // `<link id="api" href="/"` with
            // `<link id="api" href="/{api_server_ip}/"` if base if provided.
            let content = content.replace(
                "<link id=\"api\" href=\"",
                format!("<link id=\"api\" href=\"{api_server_ip}").as_str(),
            );

            // Replaces
            // `<link id="kasa_api" href="/"` with
            // `<link id="kasa_api" href="/{base}/"` if base if provided.
            let content = content.replace(
                "<link id=\"api\" href=\"",
                format!("<link id=\"api\" href=\"{kasa_api}").as_str(),
            );

            let content = if !base.is_empty() {
                // Replaces
                // `<base id="base" href="/"` with
                // `<base id="base" href="/{base}/"` if base if provided.
                //
                // Replaces
                // `<script data-event-stream="/_tower-livereload`
                // with
                // `<script data-event-stream="/{base}/_tower-livereload` if base if provided.
                //
                // Replaces wasm script block.
                content
                    .replace("href=\"/", format!("href=\"{base}/").as_str())
                    .replace(
                        "<script data-event-stream=\"/_tower-livereload",
                        format!("<<script data-event-stream=\"{}/_tower-livereload", base).as_str(),
                    )
                    .replace("'/", format!("'{}/", base).as_str())
            } else {
                content
            };

            let new_body = Full::new(Bytes::from(content))
                .map_err(|_e| unreachable!("Full body never errors"))
                .boxed_unsync();

            Ok(Response::from_parts(parts, new_body))
        })
    }
}

#[derive(Clone)]
pub struct UpdateBaseUrilHtmlLayer {
    base: String,
    api_server_ip: String,
    kasa_api: String,
}

impl<S> Layer<S> for UpdateBaseUrilHtmlLayer {
    type Service = UpdateBaseUriHtmlService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UpdateBaseUriHtmlService {
            inner,
            base: self.base.clone(),
            api_server_ip: self.api_server_ip.clone(),
            kasa_api: self.kasa_api.clone(),
        }
    }
}

impl Web {
    pub(crate) async fn setup_frontend_route(
        mut self,
        config: &FrontendServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router;

        let mut index_file = String::new();
        tokio::fs::File::open("dist/index.html")
            .await?
            .read_to_string(&mut index_file)
            .await?;
        let serve_dir = ServeDir::new("dist");

        router = router.fallback_service(serve_dir);

        #[cfg(debug_assertions)]
        {
            use notify::Watcher;
            use tower_livereload::LiveReloadLayer;

            let livereload = LiveReloadLayer::new();
            let reloader = livereload.reloader();
            router = router.layer(livereload);
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

        let base = config.get_base().unwrap_or(String::new());
        let api_server_ip = config.get_api_server_ip().to_string();
        let kasa_api = config.get_kasa_api().to_string();

        self.router = router.layer(UpdateBaseUrilHtmlLayer {
            base,
            api_server_ip,
            kasa_api,
        });

        Ok(self)
    }
}
