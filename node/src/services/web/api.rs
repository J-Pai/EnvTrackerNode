//! Logic for handling API calls from the frontend (or other clients).

use crate::config::ApiServerConfig;
use crate::config::KasaDeviceConfig;
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
                NodeClass::KasaDevice(topic, config, _) => {
                    web.setup_kasa_api_route(topic, config).await?
                }

                NodeClass::Unknown => continue,
            };
        }

        Ok(web)
    }

    async fn setup_kasa_api_route(
        self,
        topic: &String,
        config: &KasaDeviceConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self)
    }
}
