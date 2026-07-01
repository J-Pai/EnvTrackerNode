//! Logic for handling API calls from the frontend (or other clients).

use std::time::Duration;

use axum::BoxError;
use axum::error_handling::HandleErrorLayer;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing;
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;

use crate::config::ApiServerConfig;
use crate::config::NodeClass;
use crate::services::db::DeviceQuery;
use crate::services::db::QueryResult;

use super::Web;

impl Web {
    const DEFAULT_API_TIMEOUT_SECONDS: u64 = 10;

    pub(crate) async fn setup_api_route(
        self,
        config: &ApiServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut web = self;
        for node in config.get_nodes() {
            web = match node {
                NodeClass::KasaDevice(topic, _, _) => web.setup_kasa_api_route(topic).await?,

                NodeClass::Unknown => continue,
            };
        }

        Ok(web)
    }

    async fn setup_kasa_api_route(
        mut self,
        topic: &String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router;
        let db = self.db.as_ref().unwrap().clone();
        let topic = topic.clone();

        router = router
            .route(
                &format!("/api/kasa/{topic}"),
                routing::get(move |Query(query): Query<DeviceQuery>| async move {
                    match db.query_kasa_data(&topic, &query).await {
                        Ok(data) => {
                            tracing::debug!("Query complete: {:#?}", query);

                            let data = match data {
                                QueryResult::KasaDeviceInfo(data) => serde_json::to_string(&data),
                                QueryResult::Distinct(data) => serde_json::to_string(&data),
                            };

                            match data {
                                Ok(data) => data,
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to serialize data ({:#?}): {:#?}",
                                        query,
                                        e
                                    );
                                    format!("[]")
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to query data ({:#?}): {:#?}", query, e);
                            format!("[]")
                        }
                    }
                }),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(|_: BoxError| async {
                        StatusCode::REQUEST_TIMEOUT
                    }))
                    .layer(TimeoutLayer::new(Duration::from_secs(
                        Web::DEFAULT_API_TIMEOUT_SECONDS,
                    ))),
            );

        self.router = router;
        Ok(self)
    }
}
