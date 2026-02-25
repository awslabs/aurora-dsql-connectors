use aurora_dsql_sqlx_connector::{DsqlConfig, Result};

#[test]
fn test_parse_basic_connection_string() -> Result<()> {
    let config = DsqlConfig::from_connection_string(
        "dsql://admin@example.dsql.us-east-1.on.aws:5432/postgres",
    )?;

    assert_eq!(config.user, "admin");
    assert_eq!(config.host, "example.dsql.us-east-1.on.aws");
    assert_eq!(config.port, 5432);
    assert_eq!(config.database, "postgres");
    assert_eq!(config.region, None);
    Ok(())
}

#[test]
fn test_parse_with_region_param() -> Result<()> {
    let config = DsqlConfig::from_connection_string(
        "dsql://admin@example.dsql.us-west-2.on.aws/postgres?region=us-west-2",
    )?;

    assert_eq!(config.region, Some("us-west-2".to_string()));
    Ok(())
}

#[test]
fn test_parse_with_profile_param() -> Result<()> {
    let config = DsqlConfig::from_connection_string(
        "dsql://admin@example.dsql.us-east-1.on.aws/postgres?profile=dev",
    )?;

    assert_eq!(config.profile, Some("dev".to_string()));
    Ok(())
}

#[test]
fn test_parse_with_region_and_profile() -> Result<()> {
    let config = DsqlConfig::from_connection_string(
        "dsql://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1&profile=prod",
    )?;

    assert_eq!(config.region, Some("us-east-1".to_string()));
    assert_eq!(config.profile, Some("prod".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_resolve_region_from_param() -> Result<()> {
    let config = DsqlConfig::from_connection_string(
        "dsql://admin@example.dsql.us-east-1.on.aws/postgres?region=us-east-1",
    )?;

    assert_eq!(config.resolve_region().await?, "us-east-1");
    Ok(())
}

#[tokio::test]
async fn test_resolve_region_from_hostname() -> Result<()> {
    let config =
        DsqlConfig::from_connection_string("dsql://admin@example.dsql.us-west-2.on.aws/postgres")?;

    assert_eq!(config.resolve_region().await?, "us-west-2");
    Ok(())
}

#[test]
fn test_invalid_connection_string() {
    let result = DsqlConfig::from_connection_string("invalid://connection");
    assert!(result.is_err());
}
