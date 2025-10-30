/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { test } from "@jest/globals";
import { example } from "../src/index";

test("Smoke test", async () => {
  await example();
});
