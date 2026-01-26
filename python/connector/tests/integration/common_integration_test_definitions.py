# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

from datetime import datetime, timedelta, timezone

import boto3
from botocore.credentials import CredentialProvider, DeferredRefreshableCredentials


class CustomCredentialProvider(CredentialProvider):
    METHOD = "custom-test"
    CANONICAL_NAME = "custom-test"

    def __init__(self):
        super().__init__()

        # Use a flag to verify the credential provider was actually
        # called. Since we are using the default credential chain
        # internally, any bypass of the custom credentials provider
        # would be difficult to detect.
        self.load_called = False

    def load(self):
        self.load_called = True

        # Use default credential chain to get actual credentials.
        session = boto3.Session()
        creds = session.get_credentials()
        if not creds:
            return None
        return DeferredRefreshableCredentials(
            refresh_using=lambda: {
                "access_key": creds.access_key,
                "secret_key": creds.secret_key,
                "token": creds.token,
                "expiry_time": (
                    datetime.now(timezone.utc) + timedelta(hours=1)
                ).isoformat(),
            },
            method=self.METHOD,
        )
