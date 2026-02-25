use crate::{DsqlError, Result};
use aws_config::BehaviorVersion;
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};

pub async fn generate_token(host: &str, region: &str, profile: Option<&str>) -> Result<String> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_config::Region::new(region.to_string()));

    if let Some(profile_name) = profile {
        loader = loader.profile_name(profile_name);
    }

    let sdk_config = loader.load().await;

    let credentials_provider = sdk_config
        .credentials_provider()
        .ok_or_else(|| DsqlError::TokenError("No credentials provider found".into()))?;

    let signer = AuthTokenGenerator::new(
        Config::builder()
            .hostname(host)
            .region(aws_config::Region::new(region.to_string()))
            .credentials(credentials_provider)
            .build()
            .map_err(|e| DsqlError::TokenError(format!("Failed to build auth token config: {}", e)))?,
    );

    let token = signer
        .db_connect_admin_auth_token(&sdk_config)
        .await
        .map_err(|e| DsqlError::TokenError(format!("Failed to generate auth token: {}", e)))?;

    Ok(token.to_string())
}
