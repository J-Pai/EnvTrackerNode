//! Google Home service.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::routing;
use axum_oidc_client::auth_cache::AuthCache;
use gcp_auth::CustomServiceAccount;
use http::HeaderMap;
use serde_json::Value;
use tokio::sync::Mutex;
use url::Url;

use crate::services::db::Db;
use crate::services::google_home::light::DLight;

mod fulfillment;
mod light;

pub(crate) struct GoogleHome {
    db: Option<Db>,
    google_home_service_account: Option<PathBuf>,
}

trait Device {
    fn get_id(&self) -> String;
    async fn get_sync_value(&mut self) -> Value;
    async fn get_query_value(&mut self) -> Value;
    async fn execute_actions(&mut self, execution: &[Value]) -> Value;
}

impl GoogleHome {
    pub(crate) async fn new(db: Option<Db>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db,
            google_home_service_account: Some(PathBuf::from(
                "/home/jpai/.config/envtrackernode/google_home_api_service_account.json",
            )),
        })
    }

    pub(crate) async fn setup_route(
        &self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        if let Some(db) = self.db.clone()
            && let Some(path) = self.google_home_service_account.clone()
        {
            let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(db);
            let service_account = Arc::new(CustomServiceAccount::from_file(path)?);

            let dlight = Arc::new(Mutex::new(
                DLight::new(Url::from_str("http://192.168.86.22:3333")?).await?,
            ));

            let devices = HashMap::from([(dlight.lock().await.get_id(), dlight.clone())]);

            router = router.route(
                "/google_home/fulfillment",
                routing::post({
                    let db = cache.clone();
                    |headers: HeaderMap, json: Json<fulfillment::request::Request>| {
                        Self::fulfillment_handler(headers, json, db, service_account, devices)
                    }
                }),
            );
        }

        Ok(router)
    }
}
