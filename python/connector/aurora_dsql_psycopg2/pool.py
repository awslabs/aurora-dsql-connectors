# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import logging
from collections.abc import Callable
from typing import TypeVar

import psycopg2
from botocore.credentials import CredentialProvider
from psycopg2 import pool
from psycopg2.extensions import connection

from dsql_core.connection_properties import ConnectionProperties, build_application_name
from dsql_core.occ_retry import OCCRetryConfig, _retry_sync, resolve_retry_config
from dsql_core.token_manager import TokenManager

logger = logging.getLogger(__name__)

T = TypeVar("T")


class AuroraDSQLThreadedConnectionPool(pool.ThreadedConnectionPool):
    """Aurora DSQL connection pool with OCC retry support."""

    def __init__(
        self,
        minconn,
        maxconn,
        *args,
        retry: OCCRetryConfig | bool | None = None,
        custom_credentials_provider: CredentialProvider | None = None,
        **kwargs,
    ):
        self._retry = retry

        if custom_credentials_provider is not None:
            kwargs["custom_credentials_provider"] = custom_credentials_provider

        dsql_params, pool_params = ConnectionProperties.parse_properties(None, kwargs)
        self._dsql_params = dsql_params

        orm_prefix = pool_params.get("application_name")
        pool_params["application_name"] = build_application_name("psycopg2", orm_prefix)

        super().__init__(minconn, maxconn, *args, **pool_params)

    def _connect(self, key=None):
        """Create connection with fresh IAM token."""
        token = TokenManager.get_token(self._dsql_params)
        self._kwargs["password"] = token

        conn = psycopg2.connect(*self._args, **self._kwargs)
        if key is not None:
            self._used[key] = conn
            self._rused[id(conn)] = key
        else:
            self._pool.append(conn)
        return conn

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.closeall()
        return False

    def run_transaction(
        self,
        callback: Callable[[connection], T],
        *,
        retry: OCCRetryConfig | bool | None = None,
    ) -> T:
        """Execute callback in a transaction with optional OCC retry."""
        config = resolve_retry_config(self._retry, retry)

        def execute() -> T:
            conn = self.getconn()
            try:
                conn.autocommit = False
                result = callback(conn)
                conn.commit()
                return result
            except BaseException:
                try:
                    conn.rollback()
                except Exception as e:
                    logger.debug("Rollback failed: %s", e)
                raise
            finally:
                self.putconn(conn)

        if config is None:
            return execute()
        return _retry_sync(execute, config)
