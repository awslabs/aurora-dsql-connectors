// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DsqlError {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("pool error: {0}")]
    PoolError(String),

    #[error("token error: {0}")]
    TokenError(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("database error: {0}")]
    DatabaseError(String),

    #[error("OCC retry exhausted after {attempts} attempts: {message}")]
    OCCRetryExhausted {
        attempts: u32,
        message: String,
        #[source]
        source: Box<DsqlError>,
    },

    #[error("{0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, DsqlError>;
