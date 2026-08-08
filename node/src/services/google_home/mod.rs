//! Google Home service.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::routing;
use axum_oidc_client::auth_cache::AuthCache;
use gcp_auth::CustomServiceAccount;
use http::HeaderMap;
use serde_json::Value;
use tokio::sync::Mutex;
use url::Url;

use crate::services::db::Db;
use crate::services::google_home::light::DLight;
use crate::services::google_home::plug::Wemo;
use crate::services::google_home::report_state::ReportStateParams;

mod device;
mod fulfillment;
pub(crate) mod light;
pub(crate) mod plug;
mod report_state;

pub(crate) struct GoogleHome {
    db: Option<Db>,
    agent_user_id: Option<String>,
    service_account: Option<PathBuf>,
    node_uri: Option<Url>,
}

pub(crate) trait Device {
    fn get_id(&self) -> String;
    async fn get_sync_value(&mut self) -> Value;
    async fn get_query_value(&mut self) -> Value;
    async fn execute_actions(&mut self, execution: &[Value]) -> Value;
}

pub(crate) enum SupportedDevices {
    DLight(DLight),
    Wemo(Wemo),
}

impl Device for SupportedDevices {
    fn get_id(&self) -> String {
        match self {
            SupportedDevices::DLight(device) => device.get_id(),
            SupportedDevices::Wemo(device) => device.get_id(),
        }
    }

    async fn get_sync_value(&mut self) -> Value {
        match self {
            SupportedDevices::DLight(device) => device.get_sync_value().await,
            SupportedDevices::Wemo(device) => device.get_sync_value().await,
        }
    }

    async fn get_query_value(&mut self) -> Value {
        match self {
            SupportedDevices::DLight(device) => device.get_query_value().await,
            SupportedDevices::Wemo(device) => device.get_query_value().await,
        }
    }

    async fn execute_actions(&mut self, execution: &[Value]) -> Value {
        match self {
            SupportedDevices::DLight(device) => device.execute_actions(execution).await,
            SupportedDevices::Wemo(device) => device.execute_actions(execution).await,
        }
    }
}

impl GoogleHome {
    pub(crate) async fn new(
        db: Option<Db>,
        agent_user_id: Option<String>,
        service_account: Option<PathBuf>,
        node_uri: Option<Url>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            agent_user_id,
            db,
            service_account,
            node_uri,
        })
    }

    pub(crate) async fn setup_route(
        &self,
        mut router: Router,
        devices: Option<HashMap<String, Arc<Mutex<SupportedDevices>>>>,
    ) -> Result<Router, Box<dyn std::error::Error>> {
        if let Some(db) = self.db.clone()
            && let Some(uri) = self.node_uri.clone()
        {
            let cache: Arc<dyn AuthCache + Send + Sync> = Arc::new(db);

            router = router.route(
                "/google_home/fulfillment",
                routing::post({
                    let db = cache.clone();
                    |headers: HeaderMap, json: Json<fulfillment::request::Request>| {
                        Self::fulfillment_handler(headers, json, db, uri)
                    }
                }),
            );
        } else if let Some(devices) = devices
            && let Some(agent_user_id) = self.agent_user_id.clone()
            && let Some(path) = self.service_account.clone()
        {
            let service_account = Arc::new(CustomServiceAccount::from_file(path)?);
            router = router
                .route(
                    "/google_home/device",
                    routing::get({
                        let devices = devices.clone();
                        |device_ids: Query<Vec<(String, String)>>| {
                            Self::device_get_handler(devices, Some(device_ids), None)
                        }
                    }),
                )
                .route(
                    "/google_home/device/{id}",
                    routing::post({
                        let devices = devices.clone();
                        |device: Path<String>, actions: Json<Vec<Value>>| {
                            Self::device_post_handler(devices, device, actions)
                        }
                    }),
                )
                .route(
                    "/google_home/device/report_state",
                    routing::post({
                        let devices = devices.clone();
                        let agent_user_id = agent_user_id.clone();
                        |query: Json<ReportStateParams>| {
                            Self::device_report_state_handler(
                                query,
                                agent_user_id,
                                service_account,
                                devices,
                            )
                        }
                    }),
                );
        }

        Ok(router)
    }
}
