//! Logic for handling API calls from the frontend (or other clients).

use crate::config::ApiServerConfig;

use super::Web;

impl Web {
    pub(crate) async fn setup_api_route(
        self,
        _config: &ApiServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self)
    }
}
