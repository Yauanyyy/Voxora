# Voxora

Voxora is a planned Windows-first desktop voice-input application. A user-initiated Dictation Session is intended to capture speech, recognize it locally or through a user-configured cloud provider, optionally apply local processing and one global language-model step, and insert the resulting Final Text into an eligible external input target. When safe insertion is unavailable, the design preserves the text for manual copying.

## Current status

M3 now implements and locally verifies the portable Dictation Session domain, correlated live and history-retry reducers, capability ports, session-scoped application coordination, deterministic fakes, and exhaustive lifecycle tests in `voice-core`, `voice-ports`, and `voice-application`. The desktop still displays only the M2 not-yet-implemented shell: real capture, providers, Windows adapters, persistence, credentials, models, history UI, and product UX remain unimplemented. The portable proof therefore does not claim that end-user dictation currently works.

## Intended first release

The first release is planned for ordinary Windows users and open-source enthusiasts. Its documented scope includes:

- Push-to-Talk and Toggle recording with one active Dictation Session;
- user-selected local or cloud recognition, with preserved audio and recovery material on failure;
- configurable built-in text rules and an optional global, stateless, non-streaming LLM step;
- Prompt Presets, a global Hotword Library, and executable-identity Application Profiles;
- safe target resolution, clipboard paste with a SendInput fallback, and a non-focus-stealing Result Panel when automatic insertion is unsafe;
- independently configurable text and audio history with retention, deletion, playback, and recognition retry.

These capabilities remain planned until the milestone acceptance conditions in [`docs/implementation-plan.md`](docs/implementation-plan.md) are implemented and verified.

## Explicit non-goals

Voxora does not operate a project server and is not planned to include accounts, cloud synchronization, team management, telemetry, device identifiers, usage-statistic uploads, application auto-update, or a privileged helper. Users cannot author executable processing scripts, arbitrary plugins, or provider-specific Hotword metadata. macOS/Linux adapters, additional providers and models, direct audio export, and Hotword Candidate analysis are post-first-release candidates.

## Architecture at a glance

The desktop skeleton establishes this intended layering:

```text
React UI → Tauri desktop boundary → voice-application → voice-ports → voice-core
                                      ↘ selected provider/platform/history adapters
```

Portable business logic is independent of Tauri, React, Windows APIs, UI Automation types, and provider SDKs. Adapters depend inward on explicit ports; Windows-only behavior remains in `platform-windows`; React renders state and submits commands but does not orchestrate sessions. See [`docs/architecture.md`](docs/architecture.md) and [`docs/state-machine.md`](docs/state-machine.md).

## Portable development checks

The repository pins Rust 1.97.1 and Node.js 24.15.0. Common commands are:

```text
cargo fmt --all -- --check
cargo check --locked -p voice-core -p voice-ports -p voice-application --all-targets
cargo clippy --locked -p voice-core -p voice-ports -p voice-application --all-targets --all-features -- -D warnings
cargo test --locked -p voice-core -p voice-ports -p voice-application --all-targets
cargo deny check
```

From `apps/desktop`:

```text
npm ci --ignore-scripts
npm run format:check
npm run lint
npm run test
npm run build
npm run licenses:check
npm run models:check
npm run tauri build -- --no-bundle
```

The desktop build command is Windows-only in the current workspace. M3 adds no dependency, model, asset, native component, lockfile entry, or notice obligation. See [`docs/dependency-reviews/m2-workspace-ci.md`](docs/dependency-reviews/m2-workspace-ci.md) for the existing dependency, license, advisory, action-pin, and asset review.

## Privacy and recovery posture

Audio, transcripts, Prompts, Hotwords, application identities, and history are sensitive local data. Cloud credentials are planned for the Windows credential store, never ordinary SQLite, JSON, logs, fixtures, crash reports, exports, or plaintext backups. Logs use sanitized stages and codes rather than complete sensitive content. SQLite transcript/history storage is not promised to be encrypted at rest; the product will disclose reliance on per-user filesystem protection. Failed sessions are designed to preserve recorded material through recovery records, subject to the documented retention and deletion rules.

## Contributing and provenance

Implementation must be original. Do not copy, rewrite, port, translate, or derive source from SayIt or another GPL/AGPL project. New dependencies, native components, assets, provider SDKs, inference frameworks, and model files require the fail-closed review in [`docs/licensing.md`](docs/licensing.md) and corresponding notice updates. Contributions should include the relevant tests and documentation for success, cancellation, timeout, retry, late-response, fallback, recovery, privacy, and redaction behavior.

## Authoritative documentation

- [`CONTEXT.md`](CONTEXT.md) — canonical domain language.
- [`docs/product.md`](docs/product.md) — accepted product behavior and scope.
- [`docs/architecture.md`](docs/architecture.md) — boundaries and dependency direction.
- [`docs/state-machine.md`](docs/state-machine.md) — lifecycle, outcomes, and failure semantics.
- [`docs/testing.md`](docs/testing.md) — future verification obligations.
- [`docs/licensing.md`](docs/licensing.md) — dependency and model acceptance policy.
- [`docs/implementation-plan.md`](docs/implementation-plan.md) — sole delivery authority for milestones and acceptance.
- [`docs/roadmap.md`](docs/roadmap.md) — concise milestone index that defers to the master plan.
- [`docs/adr/`](docs/adr/) — accepted hard-to-reverse decisions.

## License

Voxora's project license is GNU GPL version 3 only, expressed as SPDX `GPL-3.0-only`. The complete unmodified license text is in [`LICENSE`](LICENSE). This project license does not approve any future dependency, native component, provider, framework, asset, or model; each remains subject to independent review.
