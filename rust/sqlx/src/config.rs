// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::util::{ClusterId, Host, Region, User};
use crate::{DsqlError, Result};
use derive_builder::Builder;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::Connection;
use tokio::sync::OnceCell;
use url::Url;

const DEFAULT_USER: &str = "admin";
const DEFAULT_DATABASE: &str = "postgres";
const DEFAULT_PORT: u16 = 5432;
#[cfg(feature = "pool")]
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
#[cfg(feature = "pool")]
const DEFAULT_MAX_LIFETIME_SECS: u64 = 3300;
#[cfg(feature = "pool")]
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone, Builder)]
#[builder(default)]
pub struct DsqlConfig {
    pub host: Host,
    pub port: u16,
    pub user: User,
    pub database: String,
    pub region: Option<Region>,
    pub profile: Option<String>,
    pub token_duration_secs: Option<u64>,
    #[cfg(feature = "pool")]
    pub max_connections: u32,
    #[cfg(feature = "pool")]
    pub max_lifetime_secs: u64,
    #[cfg(feature = "pool")]
    pub idle_timeout_secs: u64,
    pub application_name: Option<String>,
    pub pg_connect_options: Option<PgConnectOptions>,
    #[builder(setter(skip))]
    pub(crate) sdk_config: Arc<OnceCell<aws_config::SdkConfig>>,
}

impl Default for DsqlConfig {
    fn default() -> Self {
        Self {
            host: Host::new(""),
            port: DEFAULT_PORT,
            user: User::new(DEFAULT_USER),
            database: DEFAULT_DATABASE.to_string(),
            region: None,
            profile: None,
            token_duration_secs: None,
            #[cfg(feature = "pool")]
            max_connections: DEFAULT_MAX_CONNECTIONS,
            #[cfg(feature = "pool")]
            max_lifetime_secs: DEFAULT_MAX_LIFETIME_SECS,
            #[cfg(feature = "pool")]
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            application_name: None,
            pg_connect_options: None,
            sdk_config: Arc::new(OnceCell::new()),
        }
    }
}

