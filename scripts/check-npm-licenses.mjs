import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { dirname, resolve } from "node:path";

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

function expectedPackageName(packagePath) {
  const segments = packagePath.split("/");
  const nodeModulesIndex = segments.lastIndexOf("node_modules");
  const firstNameSegment = segments[nodeModulesIndex + 1];

  if (!firstNameSegment) {
    return null;
  }
  if (firstNameSegment.startsWith("@")) {
    const secondNameSegment = segments[nodeModulesIndex + 2];
    return secondNameSegment
      ? `${firstNameSegment}/${secondNameSegment}`
      : null;
  }

  return firstNameSegment;
}

async function checkInstalledManifest(
  lockDirectory,
  packagePath,
  lockMetadata,
  failures,
) {
  const context = packagePath || "root package";
  const manifestPath = packagePath
    ? resolve(lockDirectory, ...packagePath.split("/"), "package.json")
    : resolve(lockDirectory, "package.json");
  let installed;

  try {
    installed = JSON.parse(await readFile(manifestPath, "utf8"));
  } catch (error) {
    if (error?.code === "ENOENT" && packagePath && lockMetadata.optional) {
      return;
    }
    failures.push(
      `${context}: installed package.json is missing or unreadable`,
    );
    return;
  }

  const expectedName = packagePath
    ? expectedPackageName(packagePath)
    : lockMetadata.name;
  if (!expectedName || installed.name !== expectedName) {
    failures.push(
      `${context}: installed package identity does not match the lock`,
    );
  }
  if (installed.version !== lockMetadata.version) {
    failures.push(
      `${context}: installed package version does not match the lock`,
    );
  }

  const expectedLicense = packagePath ? lockMetadata.license : "GPL-3.0-only";
  if (installed.license !== expectedLicense) {
    failures.push(
      `${context}: installed package license does not match the lock`,
    );
  }
  if (
    packagePath
      ? !ALLOWED_LICENSES.has(installed.license)
      : installed.license !== "GPL-3.0-only"
  ) {
    failures.push(
      `${context}: installed package has an unreviewed or denied license ${JSON.stringify(installed.license)}`,
    );
  }
}

export async function checkPackageLock(lockPath) {
  const lock = JSON.parse(await readFile(lockPath, "utf8"));
  const failures = [];
  const lockDirectory = dirname(resolve(lockPath));

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
  if (root) {
    await checkInstalledManifest(lockDirectory, "", root, failures);
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

    await checkInstalledManifest(
      lockDirectory,
      packagePath,
      metadata,
      failures,
    );
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
    "npm lock and installed package identities, versions, licenses, sources, and integrity fields are reviewed",
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await main();
}
