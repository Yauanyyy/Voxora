# Implement M4 local persistence and configuration

## Status

Implemented and verified on branch
`codex/m4-local-persistence-configuration`. The final read-only verifier verdict
is `ACCEPT`. Windows/macOS/Linux, desktop, frontend, and policy checks passed on
Ready for review PR #4. The primary agent owns product and architecture interpretation,
dependency review, status documentation, Git integration, and Pull Request.

The implementation is limited to M4. It adds no desktop settings/history UI,
Tauri commands, capture, global shortcut registration, target resolution,
insertion, provider protocol, network request, local inference, model artifact,
telemetry, account, sync, updater, or server behavior.

## Objective

Implement durable local configuration and history storage with a versioned
SQLite schema, separate audio artifacts, deterministic retention/deletion and
startup cleanup, safe Prompt/rule/Hotword behavior, validated non-secret
provider configuration, and a Windows Credential Manager adapter. Secrets must
remain outside SQLite and ordinary files while recovery material remains
consistent with the M3 durability contract.

## Source decisions

- M4 deliverables and acceptance in
  [`../implementation-plan.md`](../implementation-plan.md).
- Canonical terms in [`../../CONTEXT.md`](../../CONTEXT.md).
- Configuration, Prompt, Hotword, history, retention, credential, and Base URL
  behavior in [`../product.md`](../product.md).
- Dependency direction, trust boundaries, configuration precedence, and
  persistence boundaries in [`../architecture.md`](../architecture.md).
- Persistence/recovery durability in [`../state-machine.md`](../state-machine.md).
- Persistence, redaction, and adapter tests in [`../testing.md`](../testing.md).
- Fail-closed dependency and redistribution policy in
  [`../licensing.md`](../licensing.md), ADR 0001, ADR 0004, ADR 0005, and ADR 0007.
- Agent and Git lifecycle in the two repository runbooks.

An authoritative conflict stops implementation. No conflict was found during
preflight. The exact built-in Prompt text and deterministic punctuation edge
rules below are implementation details fixed by this plan so the executor does
not invent product decisions.

## In scope

- Portable IDs and values for Prompt Presets, Hotword groups/items,
  Application Profiles, built-in rules, rule overrides, retention policies,
  LLM configuration, validated Base URLs, and storage timestamps.
- Built-in Prompt catalog with immutable project-owned entries:
  - `original_text_cleanup` (default): “Clean up the dictated text while
    preserving its original meaning, facts, tone, language, and level of
    detail. Correct obvious recognition, grammar, and punctuation issues.
    Return only the revised text.”
  - `concise_expression`: “Rewrite the dictated text concisely while preserving
    its meaning and essential details. Remove repetition and filler. Return
    only the revised text.”
  - `formal_expression`: “Rewrite the dictated text in a clear, formal style
    while preserving its meaning and facts. Return only the revised text.”
- Built-in rule catalog and deterministic transformations:
  - `remove_trailing_sentence_punctuation`, default disabled: preserve trailing
    whitespace and trailing closing quotation marks, remove the immediately
    preceding run of `.`, `?`, `!`, `…`, `。`, `？`, or `！`.
  - `replace_conversational_punctuation_with_spaces`, default disabled: replace
    common ASCII/CJK sentence, pause, question, and exclamation punctuation
    with one normalized space; preserve an ASCII period between ASCII
    alphanumeric characters, a comma between digits, a colon in numeric time or
    before `/` or `\\`, and URL query punctuation in a token containing `://`.
    Preserve non-candidate technical punctuation and collapse adjacent replaced
    punctuation/whitespace to one space without adding leading/trailing spaces.
- Prompt copy naming (`Name Copy`, then `Copy 2`, and so on), no copied
  shortcut/references, no active-Prompt change, and return of the new editable
  custom Prompt.
- Persistent global Active Prompt and Prompt-shortcut activation. Shortcut
  registration/conflict detection remains M6; M4 stores normalized nonempty
  bindings and resolves an exact configured binding to a persistent selection.
