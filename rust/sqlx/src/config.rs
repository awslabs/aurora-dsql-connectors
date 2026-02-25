use crate::{DsqlError, Result};
use sqlx::postgres::PgConnectOptions;
use url::Url;

const DEFAULT_USER: &str = "admin";
const DEFAULT_DATABASE: &str = "postgres";
const DEFAULT_PORT: u16 = 5432;

#[derive(Debug, Clone)]
pub struct DsqlConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub database: String,
    pub region: Option<String>,
    pub profile: Option<String>,
}

impl DsqlConfig {
    pub fn from_connection_string(conn_str: &str) -> Result<Self> {
        let url = Url::parse(conn_str)
            .map_err(|e| DsqlError::Error(format!("Invalid connection string: {}", e)))?;

        if url.scheme() != "dsql" {
            return Err(DsqlError::Error(
                "Connection string must start with 'dsql://'".into(),
            ));
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

        let region = url
            .query_pairs()
            .find(|(k, _)| k == "region")
            .map(|(_, v)| v.to_string());

        let profile = url
            .query_pairs()
            .find(|(k, _)| k == "profile")
            .map(|(_, v)| v.to_string());

        Ok(DsqlConfig {
            host,
            port,
            user,
            database,
            region,
            profile,
        })
    }

    pub async fn resolve_region(&self) -> Result<String> {
        // 1. Explicit region from connection string
        if let Some(ref region) = self.region {
            return Ok(region.clone());
        }

        // 2. Parse from hostname
        if let Ok(region) = parse_region_from_hostname(&self.host) {
            return Ok(region);
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

    async fn load_aws_config(&self) -> aws_config::SdkConfig {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

        if let Some(ref profile) = self.profile {
            loader = loader.profile_name(profile);
        }

        loader.load().await
    }

    pub async fn generate_token(&self) -> Result<String> {
        let region = self.resolve_region().await?;
        crate::token::generate_token(&self.host, &region, self.profile.as_deref()).await
    }

    pub fn to_pg_connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .database(&self.database)
            .ssl_mode(sqlx::postgres::PgSslMode::Require)
    }
}

fn parse_region_from_hostname(host: &str) -> Result<String> {
    if host.is_empty() {
        return Err(DsqlError::Error(
            "Hostname is required to parse region".into(),
        ));
    }

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 && parts[1].starts_with("dsql") {
        let region = parts[2].to_string();

        // Validate region format: e.g., us-east-1, eu-west-2
        if region.split('-').count() == 3 {
            return Ok(region);
        }
    }

    Err(DsqlError::Error(format!(
        "Unable to parse region from hostname: '{}'",
        host
    )))
}
