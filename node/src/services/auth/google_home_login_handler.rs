//! Handler for Login Page.
//!
//! Temporary until the main landing page is able to handle logins.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::OriginalUri;
use axum::extract::Query;
use axum::response::Html;
use axum::response::IntoResponse;
use axum_oidc_client::extractors::OptionalAuthSession;
use axum_oidc_client::jwt::DecodingKey;
use http::StatusCode;
use tokio::sync::RwLock;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;
use crate::services::auth::OAuth2AuthRequest;

impl Auth {
    pub(super) async fn google_home_login_handler(
        OptionalAuthSession(session): OptionalAuthSession,
        Query(query): Query<OAuth2AuthRequest>,
        OriginalUri(uri): OriginalUri,
        base: Url,
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        let base_path = base.path().to_string().clone();
        let (decoded_query, encoded_query) = Self::stringify_query(&uri);
        match session {
            Some(session) => {
                let expires = session
                    .expires
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "(no expiry)".to_string());

                let Ok(email) = Self::validate_session(certs, &session, &[client_json.client_id])
                    .await
                    .map_err(|e| {
                        tracing::error!("JWT validation failed: {e} - {}", session.id_token)
                    })
                else {
                    return (StatusCode::UNAUTHORIZED, Html("Unauthorized".to_string()))
                        .into_response();
                };

                tracing::info!("JWT data verified {email:#?}");

                (
                    StatusCode::OK,
                    Html(format!(
                        r#"
                            Hello World for Google Home ({email}): expires {expires}
                            <br />
                            <a href='{base}google_home/link{decoded_query}'>Authorize Link</a>
                            <br />
                            <a href='{base}auth/logout'>Logout</a>
                            <br />
                            <pre><code>{query:#?}</code></pre>
                        "#,
                    )),
                )
                    .into_response()
            }
            None => (
                StatusCode::OK,
                Html(format!(
                    r#"
                    <a href='{base}auth?redirect={base_path}google_home{encoded_query}'>
                        GOOGLE LOGIN
                    </a>
                    "#,
                )),
            )
                .into_response(),
        }
    }
}
