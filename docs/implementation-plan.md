# Voxora Master Implementation Plan

## Status and authority

This is Voxora's authoritative delivery plan. It converts the accepted product decisions into ordered, testable milestones and protects the dependency, privacy, licensing, and recovery constraints throughout implementation.

The primary agent owns this plan and all task-level executor briefs. Execution agents implement only an approved bounded brief. Verification agents review the result without editing. Product code must not begin for a milestone until its entry conditions and brief are satisfied.

`CONTEXT.md`, `docs/product.md`, `docs/architecture.md`, accepted ADRs, and this plan must agree. A conflict stops implementation until the primary agent resolves it explicitly.

## Product outcome

Voxora is an independently implemented, GPL-3.0-only, Windows-first desktop voice-input application for ordinary Windows users and open-source enthusiasts. It captures speech, recognizes it locally or through a user-configured cloud provider, optionally applies built-in local rules and one global LLM processing step, then inserts the final text into the intended external input target or preserves it for manual copying.

The project does not operate or require a public server. It has no account system, cloud sync, team management, telemetry, device identifier, usage-statistic upload, application auto-update, or server self-hosting component.

## Non-negotiable constraints

### Independent implementation and licensing

- Do not copy, rewrite, port, translate, or derive source from SayIt or other GPL/AGPL projects.
- Feature concepts and interaction patterns may inform requirements, but code and assets must be independently produced.
- Source dependencies must be GPL-3.0-only compatible and pass provenance and distribution review before adoption.
- Reject AGPL, SSPL, non-commercial, research-only, field-of-use-restricted, source-unclear, or otherwise incompatible components unless the user explicitly changes policy after a documented review.
- Model weights are independent artifacts. Framework licensing never establishes model licensing.
- Every distributed dependency, native binary, asset, and model manifest entry must be represented in `THIRD_PARTY_NOTICES.md` as applicable.

### Architecture

- Portable core code does not depend on Tauri, React, Windows APIs, UI Automation types, or provider SDKs.
- `voice-core` contains no `cfg(windows)`.
- Platform and provider behavior enters through explicit ports and adapters.
- Windows-only code stays in `platform-windows`.
- React renders state and submits commands; it does not orchestrate sessions.
- Mutable dictation state is session-scoped, not a global singleton.
- There is no catch-all orchestrator. Capture, recognition, processing, targeting, persistence, recovery, and insertion remain separable.
- Commands, events, attempts, cancellations, timeouts, and late responses carry stable session or attempt identifiers.

### Privacy and recovery

- Cloud ASR and LLM credentials use Windows Credential Manager and never ordinary SQLite, JSON, logs, fixtures, crash reports, or plaintext backups.
- Logs contain no complete Prompt, transcript, audio, provider response, credential-bearing URL, or complete private filesystem path.
- Once capture successfully completes with usable Recorded Audio, later recognition,
  processing, delivery, and persistence failures do not silently lose that audio.
  Capture start/stop/end failures have only best-effort partial-audio recovery:
  missing partial audio is valid, while any nonempty audio supplied by the adapter
  is retained. Available transcript text and Final Text keep their existing no-loss
  guarantees.
- Successful history follows user text/audio history settings. Later failures create
  recovery records for retained material until deletion or retention expiry.
- Orphaned temporary audio after a crash is deleted on the next normal startup; crash recovery is not a product guarantee.
- SQLite transcript/history content is not promised to be encrypted at rest; the product must disclose reliance on per-user filesystem protection.

## Accepted product model

### Recognition

- A `Recognition Provider` is a supported implementation type, such as Doubao or a local model family.
- A `Recognition Configuration` is the complete user-selectable ASR option. There is no separate Recognition Profile layer.
- Multiple configurations may use the same provider.
- The active ASR configuration is selected in settings, not by a shortcut.
- First cloud provider: Doubao streaming ASR.
- First local engine: sherpa-onnx with one reviewed SenseVoice Small artifact.
- Cloud Partial Transcript events exist but are not displayed during recording. If final recognition fails, the last available partial may be stored as explicitly incomplete text.
- Recognition retries from history may use a different available Recognition Configuration and create a new attempt without overwriting the original failure.

