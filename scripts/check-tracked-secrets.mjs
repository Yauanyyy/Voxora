import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const SECRET_PATTERNS = [
  ["AWS access key", /AKIA[0-9A-Z]{16}/g],
  ["GitHub token", /gh[pousr]_[A-Za-z0-9]{36,}/g],
  ["OpenAI-style key", /sk-[A-Za-z0-9]{20,}/g],
  ["private key block", /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/g],
  [
    "JWT-like token",
    /eyJ[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}/g,
  ],
];

export function findSecretPatterns(content) {
  const findings = [];

  for (const [name, pattern] of SECRET_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(content)) {
      findings.push(name);
    }
  }

  return findings;
}

async function main() {
  const result = spawnSync(
    "git",
    ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
    {
      cwd: process.cwd(),
      encoding: "buffer",
    },
  );

  if (result.status !== 0) {
    throw new Error("git ls-files failed while enumerating tracked files");
  }

  const files = result.stdout.toString("utf8").split("\0").filter(Boolean);
  const failures = [];

  for (const file of files) {
    const bytes = await readFile(file);
    if (bytes.includes(0)) {
      continue;
    }

    const findings = findSecretPatterns(bytes.toString("utf8"));
    for (const finding of findings) {
      failures.push(`${file}: ${finding}`);
    }
  }

  if (failures.length > 0) {
    console.error("tracked-file secret-pattern check failed:");
    for (const failure of failures) {
      console.error(`- ${failure}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log(
    `Checked ${files.length} tracked and untracked source files for credential-shaped content`,
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await main();
}
