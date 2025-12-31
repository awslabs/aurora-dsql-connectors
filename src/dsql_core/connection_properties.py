"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import logging
import re
from enum import Enum
from typing import Any, Dict, Optional, Tuple
from urllib.parse import parse_qs, urlparse

import boto3

logger = logging.getLogger(__name__)


class DefaultValues(Enum):
    USER = {"property_name": "user", "value": "admin"}
    DATABASE = {"property_name": "dbname", "value": "postgres"}
    TOKEN_DURATION = {"property_name": "token_duration_secs", "value": 60}


# Note: The required values don't contain the default values
# because the default values will always be supplied 'by default'
class RequiredValues(Enum):
    HOST = "host"
    REGION = "region"


class DSQLSpecific(Enum):
    REGION = "region"
    # Password is not DSQL specific but will be supplied later with generated token
    PASSWORD = "password"
    TOKEN_DURATION = "token_duration_secs"
    PROFILE = "profile"
    CREDENTIALS_PROVIDER = "custom_credentials_provider"


class ConnectionProperties:
    @staticmethod
    def parse_properties(
        dsn: Optional[str], kwargs: Dict[str, Any]
    ) -> Tuple[Dict[str, Any], Dict[str, Any]]:

        params = kwargs.copy()

        # Expand cluster ID host to full endpoint
        if "host" in params and ConnectionProperties._is_cluster_id(params["host"]):
            expanded = ConnectionProperties._construct_dsql_host_from_cluster_id(
                params["host"], params.get("region")
            )
            if expanded:
                params["host"] = expanded

        if "host" in params and "region" not in params:
            region = ConnectionProperties._extract_region_from_hostname(params["host"])
            if region:
                params["region"] = region

        if dsn:
            dsn_params = ConnectionProperties._parse_dsn(dsn, params.get("region"))
            # User params given in kwargs take precedence over dsn params.
            # On the other hand, if dsn has the intended values, kwargs would not be needed for these values.
            for key, value in dsn_params.items():
                if key not in params:
                    params[key] = value

        # Set default values if not supplied
        ConnectionProperties._set_default_values(params)

        # Check if any required parameters are missing
        ConnectionProperties._check_required_params(params)

        ConnectionProperties._verify_other_params(params)

        # Remove DSQL specific parameters for driver_params
        dsql_specific_keys = {member.value for member in DSQLSpecific}
        driver_params = {k: v for k, v in params.items() if k not in dsql_specific_keys}

        return params, driver_params

    @staticmethod
    def _parse_dsn(dsn: str, region=None) -> Dict[str, Any]:
        """Parse DSN
        The region parameter will only be used here if the dsn is just a cluster id and the client provided region value separately.
        """

        params: Dict[str, Any] = {}

        parsed = urlparse(dsn)

        if not parsed.scheme:
            # DSN is missing scheme, check if this is cluster id or treat as 'barebone' hostname
            # e.g. dsn was just cluster.dsql.us-east-1.on.aws
            if ConnectionProperties._is_cluster_id(dsn):
                host = ConnectionProperties._construct_dsql_host_from_cluster_id(
                    dsn, region
                )
                params["host"] = host if host else dsn
            else:
                params["host"] = dsn
        else:
            if parsed.hostname:
                params["host"] = parsed.hostname
            if parsed.port:
                params["port"] = parsed.port
            if parsed.path and parsed.path != "/":
                params["dbname"] = parsed.path.lstrip("/")
            if parsed.username:
                params["user"] = parsed.username
            if parsed.password:
                params["password"] = parsed.password

            # Parse query parameters
            if parsed.query:
                query_params = parse_qs(parsed.query, keep_blank_values=True)
                for key, values in query_params.items():
                    if values:
                        params[key] = values[0]

        if "host" in params and "region" not in params:
            region = ConnectionProperties._extract_region_from_hostname(params["host"])
            if region:
                params["region"] = region

        return params

    @staticmethod
    def _extract_region_from_hostname(hostname: str) -> str:
        """Extract AWS region from Aurora DSQL hostname."""
        # Aurora DSQL hostnames follow pattern: cluster.dsql[-suffix].region.on.aws
        match = re.search(r"\.dsql[^.]*\.([^.]+)\.on\.aws$", hostname)
        if match:
            return match.group(1)

        return ""

    @staticmethod
    def _get_user_local_region():
        # Get the current AWS region
        session = boto3.Session()
        return session.region_name

    @staticmethod
    def _construct_dsql_host_from_cluster_id(cluster_id: str, region=None) -> str:
        _region = None

        if region:
            _region = region
        else:
            _region = ConnectionProperties._get_user_local_region()
            logger.warning(
                f"""Using user-local region: {_region} with cluster identifier.
                    Connection will fail if the local region does not match the cluster region.
                """
            )

        if _region:
            hostname = f"{cluster_id}.dsql.{_region}.on.aws"
            return hostname

        return ""

    @staticmethod
    def _is_cluster_id(cluster_id: str) -> bool:
        return ".dsql" not in cluster_id

    @staticmethod
    def _set_default_values(params: Dict[str, Any]) -> None:
        for member in DefaultValues:
            property_name = member.value["property_name"]
            if property_name not in params:
                params[property_name] = member.value["value"]

    @staticmethod
    def _check_required_params(params: Dict[str, Any]) -> None:
        missing_values = []

        for member in RequiredValues:
            if member.value not in params:
                missing_values.append(member.value)

        if missing_values:
            missing_values_str = ", ".join(missing_values)
            if RequiredValues.REGION.value in missing_values:
                missing_values_str += (
                    f"\n  {RequiredValues.REGION.value} was not provided and could not be extracted from "
                    f"{RequiredValues.HOST.value}"
                )

            raise ValueError(f"Missing required parameters: {missing_values_str}")

    @staticmethod
    def _verify_other_params(params: Dict[str, Any]) -> None:

        issues = []
        token_duration = params.get("token_duration_secs")

        if token_duration:
            try:
                token_duration = int(token_duration)
            except ValueError:
                issues.append(f"Invalid token_duration_secs: {token_duration}")

        if issues:
            raise ValueError("\n".join(issues))
