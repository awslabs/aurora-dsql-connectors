// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Query builder with automatic OCC retry.

use crate::occ_retry::{retry_on_occ, OCCRetryConfig};
use crate::Result;
use sqlx::postgres::{PgArguments, PgQueryResult, PgRow};
use sqlx::query::Query;
use sqlx::{Encode, Executor, Postgres, Type};

/// Type-erased parameter storage for replay on retry.
trait BoundParameter: Send + Sync {
    fn bind_to<'q>(
        &self,
        query: Query<'q, Postgres, PgArguments>,
    ) -> Query<'q, Postgres, PgArguments>;
}
struct TypedParameter<T> {
    value: T,
}

impl<T> BoundParameter for TypedParameter<T>
where
    T: for<'q> Encode<'q, Postgres> + Type<Postgres> + Send + Sync + Clone + 'static,
{
    fn bind_to<'q>(
        &self,
        query: Query<'q, Postgres, PgArguments>,
    ) -> Query<'q, Postgres, PgArguments> {
        query.bind(self.value.clone())
    }
}

/// Query builder with automatic OCC retry.
///
/// Created via `executor.query()`. Most useful for single-statement writes.
/// Multi-statement transactions should use `retry_on_occ` instead
pub struct RetryQuery<'q, E> {
    sql: &'q str,
    params: Vec<Box<dyn BoundParameter>>,
    executor: E,
    config: OCCRetryConfig,
    retry_enabled: bool,
}

impl<'q, E> RetryQuery<'q, E> {
    pub(crate) fn new(sql: &'q str, executor: E, config: OCCRetryConfig) -> Self {
        Self {
            sql,
            params: Vec::new(),
            executor,
            config,
            retry_enabled: true,
        }
    }

    /// Bind a parameter. Stores value for replay on retry.
    pub fn bind<T>(mut self, value: T) -> Self
    where
        T: for<'e> Encode<'e, Postgres> + Type<Postgres> + Send + Sync + Clone + 'static,
    {
        self.params.push(Box::new(TypedParameter { value }));
        self
    }

    /// Disable retry for this query.
    pub fn without_retry(mut self) -> Self {
        self.retry_enabled = false;
        self
    }

    fn build_query(&self) -> Query<'q, Postgres, PgArguments> {
        let mut query = sqlx::query(self.sql);
        for param in &self.params {
            query = param.bind_to(query);
        }
        query
    }
}

/// Implement fetch methods with optional retry.
macro_rules! impl_fetch_method {
    ($method:ident, $ret:ty, $doc:expr) => {
        #[doc = $doc]
        pub async fn $method(self) -> Result<$ret>
        where
            for<'e> &'e E: Executor<'e, Database = Postgres>,
        {
            if !self.retry_enabled {
                return self
                    .build_query()
                    .$method(&self.executor)
                    .await
                    .map_err(crate::DsqlError::DatabaseError);
            }

            retry_on_occ(&self.config, || async {
                self.build_query().$method(&self.executor).await
            })
            .await
        }
    };
}

impl<'q, E> RetryQuery<'q, E> {
    impl_fetch_method!(
        execute,
        PgQueryResult,
        "Execute the query, returning the number of rows affected."
    );
    impl_fetch_method!(
        fetch_one,
        PgRow,
        "Fetch a single row, returning an error if no rows or multiple rows are returned."
    );
    impl_fetch_method!(
        fetch_all,
        Vec<PgRow>,
        "Fetch all rows returned by the query."
    );
    impl_fetch_method!(
        fetch_optional,
        Option<PgRow>,
        "Fetch a single row if present, or None if no rows are returned."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_stores_parameters() {
        let config = OCCRetryConfig::default();
        let query = RetryQuery::new("SELECT $1, $2", (), config)
            .bind(1_i32)
            .bind("test");

        assert_eq!(query.params.len(), 2);
        assert_eq!(query.sql, "SELECT $1, $2");
    }

    #[test]
    fn test_without_retry_disables_retry() {
        let config = OCCRetryConfig::default();
        let query = RetryQuery::new("SELECT 1", (), config).without_retry();

        assert!(!query.retry_enabled);
    }

    #[test]
    fn test_retry_enabled_by_default() {
        let config = OCCRetryConfig::default();
        let query = RetryQuery::new("SELECT 1", (), config);

        assert!(query.retry_enabled);
    }

    #[test]
    fn test_bind_chainable() {
        let config = OCCRetryConfig::default();
        let query = RetryQuery::new("SELECT $1, $2, $3", (), config)
            .bind(1_i32)
            .bind("test")
            .bind(vec![1u8, 2u8, 3u8]);

        assert_eq!(query.params.len(), 3);
    }
}
