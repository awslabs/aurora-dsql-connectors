use crate::{DsqlConfig, DsqlError, Result};
use sqlx::{Connection, PgConnection};
use std::ops::{Deref, DerefMut};

/// A single Aurora DSQL connection with IAM authentication.
///
/// This connection type generates a fresh IAM token on connect. IAM tokens are valid
/// for 15 minutes.
///
/// **Recommended for**:
/// - Short-lived operations (< 15 minutes)
/// - One-off queries or scripts
/// - Testing and development
///
/// **For production workloads**, use `DsqlPool` which provides:
/// - Automatic token refresh
/// - Connection pooling
/// - Better performance for concurrent operations
pub struct DsqlConnection {
    inner: PgConnection,
}

impl DsqlConnection {
    pub async fn connect(config: &DsqlConfig) -> Result<Self> {
        let token = config.generate_token().await?;
        let opts = config
            .to_pg_connect_options()
            .password(&token)
            .ssl_mode(sqlx::postgres::PgSslMode::Require);

        let conn = PgConnection::connect_with(&opts)
            .await
            .map_err(|e| DsqlError::ConnectionError(e.to_string()))?;

        Ok(Self { inner: conn })
    }

    pub async fn connect_with(url: &str) -> Result<Self> {
        let config = DsqlConfig::from_connection_string(url)?;
        Self::connect(&config).await
    }
}

impl Deref for DsqlConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DsqlConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
