//! AuthCache implementation for Db.

use axum_oidc_client::auth_cache::AuthCache;
use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::errors::Error;
use futures_util::future::BoxFuture;

use crate::services::db::Db;

impl AuthCache for Db {
    fn get_code_verifier(
        &self,
        challenge_state: &str,
    ) -> BoxFuture<'_, Result<Option<String>, axum_oidc_client::errors::Error>> {
        let key = Self::cv_key(challenge_state);

        Box::pin(async move {
            let db = self.db.read().await;
            let conn = db.connect().map_err(|e| Error::CacheError(e.to_string()))?;
            let now = Self::now_timestamp();

            let mut rows = conn
                .query(
                    r#"
                    SELECT cache_value FROM oidc_cache
                        WHERE cache_key = ?1 AND expires_at > ?2
                    "#,
                    (key, now),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?
            {
                Ok(Some(
                    row.get(0).map_err(|e| Error::CacheError(e.to_string()))?,
                ))
            } else {
                Ok(None)
            }
        })
    }

    fn set_code_verifier(
        &self,
        challenge_state: &str,
        code_verifier: &str,
    ) -> BoxFuture<'_, Result<(), axum_oidc_client::errors::Error>> {
        let key = Self::cv_key(challenge_state);
        let value = code_verifier.to_string();
        let expires_at = Self::expires_at(60);

        Box::pin(async move {
            let write_conn = self.write_conn.lock().await;

            write_conn
                .execute(
                    r#"
                    INSERT OR REPLACE INTO oidc_cache (cache_key, cache_value, expires_at)
                        VALUES (?1, ?2, ?3);
                    "#,
                    (key, value, expires_at),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            Ok(())
        })
    }

    fn invalidate_code_verifier(
        &self,
        challenge_state: &str,
    ) -> BoxFuture<'_, Result<(), axum_oidc_client::errors::Error>> {
        let key = Self::cv_key(challenge_state);

        Box::pin(async move {
            let write_conn = self.write_conn.lock().await;

            write_conn
                .execute(
                    r#"
                    DELETE FROM oidc_cache WHERE cache_key = ?1;
                    "#,
                    (key,),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            Ok(())
        })
    }

    fn get_auth_session(
        &self,
        session_id: &str,
    ) -> BoxFuture<
        '_,
        Result<
            Option<axum_oidc_client::auth_session::AuthSession>,
            axum_oidc_client::errors::Error,
        >,
    > {
        let key = Self::session_key(session_id);

        Box::pin(async move {
            let db = self.db.read().await;
            let conn = db.connect().map_err(|e| Error::CacheError(e.to_string()))?;
            let now = Self::now_timestamp();

            let mut rows = conn
                .query(
                    r#"
                    SELECT cache_value FROM oidc_cache
                        WHERE cache_key = ?1 AND expires_at > ?2
                    "#,
                    (key, now),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?
            {
                let data: String = row.get(0).map_err(|e| Error::CacheError(e.to_string()))?;

                Ok(Some(
                    serde_json::from_str::<AuthSession>(data.as_str())
                        .map_err(|e| Error::CacheError(e.to_string()))?,
                ))
            } else {
                Ok(None)
            }
        })
    }

    fn set_auth_session(
        &self,
        session_id: &str,
        session: AuthSession,
    ) -> BoxFuture<'_, Result<(), Error>> {
        let key = Self::session_key(session_id);
        let expires_at = session
            .expires
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|| Self::expires_at(3600));

        Box::pin(async move {
            let value =
                serde_json::to_string(&session).map_err(|e| Error::CacheError(e.to_string()))?;
            let write_conn = self.write_conn.lock().await;

            write_conn
                .execute(
                    r#"
                    INSERT OR REPLACE INTO oidc_cache (cache_key, cache_value, expires_at)
                        VALUES (?1, ?2, ?3);
                    "#,
                    (key, value, expires_at),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            Ok(())
        })
    }

    fn invalidate_auth_session(&self, session_id: &str) -> BoxFuture<'_, Result<(), Error>> {
        let key = Self::session_key(session_id);

        Box::pin(async move {
            let write_conn = self.write_conn.lock().await;

            write_conn
                .execute(
                    r#"
                    DELETE FROM oidc_cache WHERE cache_key = ?1;
                    "#,
                    (key,),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            Ok(())
        })
    }

    fn extend_auth_session(&self, session_id: &str, ttl: i64) -> BoxFuture<'_, Result<(), Error>> {
        let key = Self::session_key(session_id);
        let new_expires_at = Self::expires_at(ttl);

        Box::pin(async move {
            let write_conn = self.write_conn.lock().await;

            write_conn
                .execute(
                    r#"
                        UPDATE oidc_cache
                        SET expires_at = ?1
                        WHERE cache_key = ?2;
                    "#,
                    (new_expires_at, key),
                )
                .await
                .map_err(|e| Error::CacheError(e.to_string()))?;

            Ok(())
        })
    }
}
