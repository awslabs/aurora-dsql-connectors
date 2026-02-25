use crate::{DsqlConfig, DsqlError, Result};
use sqlx::{Connection, PgConnection};
use std::ops::{Deref, DerefMut};

pub struct DsqlConnection {
    inner: PgConnection,
}

impl DsqlConnection {
    pub async fn connect(config: &DsqlConfig) -> Result<Self> {
        let token = config.generate_token().await?;
        let opts = config.to_pg_connect_options().password(&token);

        let conn = PgConnection::connect_with(&opts)
            .await
            .map_err(|e| DsqlError::Error(e.to_string()))?;

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
