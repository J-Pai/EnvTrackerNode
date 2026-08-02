//! Handler for initiating link with Google Home.

use axum::extract::Query;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum_extra::extract::PrivateCookieJar;
use axum_oidc_client::auth::SESSION_KEY;
use axum_oidc_client::auth_session::AuthSession;
use http::HeaderMap;
use http::StatusCode;
use tower_sessions::cookie::Key;
use url::Url;

use crate::services::auth::Auth;
use crate::services::auth::ServerState;

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
        headers: HeaderMap,
        private_cookie_key: Key,
        session: AuthSession,
        Query(query): Query<OAuth2AuthRequest>,
        ServerState {
            base,
            db,
            certs,
            client_json,
            google_home_client_json,
        }: ServerState,
    ) -> impl IntoResponse {
        let Ok(_) = Self::validate_session(certs, &session, &[client_json.client_id])
            .await
            .map_err(|e| tracing::error!("JWT validation failed: {e} - {}", session.id_token))
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

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

        let jar = PrivateCookieJar::from_headers(&headers, private_cookie_key);

        let Some(session_cookie) = jar.get(SESSION_KEY) else {
            tracing::error!("No session cookie {query:#?}");
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let session_id = session_cookie.value();

        let redirect_uri = if let Some(redirect_uri) = &query.redirect_uri
            && let Ok(redirect_uri) = Url::parse(redirect_uri)
            && redirect_uri.path() == format!("/r/{}", google_home_client_json.project_id)
            && let Some(host) = redirect_uri.host_str()
            && (host == "oauth-redirect.googleusercontent.com"
                || host == "oauth-redirect-sandbox.googleusercontent.com")
        {
            redirect_uri
        } else {
            tracing::error!("Incorrect redirect {query:#?}");
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let state = if let Some(state) = query.state {
            state
        } else {
            tracing::error!("No state {query:#?}");
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
        auth_uri.query_pairs_mut().append_pair("scope", "openid");
        auth_uri
            .query_pairs_mut()
            .append_pair("access_type", "offline");
        auth_uri.query_pairs_mut().append_pair("prompt", "consent");

        if let Err(e) = db
            .set_code_verifier(
                &state,
                &format!(
                    "{}|{redirect_uri}|{}",
                    session_id, google_home_client_json.project_id
                ),
            )
            .await
        {
            tracing::error!("Issue storing Google Home code verifier {e}");
            return StatusCode::UNAUTHORIZED.into_response();
        }

        Redirect::temporary(auth_uri.as_str()).into_response()
    }
}
