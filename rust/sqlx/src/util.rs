// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Parse the AWS region from a DSQL hostname.
/// Matches pattern: *.dsql{suffix?}.{region}.on.aws
/// e.g. "cluster123.dsql.us-east-1.on.aws" → Some("us-east-1")
pub fn parse_region(host: &str) -> Option<String> {
    // Must end with .on.aws
    let stem = host.strip_suffix(".on.aws")?;
    let parts: Vec<&str> = stem.split('.').collect();
    // Need at least: {cluster}.dsql{suffix?}.{region}
    if parts.len() >= 3 {
        // Find the dsql segment
        if let Some(dsql_idx) = parts.iter().position(|p| p.starts_with("dsql")) {
            // Region is the segment after the dsql segment
            if dsql_idx + 1 < parts.len() {
                let region = parts[dsql_idx + 1];
                if !region.is_empty() {
                    return Some(region.to_string());
                }
            }
        }
    }
    None
}

/// Check if a string looks like a bare DSQL cluster ID
/// (26 lowercase alphanumeric characters, no dots).
pub fn is_cluster_id(input: &str) -> bool {
    !input.is_empty()
        && !input.contains('.')
        && input.len() == 26
        && input.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// Build a full DSQL hostname from a cluster ID and region.
/// e.g. ("abc123...", "us-east-1") → "abc123....dsql.us-east-1.on.aws"
pub fn build_hostname(cluster_id: &str, region: &str) -> String {
    format!("{}.dsql.{}.on.aws", cluster_id, region)
}

/// Get AWS region from environment variables.
/// Checks `AWS_REGION` first, then `AWS_DEFAULT_REGION`.
pub fn region_from_env() -> Option<String> {
    std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .ok()
}

/// Build the application_name string for the Postgres startup packet.
pub fn build_application_name(prefix: Option<&str>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    match prefix.map(str::trim) {
        Some(p) if !p.is_empty() => format!("{}:aurora-dsql-rust-sqlx/{}", p, version),
        _ => format!("aurora-dsql-rust-sqlx/{}", version),
    }
}
