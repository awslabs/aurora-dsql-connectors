# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import logging
import threading
from typing import Any

import boto3
import botocore.session
from botocore.exceptions import BotoCoreError, ClientError

logger = logging.getLogger(__name__)


class TokenManager:
    """Generates Aurora DSQL authentication tokens.

    Tokens are generated on every call. The boto3 DSQL client, which is expensive to build, is
    cached per (region, profile, credentials provider) and reused. Credentials are still resolved
    by the client when signing, so credential rotation keeps working.
    """

    _clients: dict[tuple[Any, ...], tuple[Any, Any]] = {}
    _clients_lock = threading.Lock()

    @classmethod
    def _get_client(cls, region: str, profile: str | None, credentials_provider: Any | None) -> Any:
        key = (region, profile, id(credentials_provider) if credentials_provider else None)
        with cls._clients_lock:
            cached = cls._clients.get(key)
            if cached is not None:
                return cached[0]

            if credentials_provider:
                botocore_session = botocore.session.Session(profile=profile)
                cred_provider = botocore_session.get_component("credential_provider")
                cred_provider.insert_before("env", credentials_provider)
                session = boto3.Session(botocore_session=botocore_session)
            elif profile:
                session = boto3.Session(profile_name=profile)
            else:
                session = boto3.Session()

            client = session.client("dsql", region_name=region)
            # Keep a reference to the provider so its id() cannot be reused while cached.
            cls._clients[key] = (client, credentials_provider)
            return client

    @classmethod
    def get_token(cls, dsql_params: dict[str, Any]) -> str:
        hostname = dsql_params.get("host")
        assert hostname is not None

        region = dsql_params.get("region")
        assert region is not None

        user = dsql_params.get("user")
        profile = dsql_params.get("profile")
        token_duration = dsql_params.get("token_duration_secs")
        credentials_provider = dsql_params.get("custom_credentials_provider")

        logger.debug(f"Generating new token for user: {user}")

        try:
            client = cls._get_client(region, profile, credentials_provider)

            # Generate token based on user type
            if user == "admin":
                token = client.generate_db_connect_admin_auth_token(
                    hostname, region, token_duration
                )
            else:
                token = client.generate_db_connect_auth_token(hostname, region, token_duration)

            logger.debug(f"Token generated successfully for user: {user}")
            return token

        except (ClientError, BotoCoreError):
            logger.error("Failed to generate authentication token")
            raise
