// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "sqlx-0_8")]
pub use sqlx_0_8 as sqlx;

#[cfg(feature = "sqlx-0_9")]
pub use sqlx_0_9 as sqlx;

#[cfg(all(feature = "sqlx-0_8", feature = "sqlx-0_9"))]
compile_error!("Features `sqlx-0_8` and `sqlx-0_9` are mutually exclusive.");

#[cfg(not(any(feature = "sqlx-0_8", feature = "sqlx-0_9")))]
compile_error!("Either `sqlx-0_8` or `sqlx-0_9` must be enabled.");
