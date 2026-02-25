use aurora_dsql_sqlx_connector::DsqlError;

#[test]
fn test_error_display() {
    let err = DsqlError::Error("test error".to_string());
    assert_eq!(format!("{}", err), "test error");
}