### Processing

- All settings are visible. Voxora has no standard/expert UI split.
- All local processing rules are built into Voxora. Users cannot author regex, scripts, plugins, or executable processing code.
- Initial built-in rules:
  - `Remove Trailing Sentence Punctuation`;
  - `Replace Conversational Punctuation With Spaces`.
- Both rules begin disabled to preserve transcription fidelity until the user opts in.
- The processing pipeline has one global order and may contain at most one optional LLM step. Built-in rules may appear before or after that step.
- Application Profiles may override each built-in rule with `inherit`, `force enabled`, or `force disabled`, but cannot change order or LLM configuration.
- When LLM processing is disabled, its step is skipped and every enabled local rule still runs in global order.
- Any processing-step failure aborts the transformed result and falls back to the Raw Transcript. Raw Transcript is always retained separately.

### LLM and Prompts

- LLM processing is global and default-off.
- Users may save multiple named `Language Model Configuration` entries. At most one is globally active and supplies a validated persisted Base URL, an opaque credential reference, model, parameters, timeout, and reasoning-mode preference.
- Absence of an active LLM configuration makes LLM processing unavailable.
- Application Profiles cannot choose or disable the LLM provider.
- A persisted Base URL must parse as an absolute URL and contain only scheme, host, optional port, and path. Userinfo, username, password, query, and fragment are rejected before persistence and before any provider request. HTTPS is required for non-loopback endpoints; HTTP is permitted only for loopback endpoints; TLS verification cannot be disabled. Credential values enter only through the opaque CredentialStore reference.
- Future non-secret provider query parameters, if needed by an adapter, use separate validated adapter settings and are never embedded in Base URL. Invalid input reports only sanitized field/error meaning and is never echoed in logs or history.
- Requests are stateless and non-streaming in the first release. They send only the current session text, Effective Prompt, and the stable supported allowed Hotword subset selected for the request.
- Reasoning mode is `provider default`, `disabled`, or `enabled`. Known adapters map supported fields; generic endpoints never receive guessed fields.
- Voxora always has an Active Prompt Preset. Built-in Prompt Presets are immutable and non-deletable but copyable.
- Initial built-in prompts: original-text cleanup, concise expression, and formal expression. Original-text cleanup is the default.
- Custom Prompt Presets contain name, content, and an optional global shortcut.
- A Prompt shortcut permanently changes the global Active Prompt Preset.
- Application Profile Prompt selection overrides the current global Prompt for that application.
- Copying any Prompt creates an editable custom preset named `Original name Copy`, then `Copy 2`, and so on. Content is copied; shortcuts and references are not. The new copy does not automatically become active.
- After copying a Prompt, the UI opens the newly created custom preset directly in its edit view.
- Deleting a Custom Prompt Preset referenced by an Application Profile requires a warning and explicit confirmation. If deletion proceeds, every affected profile stops selecting that preset and follows the global Active Prompt Preset.
- The Effective Prompt is built at request time with a Voxora-owned, immutable wrapper that appends only the stable supported allowed Hotword subset selected for that request as inert reference data. The stored Prompt Preset is never modified.

### Hotwords

- Voxora has one global Hotword Library.
- The library contains globally enabled or disabled named groups.
- Each Hotword contains only its text; it has no weight, pronunciation, alias, provider field, or application-specific selection.
- Enabled Hotwords are offered to ASR providers that support them and to an enabled LLM request.
- Provider or token limits must never cause silent omission. Voxora selects a stable supported allowed Hotword subset for each request and displays `used N of M`; history stores counts, not complete Hotword content.
- Future Hotword Candidate analysis is local-only, default-off, never auto-adds terms, and is scheduled after the first release.

