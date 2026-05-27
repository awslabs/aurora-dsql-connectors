# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import logging
from collections.abc import Awaitable, Callable
from typing import Any, TypeVar

from psycopg import AsyncConnection, Connection
from psycopg_pool import AsyncConnectionPool, ConnectionPool

from dsql_core.occ_retry import OCCRetryConfig, _retry_async, _retry_sync, resolve_retry_config

from .connection_class import DSQLAsyncConnection, DSQLConnection

logger = logging.getLogger(__name__)

T = TypeVar("T")


class AuroraDSQLPool(ConnectionPool):
    """Aurora DSQL connection pool with OCC retry support."""

    def __init__(self, *args, retry: OCCRetryConfig | bool | None = None, **kwargs):
        self._retry = retry
        super().__init__(*args, **kwargs)

    def run_transaction(
        self,
        callback: Callable[[Connection], T],
        *,
        retry: OCCRetryConfig | bool | None = None,
    ) -> T:
        """Execute callback in a transaction with optional OCC retry."""
        config = resolve_retry_config(self._retry, retry)

        def execute() -> T:
            with self.connection() as conn:
                try:
                    result = callback(conn)
                    conn.commit()
                    return result
                except BaseException:
                    try:
                        conn.rollback()
                    except Exception as e:
                        logger.debug("Rollback failed: %s", e)
                    raise

        if config is None:
            return execute()
        return _retry_sync(execute, config)


class AuroraDSQLAsyncPool(AsyncConnectionPool):
    """Aurora DSQL async connection pool with OCC retry support."""

    def __init__(self, *args, retry: OCCRetryConfig | bool | None = None, **kwargs):
        self._retry = retry
        super().__init__(*args, **kwargs)

    async def run_transaction(
        self,
        callback: Callable[[AsyncConnection], Awaitable[T]],
        *,
        retry: OCCRetryConfig | bool | None = None,
    ) -> T:
        """Execute callback in a transaction with optional OCC retry."""
        config = resolve_retry_config(self._retry, retry)

        async def execute() -> T:
            async with self.connection() as conn:
                try:
                    result = await callback(conn)
                    await conn.commit()
                    return result
                except BaseException:
                    try:
                        await conn.rollback()
                    except Exception as e:
                        logger.debug("Rollback failed: %s", e)
                    raise

        if config is None:
            return await execute()
        return await _retry_async(execute, config)


def create_pool(
    conninfo: str = "",
    *,
    retry: OCCRetryConfig | bool | None = None,
    **kwargs: Any,
) -> AuroraDSQLPool:
    """Create Aurora DSQL connection pool with IAM authentication.

    Pool is returned unopened; use ``with pool:`` or call ``pool.open()``.
    """
    if "connection_class" in kwargs:
        if not issubclass(kwargs["connection_class"], DSQLConnection):
            raise TypeError("connection_class must be a subclass of DSQLConnection for IAM auth")
    else:
        kwargs["connection_class"] = DSQLConnection
    return AuroraDSQLPool(conninfo, retry=retry, **kwargs)


def create_async_pool(
    conninfo: str = "",
    *,
    retry: OCCRetryConfig | bool | None = None,
    **kwargs: Any,
) -> AuroraDSQLAsyncPool:
    """Create Aurora DSQL async connection pool with IAM authentication.

    Pool is returned unopened; use ``async with pool:`` or call ``await pool.open()``.
    """
    if "connection_class" in kwargs:
        if not issubclass(kwargs["connection_class"], DSQLAsyncConnection):
            raise TypeError(
                "connection_class must be a subclass of DSQLAsyncConnection for IAM auth"
            )
    else:
        kwargs["connection_class"] = DSQLAsyncConnection
    return AuroraDSQLAsyncPool(conninfo, retry=retry, **kwargs)