- Confirmed deletion of a Custom Prompt with an atomic reset of every affected
  Application Profile to global-Prompt fallback. Unconfirmed referenced
  deletion returns the affected count without mutation. Built-ins cannot be
  edited or deleted.
- Stable Hotword selection from enabled groups in `(group_id, hotword_id)`
  order, greedily bounded by provider item and UTF-8 byte limits, returning the
  exact selected subset plus `used` and `total` counts. Omission is therefore
  explicit and deterministic.
- Full Base URL validation before persistence: absolute URL; only `http` or
  `https`; host required; no username, password, query, or fragment; nonempty
  port valid; HTTPS required except HTTP is permitted for loopback hosts
  (`localhost`, loopback IPv4, or loopback IPv6). Validation failures contain
  only stable field/error enums and never echo input. TLS-disable configuration
  is not representable.
- Portable ports for configuration persistence, history maintenance, and audio
  artifact storage; add credential deletion to the existing credential port.
- `history-sqlite` adapter with ordered embedded migrations and foreign keys for:
  - scalar settings and active selections;
  - recognition and Language Model configurations using opaque credential
    references only;
  - Prompt Presets and shortcuts;
  - global ordered processing steps and built-in-rule defaults;
  - Hotword groups and Hotwords;
  - Application Profiles and per-rule overrides;
  - Dictation Records, Recognition Attempts, warnings, sanitized failures,
    Raw/Processed/Final/explicitly-incomplete Partial Transcript fields,
    delivery-material flags, and audio references;
  - independent text/audio retention policy;
  - audio artifact metadata and pending artifact-deletion queue.
- SQLite configuration must enable foreign keys, use transactions for compound
  mutations, keep schema versioning fail-closed, and reject a database newer
  than the supported migration set.
- Audio artifact store under an adapter-owned root with `temporary/`,
  `committed/`, and deletion-queue behavior. SQLite stores only an opaque audio
  reference and adapter-owned relative artifact name, never audio blobs or full
  paths.
- History persistence maps every available M3 material independently. Recorded
  Audio is reported durable only when a nonempty committed artifact exists.
  Missing audio never causes text already persisted transactionally to be
  falsely reported as lost or audio to be falsely reported durable.
- Ordinary successful history obeys independent text/audio switches. Failed,
  cancelled-after-capture, manual-delivery, and uncertain-delivery records retain
  their available recovery material even when ordinary history is disabled.
  Capture-boundary records retain only a supplied nonempty artifact.
- Record deletion and retention use a database deletion queue: the transaction
  first removes/nulls database references and queues the relative artifact;
  post-commit cleanup deletes the file and clears the queue. A filesystem error
  leaves an explicit retryable queue row, never a dangling database reference.
- Retention clears expired text and audio independently, removes attempts'
  transcript material consistently, deletes empty records when no retained
  material or required metadata remains, and returns deterministic counts.
- Normal-startup maintenance removes all pre-existing temporary audio and
  retries queued artifact deletions. Crash recovery is not promised.
- A safe database backup copies SQLite only after a checkpoint/consistent
  backup operation. Because secrets never enter the database, the backup also
  contains only opaque credential references.
- `platform-windows` Credential Manager adapter using the reviewed safe keyring
  interface configured to Windows Credential Manager. The crate exposes no
  provider secret to SQLite/configuration ports and maps missing/unavailable
  errors to existing sanitized credential codes.
- Synthetic tests for all M4 acceptance paths, including scanning SQLite and
  backup bytes for a synthetic secret and credential-bearing URL.
- CI/workspace changes needed to compile/test the cross-platform persistence
  adapter on Windows/macOS/Linux and the Windows credential adapter on Windows.
- Dependency review and synchronized notice updates for every new direct or
  transitive source/native dependency and the bundled SQLite amalgamation.

## Out of scope

