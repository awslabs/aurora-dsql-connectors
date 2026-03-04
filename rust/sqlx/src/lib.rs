// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

<<<<<<< rust-connector-with-sqlx
mod config;
mod connection;
mod error;
pub mod occ_retry;
pub mod util;
#[cfg(feature = "pool")]
mod pool;
mod token;

pub use config::DsqlConfig;
pub use connection::DsqlConnection;
pub use error::{DsqlError, Result};
#[cfg(feature = "pool")]
pub use pool::DsqlPool;
=======
//! Aurora DSQL connector for SQLx.
//!
//! This crate is under active development.
>>>>>>> main
