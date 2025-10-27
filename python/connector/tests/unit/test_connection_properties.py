"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import pytest

from dsql_core.connection_properties import ConnectionProperties


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

    def test_parse_cluster_id(self):
        """Test parsing DSN with additional parameters."""
        dsn = "clusterabcdfg"
        params = ConnectionProperties._parse_dsn(dsn)

        # The result here depends whether local aws region has been set or not
        if params.get("host"):
            assert params["region"]

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
