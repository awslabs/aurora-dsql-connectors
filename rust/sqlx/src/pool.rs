use crate::{
    config::DsqlConfig,
    occ_retry::{self, OCCRetryConfig},
    token_cache::TokenCache,
    DsqlError, Result,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::ops::Deref;
use std::sync::Arc;

pub struct DsqlPool {
    pool: PgPool,
    token_cache: Arc<TokenCache>,
    retry_config: OCCRetryConfig,
}

impl DsqlPool {
    pub async fn new(conn_str: &str) -> Result<Self> {
        let config = DsqlConfig::from_connection_string(conn_str)?;
        Self::from_config(config).await
    }

    pub async fn from_config(config: DsqlConfig) -> Result<Self> {
        let region = config.resolve_region().await?;
        let token_cache = Arc::new(TokenCache::new(
            config.host.clone(),
            region,
            config.profile.clone(),
        ));

        let token = token_cache.get_token().await?;

        let pg_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&token)
            .database(&config.database)
            .ssl_mode(sqlx::postgres::PgSslMode::Require);

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect_with(pg_options)
            .await
            .map_err(|e| DsqlError::Error(format!("Failed to create pool: {}", e)))?;

        Ok(Self {
            pool,
            token_cache,
            retry_config: OCCRetryConfig::default(),
        })
    }

    async fn retry_on_occ<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
    {
        let mut attempt = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e)
                    if occ_retry::is_occ_error(&e)
                        && attempt < self.retry_config.max_attempts - 1 =>
                {
                    let delay = occ_retry::calculate_backoff(&self.retry_config, attempt);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(DsqlError::Error(e.to_string())),
            }
        }
    }

    pub async fn execute(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        self.retry_on_occ(|| async { sqlx::query(query).execute(&self.pool).await })
            .await
    }

    pub async fn fetch_one<T>(&self, query: &str) -> Result<T>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        self.retry_on_occ(|| async { sqlx::query_as(query).fetch_one(&self.pool).await })
            .await
    }

    pub async fn fetch_all<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        self.retry_on_occ(|| async { sqlx::query_as(query).fetch_all(&self.pool).await })
            .await
    }

    pub async fn clear_token_cache(&self) {
        self.token_cache.clear().await;
    }
}

impl Deref for DsqlPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
