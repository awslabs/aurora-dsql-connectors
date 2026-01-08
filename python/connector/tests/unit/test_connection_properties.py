"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from unittest.mock import patch

import pytest

from dsql_core.connection_properties import ConnectionProperties


@pytest.fixture
def mock_no_default_region():
    """Simulate default region not set."""
    with patch.object(ConnectionProperties, "_get_user_local_region", return_value=None):
        yield


@pytest.fixture
def mock_default_region():
    """Simulate default region set to us-west-2."""
    with patch.object(ConnectionProperties, "_get_user_local_region", return_value="us-west-2"):
        yield


@pytest.mark.unit
class TestDSNParsing:
    """Test DSN parsing functionality."""

    def test_parse_basic_dsn(self):
        """Test parsing basic PostgreSQL DSN."""
        dsn = "postgresql://cluster.dsql.us-east-1.on.aws/postgres?user=admin"
        params = ConnectionProperties._parse_dsn(dsn)

        assert params["host"] == "cluster.dsql.us-east-1.on.aws"
        assert params["dbname"] == "postgres"
        assert params["user"] == "admin"
        assert params["region"] == "us-east-1"

    def test_parse_with_parameters(self):
        """Test parsing DSN with additional parameters."""
        dsn = "postgresql://cluster.dsql.us-east-1.on.aws/postgres?user=admin&token_duration_secs=3600&profile=test"
        params = ConnectionProperties._parse_dsn(dsn)

        assert params["user"] == "admin"
        assert params["token_duration_secs"] == "3600"
        assert params["profile"] == "test"

    def test_parse_cluster_id(self, mock_no_default_region):
        """Test parsing cluster ID without region falls back to cluster ID as host."""
        dsn = "clusterabcdfg"
        params = ConnectionProperties._parse_dsn(dsn)
        assert params["host"] == "clusterabcdfg"
        assert "region" not in params

    def test_parse_cluster_id_with_region(self):
        """Test parsing DSN with additional parameters."""
        dsn = "clusterabcdfg"
        params = ConnectionProperties._parse_dsn(dsn, "us-east-1")

        assert params["host"] == "clusterabcdfg.dsql.us-east-1.on.aws"
        assert params["region"] == "us-east-1"

    def test_set_default_values(self):
        """Test setting default values for missing parameters."""
        params = {}
        ConnectionProperties._set_default_values(params)

        assert params["user"] == "admin"
        assert params["dbname"] == "postgres"
        assert params["token_duration_secs"] == 60

    def test_check_required_params(self):
        """Test checking for required parameters."""

        # Note:
        # The _check_required_params method does not do any parsing. It is called when all the parameters were parsed and extracted.
        # The function only verifies whether the required parameters are present in params keys given to it.
        # The example params below are missing a region. The region is technically present as a part of the value of the host key.
        # However, it is not the purpose of _check_required_params method to parse values.

        params = {"host": "cluster.dsql.us-east-1.on.aws", "user": "user_name"}

        # Exception should be raised for missing 'region'
        with pytest.raises(
            ValueError,
            match="Missing required parameters: region\n  region was not provided and could not be extracted from host",
        ):
            ConnectionProperties._check_required_params(params)

    @pytest.mark.parametrize(
        "dsn,kwargs,expected_missing",
        [
            ("", {}, {"host", "region"}),
            ("", {"host": "clusterid"}, {"region"}),
            ("", {"region": "us-east-1"}, {"host"}),
            ("clusterid", {}, {"region"}),
        ],
    )
    def test_missing_required_params(self, dsn, kwargs, expected_missing, mock_no_default_region):
        """Test error messages for missing required parameters."""
        with pytest.raises(ValueError) as exc_info:
            ConnectionProperties.parse_properties(dsn, kwargs)
        first_line = str(exc_info.value).split("\n")[0]
        assert "Missing required parameters" in first_line
        assert ("host" in first_line) == ("host" in expected_missing)
        assert ("region" in first_line) == ("region" in expected_missing)

    def test_kwargs_override_dsn(self):
        """Test that kwargs override DSN parameters."""
        dsn = "postgresql://cluster.dsql.us-east-1.on.aws/postgres?user=admin"

        _, params = ConnectionProperties.parse_properties(dsn, {"user": "non-admin"})

        # DSN adds parameters, but kwargs should take precedence
        assert params["user"] == "non-admin"  # kwargs value is used

    def test_extract_region(self):
        """Test extracting different regions."""

        test_cases = [
            ("cluster.dsql.us-west-2.on.aws", "us-west-2"),
            ("cluster.dsql.eu-west-1.on.aws", "eu-west-1"),
            ("cluster.dsql.ap-southeast-1.on.aws", "ap-southeast-1"),
        ]

        for hostname, expected_region in test_cases:
            region = ConnectionProperties._extract_region_from_hostname(hostname)
            assert region == expected_region

    def test_cluster_id_as_dsn_expands_to_full_endpoint_with_explicit_region(self):
        """Test that cluster ID as dsn is expanded to full endpoint with explicit region."""
        dsql_params, _ = ConnectionProperties.parse_properties(
            "clusterid", {"region": "us-east-1"}
        )
        assert dsql_params["host"] == "clusterid.dsql.us-east-1.on.aws"
        assert dsql_params["region"] == "us-east-1"

    def test_cluster_id_as_host_kwarg_expands_to_full_endpoint_with_explicit_region(self):
        """Test that cluster ID in host kwarg is expanded to full endpoint with explicit region."""
        dsql_params, _ = ConnectionProperties.parse_properties(
            "", {"host": "clusterid", "region": "us-east-1"}
        )
        assert dsql_params["host"] == "clusterid.dsql.us-east-1.on.aws"
        assert dsql_params["region"] == "us-east-1"

    def test_cluster_id_as_dsn_expands_to_full_endpoint_with_default_region(self, mock_default_region):
        """Test that cluster ID as dsn is expanded to full endpoint with default region."""
        dsql_params, _ = ConnectionProperties.parse_properties("clusterid", {})
        assert dsql_params["host"] == "clusterid.dsql.us-west-2.on.aws"
        assert dsql_params["region"] == "us-west-2"

    def test_cluster_id_as_host_kwarg_expands_to_full_endpoint_with_default_region(self, mock_default_region):
        """Test that cluster ID in host kwarg is expanded to full endpoint with default region."""
        dsql_params, _ = ConnectionProperties.parse_properties(
            "", {"host": "clusterid"}
        )
        assert dsql_params["host"] == "clusterid.dsql.us-west-2.on.aws"
        assert dsql_params["region"] == "us-west-2"


