# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import os

import pytest

import aurora_dsql_psycopg as dsql


# @pytest.mark.integration
class TestIntegrationPsycopg:
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

    def test_database_operations_with_auto_commit(self, cluster_config):
        """Test basic database operations."""

        conn = dsql.connect(autocommit=True, **cluster_config)
        try:
            with conn.cursor() as cur:
                # Create test table
                cur.execute(
                    """
                    CREATE TABLE IF NOT EXISTS test_integration_db_operations_auto (
                        id uuid NOT NULL DEFAULT gen_random_uuid(),
                        name TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
                """
                )
                # Insert data
                cur.execute(
                    "INSERT INTO test_integration_db_operations_auto (name) "
                    "VALUES (%s) RETURNING id",
                    ("integration_test",),
                )
                record_id = cur.fetchone()[0]

                # Query data
                cur.execute(
                    "SELECT name FROM test_integration_db_operations_auto WHERE id = %s",
                    (record_id,),
                )
                result = cur.fetchone()
                assert result[0] == "integration_test"

                # Clean up
                cur.execute(
                    "DELETE FROM test_integration_db_operations_auto WHERE id = %s",
                    (record_id,),
                )

        finally:
            conn.close()

    def test_class_basic_connection(self, cluster_config):
        """Test basic connection to Aurora DSQL."""

        conn = dsql.DSQLConnection.connect(**cluster_config)
        self._assert_connection_functional(conn)
