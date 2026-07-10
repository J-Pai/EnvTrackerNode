//! Authentication handlers.

use std::path::PathBuf;

use axum::extract::Query;
use axum::http::Uri;
use axum::response::IntoResponse;
use axum::routing;
use axum_oidc::EmptyAdditionalClaims;
use axum_oidc::OidcClaims;
use axum_oidc::OidcClient;

use crate::services::web::Web;

#[derive(serde::Deserialize, Clone, Debug)]
struct SampleQuery {
    flag: Option<bool>,
}

impl Web {
    async fn random_path_handler(
        claims: Result<OidcClaims<EmptyAdditionalClaims>, axum_oidc::error::ExtractorError>,
        Query(query): Query<SampleQuery>,
    ) -> impl IntoResponse {
        if let Ok(claims) = claims {
            format!("Hello World {query:#?}")
        } else {
            format!("Goodbye World {query:#?}")
        }
    }

    pub(crate) async fn setup_auth(
        mut self,
        oauth2_client_json: &PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = self.router;

        router = router.route("/random_path", routing::get(Self::random_path_handler));

        self.router = router;
        Ok(self)
    }
}
