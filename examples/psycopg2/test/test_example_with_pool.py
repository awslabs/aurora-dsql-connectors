"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from example_with_connection_pool import main

import pytest


# Smoke tests that our example works fine
def test_example_with_pool():
    try:
        main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
