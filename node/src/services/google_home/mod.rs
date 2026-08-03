//! Google Home service.

use std::collections::HashMap;
use std::path::PathBuf;
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
    service_account: Option<PathBuf>,
    node_uri: Option<Url>,
}

trait Device {
    fn get_id(&self) -> String;
    async fn get_sync_value(&mut self) -> Value;
    async fn get_query_value(&mut self) -> Value;
    async fn execute_actions(&mut self, execution: &[Value]) -> Value;
}

impl GoogleHome {
    pub(crate) async fn new(
        db: Option<Db>,
        service_account: Option<PathBuf>,
        node_uri: Option<Url>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db,
            service_account,
            node_uri,
        })
    }

    pub(crate) async fn setup_route(
        &self,
        mut router: Router,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        if let Some(db) = self.db.clone()
            && let Some(path) = self.service_account.clone()
            && let Some(uri) = self.node_uri.clone()
        {
            let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(db);
            let service_account = Arc::new(CustomServiceAccount::from_file(path)?);

            let dlight = Arc::new(Mutex::new(DLight::new(uri).await?));

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
