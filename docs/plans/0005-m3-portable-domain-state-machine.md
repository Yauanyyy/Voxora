# Implement the M3 portable domain and session state machine

## Status

Implemented and verified on branch `codex/m3-portable-domain-state-machine`.
The primary agent owns this plan, the final Executor Brief, review-finding
classification, validation, Git integration, and Pull Request handling. A
`sol_planner` subagent supplied read-only evidence and design advice; the
primary agent resolved the open boundaries below and authored this plan.

The implementation is confined to the three existing portable crates and their
tests. It adds no dependency, adapter, native integration, model, asset,
migration, network destination, UI behavior, lockfile change, or notice change.
Cross-platform CI remains the remote evidence gate; the verification record
below describes the completed local and read-only-agent checks.
The user later limited this run's Git integration endpoint to commit and push;
Pull Request creation and Codex Review are not part of the current completion
condition.

## Objective

Implement the complete portable Dictation Session lifecycle with deterministic
fake capabilities before any provider, persistence, Windows, Tauri command, or
product-UI integration. M3 must prove capture, recognition, transactional
processing, target-safe delivery, recovery, persistence failure, cancellation,
timeouts, retries, and stale-response rejection while retaining the exact
terminal outcome and each Recovery Artifact independently.

M3 has no user-facing UI. Its observable result is an original, cross-platform
Rust domain/API and exhaustive deterministic test suite in the three existing
portable crates.

## Source decisions

- M3, dependency direction, delivery protocol, and CI design in
  [`../implementation-plan.md`](../implementation-plan.md).
- Canonical domain terms in [`../../CONTEXT.md`](../../CONTEXT.md).
- Product lifecycle, targeting, processing, history, and privacy behavior in
  [`../product.md`](../product.md).
- Portable boundaries, trust boundaries, coordination, and recovery in
  [`../architecture.md`](../architecture.md).
- Exact phase, outcome, correlation, retry, delivery, and persistence semantics
  in [`../state-machine.md`](../state-machine.md).
- M3 fake and deterministic-test obligations in
  [`../testing.md`](../testing.md).
- Independent-implementation and fail-closed dependency policy in
  [`../licensing.md`](../licensing.md), ADR 0001, ADR 0004, ADR 0005, and ADR 0007.
- Agent delegation and Git requirements in the repository runbooks.

If an authoritative source conflicts with this plan, implementation stops until
the primary agent resolves it explicitly.

## Resolved M3 boundaries

The primary agent resolves the planning questions as follows:

1. A minimal portable `IdentifierSource` port is allowed because sessions,
   records, attempts, operations, and retries require opaque unique identifiers.
   It uses no external dependency, and identifiers are never derived from
   sensitive content.
2. Whitespace-only final recognition is empty. Accepted nonempty transcript
   text is preserved byte-for-byte and is never emitted by diagnostic formatting.
3. The two Built-in Processing Rule algorithms and catalogs remain M4 work. M3
   models ordered built-in and optional LLM steps and uses scripted processing
   fakes to prove skip, success, failure, and transactional Raw Transcript
   fallback. It does not invent underspecified punctuation-tokenization rules.
4. If automatic insertion, Result Panel presentation, and clipboard-last-resort
   all fail, the outcome is `Failed` with a sanitized delivery failure. Final
   Text remains available in memory and through any successful recovery
   persistence; it is never silently discarded.
5. A persistence failure moves retained terminal material into a separately
   correlated recovery context. It does not block a new live Dictation Session.
   A later matching persistence success may mark only the reported materials
   durable; mismatched or closed recovery callbacks are stale.
6. M3 permits either one live Dictation Session or one active record-scoped
   Recognition Attempt retry, never both. This conservative policy avoids
   unapproved capability concurrency and can be relaxed only by a later accepted
   decision.
7. Credential, history retention/deletion, model-management, and shortcut ports
   receive portable request/result shapes and scripted fakes only. Real storage,
   retention, model policy, shortcut registration, and platform behavior belong
   to later milestones.

## In scope

- Typed opaque domain IDs and checked revisions for sessions, Dictation Records,
  Recognition Attempts, configurations, operations, targets, credentials, and
  models.
- Stable dependency-free wire codes and parsing for IDs, phases, outcomes,
  warnings/failures, durability, and other persisted portable discriminants.
- Exact lifecycle phases and exactly five terminal outcomes from the state
  machine specification.
- Start mode, commands, correlated events, event dispositions, ordered effects,
  deadlines, cancellation tokens, and no-mutation stale/competing guards.
