"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from typing import Optional

import psycopg2
from botocore.credentials import CredentialProvider
from psycopg2 import pool

from dsql_core.connection_properties import ConnectionProperties
from dsql_core.token_manager import TokenManager


class AuroraDSQLThreadedConnectionPool(pool.ThreadedConnectionPool):
    """Custom ThreadedConnectionPool that generates fresh IAM tokens per connection."""

    def __init__(
        self,
        minconn,
        maxconn,
        *args,
        custom_credentials_provider: Optional[CredentialProvider] = None,
        **kwargs,
    ):

        if custom_credentials_provider is not None:
            kwargs["custom_credentials_provider"] = custom_credentials_provider

        dsql_params, pool_params = ConnectionProperties.parse_properties(None, kwargs)
        self._dsql_params = dsql_params
        # Initialize with dummy password, will be replaced per connection
        super().__init__(minconn, maxconn, *args, **pool_params)

    def _connect(self, key=None):
        """Create connection with fresh IAM token."""
        token = TokenManager.get_token(self._dsql_params)

        # Update kwargs with fresh token
        self._kwargs["password"] = token

        # Create connection
        conn = psycopg2.connect(*self._args, **self._kwargs)
        if key is not None:
            self._used[key] = conn
            self._rused[id(conn)] = key
        else:
            self._pool.append(conn)
        return conn

    def __enter__(self):
        """Enter context manager."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager and close all connections."""
        self.closeall()
        return False
