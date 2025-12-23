"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../../src/alternatives/no_connection_pool'))

import pytest
from example_with_no_connection_pool import main


# Smoke tests that our example works fine
def test_example_with_no_connection_pool():
    try:
        main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
