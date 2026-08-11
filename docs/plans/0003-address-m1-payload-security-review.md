# Address M1 payload and endpoint-security review findings

## Status

Implemented and independently verified on branch codex/m1-documentation-baseline. Ready for PR review; the user retains merge and review-thread ownership.

## Objective

Correct two confirmed PR #1 review findings so cloud requests send only the stable supported allowed Hotword subset and persisted provider Base URLs cannot contain credentials or uncontrolled query/fragment data.

This is a documentation-only correction. It introduces no product code, dependency, manifest, CI, provider integration, schema, migration, platform API, model, binary, or asset.

## Source decisions

- AGENTS.md credential-storage, log-redaction, privacy, and fail-closed rules.
- docs/implementation-plan.md LLM/Prompt, Hotword, M4 persistence, and M7 provider acceptance sections.
- docs/product.md stable allowed Hotword subset and direct-provider payload contract.
- docs/architecture.md credential-store, provider-request, persistence, and diagnostics trust boundaries.
- docs/testing.md provider-payload, serialization, redaction, and credential-store obligations.
- The two unresolved, non-outdated PR #1 review threads on docs/implementation-plan.md lines 80 and 77 at reviewed commit 149b7ff.

## Review assessment

Both findings are valid:

1. The master request rule says enabled Hotwords are sent, while the authoritative Hotword limit rule requires a stable allowed subset with used N of M reporting.
2. A Base URL containing URL userinfo or query-string credentials could be saved as ordinary configuration and leak secrets into SQLite, JSON, exports, backups, logs, or requests outside the credential-reference boundary.

## In scope

- docs/implementation-plan.md
- docs/product.md
- docs/architecture.md
- docs/testing.md
- this task plan and final verification record

## Out of scope

- Code, tests, manifests, dependencies, CI, schemas, migrations, provider/platform implementations, models, binaries, assets, or notices.
- Designing provider-specific query parameters or adding a new endpoint configuration system.
- Changing the global Hotword Library, group behavior, stable-selection algorithm, or used N of M product behavior.
- GitHub replies, reactions, thread resolution, review submission, merge, or auto-merge.

## Ownership

The execution agent has exclusive write ownership of docs/implementation-plan.md, docs/product.md, docs/architecture.md, and docs/testing.md. The primary agent owns this plan, product/security interpretation, final validation, commit, push, and PR integration. The verification agent is read-only.

The executor is not alone in the repository. It must preserve every unrelated edit and must not change other plans, state-machine semantics, licensing documents, ADRs, runbooks, AGENTS.md, configuration, or unrelated files.

## Architecture and dependency direction

No executable dependency edge changes. Base URL validation belongs at the configuration/application boundary before persistence and before adapter invocation. Credential values remain behind CredentialStore and provider adapters receive them separately from the validated Base URL.

## Security, privacy, and licensing

### Hotword payload

- ASR and LLM requests may send only the stable supported allowed subset selected from globally enabled Hotwords.
- Provider/token limits must not silently omit terms: the UI reports used N of M and history stores counts, not Hotword content.
- The Effective Prompt wrapper contains only that selected allowed subset, never the full enabled library when limits apply.

### Base URL validation

- A persisted provider Base URL must parse as an absolute URL and may contain only scheme, host, optional port, and path.
- URL userinfo, username, password, query, and fragment components are rejected before saving configuration and before any provider request.
- Credentials enter only through the opaque credential reference resolved by CredentialStore.
- HTTPS is required for non-loopback endpoints. HTTP is permitted only for loopback endpoints. TLS verification cannot be disabled.
- If a future adapter requires a non-secret provider query parameter, it must be modeled as a separate validated adapter setting; it must not be embedded in Base URL.
- Invalid or credential-bearing URL input is reported with sanitized field/error meaning and is never echoed into logs or history.

No dependency, model, native component, asset, license, or THIRD_PARTY_NOTICES change is introduced.

## State and failure behavior

This task does not change Dictation Session phases or terminal outcomes.

- Invalid Base URL input is a configuration-validation failure before persistence/request, not a provider runtime failure.
- No request is sent when endpoint validation fails.
- Hotword selection is deterministic for the same enabled library, provider capability/limit, and token budget. The request contains exactly the reported allowed subset.

## Implementation steps

