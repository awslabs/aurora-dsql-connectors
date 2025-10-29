"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import os
from datetime import datetime, timedelta, timezone

import boto3
import pytest
from botocore.credentials import CredentialProvider, DeferredRefreshableCredentials
from psycopg import Error as PsycopgError
from psycopg import OperationalError as PsycopgOperationalError
from psycopg2 import OperationalError as Psycopg2OperationalError

import aurora_dsql_psycopg as dsql
import aurora_dsql_psycopg2 as dsql2


# @pytest.mark.integration
class TestIntegrationPsycopgCommon:
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

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_basic_connection(self, cluster_config, dsql_connector):
        """Test basic connection to Aurora DSQL."""

        conn = dsql_connector.connect(**cluster_config)
        self._assert_connection_functional(conn)

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_connection_with_host_only(self, cluster_config, dsql_connector):
        """Test connection to Aurora DSQL with URL only.
        Region should be extracted from host, and the defaults will be applied"""

        conn = dsql_connector.connect(cluster_config["host"])
        self._assert_connection_functional(conn)

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_connection_with_cluster_id_only(self, cluster_config, dsql_connector):
        cluster_id = cluster_config["host"].split(".")[0]
        try:
            # Note: This works conditionally depending on the local region set correctly to match the cluster's region.
            conn = dsql_connector.connect(cluster_id)
            self._assert_connection_functional(conn)
        except Exception as e:
            pytest.skip(str(e))
        finally:
            pass

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_connection_with_cluster_id_and_region(
        self, cluster_config, dsql_connector
    ):
        cluster_id = cluster_config["host"].split(".")[0]
        config = {
            "region": cluster_config["region"],
        }
        conn = dsql_connector.connect(cluster_id, **config)
        self._assert_connection_functional(conn)

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_connection_string_format(self, cluster_config, dsql_connector):
        """Test connection using connection string."""
        conn_str = f"postgresql://{cluster_config['host']}/{cluster_config['dbname']}?user={cluster_config['user']}"

        if cluster_config.get("profile"):
            conn_str += f"&profile={cluster_config['profile']}"

        conn = dsql_connector.connect(conn_str)

        try:
            with conn.cursor() as cur:
                cur.execute("SELECT current_database(), current_user")
                result = cur.fetchone()
                assert result[0] == cluster_config["dbname"]
        finally:
            conn.close()

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_database_operations(self, cluster_config, dsql_connector):
        """Test basic database operations."""

        conn = dsql_connector.connect(**cluster_config)
        try:
            with conn.cursor() as cur:
                cur.execute(
                    """
                    CREATE TABLE IF NOT EXISTS test_integration_db_operations (
                        id uuid NOT NULL DEFAULT gen_random_uuid(),
                        name TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
                """
                )
                conn.commit()
                # Insert data
                cur.execute(
                    "INSERT INTO test_integration_db_operations (name) VALUES (%s) RETURNING id",
                    ("integration_test",),
                )
                record_id = cur.fetchone()[0]

                # Query data
                cur.execute(
                    "SELECT name FROM test_integration_db_operations WHERE id = %s",
                    (record_id,),
                )
                result = cur.fetchone()
                assert result[0] == "integration_test"

                # Clean up
                cur.execute(
                    "DELETE FROM test_integration_db_operations WHERE id = %s",
                    (record_id,),
                )
                conn.commit()
        finally:
            conn.close()

    @pytest.mark.parametrize(
        "dsql_connector, expected_error",
        [
            (dsql, PsycopgOperationalError),
            (dsql2, Psycopg2OperationalError),
        ],
    )
    def test_custom_token_duration(
        self, cluster_config, dsql_connector, expected_error
    ):
        """Test connection with custom token duration."""

        # According to https://docs.aws.amazon.com/aurora-dsql/latest/userguide/SECTION_authentication-token.html
        # the token duration must be less than a week (604800 seconds)
        # The sdk can generate tokens with duration longer than that.
        # However, the connect call to DSQL with a token that has longer duration will fail.

        config = cluster_config.copy()
        config["token_duration_secs"] = 604801  # one week + 1 sec

        with pytest.raises(expected_error) as exc_info:
            dsql_connector.connect(**config)

        error_message = exc_info.value.args[0]
        assert "X-Amz-Expires must be less than a week (in seconds)" in error_message

    @pytest.mark.parametrize(
        "dsql_connector, expected_error",
        [
            (dsql, PsycopgError),
            (dsql2, Psycopg2OperationalError),
        ],
    )
    def test_invalid_user_fails(self, cluster_config, dsql_connector, expected_error):
        """Test that invalid user fails appropriately."""

        config = cluster_config.copy()
        config["user"] = "nonexistent_user"
        with pytest.raises((RuntimeError, expected_error)):
            dsql_connector.connect(**config)

    @pytest.mark.parametrize(
        "dsql_connector",
        [
            dsql,
            dsql2,
        ],
    )
    def test_connection_with_custom_credentials_provider(
        self, cluster_config, dsql_connector
    ):
        """Test connection using custom credentials provider."""

        class CustomCredentialProvider(CredentialProvider):
            METHOD = "custom-test"
            CANONICAL_NAME = "custom-test"

            def __init__(self):
                super().__init__()

                # Use a flag to verify the credential provider was actually
                # called. Since we are using the default credential chain
                # internally, any bypass of the custom credentials provider
                # would be difficult to detect.
                self.load_called = False

            def load(self):
                self.load_called = True

                # Use default credential chain to get actual credentials.
                session = boto3.Session()
                creds = session.get_credentials()
                if not creds:
                    return None
                return DeferredRefreshableCredentials(
                    refresh_using=lambda: {
                        "access_key": creds.access_key,
                        "secret_key": creds.secret_key,
                        "token": creds.token,
                        "expiry_time": (
                            datetime.now(timezone.utc) + timedelta(hours=1)
                        ).isoformat(),
                    },
                    method=self.METHOD,
                )

        custom_provider = CustomCredentialProvider()
        conn = dsql_connector.connect(
            host=cluster_config["host"],
            user=cluster_config["user"],
            custom_credentials_provider=custom_provider,
        )

        assert (
            custom_provider.load_called
        ), "Custom credentials provider load() was not called"
        self._assert_connection_functional(conn)
