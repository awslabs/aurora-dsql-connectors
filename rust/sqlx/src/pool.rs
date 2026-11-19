// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::time::{Duration, SystemTime};

use aws_config::{Region, SdkConfig};
use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};

use crate::config::DsqlConnectOptions;
use crate::{DsqlError, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};

/// Parse a connection string, create a PgPool, verify connectivity,
/// and spawn a background token refresh task.
pub async fn connect(url: &str) -> Result<PgPool> {
    let config = DsqlConnectOptions::from_connection_string(url)?;
    connect_with(&config, PgPoolOptions::new()).await
}

/// Create a PgPool from pre-built options, verify connectivity,
/// and spawn a background token refresh task.
pub async fn connect_with(
    config: &DsqlConnectOptions,
    pool_options: PgPoolOptions,
) -> Result<PgPool> {
    let sdk_config =
        crate::config::load_aws_config(config.profile(), config.credentials_provider()).await;
    let host = config.resolve_host(&sdk_config)?;
    let region = config.resolve_region(&sdk_config)?;

    let (token, lifetime) = generate_token(config, &host, &region, &sdk_config).await?;
    let opts = config.build_connect_options(&sdk_config, &token)?;

    let pool = pool_options
        .connect_with(opts)
        .await
        .map_err(DsqlError::ConnectionError)?;

    spawn_refresh_task(
        pool.clone(),
        config.clone(),
        host,
        region,
        sdk_config,
        lifetime,
    );
    Ok(pool)
}

fn spawn_refresh_task(
    pool: PgPool,
    config: DsqlConnectOptions,
    host: String,
    region: Region,
    sdk_config: SdkConfig,
    lifetime: Duration,
) {
    tokio::spawn(async move {
        let mut delay = crate::config::refresh_interval(lifetime);
        loop {
            tokio::select! {
                _ = pool.close_event() => break,
                _ = tokio::time::sleep(delay) => {
                    match refresh_token(&config, &host, &region, &sdk_config, &pool).await {
                        Ok(lifetime) => delay = crate::config::refresh_interval(lifetime),
                        Err(e) => log::error!("token refresh failed: {:?}", e),
                    }
                }
            }
        }
    });
}

async fn refresh_token(
    config: &DsqlConnectOptions,
    host: &str,
    region: &Region,
    sdk_config: &SdkConfig,
    pool: &PgPool,
) -> Result<Duration> {
    let (token, lifetime) = generate_token(config, host, region, sdk_config).await?;
    pool.set_connect_options(config.build_connect_options(sdk_config, &token)?);
    Ok(lifetime)
}

/// Generate a token that expires no later than the credentials that sign it.
///
/// A token signed with temporary credentials stops working when they expire,
/// whatever its own lifetime. Resolves the credentials once, signs with exactly
/// those, and returns the token with its lifetime.
async fn generate_token(
    config: &DsqlConnectOptions,
    host: &str,
    region: &Region,
    sdk_config: &SdkConfig,
) -> Result<(String, Duration)> {
    let credentials = sdk_config
        .credentials_provider()
        .ok_or_else(|| DsqlError::TokenError("No credentials provider found".into()))?
        .provide_credentials()
        .await
        .map_err(|e| DsqlError::TokenError(Box::new(e)))?;
    let lifetime_secs = token_lifetime_secs(
        config.token_duration(),
        credentials.expiry(),
        SystemTime::now(),
    )?;

    let pinned = sdk_config
        .to_builder()
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .build();
    let signer = crate::token::build_signer(host, region, &pinned, Some(lifetime_secs))?;
    let user = config.pg_connect_options().get_username();
    let token = crate::token::generate_token(&signer, user, &pinned).await?;
    Ok((token, Duration::from_secs(lifetime_secs)))
}

/// The configured token lifetime, capped at the credentials' remaining lifetime.
fn token_lifetime_secs(
    configured_secs: u64,
    credentials_expiry: Option<SystemTime>,
    now: SystemTime,
) -> Result<u64> {
    let Some(expiry) = credentials_expiry else {
        return Ok(configured_secs);
    };
    match expiry.duration_since(now) {
        Ok(remaining) if remaining.as_secs() > 0 => Ok(configured_secs.min(remaining.as_secs())),
        _ => Err(DsqlError::TokenError(
            "credentials are expired or expire within one second".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_credential_types::Credentials;

    fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000)
    }

    #[test]
    fn test_token_lifetime_uses_configured_duration_when_credentials_outlive_it() {
        assert_eq!(token_lifetime_secs(900, None, now()).unwrap(), 900);
        let expiry = now() + Duration::from_secs(6 * 3600);
        assert_eq!(token_lifetime_secs(900, Some(expiry), now()).unwrap(), 900);
    }

    #[test]
    fn test_token_lifetime_caps_at_credential_expiry() {
        let expiry = now() + Duration::from_secs(40);
        assert_eq!(token_lifetime_secs(900, Some(expiry), now()).unwrap(), 40);
    }

    #[test]
    fn test_token_lifetime_rejects_expired_credentials() {
        assert!(token_lifetime_secs(900, Some(now()), now()).is_err());
        let expiry = now() - Duration::from_secs(1);
        assert!(token_lifetime_secs(900, Some(expiry), now()).is_err());
    }

    #[tokio::test]
    async fn test_generate_token_caps_lifetime_at_credential_expiry() {
        let creds = Credentials::new(
            "fake_key",
            "fake_secret",
            Some("fake_session".into()),
            Some(SystemTime::now() + Duration::from_secs(40)),
            "test",
        );
        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();
        let sdk_config =
            crate::config::load_aws_config(None, Some(&SharedCredentialsProvider::new(creds)))
                .await;
        let region = Region::new("us-east-1");

        let (token, lifetime) = generate_token(
            &config,
            "example.dsql.us-east-1.on.aws",
            &region,
            &sdk_config,
        )
        .await
        .unwrap();

        assert!((1..=40).contains(&lifetime.as_secs()), "{lifetime:?}");
        assert!(token.contains(&format!("X-Amz-Expires={}", lifetime.as_secs())));
    }
}
