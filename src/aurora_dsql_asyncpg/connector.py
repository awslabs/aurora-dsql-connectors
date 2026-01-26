# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import logging
from typing import Any, Dict, Optional

import asyncpg
from botocore.credentials import CredentialProvider

from dsql_core.connection_properties import DefaultValues
from dsql_core.connection_utils import ConnectionUtilities

logger = logging.getLogger(__name__)


async def _handleSSLParameters(params: Dict[str, Any]) -> None:

    if ("ssl" not in params) and ("sslmode" in params):
        # asyncpg does not support the sslmode parameter unless embedded in dsn.
        # It has the ssl parameter instead.
        # We are parsing dsn thus may encounter this scenario.
        # Some libraries may also pass sslmode as a parameter after parsing dsn.

        params["ssl"] = params["sslmode"]

    if (
        ("ssl" in params)
        and (params["ssl"] in ["verify-ca", "verify-full"])
        and ("sslrootcert" in params)
    ):
        # If "sslrootcert" parameter is passed for "ssl=verify-full" or "ssl=verify-ca"
        # we need to use ssl context object to pass to asyncpg

        import ssl

        ssl_cert_path = params["sslrootcert"]

        ssl_context = ssl.create_default_context()
        # Set check_hostname to true when "verify-full" and false otherwise.
        ssl_context.check_hostname = params["ssl"] == "verify-full"
        ssl_context.verify_mode = ssl.CERT_REQUIRED
        ssl_context.load_verify_locations(ssl_cert_path)

        params["ssl"] = ssl_context

    if "sslmode" in params:
        params.pop("sslmode")
    if "sslrootcert" in params:
        params.pop("sslrootcert")


async def connect(
    dsn: Optional[str] = None,
    *,
    host=None,
    port=None,
    user=None,
    password=None,
    passfile=None,
    database=None,
    loop=None,
    timeout=60,
    statement_cache_size=100,
    max_cached_statement_lifetime=300,
    max_cacheable_statement_size=1024 * 15,
    command_timeout=None,
    ssl=None,
    direct_tls=None,
    connection_class=asyncpg.Connection,
    record_class=asyncpg.Record,
    server_settings=None,
    target_session_attrs=None,
    custom_credentials_provider: Optional[CredentialProvider] = None,
    **kwargs: Any,
) -> asyncpg.Connection:
    """
    Connect to Aurora DSQL using IAM authentication.

    Args:
        dsn: Connection string in format:
             postgresql://hostname/database?user=admin&region=us-east-1&profile=myprofile&token-duration-secs=3600
        or
            hostname, e.g. cluster.dsql.us-east-1.on.aws
        custom_credentials_provider: Optional custom botocore CredentialProvider for AWS authentication

        host, port, user, database: can be passed as named arguments or in kwargs.
        password and passfile: are ignored as the IAM token will be generated and used automatically by the connector.

        All other parameters are passed directly to asyncpg.connect().
        Refer to the asyncpg documentation for details on:
        loop, timeout,
        statement_cache_size, max_cached_statement_lifetime, max_cacheable_statement_size,
        command_timeout, ssl, direct_tls, connection_class, record_class,
        server_settings, target_session_attrs, krbsrvname, gsslib

        **kwargs: Additional connection parameters

    Returns:
        Connection: asyncpg.Connection connection to Aurora DSQL

    Raises:
        ValueError: If required parameters are missing
        RuntimeError: If token generation fails
    """

    if host:
        kwargs["host"] = host
    if port:
        kwargs["port"] = port
    if user:
        kwargs["user"] = user
    if database:
        kwargs["database"] = database
    if ssl is not None:
        kwargs["ssl"] = ssl

    params = ConnectionUtilities.parse_properties_and_set_token(
        dsn,
        {**kwargs, "custom_credentials_provider": custom_credentials_provider},
        driver_name="asyncpg",
    )

    # Map default names to asyncpg parameter names
    default_dbname_param = DefaultValues.DATABASE.value["property_name"]
    if ("database" not in params) and (default_dbname_param in params):
        params["database"] = params[default_dbname_param]

    if default_dbname_param in params:
        params.pop(default_dbname_param)

    await _handleSSLParameters(params)

    # asyncpg uses server_settings for application_name, not a direct parameter
    application_name = params.pop("application_name", None)
    if application_name:
        if server_settings is None:
            server_settings = {}
        server_settings = {**server_settings, "application_name": application_name}

    # Create asyncpg connection
    logger.debug(f"Connecting to Aurora DSQL at {params.get('host')}")
    conn = await asyncpg.connect(
        loop=loop,
        timeout=timeout,
        statement_cache_size=statement_cache_size,
        max_cached_statement_lifetime=max_cached_statement_lifetime,
        max_cacheable_statement_size=max_cacheable_statement_size,
        command_timeout=command_timeout,
        direct_tls=direct_tls,
        connection_class=connection_class,
        record_class=record_class,
        server_settings=server_settings,
        target_session_attrs=target_session_attrs,
        **params,
    )

    return conn