impl DsqlConfig {
    pub async fn from_connection_string(conn_str: &str) -> Result<Self> {
        let url = Url::parse(conn_str)
            .map_err(|e| DsqlError::ConfigError(format!("Invalid connection string: {:?}", e)))?;

        match url.scheme() {
            "postgres" | "postgresql" => {}
            _ => {
                return Err(DsqlError::ConfigError(
                    "Unsupported URL scheme. Use 'postgres://' or 'postgresql://'".into(),
                ));
            }
        }

        let host = url
            .host_str()
            .ok_or_else(|| DsqlError::ConfigError("Host is required".into()))?;
        let host = Host::new(host);

        let port = url.port().unwrap_or(DEFAULT_PORT);
        let user = if url.username().is_empty() {
            User::new(DEFAULT_USER)
        } else {
            User::new(url.username())
        };

        let database = url.path().trim_start_matches('/');
        let database = if database.is_empty() {
            DEFAULT_DATABASE.to_string()
        } else {
            database.to_string()
        };

        let mut region: Option<Region> = None;
        let mut profile = None;
        let mut token_duration_secs = None;
        #[cfg(feature = "pool")]
        let mut max_connections = DEFAULT_MAX_CONNECTIONS;
        #[cfg(feature = "pool")]
        let mut max_lifetime_secs = DEFAULT_MAX_LIFETIME_SECS;
        #[cfg(feature = "pool")]
        let mut idle_timeout_secs = DEFAULT_IDLE_TIMEOUT_SECS;
        let mut application_name = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "region" => region = Some(Region::new(value.to_string())),
                "profile" => profile = Some(value.to_string()),
                "tokenDurationSecs" => {
                    token_duration_secs = Some(value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid tokenDurationSecs: '{}'", value))
                    })?);
                }
                #[cfg(feature = "pool")]
                "maxConnections" => {
                    max_connections = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid maxConnections: '{}'", value))
                    })?;
                }
                #[cfg(feature = "pool")]
                "maxLifetimeSecs" => {
                    max_lifetime_secs = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid maxLifetimeSecs: '{}'", value))
                    })?;
                }
                #[cfg(feature = "pool")]
                "idleTimeoutSecs" => {
                    idle_timeout_secs = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid idleTimeoutSecs: '{}'", value))
                    })?;
                }
                "applicationName" => {
                    application_name = Some(value.to_string());
                }
                _ => {}
            }
        }

        // Cluster ID expansion: if host is a bare cluster ID, expand to full hostname
        let host = if let Some(cluster_id) = ClusterId::new(host.as_str()) {
            let resolved_region = match region.clone() {
                Some(r) => r,
                None => crate::util::resolve_default_region().await.ok_or_else(|| {
                    DsqlError::ConfigError("region is required when host is a cluster ID".into())
                })?,
            };
            if region.is_none() {
                region = Some(resolved_region.clone());
            }
            crate::util::build_hostname(&cluster_id, &resolved_region)
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
            #[cfg(feature = "pool")]
            max_connections,
            #[cfg(feature = "pool")]
            max_lifetime_secs,
            #[cfg(feature = "pool")]
            idle_timeout_secs,
            application_name,
            pg_connect_options: None,
            sdk_config: Arc::new(OnceCell::new()),
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
        Ok(())
    }

    pub async fn resolve_region(&self) -> Result<Region> {
        let sdk_config = self.load_aws_config().await;
        self.resolve_region_with_sdk_config(sdk_config)
    }

    pub fn resolve_region_with_sdk_config(
        &self,
        sdk_config: &aws_config::SdkConfig,
    ) -> Result<Region> {
        // 1. Parse from hostname
        if let Some(region) = crate::util::parse_region(&self.host) {
            return Ok(region);
        }

        // 2. Explicit region from connection string
        if let Some(ref region) = self.region {
            return Ok(region.clone());
        }

        // 3. AWS SDK default region
        if let Some(region) = sdk_config.region() {
            return Ok(Region::new(region.to_string()));
        }

        Err(DsqlError::ConfigError(
            "Could not determine region from connection string, hostname, or AWS configuration"
                .into(),
        ))
    }

    pub async fn load_aws_config(&self) -> &aws_config::SdkConfig {
        self.sdk_config
            .get_or_init(|| async {
                let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
                if let Some(ref profile) = self.profile {
                    loader = loader.profile_name(profile);
                }
                loader.load().await
            })
            .await
    }

    pub async fn connect(&self) -> Result<sqlx::PgConnection> {
        let token = self.generate_token().await?;
        let opts = self.to_pg_connect_options(&token);

        sqlx::PgConnection::connect_with(&opts)
            .await
            .map_err(DsqlError::ConnectionError)
    }

    pub async fn generate_token(&self) -> Result<String> {
        let sdk_config = self.load_aws_config().await;
        let region = self.resolve_region_with_sdk_config(sdk_config)?;
        crate::token::generate_token_with_config(
            &self.host,
            &region,
            &self.user,
            sdk_config,
            self.token_duration_secs,
        )
        .await
    }

    pub fn to_pg_connect_options(&self, token: &str) -> PgConnectOptions {
        let app_name = self
            .application_name
            .clone()
            .unwrap_or_else(|| crate::util::build_application_name(None));

        let base = self.pg_connect_options.clone().unwrap_or_default();

        // DSQL-required settings always override the base
        base.host(self.host.as_str())
            .port(self.port)
            .username(self.user.as_str())
            .password(token)
            .database(&self.database)
            .ssl_mode(PgSslMode::VerifyFull)
            .application_name(&app_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_basic_connection_string() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws:5432/postgres",
        )
        .await?;

        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "postgres");
        assert_eq!(config.region, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_with_region_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres?region=us-west-2",
        )
        .await?;

        assert_eq!(config.region, Some(Region::new("us-west-2")));
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_with_profile_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?profile=dev",
        )
        .await?;

        assert_eq!(config.profile, Some("dev".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_with_region_and_profile() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1&profile=prod",
        )
        .await?;

        assert_eq!(config.region, Some(Region::new("us-east-1")));
        assert_eq!(config.profile, Some("prod".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_region_from_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1",
        )
        .await?;

        assert_eq!(config.resolve_region().await?, Region::new("us-east-1"));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_region_from_hostname() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres",
        )
        .await?;

        assert_eq!(config.resolve_region().await?, Region::new("us-west-2"));
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_connection_string() {
        let result = DsqlConfig::from_connection_string("invalid://connection").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_postgresql_scheme_alias() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgresql://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await?;

        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.user, User::new("admin"));
        Ok(())
    }

    #[test]
    fn test_default_values() {
        let config = DsqlConfig::default();

        assert_eq!(config.port, 5432);
        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.database, "postgres");
        #[cfg(feature = "pool")]
        {
            assert_eq!(config.max_connections, 5);
            assert_eq!(config.max_lifetime_secs, 3300);
            assert_eq!(config.idle_timeout_secs, 600);
        }
        assert_eq!(config.region, None);
        assert_eq!(config.profile, None);
        assert_eq!(config.token_duration_secs, None);
        assert_eq!(config.application_name, None);
        assert!(config.pg_connect_options.is_none());
    }

    #[tokio::test]
    async fn test_new_fields_defaults_from_connection_string() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await?;

        #[cfg(feature = "pool")]
        {
            assert_eq!(config.max_connections, 5);
            assert_eq!(config.max_lifetime_secs, 3300);
            assert_eq!(config.idle_timeout_secs, 600);
        }
        assert_eq!(config.token_duration_secs, None);
        assert_eq!(config.application_name, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_new_query_params() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
             tokenDurationSecs=900&maxConnections=20&maxLifetimeSecs=1800&\
             idleTimeoutSecs=300&applicationName=myapp",
        )
        .await?;

        assert_eq!(config.token_duration_secs, Some(900));
        #[cfg(feature = "pool")]
        {
            assert_eq!(config.max_connections, 20);
            assert_eq!(config.max_lifetime_secs, 1800);
            assert_eq!(config.idle_timeout_secs, 300);
        }
        assert_eq!(config.application_name, Some("myapp".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_cluster_id_expansion_with_region_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )
        .await?;

        assert_eq!(
            config.host,
            Host::new("abcdefghijklmnopqrstuvwxyz.dsql.us-east-1.on.aws")
        );
        assert_eq!(config.region, Some(Region::new("us-east-1")));
        Ok(())
    }

    #[tokio::test]
    async fn test_cluster_id_expansion_and_env_region() -> Result<()> {
        // These subtests share env vars, so they run in a single test to avoid races.

        // Subtest 1: cluster ID + AWS_REGION env var → expanded hostname
        std::env::set_var("AWS_REGION", "eu-west-1");
        std::env::remove_var("AWS_DEFAULT_REGION");
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres",
        )
        .await?;
        assert_eq!(
            config.host,
            Host::new("abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws")
        );
        assert_eq!(config.region, Some(Region::new("eu-west-1")));

        // Subtest 2: cluster ID without any region source → error
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("AWS_DEFAULT_REGION");
        let result = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres",
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("region is required"));

        Ok(())
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .build()
            .unwrap();

        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.port, 5432);
        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.database, "postgres");
        #[cfg(feature = "pool")]
        assert_eq!(config.max_connections, 5);
    }

    #[test]
    fn test_builder_with_custom_fields() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .user(User::new("app_user"))
            .region(Some(Region::new("us-west-2")))
            .build()
            .unwrap();

        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.user, User::new("app_user"));
        assert_eq!(config.region, Some(Region::new("us-west-2")));
    }

    #[cfg(feature = "pool")]
    #[test]
    fn test_builder_with_pool_fields() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .max_connections(20u32)
            .max_lifetime_secs(1800u64)
            .build()
            .unwrap();

        assert_eq!(config.max_connections, 20);
        assert_eq!(config.max_lifetime_secs, 1800);
    }

    #[test]
    fn test_to_pg_connect_options_without_base() {
        let config = DsqlConfig {
            host: Host::new("example.dsql.us-east-1.on.aws"),
            ..DsqlConfig::default()
        };

        let opts = config.to_pg_connect_options("test-token");
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert_eq!(opts.get_port(), 5432);
        assert_eq!(opts.get_username(), "admin");
        assert_eq!(opts.get_database().unwrap(), "postgres");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
    }

    #[test]
    fn test_to_pg_connect_options_with_custom_base() {
        // Base sets options (a pass-through PostgreSQL startup parameter)
        let base = PgConnectOptions::new().options([("search_path", "myschema")]);

        let config = DsqlConfig {
            host: Host::new("example.dsql.us-east-1.on.aws"),
            pg_connect_options: Some(base),
            ..DsqlConfig::default()
        };

        let opts = config.to_pg_connect_options("test-token");
        // Custom setting preserved from base
        let options_str = opts.get_options().expect("options should be set");
        assert!(options_str.contains("search_path"));
        // DSQL-required settings still applied
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
    }

    #[test]
    fn test_to_pg_connect_options_dsql_settings_override_base() {
        // Base tries to set host and ssl_mode — DSQL settings must win
        let base = PgConnectOptions::new()
            .host("wrong-host.example.com")
            .ssl_mode(PgSslMode::Prefer);

        let config = DsqlConfig {
            host: Host::new("example.dsql.us-east-1.on.aws"),
            pg_connect_options: Some(base),
            ..DsqlConfig::default()
        };

        let opts = config.to_pg_connect_options("test-token");
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
        assert_eq!(opts.get_username(), "admin");
    }

    #[test]
    fn test_builder_with_pg_connect_options() {
        let base = PgConnectOptions::new().options([("search_path", "custom")]);

        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .pg_connect_options(Some(base))
            .build()
            .unwrap();

        assert!(config.pg_connect_options.is_some());
        let opts = config.to_pg_connect_options("test-token");
        assert!(opts.get_options().unwrap().contains("search_path"));
    }
}
