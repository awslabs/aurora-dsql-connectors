"""
Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
"""

from example_async import main

import pytest


# Smoke tests that our async example works fine
@pytest.mark.asyncio
async def test_example_async():
    try:
        await main()
    except Exception as e:
        pytest.fail(f"Unexpected exception: {e}")
