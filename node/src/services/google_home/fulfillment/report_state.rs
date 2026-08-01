//! Handles reporting state for Googe Home QUERY and EXECUTE.

use std::sync::Arc;

use gcp_auth::TokenProvider;

use crate::services::google_home::GoogleHome;
use crate::services::google_home::fulfillment::request::Intent;
use crate::services::google_home::fulfillment::response::Response;

impl GoogleHome {
    pub(super) async fn _report_state(
        google_home_service_account: Arc<dyn TokenProvider>,
        _user_agent_id: String,
        _intent: Intent,
        _response: Response,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _token = google_home_service_account
            .token(&["https://www.googleapis.com/auth/homegraph"])
            .await
            .map_err(|e| {
                tracing::error!("Google Home API Authorization Token Failure: {e}");
                e
            })?;

        Ok(())
    }
}
