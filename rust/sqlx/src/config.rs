// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{util::ClusterId, DsqlError, Result};
use aws_config::{Region, SdkConfig};
use derive_builder::Builder;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::time::Duration;
use url::Url;

const DEFAULT_USER: &str = "admin";
const DEFAULT_DATABASE: &str = "postgres";
const DEFAULT_PORT: u16 = 5432;
const DEFAULT_TOKEN_DURATION_SECS: u64 = 900;

#[derive(Debug, Clone, Builder)]
#[builder(setter(into), build_fn(validate = "Self::validate"))]
pub struct DsqlConnectOptions {
    pg_connect_options: PgConnectOptions,
    #[builder(default)]
    region: Option<Region>,
    #[builder(default)]
    profile: Option<String>,
    #[builder(default = "DEFAULT_TOKEN_DURATION_SECS")]
    token_duration_secs: u64,
}

impl DsqlConnectOptionsBuilder {
    fn validate(&self) -> std::result::Result<(), String> {
        if let Some(ref pg) = self.pg_connect_options {
            crate::util::validate_host(pg.get_host())?;
        }
        Ok(())
    }
}

impl DsqlConnectOptions {
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
        let host = url
            .host_str()
            .ok_or_else(|| DsqlError::ConfigError("Host is required".into()))?;

        crate::util::validate_host(host).map_err(DsqlError::ConfigError)?;

        let port = url.port().unwrap_or(DEFAULT_PORT);

        let user = if !url.username().is_empty() {
            url.username()
        } else {
            DEFAULT_USER
        };

        let database = {
            let db = url.path().trim_start_matches('/');
            if db.is_empty() {
                DEFAULT_DATABASE
            } else {
                db
            }
        };

        let mut region = None;
        let mut profile = None;
        let mut token_duration_secs = DEFAULT_TOKEN_DURATION_SECS;
        let mut application_name = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "region" => {
                    region = Some(Region::new(value.to_string()));
                }
                "profile" => profile = Some(value.to_string()),
                "tokenDurationSecs" => {
                    let secs: u64 = value.parse().map_err(|_| {
                        DsqlError::ConfigError(format!("invalid tokenDurationSecs: '{}'", value))
                    })?;
                    token_duration_secs = secs;
                }
                "applicationName" => application_name = Some(value.to_string()),
                other => {
                    tracing::warn!(
                        param = other,
                        "aurora-dsql: ignoring unrecognized connection parameter"
                    );
                }
            }
        }

        let app_name =
            application_name.unwrap_or_else(|| crate::util::build_application_name(None));

        let pg = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .database(database)
            .ssl_mode(PgSslMode::VerifyFull)
            .application_name(&app_name);

        Ok(DsqlConnectOptions {
            pg_connect_options: pg,
            region,
            profile,
            token_duration_secs,
        })
    }

    /// Generate a fresh IAM token and return `PgConnectOptions` ready for use.
    ///
    /// This is the main entry point for advanced use cases where you need
    /// to supply your own `PgPoolOptions` or manage connections directly.
    pub async fn authenticated_pg_options(&self) -> Result<PgConnectOptions> {
        let sdk_config = load_aws_config(self.profile()).await;
        let host = self.resolve_host(&sdk_config)?;
        let region = self.resolve_region(&sdk_config)?;
        let signer =
            crate::token::build_signer(&host, &region, &sdk_config, Some(self.token_duration()))?;
        let user = self.pg_connect_options.get_username();
        let token = crate::token::generate_token(&signer, user, &sdk_config).await?;
        Ok(self.build_connect_options(&host, &token))
    }

    /// Clone the inner PgConnectOptions with the given token as password.
    /// The caller must pass a resolved hostname (not a bare cluster ID).
    /// Always enforces `SslMode::VerifyFull` regardless of how the config was constructed.
    pub(crate) fn build_connect_options(&self, host: &str, token: &str) -> PgConnectOptions {
        self.pg_connect_options
            .clone()
            .host(host)
            .password(token)
            .ssl_mode(PgSslMode::VerifyFull)
    }

    /// Read access to the inner PgConnectOptions.
    pub(crate) fn pg_connect_options(&self) -> &PgConnectOptions {
        &self.pg_connect_options
    }

    /// AWS profile name, if configured.
    pub(crate) fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Token validity duration in seconds. Defaults to 900s.
    pub(crate) fn token_duration(&self) -> u64 {
        self.token_duration_secs
    }

    /// How often the background refresh task should rotate tokens.
    /// Returns `token_duration * 4/5` (80%).
    pub(crate) fn refresh_interval(&self) -> Duration {
        Duration::from_secs((self.token_duration() * 4 / 5).max(1))
    }

    /// If host is a bare cluster ID, expand it to a full DSQL hostname.
    pub(crate) fn resolve_host(&self, sdk_config: &SdkConfig) -> Result<String> {
        let host = self.pg_connect_options.get_host();
        if let Some(cluster_id) = ClusterId::new(host) {
            let region = self.resolve_region(sdk_config)?;
            Ok(crate::util::build_hostname(&cluster_id, &region))
        } else {
            Ok(host.to_string())
        }
    }

    pub(crate) fn resolve_region(&self, sdk_config: &SdkConfig) -> Result<Region> {
        // 1. Parse from hostname
        let host = self.pg_connect_options.get_host();
        if let Some(region) = crate::util::parse_region(host) {
            return Ok(region);
        }

        // 2. Explicit region
        if let Some(ref region) = self.region {
            return Ok(region.clone());
        }

        // 3. AWS SDK default region
        if let Some(region) = sdk_config.region() {
            return Ok(region.clone());
        }

        Err(DsqlError::ConfigError(
            "Could not determine region from connection string, hostname, or AWS configuration"
                .into(),
        ))
    }
}