- Any React view, Tauri command/event, tray, Recording Overlay, Result Panel,
  settings screen, history screen, playback UI, or visual verification.
- Real capture or encoding of microphone audio; tests use synthetic byte arrays.
- Global shortcut registration, conflict resolution, or key suppression.
- Provider requests, Doubao, OpenAI-compatible payloads, reasoning-field
  mapping, provider parameters, provider query settings, or network clients.
- Effective Prompt transmission/wrapper construction; M7 owns provider payloads.
- Application Profile matching against a live Windows target; M4 persists and
  resolves already supplied portable identities only.
- Model management, sherpa-onnx, model manifests/artifacts, downloads, hashing,
  packaging, installer work, or release migration from a prior public version.
- Audio playback/export, cloud sync, accounts, telemetry, updater, project
  server, SQLCipher, plaintext secret fallback, or user-authored executable rule.
- Changes to accepted ADRs, product defaults, milestone order, or merge ownership.

## Ownership

The approved `luna_executor` owns the M4 implementation in:

- `crates/voice-core/` for M4 domain/configuration/rule values and tests;
- `crates/voice-ports/` for M4 ports/fakes and tests;
- `crates/voice-application/` for bounded configuration/Prompt/Hotword/
  retention services and tests;
- new `crates/history-sqlite/` and `crates/platform-windows/` crates;
- workspace/adapter manifests, `Cargo.lock`, and narrowly required CI workflow
  changes;
- `docs/dependency-reviews/m4-local-persistence-configuration.md` and
  `THIRD_PARTY_NOTICES.md` for exact dependency/native review.

The primary agent owns this plan's final status, master-plan/product/
architecture/testing/README status updates, final verification record, Git,
push, and Pull Request. The executor is not alone in the repository: preserve
unrelated edits and never revert them.

## Architecture and dependency direction

Allowed production edges are:

```text
voice-application -> voice-ports -> voice-core
                  -> voice-core

history-sqlite -> voice-ports -> voice-core
               -> voice-core

platform-windows -> voice-ports -> voice-core
                 -> voice-core

Tauri composition root -> selected application/adapters (no M4 commands/UI)
```

`voice-core` owns portable validation and deterministic product rules.
`voice-ports` owns contracts only. `voice-application` owns use-case sequencing,
including confirmation and precedence, without filesystem/SQLite/Windows types.
`history-sqlite` owns SQL, migrations, path-safe artifact layout, and mapping.
`platform-windows` owns Credential Manager integration. No inward crate imports
`rusqlite`, keyring, Windows, Tauri, React, filesystem paths, or adapter types.

Do not create a catch-all repository/service. Separate configuration, Prompt,
Hotword selection, history persistence, retention/deletion, artifact storage,
and credential responsibilities.

## Security, privacy, and licensing

- `CredentialSecret` remains redacted in `Debug`/`Display` and crosses only the
  credential port. No storage/configuration request type can contain a secret.
- Persisted provider configuration contains a validated Base URL and opaque
  credential reference. It cannot represent URL userinfo, query, fragment, or
  disabled TLS verification.
- Base URLs, Prompt content, transcripts, Hotwords, application identity, audio,
  and filesystem paths do not appear in logs or arbitrary error strings.
- SQL/adapter errors are mapped to stable sanitized meanings. Public error
  formatting contains no SQL values, paths, provider input, or secret.
- SQLite parameter binding is mandatory; do not construct SQL from user content.
- Audio relative names are derived only from checked opaque IDs. Reject path
  separators/traversal in any decoded database artifact value before filesystem
  access.
- Fixtures use obviously synthetic secrets/content, temporary task directories,
  and no private endpoint/account/path.
- Review exact locked versions, authoritative sources, licenses, advisory state,
  feature selection, native/bundled code, redistribution and notices before
  acceptance. `rusqlite` must use `default-features = false` with only the
  reviewed bundled SQLite feature needed for reproducible desktop distribution;
  no SQLCipher/OpenSSL/load-extension feature is allowed.
