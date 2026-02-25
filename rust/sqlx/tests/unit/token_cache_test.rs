use aurora_dsql_sqlx_connector::token_cache::TokenCache;

#[tokio::test]
async fn test_token_cache_clear() {
    let cache = TokenCache::new(
        "example.dsql.us-east-1.on.aws".to_string(),
        "us-east-1".to_string(),
        None,
    );

    // Clear should not panic
    cache.clear().await;
}
