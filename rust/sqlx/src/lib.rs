// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Aurora DSQL connector for SQLx.
//!
//! This crate is under active development.

mod config;
mod connection;
mod error;
#[cfg(feature = "occ")]
mod occ_retry;
#[cfg(feature = "pool")]
mod pool;
mod token;
pub(crate) mod util;

pub use config::{DsqlConfig, DsqlConfigBuilder};
pub use connection::dsql_connect;
pub use error::{DsqlError, Result};
#[cfg(all(feature = "occ", feature = "pool"))]
pub use occ_retry::with_retry;
#[cfg(feature = "occ")]
pub use occ_retry::{is_occ_dsql_error, retry_on_occ, OCCRetryConfig, OCCRetryConfigBuilder};
#[cfg(feature = "pool")]
pub use pool::DsqlPool;
pub use util::{ClusterId, Host, Region, User};
