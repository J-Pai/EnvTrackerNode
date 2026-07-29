//! Google Home service.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::routing;
use axum_oidc_client::auth_cache::AuthCache;
use http::HeaderMap;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::services::db::Db;
use crate::services::google_home::light::DLight;

mod fulfillment;
mod light;

pub(crate) struct GoogleHome {
    db: Db,
    dlight: Arc<RwLock<DLight>>,
}

trait Device {
    fn get_sync_value(&self) -> Value;
    fn get_query_value(&self) -> Value;
    fn execute_actions(&mut self, execution: &Vec<Value>) -> Value;
}

impl GoogleHome {
    pub(crate) fn new(db: Db) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db,
            dlight: Arc::new(RwLock::new(DLight {
                id: "glamp_mock_device".to_string(),
                name: "glamp".to_string(),
                brightness: 100,
                temperature: 2000,
                on: false,
            })),
        })
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
                let devices =
                    HashMap::from([(self.dlight.read().await.id.clone(), self.dlight.clone())]);
                |headers: HeaderMap, json: Json<fulfillment::request::Request>| {
                    Self::google_home_fulfillment_handler(headers, json, db, devices)
                }
            }),
        );

        Ok(router)
    }
}
