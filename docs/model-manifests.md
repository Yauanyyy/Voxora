# Model Manifest Validation Policy

M2 adds only the validation policy and tooling for future reviewed model artifacts. The repository contains no model manifest, model weight, tokenizer, vocabulary, preprocessing file, or approved model. M8 remains blocked until an exact SenseVoice Small artifact independently passes the complete review in [`licensing.md`](licensing.md).

Future approved manifests live under `model-manifests/` and must reference [`../schemas/model-manifest.schema.json`](../schemas/model-manifest.schema.json). The validator fails closed unless every manifest records an exact identity and version, HTTPS source without embedded credentials or query data, real calendar retrieval/review dates, publisher, license terms, commercial-use decision, redistribution path, distribution mechanism, every file's positive byte size and non-placeholder lowercase SHA-256, and a repository-relative approval record that exists as a tracked regular file.

The schema is structural evidence only. A syntactically valid manifest does not replace authoritative source, license-text, provenance, commercial-use, redistribution, conversion, accompanying-file, security, and reviewer checks. M2's explicit model-license allowlist contains only `Apache-2.0`; adding any other SPDX expression requires a documented compatibility decision and synchronized schema, validator, test, and notice review. Unknown or proprietary expressions therefore fail closed instead of relying on self-declared commercial-use or redistribution booleans.

Unknown fields, incomplete files, placeholder hashes, impossible dates, non-approved review state, non-user-initiated distribution paths, missing or untracked review evidence, parent traversal, backslash-separated paths, and POSIX or Windows absolute paths fail validation. Both artifact paths and review-evidence paths are checked with platform-independent POSIX and Windows semantics. An absent `model-manifests/` directory passes while explicitly approving no model.

Run the policy checks from `apps/desktop`:

```text
npm run models:check
```
