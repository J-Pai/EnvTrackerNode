//! Handler for Google Home fulfillment requests.

use axum::response::IntoResponse;
use http::HeaderMap;
use http::StatusCode;

use crate::services::auth::Auth;

impl Auth {
    pub(super) async fn google_home_fulfillment_handler(headers: HeaderMap) -> impl IntoResponse {
        tracing::info!("HEADERS : {headers:#?}");
        return StatusCode::UNAUTHORIZED.into_response();
    }
}
