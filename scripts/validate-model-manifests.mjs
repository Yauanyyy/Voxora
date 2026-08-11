import { readdir, readFile } from "node:fs/promises";
import { dirname, extname, posix, relative, resolve, win32 } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIRECTORY, "..");
const REVIEWED_MODEL_LICENSE_EXPRESSIONS = new Set(["Apache-2.0"]);

function requireExactKeys(value, requiredKeys, context) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${context} must be an object`);
  }

  const actualKeys = Object.keys(value).sort();
  const expectedKeys = [...requiredKeys].sort();
  if (actualKeys.join("\0") !== expectedKeys.join("\0")) {
    throw new Error(
      `${context} fields must be exactly: ${expectedKeys.join(", ")}`,
    );
  }
}

function requireNonEmptyString(value, context) {
  if (
    typeof value !== "string" ||
    value.trim() !== value ||
    value.length === 0
  ) {
    throw new Error(`${context} must be a non-empty trimmed string`);
  }
}

function requireHttpsUrl(value, context) {
  requireNonEmptyString(value, context);
  const url = new URL(value);
  if (
    url.protocol !== "https:" ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    throw new Error(
      `${context} must be an HTTPS URL without userinfo, query, or fragment`,
    );
  }
}

function requireRepositoryRelativePath(value, context) {
  requireNonEmptyString(value, context);
  if (
    posix.isAbsolute(value) ||
    win32.isAbsolute(value) ||
    /^[A-Za-z]:/.test(value) ||
    value.split(/[\\/]/).includes("..")
  ) {
    throw new Error(
      `${context} must be repository-relative without parent traversal`,
    );
  }
}

export function validateModelManifest(manifest) {
  requireExactKeys(
    manifest,
    [
      "$schema",
      "distribution",
      "displayName",
      "files",
      "id",
      "license",
      "publisher",
      "review",
      "schemaVersion",
      "source",
      "version",
    ],
    "manifest",
  );

  if (manifest.schemaVersion !== 1) {
    throw new Error("schemaVersion must equal 1");
  }
  if (manifest.$schema !== "../schemas/model-manifest.schema.json") {
    throw new Error(
      "$schema must reference the repository model-manifest schema",
    );
  }

  for (const field of ["id", "displayName", "version", "publisher"]) {
    requireNonEmptyString(manifest[field], field);
  }
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(manifest.id)) {
    throw new Error("id must be a lowercase kebab-case identifier");
  }

  requireExactKeys(manifest.source, ["retrievedAt", "url"], "source");
  requireHttpsUrl(manifest.source.url, "source.url");
  if (!/^\d{4}-\d{2}-\d{2}$/.test(manifest.source.retrievedAt)) {
    throw new Error("source.retrievedAt must use YYYY-MM-DD");
  }

  requireExactKeys(
    manifest.license,
    ["commercialUseAllowed", "redistribution", "spdx", "termsUrl"],
    "license",
  );
  requireNonEmptyString(manifest.license.spdx, "license.spdx");
  if (!REVIEWED_MODEL_LICENSE_EXPRESSIONS.has(manifest.license.spdx)) {
    throw new Error(
      "license.spdx is not an explicitly reviewed model license expression",
    );
  }
  if (manifest.license.commercialUseAllowed !== true) {
    throw new Error("license.commercialUseAllowed must be true");
  }
  if (
    !["allowed", "user-download-only"].includes(manifest.license.redistribution)
  ) {
    throw new Error(
      "license.redistribution must be allowed or user-download-only",
    );
  }
  requireHttpsUrl(manifest.license.termsUrl, "license.termsUrl");

  if (!["user-download", "user-import"].includes(manifest.distribution)) {
    throw new Error("distribution must be user-download or user-import");
  }

  if (!Array.isArray(manifest.files) || manifest.files.length === 0) {
    throw new Error("files must contain at least one reviewed artifact");
  }
  for (const [index, file] of manifest.files.entries()) {
    const context = `files[${index}]`;
    requireExactKeys(file, ["path", "sha256", "sizeBytes"], context);
    requireRepositoryRelativePath(file.path, `${context}.path`);
    if (!Number.isSafeInteger(file.sizeBytes) || file.sizeBytes <= 0) {
      throw new Error(`${context}.sizeBytes must be a positive safe integer`);
    }
    if (
      !/^[a-f0-9]{64}$/.test(file.sha256) ||
      /^([a-f0-9])\1{63}$/.test(file.sha256)
    ) {
      throw new Error(
        `${context}.sha256 must be a non-placeholder lowercase SHA-256`,
      );
    }
  }

  requireExactKeys(
    manifest.review,
    ["evidence", "reviewedAt", "reviewer", "status"],
    "review",
  );
  if (manifest.review.status !== "approved") {
    throw new Error("review.status must equal approved");
  }
  if (!/^\d{4}-\d{2}-\d{2}$/.test(manifest.review.reviewedAt)) {
    throw new Error("review.reviewedAt must use YYYY-MM-DD");
  }
  requireNonEmptyString(manifest.review.reviewer, "review.reviewer");
  requireRepositoryRelativePath(manifest.review.evidence, "review.evidence");
}

async function findJsonFiles(directory) {
  const files = [];
  let entries;

  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") {
      return files;
    }
    throw error;
  }

  for (const entry of entries) {
    const entryPath = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await findJsonFiles(entryPath)));
    } else if (entry.isFile() && extname(entry.name) === ".json") {
      files.push(entryPath);
    }
  }

  return files.sort();
}

export async function validateModelManifestDirectory(directory) {
  const files = await findJsonFiles(directory);
  const failures = [];

  for (const file of files) {
    try {
      const manifest = JSON.parse(await readFile(file, "utf8"));
      validateModelManifest(manifest);
    } catch (error) {
      failures.push(`${relative(REPOSITORY_ROOT, file)}: ${error.message}`);
    }
  }

  return { failures, files };
}

async function main() {
  const manifestsDirectory = resolve(REPOSITORY_ROOT, "model-manifests");
  const { failures, files } =
    await validateModelManifestDirectory(manifestsDirectory);

  if (failures.length > 0) {
    console.error("model-manifest validation failed:");
    for (const failure of failures) {
      console.error(`- ${failure}`);
    }
    process.exitCode = 1;
    return;
  }

  if (files.length === 0) {
    console.log(
      "No model manifests are present; no model artifact is approved",
    );
    return;
  }

  console.log(`Validated ${files.length} reviewed model manifest(s)`);
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await main();
}