### Recording and UI

- Push-to-Talk and Toggle bindings may both exist simultaneously.
- Toggle defaults to `Ctrl+Shift+Space`. Push-to-Talk is unbound until configured.
- Both modes use the same configurable one-to-five-minute maximum, default five minutes. Reaching the limit stops capture and continues recognition.
- Modifier-only bindings are supported with explicit conflict handling and never suppress arbitrary OS input.
- Only one Dictation Session may be active at a time.
- Esc during capture cancels intentionally, deletes audio, and creates no history.
- Esc after capture stops remaining work but preserves Recorded Audio and available results in history. The strong audio guarantee begins only after capture successfully ends with usable audio; capture-boundary failure recovery is best effort.
- The Recording Overlay is not focusable and never an insertion target. During capture it shows elapsed seconds, input amplitude, low-volume warning, and time-limit warnings. After capture it shows only a generic processing state. Failure details are not displayed there.
- Low-volume detection only warns; it never pauses or ends recording.
- Time-limit UI warns with 30 seconds remaining and displays a final ten-second countdown.
- Failure UI displays a generic failure and directs the user to history for sanitized stage and reason details.
- Start at login is configurable and default-off.

### Targeting and insertion

- At capture end, Voxora resolves the currently focused eligible input target. It no longer falls back to a previously valid input target when the current focus is an unrelated non-input control.
- The target at capture end also determines Application Profile matching.
- During recognition or processing, Voxora never steals focus. Automatic insertion occurs only if the captured target remains valid and focused.
- Voxora's windows and overlays are always excluded.
- The first Windows injector uses clipboard paste with SendInput fallback.
- Clipboard preservation is best effort for safe common formats and uses sequence checks to avoid overwriting a clipboard the user changed.
- Voxora does not elevate itself or install a privileged helper. Elevated targets fall back safely.
- If insertion is unavailable or unsafe, a non-focus-stealing Result Panel presents Final Text and a Copy action.
- If the Result Panel itself cannot appear, Final Text is written to the clipboard and the user is notified.

### History and storage

- A unified Dictation Record relates audio, recognition attempts, Raw Transcript, Processed Text, Final Text, status, and sanitized failure information.
- Text history and audio history are independently configurable, both default-on with a default 30-day retention period.
- Retention periods are user-adjustable.
- Sessions with usable audio that fail after successful capture create recovery records even when ordinary history is disabled. Capture-boundary failures retain only any partial audio actually supplied by the adapter; missing partial audio is valid.
- Users can play stored audio, delete it, and retry recognition with another configuration. Direct audio export is post-first-release.
- Users can delete one record or all records.

### Application profiles

- Application Profiles are default-off and match executable identity only.
- Classic applications use a canonical executable path stored locally and redacted from logs.
- Packaged applications use Package Family Name/AUMID-compatible identity.
- Window titles are never collected, persisted, logged, or uploaded.
- A matched profile may override built-in-rule enablement and select a Prompt Preset.
- No match uses global rules and the global Active Prompt Preset.

## Planned repository structure

Create directories only when a milestone needs them. Do not generate empty adapter crates in advance.

```text
/
├─ AGENTS.md
├─ CONTEXT.md
├─ LICENSE
├─ README.md
├─ THIRD_PARTY_NOTICES.md
├─ Cargo.toml
├─ rust-toolchain.toml
├─ deny.toml
├─ docs/
│  ├─ implementation-plan.md
│  ├─ product.md
│  ├─ architecture.md
│  ├─ state-machine.md
│  ├─ testing.md
│  ├─ roadmap.md
│  ├─ licensing.md
│  ├─ adr/
│  ├─ plans/
│  └─ runbooks/
├─ crates/
│  ├─ voice-core/
│  ├─ voice-ports/
│  ├─ voice-application/
│  ├─ history-sqlite/
│  ├─ provider-doubao/
│  ├─ provider-openai-compatible/
│  ├─ local-asr-sherpa/
│  └─ platform-windows/
├─ apps/
│  └─ desktop/
│     ├─ src/
│     └─ src-tauri/
└─ .github/
   └─ workflows/
```

