// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::util;

#[test]
fn test_parse_region_standard_hostname() {
    let region = util::parse_region("abc123.dsql.us-east-1.on.aws");
    assert_eq!(region, Some("us-east-1".to_string()));
}

#[test]
fn test_parse_region_other_regions() {
    assert_eq!(
        util::parse_region("abc123.dsql.us-west-2.on.aws"),
        Some("us-west-2".to_string())
    );
    assert_eq!(
        util::parse_region("abc123.dsql.eu-west-1.on.aws"),
        Some("eu-west-1".to_string())
    );
    assert_eq!(
        util::parse_region("abc123.dsql.ap-southeast-1.on.aws"),
        Some("ap-southeast-1".to_string())
    );
}

#[test]
fn test_parse_region_invalid_hostname() {
    assert_eq!(util::parse_region("localhost"), None);
    assert_eq!(util::parse_region("example.com"), None);
    assert_eq!(util::parse_region(""), None);
}

#[test]
fn test_is_cluster_id_valid() {
    assert!(util::is_cluster_id("abcdefghijklmnopqrstuvwxyz"));
    assert!(util::is_cluster_id("a1b2c3d4e5f6g7h8i9j0klmnop"));
}

#[test]
fn test_is_cluster_id_invalid() {
    assert!(!util::is_cluster_id(""));
    assert!(!util::is_cluster_id("too-short"));
    assert!(!util::is_cluster_id("abc.def.us-east-1.on.aws"));
    assert!(!util::is_cluster_id("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    assert!(!util::is_cluster_id("abcdefghijklmnopqrstuvwxy"));  // 25 chars
    assert!(!util::is_cluster_id("abcdefghijklmnopqrstuvwxyza")); // 27 chars
}

#[test]
fn test_build_hostname() {
    assert_eq!(
        util::build_hostname("abc123", "us-east-1"),
        "abc123.dsql.us-east-1.on.aws"
    );
}

#[test]
fn test_build_application_name_no_prefix() {
    let name = util::build_application_name(None);
    assert!(name.starts_with("aurora-dsql-rust-sqlx/"));
}

#[test]
fn test_build_application_name_with_prefix() {
    let name = util::build_application_name(Some("myapp"));
    assert!(name.starts_with("myapp:aurora-dsql-rust-sqlx/"));
}

#[test]
fn test_build_application_name_empty_prefix() {
    let name = util::build_application_name(Some(""));
    assert!(name.starts_with("aurora-dsql-rust-sqlx/"));
}

#[test]
fn test_region_from_env() {
    // Combined into one test to avoid env var races with parallel tests.

    // AWS_REGION takes priority
    std::env::set_var("AWS_REGION", "us-west-2");
    std::env::remove_var("AWS_DEFAULT_REGION");
    assert_eq!(util::region_from_env(), Some("us-west-2".to_string()));

    // Falls back to AWS_DEFAULT_REGION
    std::env::remove_var("AWS_REGION");
    std::env::set_var("AWS_DEFAULT_REGION", "eu-central-1");
    assert_eq!(util::region_from_env(), Some("eu-central-1".to_string()));

    // Returns None when neither is set
    std::env::remove_var("AWS_REGION");
    std::env::remove_var("AWS_DEFAULT_REGION");
    assert_eq!(util::region_from_env(), None);
}
