// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_bb8::DsqlConnectionManager;
use aurora_dsql_sqlx_connector::DsqlConnectOptions;

pub fn build_conn_str() -> String {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set");
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    format!("postgres://{}@{}/postgres", user, endpoint)
}

pub async fn build_pool(conn_str: &str) -> bb8::Pool<DsqlConnectionManager> {
    let opts = DsqlConnectOptions::from_connection_string(conn_str).unwrap();
    let manager = DsqlConnectionManager::new(opts);
    bb8::Pool::builder()
        .max_size(5)
        .build(manager)
        .await
        .unwrap()
}
