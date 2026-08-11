import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const directory = await mkdtemp(join(tmpdir(), "voxora-cargo-deny-"));

try {
  await mkdir(join(directory, "src"));
  await writeFile(
    join(directory, "Cargo.toml"),
    [
      "[package]",
      'name = "synthetic-unknown-license"',
      'version = "0.1.0"',
      'edition = "2024"',
      'license = "LicenseRef-Unknown"',
      "",
      "[workspace]",
      "",
    ].join("\n"),
    "utf8",
  );
  await writeFile(
    join(directory, "src", "lib.rs"),
    "#![forbid(unsafe_code)]\n",
    "utf8",
  );

  const command =
    process.platform === "win32" ? "cargo-deny.exe" : "cargo-deny";
  const result = spawnSync(
    command,
    [
      "--manifest-path",
      join(directory, "Cargo.toml"),
      "--config",
      join(repositoryRoot, "deny.toml"),
      "check",
      "licenses",
    ],
    { encoding: "utf8" },
  );

  if (result.error) {
    throw result.error;
  }
  if (result.status === 0) {
    throw new Error("cargo-deny accepted a synthetic unknown license");
  }
  if (!`${result.stdout}\n${result.stderr}`.includes("rejected")) {
    throw new Error(
      "cargo-deny failed without reporting the unknown license as rejected",
    );
  }

  console.log("cargo-deny rejects a synthetic unknown license");
} finally {
  await rm(directory, { recursive: true, force: true });
}
