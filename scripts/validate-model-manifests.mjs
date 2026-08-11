import { spawnSync } from "node:child_process";
import { lstat, readdir, readFile } from "node:fs/promises";
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
    value.includes("\\") ||
    value.split(/[\\/]/).includes("..")
  ) {
    throw new Error(
      `${context} must be repository-relative without parent traversal`,
    );
  }
}

function requireCalendarDate(value, context) {
  requireNonEmptyString(value, context);
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) {
    throw new Error(`${context} must be a real calendar date in YYYY-MM-DD`);
  }

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [
    31,
    leapYear ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];

  if (
    year < 1 ||
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > daysInMonth[month - 1]
  ) {
    throw new Error(`${context} must be a real calendar date in YYYY-MM-DD`);
  }
}

async function requireTrackedRegularFile(value, context, options) {
  const repositoryRoot = options.repositoryRoot ?? REPOSITORY_ROOT;
  const filePath = resolve(repositoryRoot, ...value.split("/"));
  let fileMetadata;

  try {
    fileMetadata = await lstat(filePath);
  } catch {
    throw new Error(`${context} must reference a tracked regular file`);
  }
  if (!fileMetadata.isFile()) {
    throw new Error(`${context} must reference a tracked regular file`);
  }

  const tracked = options.trackedFiles
    ? options.trackedFiles.has(value)
    : spawnSync("git", ["ls-files", "--error-unmatch", "--", value], {
        cwd: repositoryRoot,
        stdio: "ignore",
      }).status === 0;
  if (!tracked) {
    throw new Error(`${context} must reference a tracked regular file`);
  }
}

export async function validateModelManifest(manifest, options = {}) {
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
  requireCalendarDate(manifest.source.retrievedAt, "source.retrievedAt");

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
  requireCalendarDate(manifest.review.reviewedAt, "review.reviewedAt");
  requireNonEmptyString(manifest.review.reviewer, "review.reviewer");
  requireRepositoryRelativePath(manifest.review.evidence, "review.evidence");
  await requireTrackedRegularFile(
    manifest.review.evidence,
    "review.evidence",
    options,
  );
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

export async function validateModelManifestDirectory(directory, options = {}) {
  const files = await findJsonFiles(directory);
  const failures = [];

  for (const file of files) {
    try {
      const manifest = JSON.parse(await readFile(file, "utf8"));
      await validateModelManifest(manifest, options);
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
