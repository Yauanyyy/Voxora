# Voxora

Voxora is a planned Windows-first desktop voice-input application. A user-initiated Dictation Session is intended to capture speech, recognize it locally or through a user-configured cloud provider, optionally apply local processing and one global language-model step, and insert the resulting Final Text into an eligible external input target. When safe insertion is unavailable, the design preserves the text for manual copying.

## Current status

M4 now implements and locally verifies the portable configuration model, built-in Prompt and processing-rule catalogs, stable Hotword selection, versioned SQLite configuration/history storage, separate audio artifacts, retention/deletion/startup maintenance, safe SQLite backup, and a Windows Credential Manager adapter. M3's portable Dictation Session lifecycle and recovery behavior remain covered. The desktop still displays only the M2 not-yet-implemented shell: real capture, providers, targeting/insertion, settings/history UI, models, and product UX remain unimplemented, so end-user dictation does not yet work.

## Intended first release

The first release is planned for ordinary Windows users and open-source enthusiasts. Its documented scope includes:

- Push-to-Talk and Toggle recording with one active Dictation Session;
- user-selected local or cloud recognition, with Recorded Audio retained after a
  successfully completed capture even when later work fails; capture-boundary
  failures retain only any partial audio actually supplied by the adapter;
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
cargo check --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo clippy --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets --all-features -- -D warnings
cargo test --locked -p voice-core -p voice-ports -p voice-application -p history-sqlite --all-targets
cargo test --locked -p platform-windows --all-targets
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

The desktop build command is Windows-only in the current workspace. M4 adds reviewed `url`, bundled SQLite, and target-scoped Windows credential-store dependencies, but no provider SDK, model, network client, distributed native DLL, or asset. See [`docs/dependency-reviews/m4-local-persistence-configuration.md`](docs/dependency-reviews/m4-local-persistence-configuration.md) and [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## Privacy and recovery posture

Audio, transcripts, Prompts, Hotwords, application identities, and history are sensitive local data. The M4 Windows adapter stores cloud credentials in Windows Credential Manager; SQLite and backups contain only opaque credential references. Logs and adapter errors use sanitized stages and codes rather than complete sensitive content. SQLite transcript/history storage is not encrypted at rest; the product relies on per-user filesystem protection. Recorded Audio is stored outside SQLite blobs and is removed through a durable deletion queue. After capture successfully completes with usable Recorded Audio, later recognition, processing, delivery, and persistence failures preserve that audio through recovery records, subject to retention and deletion rules. Capture start/stop/end failures provide only best-effort partial-audio recovery: missing partial audio is valid, while any nonempty audio supplied by the adapter is retained.

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