- Recorded Audio references, Partial/Raw/Processed/Final text values, processing
  plans/results, Insertion Targets, Recognition Attempts, Dictation Records,
  warnings, sanitized failures, and independent material availability/durability.
- A pure command/event/effect reducer for a live Dictation Session.
- A separately correlated, recognition-only history-retry reducer that appends a
  fresh attempt without mutating the original terminal session or its results.
- Portable ports for audio, shortcuts, recognition, external processing,
  targeting, target validation, injection, Result Panel presentation, clipboard
  fallback, credentials, history, model management, clock, cancellation, and
  identifier generation.
- Small instance-owned application services for the one-active-work guard, live
  session routing, recognition, processing, delivery, recovery, retry, and
  effect dispatch. No service may become a catch-all policy owner.
- Standard-library-only scripted fake ports, deterministic identifier source,
  deterministic clock, ordered call recording, cancellation observation, and
  explicit callback-event injection.
- Exhaustive deterministic tests for every M3 acceptance scenario and invalid
  ID/revision/phase/mode ordering.
- Truthful status updates to the M3 task plan, master plan, README, architecture,
  product-status note, and testing-status note after implementation is verified.

## Out of scope

- React, Tauri commands/events, tray, Recording Overlay, Result Panel UI, settings,
  history UI, or any desktop-shell behavior change.
- Windows APIs or types, audio devices, global shortcut registration, target
  discovery, clipboard APIs, SendInput, UI Automation, Credential Manager, or a
  `platform-windows` crate.
- SQLite, migrations, filesystem-backed audio, retention execution, deletion,
  crash cleanup, or a `history-sqlite` crate.
- Provider protocols, HTTP, SDKs, Doubao, OpenAI-compatible processing,
  sherpa-onnx, local inference, or provider/local-ASR crates.
- Prompt catalogs, Hotword selection, Application Profile matching rules,
  persistent Recognition Configuration settings, or actual built-in-rule text
  transformations.
- Model download, hashing, installation, manifest approval, native binaries,
  model artifacts, assets, new workspace crates, or product packaging.
- Automatic retry or rollback after an uncertain insertion.
- Serde, UUID, Tokio, async-trait, thiserror, proptest, regex, zeroize, or any
  other new external dependency.
- Changes to accepted ADRs, product defaults, milestone order, CI architecture,
  or merge ownership.

## Ownership

The `luna_executor` has exclusive write ownership for the M3 implementation and
tests under:

- `crates/voice-core/src/` and `crates/voice-core/tests/`;
- `crates/voice-ports/src/` and `crates/voice-ports/tests/`;
- `crates/voice-application/src/` and `crates/voice-application/tests/`;
- the three portable crate manifests only if required for crate features or
  dev-only inward workspace dependencies, without external dependencies.

The primary agent exclusively owns this task plan, master-plan status, README,
product/architecture/testing status updates, final verification record, Git,
and Pull Request work. The executor is not alone in the repository, must preserve
all unrelated edits, and must not edit desktop, CI, ADR, dependency-review,
lockfile, notice, script, asset, or model files.

## Architecture and dependency direction

The allowed production dependency direction remains:

```text
voice-application -> voice-ports -> voice-core
                  -> voice-core
```

`voice-core` owns domain values, exact lifecycle meaning, pure transition logic,
and provider/platform-independent failure semantics. `voice-ports` owns only
portable capability contracts. `voice-application` maps ordered core effects to
individual ports and owns bounded, session-scoped coordination.

The core reducer contract is:

```text
State + Command or correlated Event
  -> Transition { state, ordered effects, disposition }
```

Rejected events distinguish stale IDs, stale revisions, unexpected phases,
wrong modes, duplicate stops, competing commands, and terminal callbacks. Every
rejected event leaves the entire state unchanged and emits no capability effect.

Live and retry correlations are separate types so invalid optional-field
combinations are not representable:

```text
Live:  Session ID + session revision + expected phase
ASR:   live correlation + Attempt ID + attempt revision
Retry: Record ID + originating Session ID + fresh Attempt ID
       + attempt revision + expected retry phase
```

Every mutable application value is owned by an ordinary service instance and
mutated through `&mut self`. No `static`, global lock, global singleton, Tauri
managed global session, platform callback mutation, or background thread is
allowed in M3.

## Domain and privacy rules

- Opaque IDs use nonzero numeric values with checked construction and canonical
  formatting. Fake identifiers are synthetic and deterministic.
- Audio references and target/application tokens are opaque values, never paths,
  HWNDs, UI Automation values, provider objects, or sensitive-content hashes.
- Sensitive text/audio/credential/application wrappers do not expose their
  content through `Debug` or diagnostic formatting. Credential secrets are not
  serializable and provide only explicit access to the consuming adapter.