@pytest.mark.unit
class TestApplicationName:
    """Test application_name functionality."""

    def test_build_application_name_default(self):
        """Test building default application_name."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg")
        assert result.startswith("aurora-dsql-python-psycopg/")
        assert "/" in result

    def test_build_application_name_with_orm_prefix(self):
        """Test building application_name with ORM prefix."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "sqlalchemy")
        assert result.startswith("sqlalchemy:aurora-dsql-python-psycopg/")

    def test_build_application_name_empty_string(self):
        """Test that empty string prefix returns default."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "")
        assert result.startswith("aurora-dsql-python-psycopg/")
        assert ":" not in result

    def test_build_application_name_whitespace_only(self):
        """Test that whitespace-only prefix returns default."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "   ")
        assert result.startswith("aurora-dsql-python-psycopg/")
        assert ":" not in result

    def test_build_application_name_with_colon(self):
        """Test that ORM prefix with colon is accepted."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "sql:alchemy")
        assert result.startswith("sql:alchemy:aurora-dsql-python-psycopg/")

    def test_build_application_name_with_at_sign(self):
        """Test that ORM prefix with @ sign is accepted."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "sql@alchemy")
        assert result.startswith("sql@alchemy:aurora-dsql-python-psycopg/")

    def test_build_application_name_with_slash(self):
        """Test that ORM prefix with slash is now accepted."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "myapp/1.0")
        assert result.startswith("myapp/1.0:aurora-dsql-python-psycopg/")

    def test_build_application_name_with_newline(self):
        """Test that ORM prefix with newline is trimmed."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "my\napp")
        # Newline should be preserved (not trimmed by strip())
        assert result.startswith("my\napp:aurora-dsql-python-psycopg/")

    def test_build_application_name_with_tab(self):
        """Test that ORM prefix with tab is trimmed."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "my\tapp")
        # Tab should be preserved (not trimmed by strip())
        assert result.startswith("my\tapp:aurora-dsql-python-psycopg/")

    def test_build_application_name_very_long_string(self):
        """Test that very long ORM prefix is accepted (PostgreSQL will truncate)."""
        from dsql_core.connection_properties import build_application_name

        long_prefix = "a" * 100  # Way over 64 char limit
        result = build_application_name("psycopg", long_prefix)
        assert result.startswith(long_prefix + ":")
        # We don't truncate - let PostgreSQL handle it

    def test_build_application_name_with_unicode(self):
        """Test that unicode characters in ORM prefix are accepted."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "日本語")
        assert result.startswith("日本語:aurora-dsql-python-psycopg/")

    def test_build_application_name_with_emoji(self):
        """Test that emoji in ORM prefix are accepted."""
        from dsql_core.connection_properties import build_application_name

        result = build_application_name("psycopg", "🚀app")
        assert result.startswith("🚀app:aurora-dsql-python-psycopg/")

    def test_build_application_name_different_drivers(self):
        """Test application_name for different drivers."""
        from dsql_core.connection_properties import build_application_name

        psycopg_name = build_application_name("psycopg")
        psycopg2_name = build_application_name("psycopg2")
        asyncpg_name = build_application_name("asyncpg")

        assert "psycopg/" in psycopg_name
        assert "psycopg2/" in psycopg2_name
        assert "asyncpg/" in asyncpg_name
