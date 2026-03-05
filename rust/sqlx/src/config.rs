// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlError, Result};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use url::Url;

const DEFAULT_USER: &str = "admin";
const DEFAULT_DATABASE: &str = "postgres";
const DEFAULT_PORT: u16 = 5432;
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_MAX_LIFETIME_SECS: u64 = 3300;
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone)]
pub struct DsqlConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub database: String,
    pub region: Option<String>,
    pub profile: Option<String>,
    pub token_duration_secs: Option<u64>,
    pub max_connections: u32,
    pub max_lifetime_secs: u64,
    pub idle_timeout_secs: u64,
    pub occ_max_retries: Option<u32>,
    pub application_name: Option<String>,
}

impl Default for DsqlConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: DEFAULT_PORT,
            user: DEFAULT_USER.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            region: None,
            profile: None,
            token_duration_secs: None,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            max_lifetime_secs: DEFAULT_MAX_LIFETIME_SECS,
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            occ_max_retries: None,
            application_name: None,
        }
    }
}

impl DsqlConfig {
    pub fn from_connection_string(conn_str: &str) -> Result<Self> {
        let url = Url::parse(conn_str)
            .map_err(|e| DsqlError::Error(format!("Invalid connection string: {}", e)))?;

        match url.scheme() {
            "postgres" | "postgresql" => {}
            _ => {
                return Err(DsqlError::Error(
                    "Unsupported URL scheme. Use 'postgres://' or 'postgresql://'".into(),
                ));
            }
        }

        let host = url
            .host_str()
            .ok_or_else(|| DsqlError::ConfigError("Host is required".into()))?
            .to_string();

        let port = url.port().unwrap_or(DEFAULT_PORT);
        let user = if url.username().is_empty() {
            DEFAULT_USER.to_string()
        } else {
            url.username().to_string()
        };

        let database = url.path().trim_start_matches('/');
        let database = if database.is_empty() {
            DEFAULT_DATABASE.to_string()
        } else {
            database.to_string()
        };

        let mut region = None;
        let mut profile = None;
        let mut token_duration_secs = None;
        let mut max_connections = DEFAULT_MAX_CONNECTIONS;
        let mut max_lifetime_secs = DEFAULT_MAX_LIFETIME_SECS;
        let mut idle_timeout_secs = DEFAULT_IDLE_TIMEOUT_SECS;
        let mut occ_max_retries = None;
        let mut application_name = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "region" => region = Some(value.to_string()),
                "profile" => profile = Some(value.to_string()),
                "tokenDurationSecs" => {
                    token_duration_secs = Some(value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid tokenDurationSecs: '{}'", value))
                    })?);
                }
                "maxConnections" => {
                    max_connections = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid maxConnections: '{}'", value))
                    })?;
                }
                "maxLifetimeSecs" => {
                    max_lifetime_secs = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid maxLifetimeSecs: '{}'", value))
                    })?;
                }
                "idleTimeoutSecs" => {
                    idle_timeout_secs = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid idleTimeoutSecs: '{}'", value))
                    })?;
                }
                "occMaxRetries" => {
                    occ_max_retries = Some(value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid occMaxRetries: '{}'", value))
                    })?);
                }
                "applicationName" => {
                    application_name = Some(value.to_string());
                }
                _ => {}
            }
        }

        // Cluster ID expansion: if host is a bare cluster ID, expand to full hostname
        let host = if crate::util::is_cluster_id(&host) {
            let resolved_region = region
                .as_deref()
                .map(String::from)
                .or_else(crate::util::region_from_env)
                .ok_or_else(|| {
                    DsqlError::ConfigError(
                        "region is required when host is a cluster ID".into(),
                    )
                })?;
            if region.is_none() {
                region = Some(resolved_region.clone());
            }
            crate::util::build_hostname(&host, &resolved_region)
        } else {
            host
        };

        let config = DsqlConfig {
            host,
            port,
            user,
            database,
            region,
            profile,
            token_duration_secs,
            max_connections,
            max_lifetime_secs,
            idle_timeout_secs,
            occ_max_retries,
            application_name,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(DsqlError::ConfigError("Host is required".into()));
        }
        if self.port == 0 {
            return Err(DsqlError::ConfigError(format!(
                "port must be between 1 and 65535, got {}",
                self.port
            )));
        }
        if let Some(retries) = self.occ_max_retries {
            if retries == 0 {
                return Err(DsqlError::ConfigError(
                    "occ_max_retries must be a positive integer".into(),
                ));
            }
        }
        Ok(())
    }

    pub async fn resolve_region(&self) -> Result<String> {
        // 1. Parse from hostname
        if let Some(region) = crate::util::parse_region(&self.host) {
            return Ok(region);
        }

        // 2. Explicit region from connection string
        if let Some(ref region) = self.region {
            return Ok(region.clone());
        }

        // 3. AWS SDK default region
        let sdk_config = self.load_aws_config().await;
        if let Some(region) = sdk_config.region() {
            return Ok(region.to_string());
        }

        Err(DsqlError::Error(
            "Could not determine region from connection string, hostname, or AWS configuration"
                .into(),
        ))
    }

    pub async fn load_aws_config(&self) -> aws_config::SdkConfig {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

        if let Some(ref profile) = self.profile {
            loader = loader.profile_name(profile);
        }

        loader.load().await
    }

    pub async fn generate_token(&self) -> Result<String> {
        let region = self.resolve_region().await?;
        crate::token::generate_token(&self.host, &region, &self.user, self.profile.as_deref(), self.token_duration_secs)
            .await
    }

    pub fn to_pg_connect_options(&self, token: &str) -> PgConnectOptions {
        let app_name = self
            .application_name
            .clone()
            .unwrap_or_else(|| crate::util::build_application_name(None));

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(token)
            .database(&self.database)
            .ssl_mode(PgSslMode::VerifyFull)
            .application_name(&app_name)
    }
}
