# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import os
import tempfile
import urllib.request
from urllib.error import URLError

import pytest
from dotenv import load_dotenv

load_dotenv()


@pytest.fixture(scope="session")
def ssl_cert_path():
    """Download Amazon root certificate if not present."""
    cert_path = os.environ.get("SSL_CERT_PATH")
    if cert_path:
        return cert_path

    cert_path = os.path.join(tempfile.gettempdir(), "AmazonRootCA1.pem")
    if not os.path.exists(cert_path):
        try:
            with urllib.request.urlopen(
                "https://www.amazontrust.com/repository/AmazonRootCA1.pem", timeout=10
            ) as response:
                with open(cert_path, "wb") as f:
                    f.write(response.read())
        except URLError as e:
            pytest.fail(
                f"Failed to download Amazon root certificate: {e}. "
                "Set SSL_CERT_PATH environment variable to a local certificate file, "
                "or fix the network connection."
            )
    return cert_path
