# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import asyncio
import os
import ssl

import pytest

import aurora_dsql_asyncpg as dsql
from dsql_core.occ_retry import OCCRetryConfig

from .common_integration_test_definitions import CustomCredentialProvider


class TestIntegrationAsyncpgPool:
    """Integration tests for Aurora DSQL asyncpg pool functionality."""

    @staticmethod
    async def _assert_pool_connection_functional(pool):
        """Verify the pool connection functions at a basic level."""
        conn = await pool.acquire()
        try:
            result = await conn.fetchval("SELECT 1")
            assert result == 1
        finally:
            await pool.release(conn)

    @pytest.fixture
    def cluster_config(self):
        """Get cluster configuration from environment variables."""
        config = {
            "host": os.getenv("CLUSTER_ENDPOINT"),
            "region": os.getenv("REGION", "us-east-1"),
            "user": os.getenv("CLUSTER_USER", "admin"),
            "database": os.getenv("DSQL_DATABASE", "postgres"),
        }
        aws_profile = os.getenv("AWS_PROFILE")
        if aws_profile:
            config["profile"] = aws_profile

        if not config["host"]:
            raise ValueError("CLUSTER_ENDPOINT environment variable not set")

        return config

    @pytest.mark.asyncio
    async def test_pool_basic_operations(self, cluster_config):
        """Test basic pool operations."""
        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_basic_operations_host_only(self, cluster_config):
        """Test basic pool operations."""
        pool = await dsql.create_pool(cluster_config["host"], min_size=2, max_size=5)

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_connection_with_custom_credentials_provider(self, cluster_config):
        """Test connection using custom credentials provider."""

        config = {
            "host": cluster_config["host"],
            "user": cluster_config["user"],
        }

        custom_provider = CustomCredentialProvider()
        config["custom_credentials_provider"] = custom_provider

        pool = await dsql.create_pool(min_size=2, max_size=5, **config)
        assert custom_provider.load_called, "Custom credentials provider load() was not called"

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_basic_operations_connection_string_format(self, cluster_config):
        """Test basic pool operations."""

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?user={cluster_config['user']}"

        if cluster_config.get("profile"):
            conn_str += f"&profile={cluster_config['profile']}"

        pool = await dsql.create_pool(conn_str, min_size=2, max_size=5)

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_basic_operations_ssl_context(self, cluster_config, ssl_cert_path):
        """Test basic pool operations."""

        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = True  # This enables hostname verification (verify-full)
        ssl_context.verify_mode = ssl.CERT_REQUIRED  # This is equivalent to verify-full
        ssl_context.load_verify_locations(ssl_cert_path)

        cluster_config["ssl"] = ssl_context

        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_basic_operations_dsn_ssl(self, cluster_config, ssl_cert_path):
        """Test basic pool operations."""

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?ssl=verify-full&sslrootcert={ssl_cert_path}"

        pool = await dsql.create_pool(conn_str, min_size=2, max_size=5)

        try:
            # Test acquiring and releasing connections
            await self._assert_pool_connection_functional(pool)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_basic_operations_context_manager(self, cluster_config):
        """Test basic pool operations."""
        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        try:
            async with pool.acquire() as conn:
                result = await conn.fetchval("SELECT 1")
                assert result == 1
        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_multiple_connections(self, cluster_config):
        """Test multiple concurrent connections."""
        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        try:
            # Test multiple connections
            connections = []
            try:
                for _ in range(5):
                    conn = await pool.acquire()
                    connections.append(conn)
                    result = await conn.fetchval("SELECT 1")
                    assert result == 1
            finally:
                for conn in connections:
                    await pool.release(conn)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_database_operations(self, cluster_config):
        """Test database operations through pool."""
        table_name = "test_pool_operations_asyncpg"
        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        try:
            conn = await pool.acquire()
            try:
                # Create table
                await conn.execute(
                    f"""
                    CREATE TABLE IF NOT EXISTS {table_name} (
                        id uuid NOT NULL DEFAULT gen_random_uuid(),
                        name TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
                """
                )

                # Insert data
                result = await conn.fetchrow(
                    f"INSERT INTO {table_name} (name) VALUES ($1) RETURNING id",
                    "pool_test",
                )
                record_id = result["id"]

                # Query data
                result = await conn.fetchrow(
                    f"SELECT name FROM {table_name} WHERE id = $1",
                    record_id,
                )
                assert result[0] == "pool_test"

                # Clean up
                await conn.execute(
                    f"DELETE FROM {table_name} WHERE id = $1",
                    record_id,
                )

            finally:
                await pool.release(conn)

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_pool_concurrent_operations(self, cluster_config):
        """Test concurrent operations using pool."""

        pool = await dsql.create_pool(min_size=2, max_size=5, **cluster_config)

        async def worker(worker_id):
            conn = await pool.acquire()
            try:
                result = await conn.fetchval("SELECT $1::int", worker_id)
                return result
            finally:
                await pool.release(conn)

        try:
            # Run multiple workers concurrently
            tasks = [worker(i) for i in range(5)]
            results = await asyncio.gather(*tasks)

            # Verify all workers completed successfully
            assert results == list(range(5))

        finally:
            await pool.close()

    @pytest.mark.asyncio
    async def test_occ_retry_deterministic_conflict(self, cluster_config):
        """Deterministic OCC test using T1/T2 interleaving:
        T1: BEGIN, SELECT
        T2: BEGIN, UPDATE, COMMIT  (forces the conflict)
        T1: UPDATE, COMMIT         (guaranteed OCC, retry must work)
        """
        table_name = "test_occ_asyncpg_deterministic"

        pool = await dsql.create_pool(
            retry=OCCRetryConfig(max_retries=5),
            min_size=3,
            max_size=5,
            **cluster_config,
        )
        try:
            async with pool.acquire() as conn:
                await conn.execute(f"""
                    CREATE TABLE IF NOT EXISTS {table_name} (
                        id INT PRIMARY KEY, value INT
                    )
                """)
                await conn.execute(
                    f"INSERT INTO {table_name} (id, value) VALUES (1, 0)"
                    f" ON CONFLICT (id) DO UPDATE SET value = 0"
                )

            t1_has_read = asyncio.Event()
            t2_has_committed = asyncio.Event()
            t1_attempts = 0

            async def t1(conn):
                nonlocal t1_attempts
                t1_attempts += 1
                row = await conn.fetchrow(f"SELECT value FROM {table_name} WHERE id = 1")
                t1_has_read.set()
                await asyncio.wait_for(t2_has_committed.wait(), timeout=10)
                await conn.execute(
                    f"UPDATE {table_name} SET value = $1 WHERE id = 1",
                    row["value"] + 1,
                )

            async def t2(conn):
                await asyncio.wait_for(t1_has_read.wait(), timeout=10)
                await conn.execute(f"UPDATE {table_name} SET value = 100 WHERE id = 1")

            async def run_t2():
                async with pool.acquire() as conn:
                    await conn.execute("BEGIN")
                    await t2(conn)
                    await conn.execute("COMMIT")
                t2_has_committed.set()

            t2_task = asyncio.create_task(run_t2())
            await pool.run_transaction(t1)
            await t2_task

            async with pool.acquire() as conn:
                result = await conn.fetchval(f"SELECT value FROM {table_name} WHERE id = 1")
                assert result == 101
            assert t1_attempts >= 2, f"Expected retry but t1 ran only {t1_attempts} time(s)"
        finally:
            for attempt in range(5):
                try:
                    async with pool.acquire() as conn:
                        await conn.execute(f"DROP TABLE IF EXISTS {table_name}")
                    break
                except Exception:
                    if attempt == 4:
                        raise
                    await asyncio.sleep(1)
            await pool.close()

    @pytest.mark.asyncio
    async def test_occ_non_occ_error_propagates_immediately(self, cluster_config):
        """A non-OCC database error should propagate without retry."""
        pool = await dsql.create_pool(
            retry=OCCRetryConfig(max_retries=5),
            min_size=2,
            max_size=5,
            **cluster_config,
        )
        try:
            call_count = 0

            async def bad_query(conn):
                nonlocal call_count
                call_count += 1
                await conn.execute("SELECT * FROM nonexistent_table_xyz_12345")

            with pytest.raises(Exception):
                await pool.run_transaction(bad_query)

            assert call_count == 1
        finally:
            await pool.close()
