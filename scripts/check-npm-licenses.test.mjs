import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { checkPackageLock } from "./check-npm-licenses.mjs";

async function withLockFile(packages, callback) {
  const directory = await mkdtemp(join(tmpdir(), "voxora-npm-license-"));
  const lockPath = join(directory, "package-lock.json");

  try {
    await writeFile(
      lockPath,
      JSON.stringify({ lockfileVersion: 3, packages }),
      "utf8",
    );
    await callback(lockPath);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

const rootPackage = { license: "GPL-3.0-only" };
const reviewedPackage = {
  version: "1.2.3",
  license: "MIT",
  resolved: "https://registry.npmjs.org/example/-/example-1.2.3.tgz",
  integrity: "sha512-synthetic-test-integrity",
};

test("accepts an exact reviewed registry package", async () => {
  await withLockFile(
    { "": rootPackage, "node_modules/example": reviewedPackage },
    async (lockPath) => {
      assert.deepEqual(await checkPackageLock(lockPath), []);
    },
  );
});

test("fails closed for a missing license", async () => {
  await withLockFile(
    {
      "": rootPackage,
      "node_modules/example": { ...reviewedPackage, license: undefined },
    },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("license")),
        true,
      );
    },
  );
});

test("fails closed for an unknown license", async () => {
  await withLockFile(
    {
      "": rootPackage,
      "node_modules/example": {
        ...reviewedPackage,
        license: "LicenseRef-Unknown",
      },
    },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("LicenseRef-Unknown")),
        true,
      );
    },
  );
});

test("rejects sources outside the reviewed npm registry", async () => {
  await withLockFile(
    {
      "": rootPackage,
      "node_modules/example": {
        ...reviewedPackage,
        resolved: "https://example.invalid/example-1.2.3.tgz",
      },
    },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("source")),
        true,
      );
    },
  );
});
