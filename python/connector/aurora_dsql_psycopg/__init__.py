# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from dsql_core._version import __version__
from dsql_core.occ_retry import OCCRetryConfig, is_occ_error

from .connection_class import DSQLAsyncConnection, DSQLConnection
from .pool import AuroraDSQLAsyncPool, AuroraDSQLPool, create_async_pool, create_pool

# DBAPI compliance
connect = DSQLConnection.connect
apilevel = "2.0"
threadsafety = 2
paramstyle = "pyformat"


__all__ = [
    "connect",
    "DSQLConnection",
    "DSQLAsyncConnection",
    "AuroraDSQLPool",
    "AuroraDSQLAsyncPool",
    "create_pool",
    "create_async_pool",
    "OCCRetryConfig",
    "is_occ_error",
    "__version__",
]
