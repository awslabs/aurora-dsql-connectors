# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from dsql_core._version import __version__

from .connection_class import DSQLAsyncConnection, DSQLConnection

# DBAPI compliance
connect = DSQLConnection.connect
apilevel = "2.0"
threadsafety = 2
paramstyle = "pyformat"


__all__ = ["connect", "DSQLConnection", "DSQLAsyncConnection", "__version__"]
