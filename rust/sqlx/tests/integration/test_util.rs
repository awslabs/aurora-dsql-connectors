// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

pub fn build_conn_str() -> String {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set");
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    format!("postgres://{}@{}/postgres", user, endpoint)
}

/// Wraps a dynamic SQL string for compatibility with sqlx 0.9's `SqlSafeStr` trait.
/// In sqlx 0.8, this is a no-op passthrough.
#[cfg(feature = "sqlx-0_9")]
macro_rules! dyn_query {
    ($($arg:tt)*) => {
        ::aurora_dsql_sqlx_connector::sqlx_compat::sqlx::query(
            ::aurora_dsql_sqlx_connector::sqlx_compat::sqlx::AssertSqlSafe(format!($($arg)*))
        )
    };
}

#[cfg(feature = "sqlx-0_8")]
macro_rules! dyn_query {
    ($($arg:tt)*) => {
        ::aurora_dsql_sqlx_connector::sqlx_compat::sqlx::query(&format!($($arg)*))
    };
}

pub(crate) use dyn_query;