- A safe maintained keyring abstraction is preferred over project-authored
  unsafe FFI. It must be target-scoped to Windows and verified to select Windows
  Credential Manager without pulling non-Windows stores into the Windows graph.
- No model, provider SDK, network destination, asset, or paid call is introduced.

## State and failure behavior

- Migrations run in order in transactions. Unknown newer schema versions fail
  without destructive downgrade.
- Compound Prompt deletion/profile reset and Active Prompt changes are atomic.
- Prompt copy failure changes neither active selection nor source Prompt.
- Referenced custom Prompt deletion without confirmation is a no-op that returns
  affected count. Confirmation clears all references before deletion in the same
  transaction.
- History persistence returns only material proven durable. Missing/uncommitted
  audio leaves audio non-durable while independently persisted text may be
  durable and recoverable through the existing M3 recovery path.
- Adapter/database failure never deletes the caller's source audio or in-memory
  text. Artifact commit uses a temporary file plus same-root atomic rename.
- Retention/deletion DB mutations commit before physical deletion; failed
  physical deletion is retried from the durable queue on maintenance/startup.
- Startup removes only files inside the exact adapter-owned `temporary`
  directory and processes exact validated queued relative names. It never
  recursively deletes the configured storage root or unrelated files.
- Credential read of a missing entry maps to `CredentialMissing`; store access,
  encoding, size, or backend failures map to `CredentialUnavailable`. Secret or
  backend error detail is not persisted or logged.
- Hotword selection never mutates the library and always reports used/total.

## Implementation steps

1. Commit this approved task plan before product-code changes.
2. Complete and commit the M4 dependency/native review together with manifest,
   lockfile, notice, and CI changes required for the approved graph.
3. Add portable M4 domain/configuration/rule values and focused core tests.
4. Add configuration, history-maintenance, artifact, and credential-delete ports
   plus deterministic fakes/contract tests.
5. Add bounded application services for Prompt copy/delete/activation,
   configuration validation/persistence, Hotword selection, and maintenance.
6. Implement `history-sqlite` migrations, mappings, artifact store, backup,
   deletion queue, retention, and startup maintenance with integration tests.
7. Implement and test the Windows Credential Manager adapter. Tests must use a
   synthetic uniquely named entry and clean it up; non-Windows builds must not
   execute or pretend to verify Windows behavior.
8. Run the full M4 validation matrix and inspect the complete diff for privacy,
   migration safety, dependency direction, unsafe path handling, scope, and
   license/notice consistency.
9. Send the approved plan and implementation to `sol_verifier`; resolve every
   in-scope finding through a revised bounded brief until accepted.
10. Update authoritative status documentation and this verification record,
    rerun final validation, commit, fetch/recheck divergence, push, and open a
    Ready for review PR. Never merge.

## Tests and validation

Tests must cover at least:

- fresh database migration, repeat open, foreign keys, built-in seeding/defaults,
  rejected newer schema, and transactional rollback;
- Base URL valid HTTPS, loopback HTTP, IPv4/IPv6 loopback and path; rejected
  relative URL, missing host, unsupported scheme, non-loopback HTTP, username,
  password, query, and fragment; sanitized errors never echo input;
- built-in Prompt immutability, default active Prompt, copy names, copy content,
  no copied shortcut/reference/activation, shortcut activation persistence,
  referenced-delete confirmation/no-op, confirmed atomic profile reset, and
  unreferenced deletion;
- exact built-in-rule transformations including quotes, ellipses, CJK
  punctuation, decimals, domains, abbreviations, numeric commas/times, URLs,
  technical tokens, normalized whitespace, and disabled defaults;
- Hotword enabled groups, stable ordering, item/byte limits, oversized entries,
  deterministic repeatability, and explicit `used N of M` counts;
- LLM configuration round trip with only validated Base URL and opaque
  credential reference; database byte scan proves the synthetic secret and a
  credential-bearing rejected URL are absent;
