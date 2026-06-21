//! Logic for handling API calls from the frontend (or other clients).

use axum::routing;

use crate::config::ApiServerConfig;
use crate::config::NodeClass;

use super::Web;

impl Web {
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

        router = router.route(
            &format!("/api/kasa/{}", topic),
            routing::get(move || async move { format!("[]") }),
        );

        self.router = router;
        Ok(self)
    }
}