## Dependency direction

```text
React UI
   ↓ commands / state events
Tauri desktop composition root
   ↓
voice-application
   ↓
voice-ports ← adapter implementations
   ↓          ├─ history-sqlite
voice-core    ├─ provider-doubao
              ├─ provider-openai-compatible
              ├─ local-asr-sherpa
              └─ platform-windows
```

Rules:

- `voice-core` owns domain values, deterministic transitions, and provider/platform-independent error meaning.
- `voice-ports` owns capability contracts and portable request/result types and depends only on `voice-core`.
- `voice-application` owns bounded use cases and session-scoped coordination and depends only on core and ports.
- Adapters depend inward on ports/core. No inward crate depends on an adapter.
- The Tauri crate is the only production composition root and may depend on application and selected adapters.
- Frontend types are generated or mapped at the Tauri boundary; React never imports Rust/provider concepts directly.

## Delivery protocol

Every milestone and material task follows `docs/runbooks/agent-execution.md`:

1. The primary agent reads current facts and writes a task plan or Executor Brief under `docs/plans/`.
2. The brief defines ownership, constraints, exact acceptance criteria, validation, and non-goals.
3. An execution subagent implements only that brief and reports evidence.
4. A read-only verification subagent checks the implementation against the brief, architecture, tests, security, privacy, and licensing requirements.
5. The primary agent resolves findings, updates the plan, validates the final diff, and performs the allowed Git workflow.

The primary agent does not delegate unresolved product decisions. Execution agents do not redesign the architecture.

## Milestones

### M0 — Governance and master plan

Objective: establish shared language, repository rules, agent roles, Git/PR policy, and the authoritative delivery plan.

Deliverables:

- `CONTEXT.md`;
- `AGENTS.md`;
- `.codex/config.toml` and scoped agent definitions;
- repository workflow;
- agent execution workflow;
- this master plan.

Acceptance:

- no product code or runtime dependency is added;
- documents agree on independent implementation, user-owned merge, privacy, and dependency direction;
- Git diff passes whitespace validation;
- initial commit contains no credentials or private user content.

### M1 — Product, architecture, licensing, and decision baseline

Objective: create the documentation required to judge every later implementation.

Deliverables:

- `README.md`, GPL-3.0-only `LICENSE`, and initial `THIRD_PARTY_NOTICES.md`;
- `docs/product.md`, `docs/architecture.md`, `docs/state-machine.md`, `docs/testing.md`, `docs/roadmap.md`, and `docs/licensing.md`;
- ADRs for Windows-first portable core, Tauri/Rust/React, no project server, GPL-3.0-only, ports/adapters, sherpa-onnx, and credentials outside SQLite;
- documented dependency and model acceptance checklist.

Acceptance:

- all accepted product decisions in this plan are traceable to product or architecture documentation;
- state and error semantics cover cancellation, timeout, retry, failure fallback, recovery, late responses, and insertion safety;
- model weights are explicitly treated separately from sherpa-onnx;
- no unreviewed third-party dependency is introduced.

### M2 — Workspace and CI skeleton

Task status: implemented, reviewed, and merged into `main` through Pull Request #2 by the user.

Objective: establish the smallest buildable cross-platform workspace without speculative provider/platform implementations.

Deliverables:

- Rust workspace with `voice-core`, `voice-ports`, and `voice-application`;
- Tauri 2 + React + TypeScript + Vite desktop shell;
- Vitest and Rust test baseline;
- GitHub Actions for common Rust crates on Windows/macOS/Linux, Windows desktop build, frontend build/lint/test, and dependency-license checks;
- formatting, linting, lockfiles, and model-manifest validation policy.

