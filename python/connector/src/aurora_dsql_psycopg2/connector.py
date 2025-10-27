"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import logging
from typing import Any, Optional

import psycopg2
from botocore.credentials import CredentialProvider

from dsql_core.connection_utils import ConnectionUtilities

logger = logging.getLogger(__name__)


def connect(
    dsn: Optional[str] = None,
    *,
    connection_factory=None,
    cursor_factory=None,
    custom_credentials_provider: Optional[CredentialProvider] = None,
    **kwargs: Any,
):
    """
    Connect to Aurora DSQL using IAM authentication.

    Args:
        dsn: Connection string in format:
            postgresql://hostname/database?user=admin&region=us-east-1&profile=myprofile&token_duration_secs=3600
            or
            hostname, e.g. cluster.dsql.us-east-1.on.aws
        custom_credentials_provider: Optional custom botocore CredentialProvider for AWS authentication
        **kwargs: Additional connection parameters

        The connection_factory and  cursor_factory parameters are psycopg2 connect parameters.
        Refer to the psycopg documentation.
        This method just passes them 'as is' to the psycopg2.connect call.

    Returns:
        Connection: psycopg2 connection to Aurora DSQL

    Raises:
        ValueError: If required parameters are missing
        ClientError or BotoCoreError: If token generation fails
    """

    params = ConnectionUtilities.parse_properties_and_set_token(
        dsn, {**kwargs, "custom_credentials_provider": custom_credentials_provider}
    )

    # Create psycopg2 connection
    conn = psycopg2.connect(
        connection_factory=connection_factory, cursor_factory=cursor_factory, **params
    )

    return conn
