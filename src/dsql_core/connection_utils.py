"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from typing import Any, Dict, Optional

from dsql_core.connection_properties import ConnectionProperties
from dsql_core.token_manager import TokenManager


class ConnectionUtilities:
    @staticmethod
    def parse_properties_and_set_token(
        dsn: Optional[str], kwargs: Dict[str, Any]
    ) -> Dict[str, Any]:

        dsql_params, params = ConnectionProperties.parse_properties(dsn, kwargs)
        token = TokenManager.get_token(dsql_params)
        params["password"] = token

        return params