1. Correct the master LLM payload rule to say current session/pipeline text, Effective Prompt, and the supported allowed Hotword subset.
2. Correct the master Prompt wrapper wording so it appends only the selected allowed subset.
3. Add the fail-closed Base URL validation contract to the accepted LLM model and M7 acceptance; add the persistence/serialization gate where appropriate in M4.
4. Align product documentation with the allowed-subset wrapper and Base URL validation/user-visible behavior.
5. Align architecture trust and persistence boundaries so validated Base URL data and CredentialStore secrets remain separate.
6. Add tests for exact allowed-subset payload/reporting and rejection/non-persistence of userinfo, query, fragment, non-loopback HTTP, and disabled TLS verification.
7. Run all validation below and report exact results.

## Tests and validation

Run from the repository root:

    git status --short --branch
    git diff --check
    git diff --name-status
    git diff --stat

Also verify:

- only this plan and the four authorized documentation files change;
- every ASR/LLM payload description uses the supported/selected allowed Hotword subset rather than the complete enabled library;
- used N of M and history count-only behavior remain unchanged;
- Base URL is absolute and limited to scheme/host/port/path;
- userinfo, username/password, query, and fragment are rejected before persistence and request;
- HTTPS/non-loopback, loopback HTTP, and non-disableable TLS verification rules are consistent;
- credential values still enter only through CredentialStore references;
- serialization tests forbid credential-bearing URLs in SQLite, JSON, exports, and backups;
- validation errors and logs never echo credential-bearing URLs;
- relative links, whitespace, unfinished-marker, incorrect-license-declaration, sensitive-content, and private-path checks pass;
- no code, dependency, manifest, CI, schema, migration, provider/platform implementation, model, binary, asset, or notice change appears.

No build, paid provider call, platform test, network integration test, or model download is appropriate for this documentation-only correction.

## Acceptance criteria

- Both PR review findings are fully resolved without contradicting product, architecture, persistence, or testing documents.
- Requests contain exactly the stable reported allowed Hotword subset.
- Credential-bearing or uncontrolled Base URLs cannot reach ordinary persistence or provider requests.
- Separate CredentialStore handling remains the only credential path.
- Existing privacy, redaction, five-outcome, licensing, recovery, and no-server guarantees remain unchanged.
- A read-only verification agent returns ACCEPT.

## Rollback and recovery

The correction changes Markdown only and has no runtime, data, credential, schema, migration, or model effect. Before merge it can be corrected with ordinary additive commits or left unmerged. Shared history must not be rewritten.

## Verification record

The execution agent changed only docs/implementation-plan.md, docs/product.md, docs/architecture.md, and docs/testing.md. The primary agent clarified one product-document sentence so query and fragment components are not grammatically presented as parts of URL userinfo. The read-only verification agent then returned ACCEPT with no actionable findings.

Verified evidence covers exact selected Hotword-subset payloads, `used N of M` reporting, count-only Hotword history, selected-subset-only Effective Prompt wrapping, fail-closed Base URL validation before persistence and request, opaque CredentialStore references, endpoint transport rules, TLS-verification enforcement, sanitized diagnostics, scope allowlisting, relative links, whitespace, sensitive content, private paths, GPL-3.0-only invariants, and absence of executable, dependency, model, asset, schema, migration, CI, or notice changes. The verified canonical GPLv3 LICENSE SHA-256 remains `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`. Builds, provider calls, platform tests, network integration tests, and model downloads were not run because this task is documentation-only.

## Executor Brief

Implement docs/plans/0003-address-m1-payload-security-review.md exactly on codex/m1-documentation-baseline.

Read AGENTS.md, CONTEXT.md, docs/implementation-plan.md, docs/product.md, docs/architecture.md, docs/testing.md, docs/plans/0001-m1-documentation-baseline.md, docs/plans/0002-address-m1-review-findings.md, and this complete plan before editing.

You have exclusive write ownership only of docs/implementation-plan.md, docs/product.md, docs/architecture.md, and docs/testing.md. You are not alone in the repository: preserve unrelated edits and stop on any conflict with authoritative sources.

Correct both confirmed review findings using the exact Hotword payload and Base URL validation rules in this plan. Keep credentials exclusively behind CredentialStore references and keep the change documentation-only.

Do not edit any other file. Do not add code, dependencies, manifests, CI, schemas, migrations, provider/platform implementations, models, binaries, assets, notices, or new product features. Do not stage, commit, push, reply to GitHub, resolve threads, submit a review, or merge.

Run every validation in this plan and report changed files, exact results, remaining risks, and anything unverified.