Acceptance:

- common crates compile and test on all three OS runners without Windows conditionals in `voice-core`;
- Windows runner builds the desktop application;
- frontend build, lint, and tests pass;
- license checks fail closed for denied or unknown licenses according to documented policy;
- no empty provider or platform crate is added merely to match the future tree.

### M3 — Portable domain and session state machine

Task status: implemented and locally verified on `codex/m3-portable-domain-state-machine`; the final read-only verifier verdict is `ACCEPT`. Cross-platform CI remains remote evidence before merge. See [`docs/plans/0005-m3-portable-domain-state-machine.md`](plans/0005-m3-portable-domain-state-machine.md).

Objective: prove the complete dictation lifecycle using fakes before any native or provider integration.

Deliverables:

- domain IDs, values, events, phases, attempts, records, targets, transcripts, processing results, and structured failures;
- ports for audio, shortcuts, recognition, processing, injection, target resolution, credentials, history, model management, and clock;
- session-scoped application services without global mutable state;
- Fake ports and deterministic clock;
- exhaustive state-machine tests.

Acceptance scenarios:

- PTT press/release and Toggle start/stop;
- automatic stop at the configured maximum;
- Esc cancellation during capture and post-capture stop semantics;
- cloud Partial Transcript acceptance and stale partial rejection;
- ASR failure with preserved audio and optional incomplete text;
- local-rule and LLM failure fallback to Raw Transcript;
- timeout, cancellation token, retry attempt, and late-response rejection;
- target invalidation and Result Panel fallback;
- injection uncertainty and clipboard-last-resort behavior;
- history failure without silent loss;
- competing shortcut events cannot create concurrent sessions.

### M4 — Local persistence and configuration

Objective: persist safe configuration and history while keeping credentials outside SQLite.

Deliverables:

- SQLite schema and migrations for settings, prompts, hotwords, application profiles, dictation records, recognition attempts, and retention metadata;
- audio artifact store outside SQLite blobs;
- retention and deletion services;
- Windows Credential Manager adapter;
- built-in Prompt and rule catalogs;
- copyable Prompt behavior and global Prompt shortcuts;
- hotword groups and stable provider-limit selection.

Acceptance:

- API keys and credential-bearing URLs never appear in SQLite, JSON, logs, exported settings, or backups; credentials are represented only by opaque CredentialStore references;
- persisted Base URLs parse as absolute URLs and are limited to scheme, host, optional port, and path; userinfo, username, password, query, and fragment are rejected before persistence;
- built-in prompts cannot be edited/deleted and copies do not inherit shortcuts;
- deleting a referenced Custom Prompt Preset requires confirmation and resets affected Application Profiles to follow the global Active Prompt Preset;
- deletion and retention remove associated artifacts consistently;
- failures after successful capture retain usable Recorded Audio according to policy, while capture-boundary failures recover only any partial audio supplied by the adapter;
- orphan temporary audio cleanup is tested;
- history records raw and final text independently.

### M5 — Desktop UX with fake adapters

Objective: validate the complete user journey before adding native capture, provider calls, or real injection.

Deliverables:

- tray lifecycle and settings/history windows;
- Recording Overlay animation, elapsed time, input amplitude, low-volume warning, deadline warning, and generic processing/failure state;
- Result Panel with Copy action and no initial focus theft;
- settings for ASR, LLM, prompts, shortcuts, rules, hotwords, retention, microphone, and application profiles;
- Tauri command/event boundary using fake adapters;
- React unit and interaction tests plus rendered visual verification.

Acceptance:

- React contains no dictation orchestration;
- Partial Transcript is not displayed in ordinary recording UI;
- overlay and Voxora windows never become insertion targets;
- failure details appear in history, not the processing overlay;
- settings precedence and Prompt copying match this plan;
- copying a Prompt opens the new editable copy without changing the Active Prompt Preset or duplicating its shortcut;
- start-at-login can be enabled or disabled and defaults to disabled;
- complete fake workflow reaches insertion or Result Panel without losing text.

