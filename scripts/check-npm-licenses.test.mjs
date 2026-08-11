import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { checkPackageLock } from "./check-npm-licenses.mjs";

async function withLockFile(
  packages,
  callback,
  installedPackage = reviewedInstalledPackage,
) {
  const directory = await mkdtemp(join(tmpdir(), "voxora-npm-license-"));
  const lockPath = join(directory, "package-lock.json");

  try {
    await writeFile(
      lockPath,
      JSON.stringify({ lockfileVersion: 3, packages }),
      "utf8",
    );
    await writeFile(
      join(directory, "package.json"),
      JSON.stringify(rootPackage),
      "utf8",
    );
    if (installedPackage) {
      const installedDirectory = join(directory, "node_modules", "example");
      await mkdir(installedDirectory, { recursive: true });
      await writeFile(
        join(installedDirectory, "package.json"),
        JSON.stringify(installedPackage),
        "utf8",
      );
    }
    await callback(lockPath);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

const rootPackage = {
  name: "synthetic-root",
  version: "0.1.0",
  license: "GPL-3.0-only",
};
const reviewedPackage = {
  version: "1.2.3",
  license: "MIT",
  resolved: "https://registry.npmjs.org/example/-/example-1.2.3.tgz",
  integrity: "sha512-synthetic-test-integrity",
};
const reviewedInstalledPackage = {
  name: "example",
  version: "1.2.3",
  license: "MIT",
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

test("rejects an installed license hidden by edited lock metadata", async () => {
  await withLockFile(
    { "": rootPackage, "node_modules/example": reviewedPackage },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("installed package has")),
        true,
      );
      assert.equal(
        failures.some((failure) => failure.includes("does not match the lock")),
        true,
      );
    },
    { ...reviewedInstalledPackage, license: "AGPL-3.0-only" },
  );
});

test("binds an installed package to the locked identity and version", async () => {
  await withLockFile(
    { "": rootPackage, "node_modules/example": reviewedPackage },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("identity")),
        true,
      );
      assert.equal(
        failures.some((failure) => failure.includes("version")),
        true,
      );
    },
    { ...reviewedInstalledPackage, name: "different", version: "9.9.9" },
  );
});

test("fails when a required installed package is missing", async () => {
  await withLockFile(
    { "": rootPackage, "node_modules/example": reviewedPackage },
    async (lockPath) => {
      const failures = await checkPackageLock(lockPath);
      assert.equal(
        failures.some((failure) => failure.includes("missing or unreadable")),
        true,
      );
    },
    null,
  );
});
