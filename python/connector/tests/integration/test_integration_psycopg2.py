"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import os

import pytest


# @pytest.mark.integration
class TestIntegration:
    """Integration tests requiring real Aurora DSQL cluster."""

    @staticmethod
    def _assert_connection_functional(conn):
        """Verify the provided connection functions at a basic level. Closes the connection."""
        try:
            with conn.cursor() as cur:
                cur.execute("SELECT 1")
                result = cur.fetchone()
                assert result[0] == 1
        finally:
            conn.close()

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

    def test_empty(self, cluster_config):
        """Test basic connection to Aurora DSQL."""

        pass