### M6 — Windows audio, shortcuts, targeting, and insertion

Objective: replace fake platform behavior with safe Windows adapters.

Deliverables:

- system-default and pinned microphone capture;
- PTT, Toggle, Prompt shortcuts, modifier-only detection, and conflict validation;
- current-focus target resolution for classic and packaged applications;
- clipboard-paste injector with supported-format preservation and sequence checks;
- SendInput fallback;
- permission mismatch, disappearing target, focus change, and injection-timeout handling.

Acceptance:

- pinned-device loss does not silently switch microphones;
- low-volume analysis remains local and warning-only;
- modifier-only Toggle fires on clean release; modifier-only PTT observes hold semantics without suppressing arbitrary OS input;
- target is captured at recording stop and never forcibly reactivated;
- focus change during processing routes Final Text to Result Panel;
- elevated-target and uncertain-insertion cases preserve Final Text;
- all Windows types remain inside `platform-windows`.

### M7 — Doubao ASR and OpenAI-compatible processing

Objective: add the first cloud recognition and optional LLM adapters after documentation and license review.

Deliverables:

- multiple saved Doubao Recognition Configurations with secure credentials;
- streaming audio and Partial Transcript events;
- hotword capability negotiation and limit reporting;
- OpenAI-compatible stateless, non-streaming text processing;
- validated Base URL, model, parameters, timeout, Prompt, hotword wrapper, and capability-aware reasoning mode;
- sanitized provider error mapping.

Acceptance:

- no request is sent without explicit user configuration;
- invalid Base URLs are rejected before persistence and before a request; userinfo, username, password, query, and fragment are not accepted;
- non-loopback HTTP endpoints are rejected, loopback HTTP is permitted, and TLS verification cannot be disabled;
- Base URL validation failures report sanitized field/error meaning without echoing the input URL;
- LLM default remains off;
- provider response bodies and sensitive request data are not logged;
- final ASR, partial failure, LLM timeout, empty result, malformed response, cancellation, and late response are tested;
- LLM failure falls back to Raw Transcript and records only sanitized failure details.

### M8 — Local ASR and model manager

Objective: provide CPU-capable offline recognition with reviewed artifacts.

Entry gate: exact SenseVoice Small artifact, source, SHA-256, size, license, commercial-use rights, and sherpa-onnx integration terms are verified and documented.

Deliverables:

- sherpa-onnx adapter behind Recognition Engine port;
- local one-shot recognition after capture;
- model manager with user-initiated download, cancellation/resume as supported, SHA-256 verification, license confirmation, deletion, and manual update;
- import of local packages only when they exactly match a known reviewed manifest;
- CPU progress and cancellation handling.

Acceptance:

- application package contains no model weights;
- corrupted, partial, wrong-version, or hash-mismatched downloads never become active;
- every installed model directory records source, version, size, hash, and license;
- local recognition runs on an ordinary CPU-only Windows test machine within benchmarked expectations;
- no automatic model update or background check occurs;
- local and cloud configurations remain user-selectable without automatic privacy-changing fallback.

### M9 — Release hardening

Objective: prepare a transparent Windows-first public release without weakening portable-core guarantees.

Deliverables:

- Windows x86_64 per-user NSIS package;
- CI signing hook with unsigned-build behavior documented;
- accessibility, privacy, failure-recovery, history-retention, disk-use, and upgrade tests;
- dependency/model notice audit;
- user documentation for credentials, cloud transmission, local models, history storage, elevated targets, clipboard fallback, and SmartScreen;
- release checklist and rollback guidance.

Acceptance:

- Windows 10 22H2 and Windows 11 validation is documented;
- common crates still compile/test on Windows/macOS/Linux;
- no telemetry, server dependency, account, sync, or application updater is introduced;
- install/uninstall does not delete user data without an explicit user choice;
- all required checks pass and residual risks are documented before a Ready PR.

