"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import pytest
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../src'))
from example_preferred import main


# Smoke tests that our example works fine
def test_example_preferred():
    try:
        main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
