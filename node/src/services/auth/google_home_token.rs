//! Handler for Google Home token requests.

use axum::Form;
use axum::response::IntoResponse;
use axum::response::Redirect;
use url::Url;

use crate::services::auth::Auth;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct OAuth2TokenRequest {
    client_id: Option<String>,
    client_secret: Option<String>,
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    refresh_token: Option<String>,
}

impl Auth {
    pub(super) async fn google_home_token_handler(
        Form(params): Form<OAuth2TokenRequest>,
        base: Url,
    ) -> impl IntoResponse {
        tracing::info!("TOKEN ENDPOINT\n{params:#?}");

        Redirect::to(format!("{base}google_home").as_str()).into_response()
    }
}