- Dictation Record/Recognition Attempt round trips, sanitized failures/warnings,
  and independently stored Raw and Final Text (including different values);
- committed audio durability, missing/uncommitted audio partial persistence,
  recovery material despite disabled ordinary history, and capture-boundary
  record with no supplied audio;
- independent text/audio retention, one-record/all-record deletion, cascade of
  attempts/warnings/profile references as applicable, queued file deletion,
  retry after simulated filesystem failure, and no dangling database reference;
- startup orphan temporary-audio cleanup without touching committed/unrelated
  files, traversal rejection, and idempotent maintenance;
- consistent backup plus byte scans proving the synthetic secret is absent;
- credential fake no serialization; Windows real adapter write/read/delete,
  missing-entry and sanitized-error behavior when running on Windows;
- no Tauri/React/Windows/provider types in portable crates, no `cfg(windows)` in
  `voice-core`, and no unsafe code added by Voxora.

Run from the repository root using the pinned toolchain:

```text
cargo fmt --all -- --check
cargo check --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo clippy --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets --all-features -- -D warnings
cargo test --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo check --locked -p platform-windows --all-targets
cargo clippy --locked -p platform-windows --all-targets --all-features -- -D warnings
cargo test --locked -p platform-windows --all-targets
cargo check --locked -p voxora-desktop --all-targets
cargo deny check
node --test scripts/check-tracked-secrets.test.mjs
node scripts/check-tracked-secrets.mjs
git grep -n -E "cfg[[:space:]]*\\([[:space:]]*windows|tauri|HWND|UIAutomation|windows::|rusqlite|keyring" -- crates/voice-core crates/voice-ports crates/voice-application
git grep -n -E "CredentialSecret|credential-bearing|synthetic-secret" -- crates/history-sqlite crates/platform-windows
git diff --check
git status --short --branch
```

Also inspect `cargo tree` for inward-only edges and the Windows target graph;
verify the selected bundled SQLite and credential backend features; inspect
SQLite/backup test bytes; review migrations and all destructive filesystem
targets; and review the complete diff for sensitive content, full paths,
credential-bearing URLs, misleading validation claims, and M5+ scope. CI owns
final cross-platform/Windows runner evidence.

## Acceptance criteria

- All M4 schema areas exist behind ordered migrations and round-trip their
  portable values without secret-bearing columns or audio blobs.
- Secrets exist only behind opaque `CredentialReferenceId` values in portable
  configuration/persistence and in Windows Credential Manager at runtime.
- Invalid or credential-bearing Base URLs cannot be persisted and no validation
  error echoes them.
- Built-in Prompts/rules are seeded, immutable/default-off as applicable, and
  their exact copy/delete/shortcut behavior passes deterministic tests.
- Referenced custom Prompt deletion requires explicit confirmation and confirmed
  deletion atomically resets every affected profile to global fallback.
- Hotword selection is stable, limit-aware, explicit about omissions, and stores
  counts rather than selected content in history metadata.
- Audio is outside SQLite; history reports audio durable only for a committed
  nonempty artifact; later failure/capture-boundary preservation follows policy.
- Raw and Final Text persist independently and can differ on round trip.
- Retention and deletion independently remove text/audio and use the durable
  queue to avoid dangling database references after filesystem failure.
- Startup cleanup deletes orphan temporary audio only within its owned directory.
- SQLite and backups contain neither the synthetic API secret nor rejected
  credential-bearing URL, and diagnostics contain no full sensitive value/path.
- Windows Credential Manager adapter passes its Windows test; non-Windows CI
  still builds/tests the portable/history crates without Windows leakage.
- Dependency/native review and `THIRD_PARTY_NOTICES.md` match the exact lockfile
  and features. No denied/unknown license, unreviewed binary, model, provider,
  network, UI, or M5+ scope is introduced.
- Required local validation passes; remote CI evidence is reported accurately.

