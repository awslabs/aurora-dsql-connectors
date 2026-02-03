/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package transaction_test

import (
	"os"
	"testing"

	"github.com/awslabs/aurora-dsql-connectors/go/pgx/example/src/transaction"
)

func TestTransactionExample(t *testing.T) {
	if os.Getenv("CLUSTER_ENDPOINT") == "" {
		t.Skip("CLUSTER_ENDPOINT required for integration test")
	}

	err := transaction.Example()
	if err != nil {
		t.Errorf("Transaction example failed: %v", err)
	}
}
