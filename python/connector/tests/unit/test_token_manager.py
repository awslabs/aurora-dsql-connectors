"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from dataclasses import dataclass
from unittest.mock import Mock, patch

import pytest
from botocore.credentials import CredentialProvider
from botocore.exceptions import BotoCoreError, ClientError

from dsql_core.connection_properties import DefaultValues
from dsql_core.token_manager import TokenManager


@dataclass
class MockSessions:
    """Container for session mocks."""

    botocore: Mock
    boto3: Mock
    cred_provider: Mock


@pytest.mark.unit
class TestTokenManager:
    ADMIN_USER = "admin"
    TEST_USER = "testuser"
    ADMIN_TOKEN = "admin-token"
    USER_TOKEN = "user-token"

    @staticmethod
    def _build_params(**overrides):
        """Build DSQL config params based on default values."""
        default_region = "us-east-1"
        params = {
            "host": f"fake-cluster.dsql.${default_region}.on.aws",
            "region": default_region,
            "user": DefaultValues.USER.value["value"],
            "token_duration_secs": DefaultValues.TOKEN_DURATION.value["value"],
        }
        params.update(overrides)
        return params

    @pytest.fixture
    def mocks(self):
        mock_client = Mock()
        mock_client.generate_db_connect_admin_auth_token.return_value = self.ADMIN_TOKEN
        mock_client.generate_db_connect_auth_token.return_value = self.USER_TOKEN

        mock_boto3_session = Mock()
        mock_boto3_session.client.return_value = mock_client

        mock_cred_provider = Mock()
        mock_botocore_session = Mock()
        mock_botocore_session.get_component.return_value = mock_cred_provider

        with (
            patch("dsql_core.token_manager.botocore.session.Session") as mock_botocore,
            patch("dsql_core.token_manager.boto3.Session") as mock_boto3,
        ):
            mock_botocore.return_value = mock_botocore_session
            mock_boto3.return_value = mock_boto3_session
            yield MockSessions(
                botocore=mock_botocore,
                boto3=mock_boto3,
                cred_provider=mock_cred_provider,
            )

    @pytest.mark.parametrize(
        "user,expected_token",
        [
            (ADMIN_USER, ADMIN_TOKEN),
            (TEST_USER, USER_TOKEN),
        ],
    )
    def test_token_generation_by_user_type(self, mocks, user, expected_token):
        """Test that correct token generation method is called based on user type."""
        token = TokenManager.get_token(self._build_params(user=user))
        assert token == expected_token

    @pytest.mark.parametrize(
        "profile,expected_call",
        [
            (None, {}),
            ("myprofile", {"profile_name": "myprofile"}),
        ],
    )
    def test_session_creation_with_profile(self, mocks, profile, expected_call):
        """Test that boto3 session is created with correct profile parameter."""
        dsql_params = self._build_params()
        if profile:
            dsql_params["profile"] = profile

        TokenManager.get_token(dsql_params)
        mocks.boto3.assert_called_once_with(**expected_call)

    @pytest.mark.parametrize("profile", [None, "myprofile"])
    def test_custom_credentials_provider_with_optional_profile(self, mocks, profile):
        """Test custom credentials provider works with and without profile."""
        custom_provider = Mock(spec=CredentialProvider)
        dsql_params = self._build_params(custom_credentials_provider=custom_provider)
        if profile:
            dsql_params["profile"] = profile

        token = TokenManager.get_token(dsql_params)

        mocks.botocore.assert_called_once_with(profile=profile)
        mocks.cred_provider.insert_before.assert_called_once_with(
            "env", custom_provider
        )
        mocks.boto3.assert_called_once_with(
            botocore_session=mocks.botocore.return_value
        )
        assert token is not None

    @pytest.mark.parametrize("region", ["us-east-1", "us-west-2", "eu-west-1"])
    def test_client_creation_with_region(self, mocks, region):
        """Test that DSQL client is created with correct region."""
        TokenManager.get_token(
            self._build_params(host=f"cluster.dsql.{region}.on.aws", region=region)
        )
        mocks.boto3.return_value.client.assert_called_once_with(
            "dsql", region_name=region
        )

    @pytest.mark.parametrize(
        "hostname,user,token_duration",
        [
            ("cluster1.dsql.us-east-1.on.aws", ADMIN_USER, 60),
            ("cluster2.dsql.us-west-2.on.aws", TEST_USER, 3600),
            ("cluster3.dsql.eu-west-1.on.aws", ADMIN_USER, 900),
        ],
    )
    def test_token_generation_parameters(self, mocks, hostname, user, token_duration):
        """Test that token generation is called with provided parameters."""
        region = hostname.split(".")[2]
        TokenManager.get_token(
            self._build_params(
                host=hostname,
                region=region,
                user=user,
                token_duration_secs=token_duration,
            )
        )

        mock_client = mocks.boto3.return_value.client.return_value

        if user == self.ADMIN_USER:
            mock_client.generate_db_connect_admin_auth_token.assert_called_once_with(
                hostname, region, token_duration
            )
        else:
            mock_client.generate_db_connect_auth_token.assert_called_once_with(
                hostname, region, token_duration
            )

    @pytest.mark.parametrize(
        "exception_type",
        [
            "ClientError",
            "BotoCoreError",
        ],
    )
    def test_token_generation_errors_not_suppressed(self, mocks, exception_type):
        """Test that AWS errors are properly raised and not suppressed by the implementation."""
        mock_client = mocks.boto3.return_value.client.return_value

        if exception_type == "ClientError":
            error = ClientError(
                {"Error": {"Code": "InvalidToken", "Message": "Invalid"}}, "operation"
            )
        else:
            error = BotoCoreError()

        mock_client.generate_db_connect_admin_auth_token.side_effect = error

        with pytest.raises((ClientError, BotoCoreError)):
            TokenManager.get_token(self._build_params())