- `SanitizedFailure` accepts only project-owned stage/code/retry/certainty enums;
  provider bodies and arbitrary strings cannot enter failure metadata.
- Available material is independently `NonDurable` or `Durable`; absent material
  has no durability claim. Recorded Audio, incomplete Partial Transcript, Raw
  Transcript, Processed Text, Final Text, Result Panel presentation, and
  clipboard fallback remain independently observable.
- No fixture contains a real credential, endpoint, Prompt, transcript, Hotword,
  audio recording, account identifier, private application identity, or private
  path. Test text is visibly synthetic.
- M3 adds no network destination, dependency, native component, model, asset, or
  redistribution obligation; `Cargo.lock` and `THIRD_PARTY_NOTICES.md` therefore
  must remain unchanged.

## State and failure behavior

### Capture and one-active-work guard

- Start creates correlated IDs through `IdentifierSource`, binds Push-to-Talk or
  Toggle, records checked maximum-duration and recognition deadlines, and emits
  one capture-start effect.
- A competing start or live/retry concurrency attempt is rejected without
  changing existing work.
- Only the matching mode, Session ID, revision, and `Capturing` phase may stop.
  Cross-mode, stale, duplicate, and post-capture stops are no-ops.
- Maximum duration stops capture, records a warning, and continues recognition.
- Esc during capture cancels and discards audio, selects `Cancelled`, creates no
  history/recovery effect, and advances correlation before cancellation.
- Capture failure preserves actual nonempty audio if present. Empty audio is
  `Failed`, makes no recognition request, and creates no zero-length Recovery
  Artifact.

### Recognition and retry

- Capture end resolves the current target once and begins recognition without
  retaining any earlier target. Target and recognition results may arrive in
  either order without becoming stale merely because the other completed first.
- Only the active attempt tuple accepts partial/final/failure callbacks. The last
  matching partial may be retained only as explicitly incomplete recovery text.
- Empty/whitespace-only final, timeout, provider failure, or internal cancellation
  without Esc selects `Failed` and preserves available audio/partial material.
- Timeout and cancellation advance the revision before cancellation is emitted;
  all later callbacks are stale.
- A history retry requires a durable record with durable usable Recorded Audio,
  no live session, and no other retry. It allocates a fresh attempt, increments
  the checked revision, and persists the pending attempt before recognition.
- Retry success/failure affects only the new attempt. It never processes text,
  calls an LLM, resolves/reuses a target, presents a Result Panel, writes the
  clipboard, or injects text. Original outcome and results are immutable.

### Processing and delivery

- Processing starts only from retained Raw Transcript and uses a working copy.
  Ordered fake/scripted steps model built-in rules and at most one optional LLM
  step. Disabled or unavailable LLM is skipped while later local steps continue.
- Any enabled step failure or timeout discards the entire transformed working
  copy, leaves Processed Text absent, selects Raw Transcript as Final Text, and
  adds a processing-fallback warning.
- Delivery revalidates only the captured target and never reactivates or replaces
  it. Target invalidation or focus change starts manual preservation.
- Esc may cancel safely before insertion becomes irreversible. After an explicit
  irreversible marker, confirmed delivery remains `DeliveredAutomatically` and
  unconfirmed delivery remains `DeliveryUncertain`; neither path rolls back or
  automatically retries.
- Definite insertion failure uses Result Panel then clipboard-last-resort and
  selects `ManualDeliveryRequired` when either preserves Final Text. If both fail,
  select `Failed` while retaining Final Text in memory/recovery.

### Persistence and recovery

- Capture-time cancellation creates no history. Failed and post-capture-cancelled
  sessions request recovery persistence even when ordinary history is disabled.
- Persistence success marks only the explicitly reported material durable.
- Persistence failure preserves the previously selected outcome when one exists,
  adds a non-durable warning, erases nothing, requests a generic unsaved-history
  notification, and uses manual preservation when Final Text was not confirmed
  delivered.
- A separately correlated recovery context accepts a later matching persistence
  success without keeping a live session active. Closed or mismatched recovery
  callbacks are stale. A persistence warning never becomes a sixth outcome.

## Implementation steps

1. Commit this approved M3 task plan before product-code changes.
2. Add `voice-core` modules for IDs/revisions, time limits, sensitive values,
   materials, failures/warnings, attempts/records, processing/target/delivery
   values, live state/effects, and retry state/effects.
3. Implement stable explicit wire codes and redacted diagnostics without adding
   serialization or error-helper dependencies.
