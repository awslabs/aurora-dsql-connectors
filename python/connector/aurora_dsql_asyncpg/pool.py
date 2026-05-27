# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import logging
from collections.abc import Awaitable, Callable
from typing import Any, TypeVar

import asyncpg

from dsql_core.occ_retry import OCCRetryConfig, _retry_async, resolve_retry_config

from .connector import connect as dsql_connect

logger = logging.getLogger(__name__)

T = TypeVar("T")


class AuroraDSQLPool(asyncpg.Pool):
    """Aurora DSQL connection pool with OCC retry support."""

    __slots__ = ("_retry",)

    def __init__(self, *connect_args, retry: OCCRetryConfig | bool | None = None, **kwargs):
        self._retry = retry
        super().__init__(*connect_args, **kwargs)

    async def run_transaction(
        self,
        callback: Callable[[asyncpg.Connection], Awaitable[T]],
        *,
        retry: OCCRetryConfig | bool | None = None,
    ) -> T:
        """Execute callback in a transaction with optional OCC retry."""
        config = resolve_retry_config(self._retry, retry)

        async def execute() -> T:
            conn = await self.acquire()
            try:
                await conn.execute("BEGIN")
                try:
                    result = await callback(conn)
                    await conn.execute("COMMIT")
                    return result
                except BaseException:
                    try:
                        await conn.execute("ROLLBACK")
                    except Exception as e:
                        logger.debug("Rollback failed: %s", e)
                    raise
            finally:
                await self.release(conn)

        if config is None:
            return await execute()
        return await _retry_async(execute, config)


async def create_pool(
    dsn=None,
    *,
    retry: OCCRetryConfig | bool | None = None,
    min_size=10,
    max_size=10,
    max_queries=50000,
    max_inactive_connection_lifetime=300.0,
    setup=None,
    init=None,
    reset=None,
    loop=None,
    connection_class=asyncpg.Connection,
    record_class=asyncpg.Record,
    **connect_kwargs: Any,
) -> AuroraDSQLPool:
    """Create Aurora DSQL connection pool with fresh token generation."""

    async def reset_connection(conn):
        await conn.execute("RESET ALL;")

    return await AuroraDSQLPool(
        dsn,
        retry=retry,
        min_size=min_size,
        max_size=max_size,
        max_queries=max_queries,
        max_inactive_connection_lifetime=max_inactive_connection_lifetime,
        connect=dsql_connect,
        setup=setup,
        init=init,
        reset=reset or reset_connection,
        loop=loop,
        connection_class=connection_class,
        record_class=record_class,
        **connect_kwargs,
    )
