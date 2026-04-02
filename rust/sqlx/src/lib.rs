// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Aurora DSQL connector for SQLx.

mod config;
pub mod connection;
mod error;
#[cfg(feature = "occ")]
mod occ_retry;
#[cfg(feature = "occ")]
mod occ_trait;
#[cfg(feature = "pool")]
pub mod pool;
#[cfg(feature = "occ")]
mod retry_query;
mod token;
pub(crate) mod util;

pub use aws_config::Region;
pub use config::{DsqlConnectOptions, DsqlConnectOptionsBuilder};
pub use error::{DsqlError, Result};
#[cfg(feature = "occ")]
pub use occ_retry::{is_occ_error, retry_on_occ, OCCRetryConfig, OCCRetryConfigBuilder};
#[cfg(feature = "occ")]
pub use occ_trait::{RetryExecutor, RetryPool};
#[cfg(feature = "occ")]
pub use retry_query::RetryQuery;