4. Implement the pure live and retry reducers with checked correlations,
   revisions, deadlines, transactional processing, delivery irreversibility,
   persistence durability, and no-mutation rejection.
5. Add `voice-ports` contracts for every M3 capability and standard-library
   cancellation/identifier/clock abstractions.
6. Add reusable scripted fake ports and deterministic clock/identifier source,
   with ordered call recording and explicit result-event injection.
7. Add separate `voice-application` supervisor, live-session, recognition,
   pipeline, delivery, recovery, retry, and effect-dispatch responsibilities.
8. Add deterministic core transition tables, port/fake contract tests, and full
   application lifecycle scenarios covering every M3 acceptance condition.
9. Run the complete M3 validation matrix. The executor reports evidence and does
   not commit, push, or edit primary-owned documentation.
10. Send the implementation and this plan to `sol_verifier`; classify and fix
    every in-scope finding through a revised brief until accepted.
11. Update authoritative status documentation and the verification record, run
    final validation, commit, fetch/recheck divergence, push, and open a Ready
    Pull Request.
12. Request Codex Review and repeat in-scope actionable correction, validation,
    commit, push, and review until the latest review has no M3 issue. Never merge.

## Tests and validation

Deterministic tests must cover every required row in `docs/state-machine.md`,
including:

- PTT and Toggle starts/stops, wrong-mode/stale/duplicate stops, competing starts,
  one-live-work enforcement, and maximum duration;
- capture failure, empty audio, capture-time Esc deletion/no-history, post-capture
  Esc preservation, and Esc after irreversible delivery;
- matching/stale partials, final, empty, timeout, provider failure, internal
  cancellation, cancellation tokens, revisions, and late callbacks;
- ordered processing, disabled/unavailable LLM skip, step success, local/LLM
  failure fallback, Raw/Processed/Final separation, and fallback warning;
- target capture at stop, target invalidation, focus change, no reactivation,
  insertion success, definite failure, uncertainty, Result Panel, clipboard last
  resort, and failure of both manual mechanisms;
- exact terminal outcome plus independent warnings/failures, availability, and
  durability for every terminal scenario;
- persistence success/failure, unsaved-history notification, later correlated
  recovery success, and stale recovery callbacks;
- retry eligibility, pending-attempt persistence, success, empty, timeout,
  cancellation, provider failure, stale full-tuple callbacks, immutable original
  results/outcome, and absence of processing/target/delivery side effects;
- stable ID/enum code round trips, unknown-code rejection, checked overflow, and
  redacted diagnostic formatting;
- every command/event against incompatible phases where practical, asserting an
  unchanged snapshot and zero effects.

Run from the repository root with the isolated Rust 1.97.1 toolchain discovered
during preflight configured through task-local `CARGO_HOME` and `RUSTUP_HOME`
environment values. Do not record their machine-specific absolute paths:

```text
cargo fmt --all -- --check
cargo check --locked -p voice-core -p voice-ports -p voice-application --all-targets
cargo clippy --locked -p voice-core -p voice-ports -p voice-application --all-targets --all-features -- -D warnings
cargo test --locked -p voice-core -p voice-ports -p voice-application --all-targets
cargo deny check
node scripts/check-tracked-secrets.mjs
git grep -n -E "cfg[[:space:]]*\\([[:space:]]*windows|tauri|HWND|UIAutomation|windows::" -- crates/voice-core crates/voice-ports crates/voice-application
git diff --check
git status --short --branch
```

Also inspect `cargo tree` for inward-only workspace edges, confirm no external
dependency or lockfile/notice change, and review the entire diff for sensitive
`Debug`/`Display` exposure, provider/platform leakage, private paths, real-looking
fixtures, accidental outcome/durability coupling, and scope expansion. CI owns
final Windows/macOS/Linux runner evidence.

## Acceptance criteria

- The exact M3 phases and five outcomes are implemented without combining
  warnings or durability into outcome.
- Every asynchronous mutation requires the complete applicable correlation and
  rejects stale/mismatched events without state change or effect.
- PTT, Toggle, maximum duration, Esc, recognition, processing, targeting,
  delivery, persistence, recovery, retry, and competing-shortcut scenarios have
  deterministic passing tests with exact material assertions.
- Processing is transactional; any enabled-step failure selects Raw Transcript
  as Final Text, and LLM skip does not prevent later local steps.
- Target validation never reactivates or replaces a target, and uncertain
  insertion never rolls back or automatically retries.
- Persistence failure loses no available material, invents no sixth outcome, and
  supports separately correlated later recovery success.
- History retry appends only a fresh Recognition Attempt, preserves the original
  terminal session, and has no processing/target/delivery side effects.
