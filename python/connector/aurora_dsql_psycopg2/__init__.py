# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from dsql_core._version import __version__
from dsql_core.occ_retry import OCCRetryConfig, is_occ_error

from .connector import connect
from .pool import AuroraDSQLThreadedConnectionPool

apilevel = "2.0"
threadsafety = 2
paramstyle = "pyformat"

__all__ = [
    "connect",
    "AuroraDSQLThreadedConnectionPool",
    "OCCRetryConfig",
    "is_occ_error",
    "__version__",
]
