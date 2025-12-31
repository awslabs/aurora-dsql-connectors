"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import os
import ssl

import pytest
from asyncpg.exceptions import ProtocolViolationError

import aurora_dsql_asyncpg as dsql

from .common_integration_test_definitions import CustomCredentialProvider


# @pytest.mark.integration
class TestIntegrationAsyncpg:
    """Integration tests requiring real Aurora DSQL cluster."""

    @staticmethod
    async def _assert_connection_functional(conn):
        """Verify the provided connection functions at a basic level. Closes the connection."""
        try:
            row = await conn.fetchrow("SELECT 1")
            assert row[0] == 1
        finally:
            await conn.close()

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
    async def test_basic_connection(self, cluster_config):
        """Test basic connection to Aurora DSQL."""

        conn = await dsql.connect(**cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_with_named_parameters(self, cluster_config):
        """Test basic connection to Aurora DSQL with named parameters."""

        conn = await dsql.connect(
            host=cluster_config["host"],
            user=cluster_config["user"],
            database=cluster_config["database"],
        )
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connection_with_host_only(self, cluster_config):
        """Test connection to Aurora DSQL with URL only.
        Region should be extracted from host, and the defaults will be applied"""

        conn = await dsql.connect(cluster_config["host"])
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connection_with_cluster_id_only(self, cluster_config, monkeypatch):
        """Test connection with cluster ID using default region from environment."""
        monkeypatch.setenv("AWS_DEFAULT_REGION", cluster_config["region"])

        cluster_id = cluster_config["host"].split(".")[0]
        conn = await dsql.connect(cluster_id)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connection_with_cluster_id_and_region(self, cluster_config):
        cluster_id = cluster_config["host"].split(".")[0]
        config = {
            "region": cluster_config["region"],
        }
        conn = await dsql.connect(cluster_id, **config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connection_string_format(self, cluster_config):
        """Test connection using connection string."""
        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?user={cluster_config['user']}"

        if cluster_config.get("profile"):
            conn_str += f"&profile={cluster_config['profile']}"

        conn = await dsql.connect(conn_str)

        try:
            row = await conn.fetchrow("SELECT current_database(), current_user")
            assert row["current_database"] == cluster_config["database"]
        finally:
            await conn.close()

    @pytest.mark.asyncio
    async def test_database_operations(self, cluster_config):
        """Test basic database operations.
        Note: asyncpg uses uses autocommit by default.
        """

        table_name = "test_integration_db_operations_asyncpg"
        conn = await dsql.connect(**cluster_config)
        try:

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
                "integration_test",
            )
            record_id = result["id"]

            # Query data
            result = await conn.fetchrow(
                f"SELECT name FROM {table_name} WHERE id = $1",
                record_id,
            )
            assert result[0] == "integration_test"

            # Clean up
            await conn.execute(
                f"DELETE FROM {table_name} WHERE id = $1",
                record_id,
            )
        finally:
            await conn.close()

    @pytest.mark.asyncio
    async def test_invalid_user_fails(self, cluster_config):
        """Test that invalid user fails appropriately."""

        config = cluster_config.copy()
        config["user"] = "nonexistent_user"
        with pytest.raises((RuntimeError, ProtocolViolationError)):
            await dsql.connect(**config)

    @pytest.mark.asyncio
    async def test_connection_with_custom_credentials_provider(self, cluster_config):
        """Test connection using custom credentials provider."""

        custom_provider = CustomCredentialProvider()
        conn = await dsql.connect(
            host=cluster_config["host"],
            user=cluster_config["user"],
            custom_credentials_provider=custom_provider,
        )

        assert (
            custom_provider.load_called
        ), "Custom credentials provider load() was not called"
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connection_with_custom_credentials_provider_in_kwargs(
        self, cluster_config
    ):
        """Test connection using custom credentials provider."""

        config = {
            "host": cluster_config["host"],
            "user": cluster_config["user"],
        }

        custom_provider = CustomCredentialProvider()
        config["custom_credentials_provider"] = custom_provider
        conn = await dsql.connect(**config)

        assert (
            custom_provider.load_called
        ), "Custom credentials provider load() was not called"
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_false(self, cluster_config):

        with pytest.raises((RuntimeError, ProtocolViolationError)) as exc_info:
            await dsql.connect(ssl=False, **cluster_config)

        error_message = exc_info.value.args[0]
        assert "SSL is mandatory" in error_message

    @pytest.mark.asyncio
    async def test_basic_connection_ssl(self, cluster_config):

        conn = await dsql.connect(ssl=True, **cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_require(self, cluster_config):

        conn = await dsql.connect(ssl="require", **cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_context_in_named_param(self, cluster_config):

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = (
            True  # This enables hostname verification (verify-full)
        )
        ssl_context.verify_mode = ssl.CERT_REQUIRED  # This is equivalent to verify-full
        ssl_context.load_verify_locations(ssl_cert_path)

        conn = await dsql.connect(ssl=ssl_context, **cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_context_no_cert_specified(self, cluster_config):

        ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ssl_context.check_hostname = True
        ssl_context.verify_mode = ssl.CERT_REQUIRED

        with pytest.raises((RuntimeError, ssl.SSLCertVerificationError)) as exc_info:
            await dsql.connect(ssl=ssl_context, **cluster_config)

        error_message = exc_info.value.args[1]
        assert "certificate verify failed" in error_message

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_context_in_kwargs(self, cluster_config):

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = (
            True  # This enables hostname verification (verify-full)
        )
        ssl_context.verify_mode = ssl.CERT_REQUIRED  # This is equivalent to verify-full
        ssl_context.load_verify_locations(ssl_cert_path)

        cluster_config["ssl"] = ssl_context

        conn = await dsql.connect(**cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_sslrootcert_not_needed_with_context(
        self, cluster_config
    ):

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = (
            True  # This enables hostname verification (verify-full)
        )
        ssl_context.verify_mode = ssl.CERT_REQUIRED  # This is equivalent to verify-full
        ssl_context.load_verify_locations(ssl_cert_path)

        cluster_config["ssl"] = ssl_context
        cluster_config["sslrootcert"] = (
            ssl_cert_path  # This parameter should be ignored given that ssl parameter is already specifying full context
        )

        conn = await dsql.connect(**cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_context_with_verify_ca(self, cluster_config):

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        ssl_context = ssl.create_default_context()
        ssl_context.verify_mode = ssl.CERT_OPTIONAL  # This is equivalent to verify-ca
        ssl_context.load_verify_locations(ssl_cert_path)

        conn = await dsql.connect(ssl=ssl_context, **cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_context_no_cert_specified_with_verify_ca(
        self, cluster_config
    ):

        ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ssl_context.check_hostname = True
        ssl_context.verify_mode = ssl.CERT_OPTIONAL  # This is equivalent to verify-ca

        with pytest.raises((RuntimeError, ssl.SSLCertVerificationError)) as exc_info:
            await dsql.connect(ssl=ssl_context, **cluster_config)

        error_message = exc_info.value.args[1]
        assert "certificate verify failed" in error_message

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_sslrootcert_direct(self, cluster_config):

        # This test works because handling was added in the connector to support these parameters from dsn.
        # The connector internally will use ssl context when handling this scenario.

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        cluster_config["ssl"] = "verify-full"
        cluster_config["sslrootcert"] = ssl_cert_path

        conn = await dsql.connect(**cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_basic_connection_ssl_sslrootcert_direct_with_verify_ca(
        self, cluster_config
    ):

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        cluster_config["ssl"] = "verify-ca"
        cluster_config["sslrootcert"] = ssl_cert_path

        conn = await dsql.connect(**cluster_config)
        await self._assert_connection_functional(conn)

    @pytest.mark.asyncio
    async def test_connect_with_dsn_ssl(self, cluster_config):
        # The DSN format allows specifying SSL modes and file paths as query parameters

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?ssl=verify-full&sslrootcert={ssl_cert_path}"

        conn = await dsql.connect(dsn=conn_str)
        await conn.close()

    @pytest.mark.asyncio
    async def test_connect_with_dsn_sslmode(self, cluster_config):
        # The DSN format allows specifying SSL modes and file paths as query parameters

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?sslmode=verify-full&sslrootcert={ssl_cert_path}"

        conn = await dsql.connect(dsn=conn_str)
        await conn.close()

    @pytest.mark.asyncio
    async def test_connect_with_dsn_sslmode_verify_ca(self, cluster_config):
        # The DSN format allows specifying SSL modes and file paths as query parameters

        ssl_cert_path = os.getenv("SSL_CERT_PATH")
        if not ssl_cert_path:
            raise ValueError("SSL_CERT_PATH environment variable not set")

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?sslmode=verify-ca&sslrootcert={ssl_cert_path}"

        conn = await dsql.connect(dsn=conn_str)
        await conn.close()

    @pytest.mark.asyncio
    async def test_connect_with_dsn_sslmode_disable(self, cluster_config):

        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['database']}?sslmode=disable"

        with pytest.raises((RuntimeError, ProtocolViolationError)) as exc_info:
            await dsql.connect(dsn=conn_str)

        error_message = exc_info.value.args[0]
        assert "SSL is mandatory" in error_message
