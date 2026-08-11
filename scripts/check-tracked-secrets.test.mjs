import assert from "node:assert/strict";
import test from "node:test";
import { findSecretPatterns } from "./check-tracked-secrets.mjs";

test("ordinary source text has no findings", () => {
  assert.deepEqual(findSecretPatterns("Voxora M2 workspace skeleton"), []);
});

test("credential-shaped synthetic text is rejected", () => {
  const syntheticAwsKey = `AKIA${"A".repeat(16)}`;
  assert.deepEqual(findSecretPatterns(syntheticAwsKey), ["AWS access key"]);
});
