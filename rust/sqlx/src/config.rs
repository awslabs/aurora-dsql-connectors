// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    util::{ClusterId, Host, Region, User},
    DsqlError, Result,
};
use derive_builder::Builder;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::Connection;
use url::Url;

const DEFAULT_USER: &str = "admin";
const DEFAULT_DATABASE: &str = "postgres";
const DEFAULT_PORT: u16 = 5432;

#[derive(Debug, Clone, Builder)]
pub struct DsqlConfig {
    pub host: Host,
    #[builder(default = "DEFAULT_PORT")]
    pub port: u16,
    #[builder(default = "User::new(DEFAULT_USER)")]
    pub user: User,
    #[builder(default = "DEFAULT_DATABASE.to_string()")]
    pub database: String,
    #[builder(default)]
    pub region: Option<Region>,
    #[builder(default)]
    pub profile: Option<String>,
    #[builder(default)]
    pub token_duration_secs: Option<u64>,
    #[builder(default)]
    pub application_name: Option<String>,
    #[builder(default)]
    pub pg_connect_options: Option<PgConnectOptions>,
}

#[cfg(feature = "pool")]
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
#[cfg(feature = "pool")]
const DEFAULT_MAX_LIFETIME_SECS: u64 = 3300;
#[cfg(feature = "pool")]
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

#[cfg(feature = "pool")]
#[derive(Debug, Clone, Builder)]
pub struct DsqlPoolConfig {
    pub connection: DsqlConfig,
    #[builder(default = "DEFAULT_MAX_CONNECTIONS")]
    pub max_connections: u32,
    #[builder(default = "DEFAULT_MAX_LIFETIME_SECS")]
    pub max_lifetime_secs: u64,
    #[builder(default = "DEFAULT_IDLE_TIMEOUT_SECS")]
    pub idle_timeout_secs: u64,
    #[builder(default)]
    pub occ_max_retries: Option<u32>,
}

impl DsqlConfig {
    pub fn from_connection_string(conn_str: &str) -> Result<Self> {
        let url = Self::parse_url(conn_str)?;
        Self::from_url(&url)
    }

    fn parse_url(conn_str: &str) -> Result<Url> {
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

        Ok(url)
    }

    fn from_url(url: &Url) -> Result<Self> {
        let host = Host::new(
            url.host_str()
                .ok_or_else(|| DsqlError::ConfigError("Host is required".into()))?,
        );
        let port = url.port().unwrap_or(DEFAULT_PORT);

        let user = if !url.username().is_empty() {
            User::new(url.username())
        } else {
            User::new(DEFAULT_USER)
        };

        let database = {
            let db = url.path().trim_start_matches('/');
            if db.is_empty() {
                DEFAULT_DATABASE.to_string()
            } else {
                db.to_string()
            }
        };

        let mut region = None;
        let mut profile = None;
        let mut token_duration_secs = None;
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
                "applicationName" => application_name = Some(value.to_string()),
                _ => {}
            }
        }

        let cfg = DsqlConfig {
            host,
            port,
            user,
            database,
            region,
            profile,
            token_duration_secs,
            application_name,
            pg_connect_options: None,
        };

        cfg.validate()?;
        Ok(cfg)
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

    /// If host is a bare cluster ID, expand it to a full DSQL hostname.
    /// Requires the SDK config for default region resolution when no
    /// explicit region is set.
    pub fn resolve_host(&self, sdk_config: &aws_config::SdkConfig) -> Result<Host> {
        if let Some(cluster_id) = ClusterId::new(self.host.as_str()) {
            let region = self.resolve_region(sdk_config)?;
            Ok(crate::util::build_hostname(&cluster_id, &region))
        } else {
            Ok(self.host.clone())
        }
    }

    pub fn resolve_region(&self, sdk_config: &aws_config::SdkConfig) -> Result<Region> {
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

    pub async fn load_aws_config(&self) -> aws_config::SdkConfig {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(ref profile) = self.profile {
            loader = loader.profile_name(profile);
        }
        loader.load().await
    }

    pub async fn connect(&self) -> Result<sqlx::PgConnection> {
        let sdk_config = self.load_aws_config().await;
        let host = self.resolve_host(&sdk_config)?;
        let region = self.resolve_region(&sdk_config)?;

        let token = crate::token::generate_token_with_config(
            &host,
            &region,
            &self.user,
            &sdk_config,
            self.token_duration_secs,
        )
        .await?;

        let opts = self.build_pg_connect_options(&host, &token);

        sqlx::PgConnection::connect_with(&opts)
            .await
            .map_err(DsqlError::ConnectionError)
    }

    pub async fn generate_token(&self) -> Result<String> {
        let sdk_config = self.load_aws_config().await;
        let host = self.resolve_host(&sdk_config)?;
        let region = self.resolve_region(&sdk_config)?;
        crate::token::generate_token_with_config(
            &host,
            &region,
            &self.user,
            &sdk_config,
            self.token_duration_secs,
        )
        .await
    }

    pub fn to_pg_connect_options(&self, token: &str) -> PgConnectOptions {
        self.build_pg_connect_options(&self.host, token)
    }

    fn build_pg_connect_options(&self, host: &Host, token: &str) -> PgConnectOptions {
        let app_name = self
            .application_name
            .clone()
            .unwrap_or_else(|| crate::util::build_application_name(None));

        let base = self.pg_connect_options.clone().unwrap_or_default();

        // DSQL-required settings always override the base
        base.host(host.as_str())
            .port(self.port)
            .username(self.user.as_str())
            .password(token)
            .database(&self.database)
            .ssl_mode(PgSslMode::VerifyFull)
            .application_name(&app_name)
    }
}

