import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  validateModelManifest,
  validateModelManifestDirectory,
} from "./validate-model-manifests.mjs";

const validManifest = {
  $schema: "../schemas/model-manifest.schema.json",
  schemaVersion: 1,
  id: "synthetic-model",
  displayName: "Synthetic Model",
  version: "1.0.0",
  publisher: "Synthetic Publisher",
  source: {
    url: "https://example.invalid/models/synthetic-model",
    retrievedAt: "2026-08-12",
  },
  license: {
    spdx: "Apache-2.0",
    termsUrl: "https://example.invalid/licenses/apache-2.0",
    commercialUseAllowed: true,
    redistribution: "user-download-only",
  },
  distribution: "user-download",
  files: [
    {
      path: "synthetic/model.onnx",
      sizeBytes: 1234,
      sha256:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    },
  ],
  review: {
    status: "approved",
    reviewedAt: "2026-08-12",
    reviewer: "Synthetic Reviewer",
    evidence: "docs/dependency-reviews/synthetic-model.md",
  },
};

test("accepts a structurally complete reviewed manifest", () => {
  assert.doesNotThrow(() => validateModelManifest(validManifest));
});

test("rejects unknown fields", () => {
  assert.throws(
    () => validateModelManifest({ ...validManifest, unexpected: true }),
    /fields must be exactly/,
  );
});

test("rejects placeholder hashes", () => {
  assert.throws(
    () =>
      validateModelManifest({
        ...validManifest,
        files: [{ ...validManifest.files[0], sha256: "0".repeat(64) }],
      }),
    /non-placeholder/,
  );
});

test("rejects licenses denied by project policy", () => {
  assert.throws(
    () =>
      validateModelManifest({
        ...validManifest,
        license: { ...validManifest.license, spdx: "AGPL-3.0-only" },
      }),
    /denied by project policy/,
  );
});

test("an absent manifest directory approves no model and passes", async () => {
  const directory = await mkdtemp(join(tmpdir(), "voxora-model-manifest-"));
  const missingDirectory = join(directory, "missing");

  try {
    assert.deepEqual(await validateModelManifestDirectory(missingDirectory), {
      failures: [],
      files: [],
    });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("a malformed manifest fails directory validation", async () => {
  const directory = await mkdtemp(join(tmpdir(), "voxora-model-manifest-"));

  try {
    await writeFile(
      join(directory, "invalid.json"),
      JSON.stringify({}),
      "utf8",
    );
    const result = await validateModelManifestDirectory(directory);
    assert.equal(result.failures.length, 1);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
