"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

import pytest
from example import main


# Smoke tests that our example works fine
def test_example():
    try:
        main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
