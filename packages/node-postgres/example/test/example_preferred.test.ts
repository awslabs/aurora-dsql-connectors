/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { test } from "@jest/globals";
import { example } from "../src/example_preferred";

test("Preferred example (connection pool concurrent)", async () => {
  await example();
});
