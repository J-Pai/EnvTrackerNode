//! Google Home service.

use std::sync::Arc;

use axum::{Json, Router, routing};
use axum_oidc_client::auth_cache::AuthCache;
use http::HeaderMap;

use crate::services::db::Db;
use crate::services::google_home::fulfillment::FulfillmentRequest;

mod fulfillment;

pub(crate) struct GoogleHome {
    db: Db,
}

impl GoogleHome {
    pub(crate) fn new(db: Db) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { db })
    }

    pub(crate) async fn setup_route(
        &self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(self.db.clone());

        router = router.route(
            "/google_home/fulfillment",
            routing::post({
                let db = cache.clone();
                |headers: HeaderMap, json: Json<FulfillmentRequest>| {
                    Self::google_home_fulfillment_handler(headers, json, db)
                }
            }),
        );

        Ok(router)
    }
}