- All listed capability ports and reusable deterministic fakes exist while all
  mutable state remains instance-owned.
- Portable crates contain no Tauri, Windows, provider SDK, UI Automation, native
  path, or global mutable state leakage.
- No dependency, lockfile, notice, model, asset, network, provider, persistence,
  platform, UI, or new-crate scope is added.
- Required local validation passes and cross-platform CI is reported accurately.

## Rollback and recovery

M3 introduces no migration, user data, external service, native integration, or
irreversible runtime effect. Before merge, corrections use additive commits on
the task branch without rewriting shared history. Abandoning M3 means leaving
the branch and Pull Request unmerged. A normal revert of the portable-crate and
status-document commits restores the M2 skeleton without data cleanup.

## Verification record

- The approved `luna_executor` implemented the portable domain, ports,
  application coordination, deterministic fakes, and acceptance tests in the
  three existing portable crates only.
- The approved `sol_verifier` completed successive read-only review rounds.
  Earlier rounds identified in-scope correlation, recovery, cleanup, cancellation,
  processing, retry, wire-code, and coverage defects. Those findings were
  corrected through bounded revised Executor Briefs. The final verdict was
  `ACCEPT`, with no remaining actionable M3 issue.
- The final local suite contains 58 portable Rust tests: 14 application workflow
  tests, 32 core acceptance tests, six core unit tests, and six ports unit tests.
- Final local validation passed formatting, locked portable-crate check, Clippy
  with warnings denied, locked portable-crate tests, `cargo deny check`, tracked
  secret scanning, portable platform-leak scanning, inward dependency-tree
  inspection, and `git diff --check`.
- No manifest, lockfile, `THIRD_PARTY_NOTICES.md`, desktop, CI, provider,
  persistence-adapter, platform-adapter, model, asset, or migration file changed.
- Windows/macOS/Linux CI is not claimed by this local record and remains the
  remote branch/PR validation responsibility.

## Executor Brief

Implement the approved M3 portable domain and lifecycle exactly as specified in
this plan. You own only the three portable crates' source, tests, and narrowly
necessary manifests. You are not alone in the repository: preserve all unrelated
work and do not revert or edit primary-owned documentation, desktop, CI, ADR,
dependency-review, lockfile, notice, script, asset, or model files.

Read the repository guidance and every source decision named above. In
`voice-core`, implement typed opaque IDs and checked revisions; the exact phases
and five outcomes; sensitive Recorded Audio/text/target values; Recognition
Attempts and Dictation Records; independent material availability/durability;
project-owned sanitized failures/warnings; stable dependency-free wire codes;
and pure live/retry command-event-effect reducers. Reject stale IDs, revisions,
phases, modes, duplicate stops, competing work, and terminal callbacks without
mutation or effects. Preserve whitespace-exact nonempty text, treat
whitespace-only final recognition as empty, and never expose sensitive values in
diagnostics.

In `voice-ports`, define portable synchronous command-submission contracts for
audio, shortcuts, recognition, external processing, target resolution and
validation, injection, Result Panel, clipboard fallback, credentials, history,
model management, clock, cancellation, and identifier generation. Outcomes feed
back as fully correlated events. Add reusable standard-library-only scripted
fakes, deterministic clock/IDs, ordered call recording, and cancellation
observation. Do not add an async runtime, external dependency, platform/provider
type, or real adapter behavior.

In `voice-application`, implement separate instance-owned responsibilities for
the one-active-live-or-retry guard, live session, recognition, transactional
pipeline, delivery, recovery, recognition retry, and effect dispatch. Enforce
mode-owned stops, clock-driven limits/timeouts, revision-before-cancel, target
capture/revalidation without reactivation, insertion irreversibility, manual
preservation, non-durable persistence failure, later correlated recovery, and
recognition-only retry. A retry must never mutate the original session outcome or
run processing, LLM, targeting, Result Panel, clipboard, or injection.

Use scripted processing results to prove ordering, disabled/unavailable LLM skip,
and whole-pipeline Raw Transcript fallback. Do not implement the M4 built-in-rule
catalog or punctuation algorithms. Add exhaustive deterministic tests for every
M3 scenario and invalid ordering, asserting exact outcome, warning/failure,
material availability/durability, ordered port calls, cancellation, and unchanged
snapshots for stale events. Use only visibly synthetic content and no sleeps,
threads, network, filesystem, Windows APIs, Tauri, or global mutable state.

Run every validation command in this plan and report changed files, exact results,
acceptance coverage, remaining risks, and anything unverified. Stop on an
authoritative conflict or any need for an external dependency. Do not commit,
push, publish, open or merge a Pull Request.
