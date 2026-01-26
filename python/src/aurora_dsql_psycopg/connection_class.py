# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import psycopg
from botocore.credentials import CredentialProvider

from dsql_core.connection_utils import ConnectionUtilities


class DSQLConnection(psycopg.Connection):
    """Connection class that refreshes IAM tokens on connect."""

    @classmethod
    def connect(
        cls,
        conninfo="",
        *,
        autocommit=False,
        prepare_threshold=5,
        context=None,
        row_factory=None,
        cursor_factory=None,
        custom_credentials_provider: CredentialProvider | None = None,
        **kwargs,
    ):
        """
        Connect to Aurora DSQL using IAM authentication.

        Args:
            conninfo: Connection string in format:
                postgresql://hostname/database?user=admin&region=us-east-1&profile=myprofile&token_duration_secs=3600
                or
                hostname, e.g. cluster.dsql.us-east-1.on.aws
            custom_credentials_provider: Optional custom botocore CredentialProvider
                for AWS authentication
            **kwargs: Additional connection parameters

            The other parameters are psycopg connect parameters. Refer to the psycopg documentation.
            This method just passes them 'as is' to the psycopg.connect call.

        Returns:
            Connection: psycopg connection to Aurora DSQL

        Raises:
            ValueError: If required parameters are missing
            ClientError or BotoCoreError: If token generation fails
        """

        params = ConnectionUtilities.parse_properties_and_set_token(
            conninfo,
            {**kwargs, "custom_credentials_provider": custom_credentials_provider},
            driver_name="psycopg",
        )

        return super().connect(
            autocommit=autocommit,
            prepare_threshold=prepare_threshold,
            context=context,
            row_factory=row_factory,
            cursor_factory=cursor_factory,
            **params,
        )


class DSQLAsyncConnection(psycopg.AsyncConnection):
    """Async connection class that refreshes IAM tokens on connect."""

    @classmethod
    async def connect(
        cls,
        conninfo="",
        *,
        autocommit=False,
        prepare_threshold=5,
        context=None,
        row_factory=None,
        cursor_factory=None,
        custom_credentials_provider: CredentialProvider | None = None,
        **kwargs,
    ):
        """
        Connect to Aurora DSQL using IAM authentication.

        Args:
            conninfo: Connection string in format:
                postgresql://hostname/database?user=admin&region=us-east-1&profile=myprofile&token_duration_secs=3600
                or
                hostname, e.g. cluster.dsql.us-east-1.on.aws
            custom_credentials_provider: Optional custom botocore CredentialProvider
                for AWS authentication
            **kwargs: Additional connection parameters

            The other parameters are psycopg connect parameters. Refer to the psycopg documentation.
            This method just passes them 'as is' to the psycopg.connect call.

        Returns:
            Connection: psycopg connection to Aurora DSQL

        Raises:
            ValueError: If required parameters are missing
            ClientError or BotoCoreError: If token generation fails
        """

        params = ConnectionUtilities.parse_properties_and_set_token(
            conninfo,
            {**kwargs, "custom_credentials_provider": custom_credentials_provider},
            driver_name="psycopg",
        )

        return await super().connect(
            autocommit=autocommit,
            prepare_threshold=prepare_threshold,
            context=context,
            row_factory=row_factory,
            cursor_factory=cursor_factory,
            **params,
        )