/// Load AWS SDK config, optionally using a named profile.
pub(crate) async fn load_aws_config(profile: Option<&str>) -> SdkConfig {
    let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
    if let Some(profile) = profile {
        loader = loader.profile_name(profile);
    }
    loader.load().await
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- from_connection_string tests ---

    #[test]
    fn test_parse_basic_connection_string() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws:5432/postgres",
        )?;

        assert_eq!(config.pg_connect_options.get_username(), "admin");
        assert_eq!(
            config.pg_connect_options.get_host(),
            "example.dsql.us-east-1.on.aws"
        );
        assert_eq!(config.pg_connect_options.get_port(), 5432);
        assert_eq!(
            config.pg_connect_options.get_database().unwrap(),
            "postgres"
        );
        assert!(config.region.is_none());
        Ok(())
    }

    #[test]
    fn test_parse_with_region_param() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres?region=us-west-2",
        )?;

        assert_eq!(
            config.region.as_ref().map(|r| r.as_ref()),
            Some("us-west-2")
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_profile_param() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?profile=dev",
        )?;

        assert_eq!(config.profile, Some("dev".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_with_region_and_profile() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1&profile=prod",
        )?;

        assert_eq!(
            config.region.as_ref().map(|r| r.as_ref()),
            Some("us-east-1")
        );
        assert_eq!(config.profile, Some("prod".to_string()));
        Ok(())
    }

    #[test]
    fn test_invalid_connection_string() {
        let result = DsqlConnectOptions::from_connection_string("invalid://connection");
        assert!(result.is_err());
    }

    #[test]
    fn test_postgresql_scheme_alias() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgresql://admin@example.dsql.us-east-1.on.aws/postgres",
        )?;

        assert_eq!(
            config.pg_connect_options.get_host(),
            "example.dsql.us-east-1.on.aws"
        );
        assert_eq!(config.pg_connect_options.get_username(), "admin");
        Ok(())
    }

    #[test]
    fn test_parse_query_params() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
             tokenDurationSecs=900&applicationName=myapp",
        )?;

        assert_eq!(config.token_duration_secs, 900);
        assert_eq!(
            config.pg_connect_options.get_application_name().unwrap(),
            "myapp"
        );
        Ok(())
    }

    #[test]
    fn test_parse_cluster_id_stores_raw_host() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )?;

        assert_eq!(
            config.pg_connect_options.get_host(),
            "abcdefghijklmnopqrstuvwxyz"
        );
        assert_eq!(
            config.region.as_ref().map(|r| r.as_ref()),
            Some("us-east-1")
        );
        Ok(())
    }

    // --- resolve_host / resolve_region tests ---

    #[tokio::test]
    async fn test_resolve_region_from_param() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1",
        )?;

        let sdk_config = load_aws_config(config.profile()).await;
        let region = config.resolve_region(&sdk_config)?;
        assert_eq!(region.as_ref(), "us-east-1");
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_region_from_hostname() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-west-2.on.aws/postgres",
        )?;

        let sdk_config = load_aws_config(config.profile()).await;
        let region = config.resolve_region(&sdk_config)?;
        assert_eq!(region.as_ref(), "us-west-2");
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_host_expands_cluster_id() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )?;

        let sdk_config = load_aws_config(config.profile()).await;
        let host = config.resolve_host(&sdk_config)?;
        assert_eq!(host, "abcdefghijklmnopqrstuvwxyz.dsql.us-east-1.on.aws");
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_host_noop_for_full_hostname() -> Result<()> {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )?;

        let sdk_config = load_aws_config(config.profile()).await;
        let host = config.resolve_host(&sdk_config)?;
        assert_eq!(host, "example.dsql.us-east-1.on.aws");
        Ok(())
    }

    // --- builder tests ---

    #[test]
    fn test_builder_rejects_empty_host() {
        let pg = PgConnectOptions::new()
            .host("")
            .username("admin")
            .database("postgres");

        let result = DsqlConnectOptionsBuilder::default()
            .pg_connect_options(pg)
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Host is required"),
            "Expected host error, got: {}",
            err
        );
    }

    // --- build_connect_options tests ---

    #[test]
    fn test_build_connect_options() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        let opts = config.build_connect_options("example.dsql.us-east-1.on.aws", "test-token");
        assert_eq!(opts.get_host(), "example.dsql.us-east-1.on.aws");
        assert_eq!(opts.get_port(), 5432);
        assert_eq!(opts.get_username(), "admin");
        assert_eq!(opts.get_database().unwrap(), "postgres");
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
    }

    #[test]
    fn test_build_connect_options_with_resolved_cluster_id() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@abcdefghijklmnopqrstuvwxyz/postgres?region=us-east-1",
        )
        .unwrap();

        let resolved_host = "abcdefghijklmnopqrstuvwxyz.dsql.us-east-1.on.aws";
        let opts = config.build_connect_options(resolved_host, "test-token");
        assert_eq!(
            opts.get_host(),
            "abcdefghijklmnopqrstuvwxyz.dsql.us-east-1.on.aws",
        );
    }

    #[test]
    fn test_connect_options_default_application_name() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        let opts = config.build_connect_options("example.dsql.us-east-1.on.aws", "test-token");
        let app_name = opts
            .get_application_name()
            .expect("application_name should be set");
        assert!(app_name.starts_with("aurora-dsql-rust-sqlx/"));
    }

    #[test]
    fn test_connect_options_custom_application_name() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?applicationName=my-service",
        )
        .unwrap();

        let opts = config.build_connect_options("example.dsql.us-east-1.on.aws", "test-token");
        assert_eq!(
            opts.get_application_name().unwrap(),
            "my-service",
            "Custom application_name should override the default"
        );
    }

    #[test]
    fn test_ssl_mode_always_verify_full() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        assert!(matches!(
            config.pg_connect_options.get_ssl_mode(),
            PgSslMode::VerifyFull
        ));
    }

    #[test]
    fn test_ssl_mode_enforced_via_builder() {
        let pg = PgConnectOptions::new()
            .host("example.dsql.us-east-1.on.aws")
            .username("admin")
            .database("postgres")
            .ssl_mode(PgSslMode::Prefer); // intentionally weak

        let config = DsqlConnectOptionsBuilder::default()
            .pg_connect_options(pg)
            .build()
            .unwrap();

        let opts = config.build_connect_options("example.dsql.us-east-1.on.aws", "test-token");
        assert!(
            matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull),
            "SSL must be VerifyFull regardless of builder input"
        );
    }

    // --- refresh_interval tests ---

    #[test]
    fn test_refresh_interval_default() {
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        assert_eq!(config.refresh_interval(), Duration::from_secs(720));
    }

    #[test]
    fn test_refresh_interval_floors_to_one_second() {
        let pg = PgConnectOptions::new()
            .host("example.dsql.us-east-1.on.aws")
            .username("admin")
            .database("postgres");

        let config = DsqlConnectOptionsBuilder::default()
            .pg_connect_options(pg)
            .token_duration_secs(1u64)
            .build()
            .unwrap();

        assert_eq!(config.refresh_interval(), Duration::from_secs(1));
    }
}
