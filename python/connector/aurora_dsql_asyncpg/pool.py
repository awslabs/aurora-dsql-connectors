# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from typing import Any

import asyncpg

from .connector import connect as dsql_connect


async def create_pool(
    dsn=None,
    *,
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
):
    """Create Aurora DSQL connection pool with fresh token generation."""

    async def reset_connection(conn):
        await conn.execute("RESET ALL;")

    pool = await asyncpg.create_pool(
        dsn,
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

    return pool
