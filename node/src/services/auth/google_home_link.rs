//! Handler for initiating link with Google Home.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::OriginalUri;
use axum::extract::Query;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::jwt::DecodingKey;
use http::StatusCode;
use tokio::sync::RwLock;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ClientJsonWeb;
use crate::services::db::Db;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(super) struct OAuth2AuthRequest {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    state: Option<String>,
    scope: Option<String>,
    response_type: Option<String>,
}

impl Auth {
    pub(super) async fn google_home_link_handler(
        session: AuthSession,
        Query(query): Query<OAuth2AuthRequest>,
        OriginalUri(uri): OriginalUri,
        base: Url,
        certs: Arc<RwLock<HashMap<String, DecodingKey>>>,
        db: Db,
        client_json: ClientJsonWeb,
        google_home_client_json: ClientJsonWeb,
    ) -> impl IntoResponse {
        let Ok(_) = Self::validate_session(certs, &session, &[client_json.client_id])
            .await
            .map_err(|e| tracing::error!("JWT validation failed: {e} - {}", session.id_token))
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let (decoded_query, encoded_query) = Self::stringify_query(&uri);

        tracing::info!("{decoded_query}\n{encoded_query}\n{query:#?}");

        let client_id = if let Some(client_id) = query.client_id.clone()
            && client_id == google_home_client_json.client_id
        {
            client_id
        } else {
            return (
                StatusCode::OK,
                Html(format!(
                    r#"
                        <pre><code>{query:#?}</code></pre>
                        "#,
                )),
            )
                .into_response();
        };

        let redirect_uri = if let Some(redirect_uri) = query.redirect_uri
            && let Ok(redirect_uri) = Url::parse(&redirect_uri)
            && redirect_uri.path() == format!("/r/{}", google_home_client_json.project_id)
            && let Some(host) = redirect_uri.host_str()
            && (host == "oauth-redirect.googleusercontent.com"
                || host == "oauth-redirect-sandbox.googleusercontent.com")
        {
            redirect_uri
        } else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let state = if let Some(state) = query.state {
            state
        } else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Ok(mut auth_uri) = Url::parse(&google_home_client_json.auth_uri) else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        auth_uri
            .query_pairs_mut()
            .append_pair("client_id", &client_id);
        auth_uri.query_pairs_mut().append_pair(
            "redirect_uri",
            base.join("google_home/callback").unwrap().as_str(),
        );
        auth_uri.query_pairs_mut().append_pair("state", &state);
        auth_uri
            .query_pairs_mut()
            .append_pair("response_type", "code");
        auth_uri
            .query_pairs_mut()
            .append_pair("scope", "openid email profile");

        if let Err(e) = db
            .set_code_verifier(
                &state,
                &format!(
                    "{}|{redirect_uri}|{}",
                    &session.access_token, google_home_client_json.project_id
                ),
            )
            .await
        {
            tracing::error!("Issue storing Google Home code verifier {e}");
            return StatusCode::UNAUTHORIZED.into_response();
        }

        Redirect::to(auth_uri.as_str()).into_response()
    }
}
