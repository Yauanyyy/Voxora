import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";

const ALLOWED_LICENSES = new Set([
  "Apache-2.0",
  "Apache-2.0 OR MIT",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "BlueOak-1.0.0",
  "ISC",
  "MIT",
  "MPL-2.0",
]);

const PUBLIC_REGISTRY_PREFIX = "https://registry.npmjs.org/";

export async function checkPackageLock(lockPath) {
  const lock = JSON.parse(await readFile(lockPath, "utf8"));
  const failures = [];

  if (lock.lockfileVersion !== 3 || typeof lock.packages !== "object") {
    failures.push(
      "package-lock.json must use lockfileVersion 3 with package metadata",
    );
    return failures;
  }

  const root = lock.packages[""];
  if (root?.license !== "GPL-3.0-only") {
    failures.push("the root npm package must declare GPL-3.0-only");
  }

  for (const [packagePath, metadata] of Object.entries(lock.packages)) {
    if (packagePath === "") {
      continue;
    }

    if (!packagePath.includes("node_modules/")) {
      failures.push(`${packagePath}: unexpected non-registry package path`);
      continue;
    }

    if (typeof metadata.version !== "string" || metadata.version.length === 0) {
      failures.push(`${packagePath}: missing exact version`);
    }

    if (!ALLOWED_LICENSES.has(metadata.license)) {
      failures.push(
        `${packagePath}: unreviewed or denied license ${JSON.stringify(metadata.license)}`,
      );
    }

    if (
      typeof metadata.resolved !== "string" ||
      !metadata.resolved.startsWith(PUBLIC_REGISTRY_PREFIX)
    ) {
      failures.push(
        `${packagePath}: source is not the reviewed public npm registry`,
      );
    }

    if (
      typeof metadata.integrity !== "string" ||
      !metadata.integrity.startsWith("sha512-")
    ) {
      failures.push(`${packagePath}: missing SHA-512 registry integrity`);
    }
  }

  return failures;
}

async function main() {
  const lockPath = resolve(process.cwd(), "package-lock.json");
  const failures = await checkPackageLock(lockPath);

  if (failures.length > 0) {
    console.error("npm dependency review failed:");
    for (const failure of failures) {
      console.error(`- ${failure}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log(
    "npm dependency licenses, sources, versions, and integrity fields are reviewed",
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await main();
}
