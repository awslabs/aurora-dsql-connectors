"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from unittest.mock import Mock, patch

import pytest
from psycopg_pool import ConnectionPool as PsycopgPool

import aurora_dsql_psycopg as dsql
from dsql_core.connection_properties import DefaultValues


# @pytest.mark.integration
class TestPsycopgPool:

    @pytest.fixture
    def cluster_config(self):

        default_region = "us-east-1"
        config = {
            "host": f"fake-cluster.dsql.${default_region}.on.aws",
            "region": default_region,
            "user": DefaultValues.USER.value["value"],
            "token_duration_secs": DefaultValues.TOKEN_DURATION.value["value"],
        }

        if not config["host"]:
            raise ValueError("CLUSTER_ENDPOINT environment variable not set")

        return config

    @patch("dsql_core.token_manager.TokenManager.get_token")
    @patch("psycopg.Connection.connect")  # Mock the parent class connect method
    def test_dsql_connection_calls_get_token(
        self, mock_psycopg_connect, mock_get_token, cluster_config
    ):
        """Test that DSQLConnection.connect calls TokenManager.get_token."""

        # Setup mocks
        mock_get_token.return_value = "mock_token"

        # Create a mock connection that supports context manager
        mock_connection = Mock()
        mock_connection.__enter__ = Mock(return_value=mock_connection)
        mock_connection.__exit__ = Mock(return_value=None)
        mock_psycopg_connect.return_value = mock_connection

        # Directly test DSQLConnection.connect
        _ = dsql.DSQLConnection.connect(**cluster_config)

        # Verify that TokenManager.get_token was called
        mock_get_token.assert_called()
        mock_psycopg_connect.assert_called()

        # Check how many times get_token was called
        print(f"get_token called {mock_get_token.call_count} times")
        assert mock_get_token.call_count == 1

    @patch("dsql_core.token_manager.TokenManager.get_token")
    @patch("psycopg.Connection.connect")  # Mock the parent class connect method
    def test_connection_pool_calls_get_token(
        self, mock_psycopg_connect, mock_get_token, cluster_config
    ):
        """Test that connection pool with DSQLConnection calls TokenManager.get_token."""

        # Setup mocks
        mock_get_token.return_value = "mock_token"

        # Create a mock connection that supports context manager
        mock_connection = Mock()
        mock_connection.__enter__ = Mock(return_value=mock_connection)
        mock_connection.__exit__ = Mock(return_value=None)
        mock_psycopg_connect.return_value = mock_connection

        pool = PsycopgPool(
            "",  # Empty conninfo
            connection_class=dsql.DSQLConnection,
            kwargs=cluster_config,  # Pass IAM params as kwargs
            min_size=3,
            max_size=5,
        )

        with pool as p:
            with p.connection() as _:
                pass  # Just getting a connection should trigger the calls

        # Verify that TokenManager.get_token was called
        mock_get_token.assert_called()

        # Check how many times get_token was called
        print(f"get_token called {mock_get_token.call_count} times")
        assert (
            mock_get_token.call_count >= 1
        )  # Pool might call it multiple times for min_size
