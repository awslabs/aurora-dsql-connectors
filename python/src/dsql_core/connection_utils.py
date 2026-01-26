# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from typing import Any

from dsql_core.connection_properties import ConnectionProperties, build_application_name
from dsql_core.token_manager import TokenManager


class ConnectionUtilities:
    @staticmethod
    def parse_properties_and_set_token(
        dsn: str | None,
        kwargs: dict[str, Any],
        driver_name: str = "unknown",
    ) -> dict[str, Any]:
        dsql_params, params = ConnectionProperties.parse_properties(dsn, kwargs)
        token = TokenManager.get_token(dsql_params)
        params["password"] = token

        # Set application_name with optional ORM prefix
        orm_prefix = params.get("application_name")
        params["application_name"] = build_application_name(driver_name, orm_prefix)

        return params
