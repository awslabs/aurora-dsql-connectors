// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::util::{Host, Region, User};
use crate::{DsqlError, Result};
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};

pub async fn generate_token_with_config(
    host: &Host,
    region: &Region,
    user: &User,
    sdk_config: &aws_config::SdkConfig,
    token_duration_secs: Option<u64>,
) -> Result<String> {
    let credentials_provider = sdk_config
        .credentials_provider()
        .ok_or_else(|| DsqlError::TokenError("No credentials provider found".into()))?;

    let mut builder = Config::builder()
        .hostname(host.as_str())
        .region(aws_config::Region::new(region.to_string()))
        .credentials(credentials_provider);

    if let Some(duration) = token_duration_secs {
        builder = builder.expires_in(duration);
    }

    let signer = AuthTokenGenerator::new(builder.build().map_err(|e| {
        DsqlError::TokenError(format!("Failed to build auth token config: {:?}", e))
    })?);

    let token = if user.is_admin() {
        signer.db_connect_admin_auth_token(sdk_config).await
    } else {
        signer.db_connect_auth_token(sdk_config).await
    }
    .map_err(|e| DsqlError::TokenError(format!("Failed to generate auth token: {:?}", e)))?;

    Ok(token.to_string())
}

#[cfg(test)]
mod tests {
    use crate::util::Host;
    use crate::DsqlConfig;

    #[tokio::test]
    async fn test_generate_token_admin_user() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await
        .unwrap();

        // Token generation is a local SigV4 presigning operation.
        // Succeeds if AWS credentials are available, fails gracefully otherwise.
        let result = config.generate_token().await;
        match &result {
            Ok(token) => assert!(!token.is_empty(), "Token should not be empty"),
            Err(e) => {
                // Expected in environments without AWS credentials
                let msg = e.to_string();
                assert!(
                    msg.contains("token") || msg.contains("credential"),
                    "Expected token or credentials error, got: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_generate_token_non_admin_user() {
        let config = DsqlConfig::from_connection_string(
            "postgres://regular_user@example.dsql.us-east-1.on.aws/postgres",
        )
        .await
        .unwrap();

        let result = config.generate_token().await;
        match &result {
            Ok(token) => assert!(!token.is_empty(), "Token should not be empty"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("token") || msg.contains("credential"),
                    "Expected token or credentials error, got: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_generate_token_with_custom_duration() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?tokenDurationSecs=600",
        )
        .await
        .unwrap();

        assert_eq!(config.token_duration_secs, Some(600));
        // Should not panic regardless of credential availability
        let _ = config.generate_token().await;
    }

    #[tokio::test]
    async fn test_generate_token_with_profile() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?profile=nonexistent",
        )
        .await
        .unwrap();

        assert_eq!(config.profile, Some("nonexistent".to_string()));
        // May fail due to invalid profile, but should not panic
        let _ = config.generate_token().await;
    }

    #[tokio::test]
    async fn test_generate_token_requires_resolvable_region() {
        let config = DsqlConfig {
            host: Host::new("localhost"),
            region: None,
            ..DsqlConfig::default()
        };

        // Without a DSQL hostname, explicit region, or AWS_REGION env var,
        // region resolution may fail
        let result = config.generate_token().await;
        match &result {
            Ok(_) => {
                // AWS_REGION might be set in the environment, allowing region resolution
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("region") || msg.contains("token") || msg.contains("credential"),
                    "Expected region, token, or credentials error, got: {}",
                    msg
                );
            }
        }
    }
}
