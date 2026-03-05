// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn test_no_pool_example() {
    if std::env::var("CLUSTER_ENDPOINT").is_err() {
        eprintln!("CLUSTER_ENDPOINT not set, skipping example test");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--bin", "no_pool"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run no_pool");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "no_pool example failed.\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );

    assert!(
        stdout.contains("Connection exercised successfully"),
        "Expected success message in stdout: {}",
        stdout
    );
}