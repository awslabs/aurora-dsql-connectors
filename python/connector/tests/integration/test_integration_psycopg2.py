# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import os
import threading
import time

import pytest

import aurora_dsql_psycopg2 as dsql
from dsql_core.occ_retry import OCCRetryConfig


# @pytest.mark.integration
class TestIntegrationPsycopg2:
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

    def test_occ_retry_deterministic_conflict(self, cluster_config):
        """Deterministic OCC test using T1/T2 interleaving:
        T1: BEGIN, SELECT
        T2: BEGIN, UPDATE, COMMIT  (forces the conflict)
        T1: UPDATE, COMMIT         (guaranteed OCC, retry must work)
        """
        table_name = "test_occ_psycopg2_deterministic"

        pool = dsql.AuroraDSQLThreadedConnectionPool(
            minconn=3,
            maxconn=5,
            retry=OCCRetryConfig(max_retries=5),
            **cluster_config,
        )

        try:
            conn = pool.getconn()
            try:
                conn.autocommit = True
                with conn.cursor() as cur:
                    cur.execute(f"""
                        CREATE TABLE IF NOT EXISTS {table_name} (
                            id INT PRIMARY KEY, value INT
                        )
                    """)
                    cur.execute(
                        f"INSERT INTO {table_name} (id, value) VALUES (1, 0)"
                        f" ON CONFLICT (id) DO UPDATE SET value = 0"
                    )
                conn.autocommit = False
            finally:
                pool.putconn(conn)

            t1_has_read = threading.Event()
            t2_has_committed = threading.Event()
            t1_attempts = 0

            def t1(conn):
                nonlocal t1_attempts
                t1_attempts += 1
                with conn.cursor() as cur:
                    cur.execute(f"SELECT value FROM {table_name} WHERE id = 1")
                    row = cur.fetchone()
                t1_has_read.set()
                assert t2_has_committed.wait(timeout=10)
                with conn.cursor() as cur:
                    cur.execute(
                        f"UPDATE {table_name} SET value = %s WHERE id = 1",
                        (row[0] + 1,),
                    )

            def run_t2():
                conn = pool.getconn()
                try:
                    assert t1_has_read.wait(timeout=10)
                    with conn.cursor() as cur:
                        cur.execute(f"UPDATE {table_name} SET value = 100 WHERE id = 1")
                    conn.commit()
                finally:
                    pool.putconn(conn)
                t2_has_committed.set()

            t2_thread = threading.Thread(target=run_t2)
            t2_thread.start()
            pool.run_transaction(t1)
            t2_thread.join()

            conn = pool.getconn()
            try:
                with conn.cursor() as cur:
                    cur.execute(f"SELECT value FROM {table_name} WHERE id = 1")
                    result = cur.fetchone()
                    assert result[0] == 101
            finally:
                pool.putconn(conn)
            assert t1_attempts >= 2, f"Expected retry but t1 ran only {t1_attempts} time(s)"
        finally:
            for attempt in range(5):
                conn = pool.getconn()
                try:
                    conn.autocommit = True
                    with conn.cursor() as cur:
                        cur.execute(f"DROP TABLE IF EXISTS {table_name}")
                    break
                except Exception:
                    if attempt == 4:
                        raise
                    time.sleep(1)
                finally:
                    pool.putconn(conn)
            pool.closeall()

    def test_occ_non_occ_error_propagates_immediately(self, cluster_config):
        """A non-OCC database error should propagate without retry."""
        pool = dsql.AuroraDSQLThreadedConnectionPool(
            minconn=2,
            maxconn=5,
            retry=OCCRetryConfig(max_retries=5),
            **cluster_config,
        )

        try:
            call_count = 0

            def bad_query(conn):
                nonlocal call_count
                call_count += 1
                with conn.cursor() as cur:
                    cur.execute("SELECT * FROM nonexistent_table_xyz_12345")

            with pytest.raises(Exception):
                pool.run_transaction(bad_query)

            assert call_count == 1
        finally:
            pool.closeall()
