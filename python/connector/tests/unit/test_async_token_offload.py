# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

"""Async connectors must not generate tokens on the event loop thread."""

import asyncio
import threading
import time
from unittest.mock import AsyncMock, patch

import pytest

HOST = "fake-cluster.dsql.us-east-1.on.aws"
SLOW_SECS = 0.3


def _slow_token_call(threads):
    def fake(dsn, kwargs, driver_name=None):
        threads.append(threading.current_thread())
        time.sleep(SLOW_SECS)
        return {"host": HOST, "user": "admin", "password": "token"}

    return fake


async def _max_loop_stall(coro):
    """Run coro while a ticker measures how long the event loop is blocked."""
    stalls = []

    async def ticker():
        while True:
            start = time.perf_counter()
            await asyncio.sleep(0.01)
            stalls.append(time.perf_counter() - start - 0.01)

    task = asyncio.create_task(ticker())
    await asyncio.sleep(0.05)
    try:
        await coro
    finally:
        task.cancel()
    return max(stalls)


@pytest.mark.unit
class TestAsyncTokenOffload:
    @pytest.mark.asyncio
    async def test_asyncpg_token_generation_off_event_loop(self):
        import aurora_dsql_asyncpg as dsql

        threads = []
        with (
            patch(
                "aurora_dsql_asyncpg.connector.ConnectionUtilities.parse_properties_and_set_token",
                side_effect=_slow_token_call(threads),
            ),
            patch(
                "aurora_dsql_asyncpg.connector.asyncpg.connect",
                new=AsyncMock(return_value=object()),
            ),
        ):
            stall = await _max_loop_stall(dsql.connect(host=HOST))

        assert threads[0] is not threading.main_thread()
        assert stall < SLOW_SECS / 2

    @pytest.mark.asyncio
    async def test_psycopg_async_token_generation_off_event_loop(self):
        import psycopg

        from aurora_dsql_psycopg import DSQLAsyncConnection

        threads = []
        with (
            patch(
                "aurora_dsql_psycopg.connection_class.ConnectionUtilities."
                "parse_properties_and_set_token",
                side_effect=_slow_token_call(threads),
            ),
            patch.object(
                psycopg.AsyncConnection,
                "connect",
                new=AsyncMock(return_value=object()),
            ),
        ):
            stall = await _max_loop_stall(DSQLAsyncConnection.connect(f"host={HOST}"))

        assert threads[0] is not threading.main_thread()
        assert stall < SLOW_SECS / 2
