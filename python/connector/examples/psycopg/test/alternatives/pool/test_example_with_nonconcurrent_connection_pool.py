"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import pytest
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../src/alternatives/pool'))
from example_with_nonconcurrent_connection_pool import main


# Smoke tests that our example works fine
def test_example_with_nonconcurrent_connection_pool():
    try:
        main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
