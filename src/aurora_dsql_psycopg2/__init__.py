"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from .connector import connect
from .pool import AuroraDSQLThreadedConnectionPool

apilevel = "2.0"
threadsafety = 2
paramstyle = "pyformat"

__all__ = ["connect", "AuroraDSQLThreadedConnectionPool"]
