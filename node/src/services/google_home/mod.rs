//! Google Home service.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::routing;
use axum_oidc_client::auth_cache::AuthCache;
use gcp_auth::CustomServiceAccount;
use gcp_auth::TokenProvider;
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
    google_home_service_account: Arc<dyn TokenProvider>,
}

trait Device {
    fn get_sync_value(&self) -> Value;
    fn get_query_value(&self) -> Value;
    fn execute_actions(&mut self, execution: &Vec<Value>) -> Value;
}

impl GoogleHome {
    pub(crate) fn new(db: Db) -> Result<Self, Box<dyn std::error::Error>> {
        let credentials_path =
            PathBuf::from("/home/jpai/.config/envtrackernode/google_home_api_service_account.json");
        let service_account = CustomServiceAccount::from_file(credentials_path)?;
        Ok(Self {
            db,
            dlight: Arc::new(RwLock::new(DLight {
                id: "glamp_mock_device".to_string(),
                name: "glamp".to_string(),
                brightness: 100,
                temperature: 2000,
                on: false,
            })),
            google_home_service_account: Arc::new(service_account),
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
                let service_account = self.google_home_service_account.clone();
                |headers: HeaderMap, json: Json<fulfillment::request::Request>| {
                    Self::google_home_fulfillment_handler(
                        headers,
                        json,
                        db,
                        service_account,
                        devices,
                    )
                }
            }),
        );

        Ok(router)
    }
}
