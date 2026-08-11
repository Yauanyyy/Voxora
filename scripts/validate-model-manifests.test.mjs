import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, test } from "node:test";
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

let repositoryRoot;

function runGit(...args) {
  const result = spawnSync("git", args, {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
}

before(async () => {
  repositoryRoot = await mkdtemp(join(tmpdir(), "voxora-model-review-"));
  const evidenceDirectory = join(repositoryRoot, "docs", "dependency-reviews");
  await mkdir(evidenceDirectory, { recursive: true });
  await writeFile(
    join(evidenceDirectory, "synthetic-model.md"),
    "# Synthetic model review\n",
    "utf8",
  );
  runGit("init", "--quiet");
  runGit("add", "--", validManifest.review.evidence);
});

after(async () => {
  await rm(repositoryRoot, { recursive: true, force: true });
});

test("accepts a complete manifest with tracked review evidence", async () => {
  await assert.doesNotReject(
    validateModelManifest(validManifest, { repositoryRoot }),
  );
});

test("rejects unknown fields", async () => {
  await assert.rejects(
    validateModelManifest(
      { ...validManifest, unexpected: true },
      { repositoryRoot },
    ),
    /fields must be exactly/,
  );
});

test("rejects placeholder hashes", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        files: [{ ...validManifest.files[0], sha256: "0".repeat(64) }],
      },
      { repositoryRoot },
    ),
    /non-placeholder/,
  );
});

for (const spdx of ["AGPL-3.0-only", "LicenseRef-Proprietary-Research-Only"]) {
  test(`rejects unreviewed model license expression ${spdx}`, async () => {
    await assert.rejects(
      validateModelManifest(
        {
          ...validManifest,
          license: { ...validManifest.license, spdx },
        },
        { repositoryRoot },
      ),
      /not an explicitly reviewed model license expression/,
    );
  });
}

test("rejects a Windows absolute model file path on every host OS", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        files: [
          {
            ...validManifest.files[0],
            path: "C:\\absolute\\model.onnx",
          },
        ],
      },
      { repositoryRoot },
    ),
    /repository-relative/,
  );
});

test("rejects a Windows absolute review evidence path on every host OS", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        review: {
          ...validManifest.review,
          evidence: "D:\\absolute\\model-review.md",
        },
      },
      { repositoryRoot },
    ),
    /repository-relative/,
  );
});

test("rejects a missing review evidence file", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        review: {
          ...validManifest.review,
          evidence: "docs/dependency-reviews/missing.md",
        },
      },
      { repositoryRoot },
    ),
    /tracked regular file/,
  );
});

test("rejects an untracked review evidence file", async () => {
  const evidence = "docs/dependency-reviews/untracked.md";
  await writeFile(
    join(repositoryRoot, ...evidence.split("/")),
    "# Untracked review\n",
    "utf8",
  );

  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        review: { ...validManifest.review, evidence },
      },
      { repositoryRoot },
    ),
    /tracked regular file/,
  );
});

test("rejects an impossible source retrieval date", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        source: { ...validManifest.source, retrievedAt: "2026-99-99" },
      },
      { repositoryRoot },
    ),
    /real calendar date/,
  );
});

test("rejects an impossible review date", async () => {
  await assert.rejects(
    validateModelManifest(
      {
        ...validManifest,
        review: { ...validManifest.review, reviewedAt: "0000-00-00" },
      },
      { repositoryRoot },
    ),
    /real calendar date/,
  );
});

test("an absent manifest directory approves no model and passes", async () => {
  const directory = await mkdtemp(join(tmpdir(), "voxora-model-manifest-"));
  const missingDirectory = join(directory, "missing");

  try {
    assert.deepEqual(
      await validateModelManifestDirectory(missingDirectory, {
        repositoryRoot,
      }),
      {
        failures: [],
        files: [],
      },
    );
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
    const result = await validateModelManifestDirectory(directory, {
      repositoryRoot,
    });
    assert.equal(result.failures.length, 1);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