#[cfg(feature = "pool")]
impl DsqlPoolConfig {
    pub fn from_connection_string(conn_str: &str) -> Result<Self> {
        let url = DsqlConfig::parse_url(conn_str)?;
        let connection = DsqlConfig::from_url(&url)?;

        let mut max_connections = DEFAULT_MAX_CONNECTIONS;
        let mut max_lifetime_secs = DEFAULT_MAX_LIFETIME_SECS;
        let mut idle_timeout_secs = DEFAULT_IDLE_TIMEOUT_SECS;
        let mut occ_max_retries = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
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
                _ => {}
            }
        }

        Ok(DsqlPoolConfig {
            connection,
            max_connections,
            max_lifetime_secs,
            idle_timeout_secs,
            occ_max_retries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_connection_string() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws:5432/postgres",
        )?;

        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "postgres");
        assert_eq!(config.region, None);
        Ok(())
    }

    #[test]
    fn test_parse_with_region_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres?region=us-west-2",
        )?;

        assert_eq!(config.region, Some(Region::new("us-west-2")));
        Ok(())
    }

    #[test]
    fn test_parse_with_profile_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?profile=dev",
        )?;

        assert_eq!(config.profile, Some("dev".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_with_region_and_profile() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1&profile=prod",
        )?;

        assert_eq!(config.region, Some(Region::new("us-east-1")));
        assert_eq!(config.profile, Some("prod".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_region_from_param() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1",
        )?;

        let sdk_config = config.load_aws_config().await;
        assert_eq!(
            config.resolve_region(&sdk_config)?,
            Region::new("us-east-1")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_region_from_hostname() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres",
        )?;

        let sdk_config = config.load_aws_config().await;
        assert_eq!(
            config.resolve_region(&sdk_config)?,
            Region::new("us-west-2")
        );
        Ok(())
    }

    #[test]
    fn test_invalid_connection_string() {
        let result = DsqlConfig::from_connection_string("invalid://connection");
        assert!(result.is_err());
    }

    #[test]
    fn test_postgresql_scheme_alias() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgresql://admin@example.dsql.us-east-1.on.aws/postgres",
        )?;

        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.user, User::new("admin"));
        Ok(())
    }

    #[test]
    fn test_defaults_from_connection_string() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )?;

        assert_eq!(config.port, 5432);
        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.database, "postgres");
        assert_eq!(config.region, None);
        assert_eq!(config.profile, None);
        assert_eq!(config.token_duration_secs, None);
        assert_eq!(config.application_name, None);
        assert!(config.pg_connect_options.is_none());
        Ok(())
    }

    #[test]
    fn test_parse_query_params() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
             tokenDurationSecs=900&applicationName=myapp",
        )?;

        assert_eq!(config.token_duration_secs, Some(900));
        assert_eq!(config.application_name, Some("myapp".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_cluster_id_stores_raw_host() -> Result<()> {
        // from_connection_string no longer expands cluster IDs — it stores
        // the raw host. Expansion happens at connect/pool creation time
        // via resolve_host().
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )?;

        assert_eq!(config.host, Host::new("abcdefghijklmnopqrstuvwxyz"));
        assert_eq!(config.region, Some(Region::new("us-east-1")));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_host_expands_cluster_id() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )?;

        let sdk_config = config.load_aws_config().await;
        let host = config.resolve_host(&sdk_config)?;
        assert_eq!(
            host,
            Host::new("abcdefghijklmnopqrstuvwxyz.dsql.us-east-1.on.aws")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_host_noop_for_full_hostname() -> Result<()> {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )?;

        let sdk_config = config.load_aws_config().await;
        let host = config.resolve_host(&sdk_config)?;
        assert_eq!(host, Host::new("example.dsql.us-east-1.on.aws"));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_host_cluster_id_with_env_region() -> Result<()> {
        std::env::set_var("AWS_REGION", "eu-west-1");
        std::env::remove_var("AWS_DEFAULT_REGION");

        let config = DsqlConfig::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres",
        )?;

        // sdk_config picks up AWS_REGION from env
        let sdk_config = config.load_aws_config().await;
        let host = config.resolve_host(&sdk_config)?;
        assert_eq!(
            host,
            Host::new("abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws")
        );

        // Clean up
        std::env::remove_var("AWS_REGION");
        Ok(())
    }

    #[test]
    fn test_builder_with_required_host() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .build()
            .unwrap();

        assert_eq!(config.host, Host::new("example.dsql.us-east-1.on.aws"));
        assert_eq!(config.port, 5432);
        assert_eq!(config.user, User::new("admin"));
        assert_eq!(config.database, "postgres");
    }

    #[test]
    fn test_builder_fails_without_host() {
        let result = DsqlConfigBuilder::default().build();
        assert!(result.is_err(), "Builder should fail without host");
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

    #[test]
    fn test_to_pg_connect_options_without_base() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .build()
            .unwrap();

        let opts = config.to_pg_connect_options("test-token");
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert_eq!(opts.get_port(), 5432);
        assert_eq!(opts.get_username(), "admin");
        assert_eq!(opts.get_database().unwrap(), "postgres");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
        let app_name = opts
            .get_application_name()
            .expect("application_name should be set");
        assert!(app_name.starts_with("aurora-dsql-rust-sqlx/"));
    }

    #[test]
    fn test_to_pg_connect_options_with_custom_base() {
        // Base sets options (a pass-through PostgreSQL startup parameter)
        let base = PgConnectOptions::new().options([("search_path", "myschema")]);

        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .pg_connect_options(Some(base))
            .build()
            .unwrap();

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
        // Base tries to set host, ssl_mode, and application_name — DSQL settings must win
        let base = PgConnectOptions::new()
            .host("wrong-host.example.com")
            .ssl_mode(PgSslMode::Prefer)
            .application_name("base-app");

        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .application_name(Some("my-service".to_string()))
            .pg_connect_options(Some(base))
            .build()
            .unwrap();

        let opts = config.to_pg_connect_options("test-token");
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
        assert_eq!(opts.get_username(), "admin");
        assert_eq!(
            opts.get_application_name().unwrap(),
            "my-service",
            "DsqlConfig application_name should override base"
        );
    }

    #[test]
    fn test_to_pg_connect_options_custom_application_name() {
        let config = DsqlConfigBuilder::default()
            .host(Host::new("example.dsql.us-east-1.on.aws"))
            .application_name(Some("my-service".to_string()))
            .build()
            .unwrap();

        let opts = config.to_pg_connect_options("test-token");
        assert_eq!(
            opts.get_application_name().unwrap(),
            "my-service",
            "Custom application_name should override the default"
        );
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

    #[cfg(feature = "pool")]
    mod pool_config_tests {
        use super::*;

        #[test]
        fn test_pool_config_from_connection_string() -> Result<()> {
            let config = DsqlPoolConfig::from_connection_string(
                "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
            )?;

            assert_eq!(
                config.connection.host,
                Host::new("example.dsql.us-east-1.on.aws")
            );
            assert_eq!(config.max_connections, 5);
            assert_eq!(config.max_lifetime_secs, 3300);
            assert_eq!(config.idle_timeout_secs, 600);
            assert_eq!(config.occ_max_retries, None);
            Ok(())
        }

        #[test]
        fn test_pool_config_parse_query_params() -> Result<()> {
            let config = DsqlPoolConfig::from_connection_string(
                "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
                 tokenDurationSecs=900&maxConnections=20&maxLifetimeSecs=1800&\
                 idleTimeoutSecs=300&occMaxRetries=5&applicationName=myapp",
            )?;

            assert_eq!(config.connection.token_duration_secs, Some(900));
            assert_eq!(
                config.connection.application_name,
                Some("myapp".to_string())
            );
            assert_eq!(config.max_connections, 20);
            assert_eq!(config.max_lifetime_secs, 1800);
            assert_eq!(config.idle_timeout_secs, 300);
            assert_eq!(config.occ_max_retries, Some(5));
            Ok(())
        }

        #[test]
        fn test_pool_config_builder_with_defaults() {
            let connection = DsqlConfigBuilder::default()
                .host(Host::new("example.dsql.us-east-1.on.aws"))
                .build()
                .unwrap();

            let config = DsqlPoolConfigBuilder::default()
                .connection(connection)
                .build()
                .unwrap();

            assert_eq!(config.max_connections, 5);
            assert_eq!(config.max_lifetime_secs, 3300);
            assert_eq!(config.idle_timeout_secs, 600);
            assert_eq!(config.occ_max_retries, None);
        }

        #[test]
        fn test_pool_config_builder_with_custom_fields() {
            let connection = DsqlConfigBuilder::default()
                .host(Host::new("example.dsql.us-east-1.on.aws"))
                .build()
                .unwrap();

            let config = DsqlPoolConfigBuilder::default()
                .connection(connection)
                .max_connections(20u32)
                .max_lifetime_secs(1800u64)
                .occ_max_retries(Some(3u32))
                .build()
                .unwrap();

            assert_eq!(config.max_connections, 20);
            assert_eq!(config.max_lifetime_secs, 1800);
            assert_eq!(config.idle_timeout_secs, 600); // default
            assert_eq!(config.occ_max_retries, Some(3));
        }

        #[test]
        fn test_pool_config_builder_fails_without_connection() {
            let result = DsqlPoolConfigBuilder::default()
                .max_connections(10u32)
                .build();
            assert!(result.is_err(), "Builder should fail without connection");
        }
    }
}
