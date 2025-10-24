"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import os
import pytest

import aurora_dsql_psycopg as dsql


# @pytest.mark.integration
class TestIntegrationAsync:
    """Integration tests requiring real Aurora DSQL cluster."""

    @staticmethod
    async def _assert_connection_functional_async(conn):
        """Verify the provided connection functions at a basic level. Closes the connection."""

        assert conn

        try:
            async with conn.cursor() as cur:
                await cur.execute("SELECT 1")
                result = await cur.fetchone()
                assert result[0] == 1
        finally:
            await conn.close()

    @pytest.fixture
    def cluster_config(self):
        """Get cluster configuration from environment variables."""
        config = {
            "host": os.getenv("CLUSTER_ENDPOINT"),
            "region": os.getenv("REGION", "us-east-1"),
            "user": os.getenv("CLUSTER_USER", "admin"),
            "dbname": os.getenv("DSQL_DATABASE", "postgres"),
        }
        aws_profile = os.getenv("AWS_PROFILE")
        if aws_profile:
            config["profile"] = aws_profile

        if not config["host"]:
            raise ValueError("CLUSTER_ENDPOINT environment variable not set")

        return config

    @pytest.mark.asyncio
    async def test_class_basic_connection_async(self, cluster_config):
        """Test basic async connection to Aurora DSQL using DSQLAsyncConnection."""

        conn = await dsql.DSQLAsyncConnection.connect(**cluster_config)

        try:
            async with conn.cursor() as cur:
                await cur.execute("SELECT 1")
                result = await cur.fetchone()
                assert result[0] == 1
        finally:
            await conn.close()
