"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from dsql_core._version import __version__

from .connector import connect
from .pool import create_pool

__all__ = ["connect", "create_pool", "__version__"]