## Rollback and recovery

M4 is pre-release and has no existing user database to preserve. Before merge,
abandoning the branch or reverting its additive commits removes the new crates
and schema. Migrations are forward-only and never silently downgrade. Runtime
record deletion is recoverable only through a user-created safe database/audio
backup; the implementation must not advertise automatic crash recovery.
Credential deletion affects only the exact opaque Voxora entry selected by the
user command and never enumerates or deletes unrelated Credential Manager data.

## Verification record

Implementation completed within M4 scope and received final read-only verifier
verdict `ACCEPT` after all findings were resolved. Local validation on Windows
with Rust 1.97.1 passed:

```text
cargo fmt --all -- --check
cargo check --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo clippy --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets --all-features -- -D warnings
cargo test --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo check --locked -p platform-windows --all-targets
cargo clippy --locked -p platform-windows --all-targets --all-features -- -D warnings
cargo test --locked -p platform-windows --all-targets
cargo check --locked -p voxora-desktop --all-targets
cargo deny check
node --test scripts/check-tracked-secrets.test.mjs
node scripts/check-tracked-secrets.mjs
git diff --check
```

The Rust test matrix passed 93 tests, including 22 SQLite integration tests and
three Windows credential-adapter tests with a real synthetic Credential Manager
write/read/delete round trip. `cargo deny 0.20.2` reported advisories, bans,
licenses, and sources all `ok`; the secret scanner checked 97 tracked and
untracked source files. Dependency-feature and portable-boundary inspection
confirmed bundled SQLite only, target-scoped Windows credentials, no inward
adapter leakage, and no Voxora-authored unsafe code. Ready for review PR #4 then
passed the Windows/macOS/Linux portable and persistence matrix, Windows desktop
and credential-adapter job, frontend job, and dependency/artifact policy job.

## Executor Brief

Implement the approved M4 local persistence and configuration milestone exactly
as specified above. You own only the M4 files and responsibilities listed in
Ownership. You are not alone in the repository: preserve all unrelated edits and
do not alter accepted ADRs, M5+ UI/native/provider scope, model files, product
defaults, or merge ownership.

Read every source decision and the current M3 domain/ports/application code.
Create portable configuration types, validation, catalogs, deterministic
built-in-rule algorithms, Prompt semantics, and stable Hotword selection in the
inward crates. Keep sensitive wrappers redacted and make secret-bearing storage
requests unrepresentable. Add narrow configuration/history-maintenance/audio
artifact ports and fakes; add credential deletion without broadening the port.

Implement `history-sqlite` with reviewed minimal dependencies, ordered embedded
migrations, parameterized SQL, foreign keys, transactionally seeded defaults,
history/attempt mapping, separate Raw/Final text, audio metadata but no blobs,
an owned temporary/committed artifact layout, consistent backup, independent
retention, record/all deletion, a durable artifact-deletion queue, and safe
startup cleanup. Report only proven durable materials. Preserve recovery
material when ordinary history is disabled according to the exact outcome rules
in this plan, and never delete caller-owned non-durable material on a failed
persistence attempt.

Implement `platform-windows` with a reviewed safe Windows Credential Manager
backend, target-scoped features, exact entry naming from opaque IDs, sanitized
missing/unavailable mapping, and write/read/delete tests using a unique synthetic
entry that is cleaned up. Do not write project-authored unsafe FFI if the safe
reviewed backend satisfies the port.

Add the exact dependency/native review and notice updates before accepting the
graph. Update only the CI/workspace composition needed to test M4 adapters; do
not add product commands or UI. Use only synthetic data and temporary
adapter-owned directories. Run every validation command, inspect dependency
features and the complete diff, and report changed files, exact test/check
results, acceptance coverage, remaining risk, and unverified remote evidence.
Stop on an authoritative conflict or a need to broaden scope. Do not commit,
push, publish, open a Pull Request, request review, resolve threads, or merge.