### M10 — Post-first-release candidates

Not part of first-release acceptance:

- local Hotword Candidate analysis from history;
- a second local ASR model;
- direct audio export;
- macOS/Linux platform adapters and desktop packaging;
- additional cloud ASR providers;
- additional safe built-in processing rules.

## Testing strategy

### Portable Rust tests

- deterministic state transition tables;
- property tests for invalid event ordering and stale attempt IDs where valuable;
- fake-clock timeout and cancellation tests;
- fake-port contract tests;
- serialization compatibility tests for persisted portable values;
- no Windows target required.

### Adapter contract tests

- every adapter runs a shared conformance suite for success, cancellation, timeout, malformed response, retryability, and redaction;
- provider fixtures are synthetic and contain no real transcript, Prompt, key, endpoint, or account value;
- platform adapters expose provider/platform-independent failure meaning.

### Frontend tests

- reducers/view models for overlay phases and settings precedence;
- Prompt copy naming and shortcut conflict behavior;
- application overrides and global fallback;
- history deletion and recovery actions;
- no full provider error details in the overlay;
- visual verification for settings, Recording Overlay, Result Panel, history, empty states, and failures.

### End-to-end and manual Windows tests

- common editors, browsers, chat applications, classic Win32 apps, packaged apps, and elevated-target fallback;
- focus changes during recording and processing;
- clipboard changes during injection;
- microphone disappearance and low-volume warning;
- modifier-only shortcut conflicts;
- model download corruption and disk exhaustion;
- unsigned installer/SmartScreen documentation.

## CI design

- Windows/macOS/Linux: format check, common-crate compile, clippy, and tests.
- Windows: Tauri desktop build and Windows-gated adapter tests.
- Frontend: deterministic install, TypeScript build, lint, and Vitest.
- Licensing: Rust license/source/advisory policy, frontend license inventory, native/model manifest validation, and `THIRD_PARTY_NOTICES.md` consistency.
- Security: secret-pattern checks limited to tracked files, log-redaction tests, and fixtures that prove credentials are not serialized.
- CI must not make paid provider calls or download large model weights.

## Known risks and planned controls

- **SenseVoice artifact license uncertainty:** block M8 until exact artifact review is recorded.
- **Framework/model license confusion:** separate notice and manifest records; never infer.
- **Doubao protocol drift:** isolate protocol in adapter, use official documentation, and keep contract fixtures synthetic.
- **Reasoning-disable incompatibility:** capability-aware mapping with provider-default fallback and visible unsupported status.
- **Modifier-only shortcut conflicts:** exact conflict registry, hold/release semantics, warnings, and no broad key suppression.
- **Clipboard restoration limits:** supported-format best effort, sequence checks, Result Panel, and clipboard-last-resort fallback.
- **Elevated target restrictions:** no self-elevation; preserve text and report the limitation.
- **Plaintext transcript history:** explicit disclosure, per-user paths, deletion controls, no secret co-location.
- **Audio disk growth:** separate retention, storage reporting, no silent deletion inside retention without an explicit policy.
- **Unsigned installer reputation:** signing hook and clear SmartScreen documentation.
- **No remote at bootstrap:** create a reviewed local bootstrap commit; push and PR start only after the user configures GitHub.

## Completion definition

Voxora's first release is complete only when:

- accepted product behavior is implemented without unresolved documentation conflict;
- every milestone acceptance condition through M9 is satisfied or explicitly removed by the user;
- required CI passes;
- dependency and model notices are complete;
- credentials, private content, and full sensitive paths are absent from logs and tracked fixtures;
- failure paths preserve Recorded Audio after successful capture, and preserve available text according to policy; capture-boundary audio recovery remains best effort;
- the Windows package is tested and residual limitations are documented;
- a Ready for review PR exists when a remote is available;
- the user, not Codex, decides and performs the merge.
