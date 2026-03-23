// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DsqlError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DsqlError {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("token error: {0}")]
    TokenError(String),

    #[error("connection error: {0}")]
    ConnectionError(#[source] sqlx::Error),

    #[error("database error: {0}")]
    DatabaseError(#[source] sqlx::Error),

    #[error("OCC retry exhausted after {attempts} attempts: {source}")]
    OCCRetryExhausted {
        attempts: u32,
        #[source]
        source: Box<DsqlError>,
    },
}
