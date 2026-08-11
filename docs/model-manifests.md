# Model Manifest Validation Policy

M2 adds only the validation policy and tooling for future reviewed model artifacts. The repository contains no model manifest, model weight, tokenizer, vocabulary, preprocessing file, or approved model. M8 remains blocked until an exact SenseVoice Small artifact independently passes the complete review in [`licensing.md`](licensing.md).

Future approved manifests live under `model-manifests/` and must reference [`../schemas/model-manifest.schema.json`](../schemas/model-manifest.schema.json). The validator fails closed unless every manifest records an exact identity and version, HTTPS source without embedded credentials or query data, retrieval date, publisher, license terms, commercial-use decision, redistribution path, distribution mechanism, every file's positive byte size and non-placeholder lowercase SHA-256, and a repository-relative approval record.

The schema is structural evidence only. A syntactically valid manifest does not replace authoritative source, license-text, provenance, commercial-use, redistribution, conversion, accompanying-file, security, and reviewer checks. Unknown fields, incomplete files, placeholder hashes, non-approved review state, denied license markers, and non-user-initiated distribution paths fail validation. An absent `model-manifests/` directory passes while explicitly approving no model.

Run the policy checks from `apps/desktop`:

```text
npm run models:check
```
