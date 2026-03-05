// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlError, Result};
use aws_config::BehaviorVersion;
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};

pub async fn generate_token(host: &str, region: &str, user: &str, profile: Option<&str>, token_duration_secs: Option<u64>) -> Result<String> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_config::Region::new(region.to_string()));

    if let Some(profile_name) = profile {
        loader = loader.profile_name(profile_name);
    }

    let sdk_config = loader.load().await;

    generate_token_with_config(host, region, user, &sdk_config, token_duration_secs).await
}

pub async fn generate_token_with_config(
    host: &str,
    region: &str,
    user: &str,
    sdk_config: &aws_config::SdkConfig,
    token_duration_secs: Option<u64>,
) -> Result<String> {
    let credentials_provider = sdk_config
        .credentials_provider()
        .ok_or_else(|| DsqlError::TokenError("No credentials provider found".into()))?;

    let mut builder = Config::builder()
        .hostname(host)
        .region(aws_config::Region::new(region.to_string()))
        .credentials(credentials_provider);

    if let Some(duration) = token_duration_secs {
        builder = builder.expires_in(duration);
    }

    let signer = AuthTokenGenerator::new(
        builder
            .build()
            .map_err(|e| DsqlError::TokenError(format!("Failed to build auth token config: {}", e)))?,
    );

    let token = if user == "admin" {
        signer.db_connect_admin_auth_token(sdk_config).await.map_err(|e| DsqlError::TokenError(format!("Failed to generate auth token: {}", e)))?
    } else {
        signer.db_connect_auth_token(sdk_config).await.map_err(|e| DsqlError::TokenError(format!("Failed to generate auth token: {}", e)))?
    };

    Ok(token.to_string())
}
