# Testing Strategy

## Status and test philosophy

M3 implements the portable lifecycle baseline with deterministic fakes before native capture, provider calls, model downloads, persistence adapters, or real insertion are introduced. The final local suite contains 59 portable Rust tests across application workflow, core acceptance, core unit, and ports unit coverage. Later adapter, frontend, persistence, native, provider, and end-to-end obligations below remain future work. No fixture may contain a real credential, endpoint, Prompt, transcript, Hotword, account identifier, audio sample, or private path.

## Portable state and domain tests

Use deterministic transition tables and, where useful, property tests for invalid event ordering and stale identifiers. The tables must cover:

- Push-to-Talk and Toggle start/stop, mode-owned stop gestures guarded by starting mode plus Session ID, competing shortcuts, one-active-session enforcement, maximum-duration stop, capture failure, and empty audio;
- Esc during capture versus Esc after capture before and after irreversible delivery, including deletion/no-history, preservation/recovery, DeliveryUncertain, and no-rollback/no-automatic-retry differences;
- Partial Transcript acceptance, hidden normal UI behavior, final recognition, empty final, timeout, cancellation, provider failure, retry attempts, and late/stale response rejection;
- transactional local-rule/LLM processing, unavailable or disabled LLM skip, Raw Transcript fallback, and `DeliveredAutomatically` with a processing-fallback warning when fallback text is inserted;
- target/profile resolution at capture end, target invalidation, focus changes without reactivation, insertion success, definite failure, delivery uncertainty, Result Panel, and clipboard-last-resort paths;
- persistence failure, recoverable-material flags, retention/deletion decisions, and record-scoped retry as another Recognition Attempt in the same Dictation Record.

Use a fake clock to drive deadlines and cancellation without sleeping. Validate serialization compatibility for portable identifiers, phases, the exact terminal outcomes, orthogonal warning/failure metadata, recoverable-material availability, durability flags, and sanitized errors.

## Exact outcome and durability obligations

Every state-transition table must assert one exact terminal outcome and independently assert warning/failure metadata plus material availability/durability:

| Scenario | Exact outcome | Required material and metadata |
| --- | --- | --- |
| Capture start/stop/end failure | `Failed` | Partial audio recovery is best effort: missing audio is valid, while any nonempty `RecordedAudio` supplied by the adapter remains available. |
| Empty audio | `Failed` | No provider call and no recoverable zero-length audio artifact; sanitized empty-audio metadata. |
| Esc during capture | `Cancelled` | Audio is deleted and no history or Recovery Artifact is created. |
| Esc after capture before irreversible delivery | `Cancelled` | Recorded Audio and available results remain available, durable only after persistence succeeds. |
| Esc after irreversible delivery begins | Preserve `DeliveredAutomatically` when delivery is confirmed, or `DeliveryUncertain` when it is not; Esc after terminal delivery is stale. | No rollback or automatic retry; preserve Final Text through the applicable delivery/recovery path. |
| Recognition empty, timeout, or provider failure after successful capture | `Failed` | Recorded Audio, any explicitly incomplete partial, and sanitized warning/failure metadata remain recoverable. |
| Recognition cancellation without higher-level user cancellation | `Failed` | The attempt stops without replacing prior attempts; stale responses cannot mutate the record. |
| Processing fallback followed by confirmed insertion | `DeliveredAutomatically` | Raw Transcript remains separately retained, Recorded Audio remains available, and a processing-fallback warning is present. |
| Confirmed insertion | `DeliveredAutomatically` | Final Text delivery is confirmed and Recorded Audio remains available after successful capture. |
| Definite insertion failure | `ManualDeliveryRequired` | Result Panel then clipboard-last-resort preserves Final Text and Recorded Audio remains available after successful capture. |
| Insertion uncertainty | `DeliveryUncertain` | No automatic retry; Final Text and Recorded Audio remain available after successful capture to prevent loss or duplicate delivery. |
| Persistence failure | Preserve the existing `DeliveredAutomatically`, `ManualDeliveryRequired`, `DeliveryUncertain`, `Cancelled`, or `Failed` outcome | Existing Recovery Artifacts and in-memory text are not erased; all material retained after successful capture is non-durable until `PersistenceSucceeded`; show a generic unsaved-history warning and use Result Panel/clipboard-last-resort if Final Text is not confirmed delivered. Do not claim non-durable audio survives exit or crash. |

The matrix must also prove that a persistence warning never becomes a sixth terminal outcome and that late responses after cancellation, timeout, retry, or terminal completion cannot alter the selected outcome or durability flags.

## Mode guards, irreversible delivery, and history retry

The transition suite must include these deterministic cases:

- `StartPushToTalk` and `StartToggle` bind the active Session ID and starting mode; only a matching `ReleasePushToTalk` or `StopToggle` can stop that session. Cross-mode releases/stops, stale Session IDs, duplicate stops, and post-capture stops are rejected without mutation.
- Esc before delivery becomes irreversible yields `Cancelled` and preserves applicable materials. Once clipboard paste or SendInput may be irreversible, Esc cannot change the result to `Cancelled`: confirmed delivery remains `DeliveredAutomatically`, unconfirmed delivery remains `DeliveryUncertain`, and neither rollback nor automatic retry occurs. Esc after terminal delivery is stale.
- `RetryRecognition` is accepted only for a durable Dictation Record with usable Recorded Audio, no live Dictation Session, and no other active retry. It retains the originating Session ID, creates a fresh Recognition Attempt ID, increments the attempt revision, and enters the record-scoped retry phase without creating a new session.
- Retry responses are accepted only when Dictation Record ID, originating Session ID, fresh Attempt ID, revision, and expected retry phase match. Success appends a successful attempt-scoped Raw Transcript for manual use; empty, timeout, cancellation, and provider failure mark only that attempt failed. Prior attempts, the original terminal session, and its results remain immutable.
- Retry never runs the Processing Pipeline, calls an LLM, resolves or reuses an Insertion Target, shows an insertion Result Panel, or injects text. Responses from an earlier attempt, closed retry, or mismatched identifiers are stale and cannot mutate the record.

## Fake ports and contract coverage

The portable application tests should use fakes for each applicable port:

| Fake | Obligations |
| --- | --- |
| `FakeAudioCapture` | Scripted start/stop success or failure, amplitude warning, maximum duration, empty audio, and cancellation. Injected core capture completion/failure events separately cover both adapter-supplied `Some(audio)` and valid `None`; the fake port does not return partial audio from start/stop. |
| `FakeRecognitionEngine` | Partial/final events, empty result, provider failure, timeout, cancellation, retry, and late response, with exact `Failed`/`Cancelled` assignment. |
| `FakeTextProcessor` | Ordered built-in rules and their punctuation-preservation semantics, optional LLM skip, processing failure, Raw Transcript fallback, and `DeliveredAutomatically` warning behavior after confirmed insertion. |
| `FakeTextInjector` | Success, definite failure, focus/target invalidation, irreversible start, and delivery uncertainty. |
| Fake target resolver | Current-focus capture, profile identity, ineligible target, disappearing target, and no reactivation. |
| Fake history store | Successful persistence, failed-session recovery, retention/deletion, audio references, durability transitions, generic unsaved-history warning, and persistence failure without erasing Recovery Artifacts or in-memory text. |
| Fake credential store | Opaque references only; prove secret values never enter SQLite/JSON/logging. |
| Fake model manager | Manifest gating, hash mismatch, cancellation, partial/corrupt artifact, deletion, and no automatic update. |
| Fake shortcut registry | PTT/Toggle coexistence, modifier-only conflict handling, and competing events. |
| Fake clock | Deterministic deadlines, timeout, and warning countdown. |

Every adapter must also run a shared conformance suite for success, cancellation, timeout, malformed response, retryability, and redaction. Provider fixtures are synthetic and do not make paid calls.

Provider payload tests must prove the trust boundary independently for each path:

- a configured cloud ASR Recognition Configuration sends only Recorded Audio and the stable supported allowed Hotword subset selected for that request directly to that provider, never through a project proxy and never with an automatic privacy-changing fallback;
- an enabled LLM request sends only current pipeline text, the Effective Prompt, and the stable supported allowed Hotword subset selected for that request to the one globally active Language Model Configuration endpoint, never audio and never through an automatic privacy-changing fallback.
- payload assertions require the exact selected subset, `used N of M` reporting, an Effective Prompt wrapper containing only that subset, and history storing counts rather than Hotword content.

## Persistence, privacy, and redaction tests

Future persistence tests must verify independent text/audio retention, record and artifact deletion, orphan temporary-audio cleanup on normal startup, failed-session recovery even when ordinary history is disabled, and independent Raw/Processed/Final text storage. Credential serialization tests must assert that secrets and credential-bearing Base URLs do not appear in SQLite, JSON, logs, exports, backups, or crash reports, and that credentials remain opaque CredentialStore references.

Base URL validation tests must reject relative/non-absolute URLs plus userinfo, username, password, query, and fragment components before persistence and before any request; require HTTPS for non-loopback endpoints; permit HTTP only for loopback endpoints; and reject any attempt to disable TLS verification. Future adapter-setting tests must keep non-secret provider query parameters separate from Base URL and validate them independently. Invalid or credential-bearing URL input must not be echoed in validation errors, logs, or history.

Log-redaction tests must reject complete Prompts, transcripts, provider response bodies, audio, Hotword content, credential-bearing URLs, account identifiers, and complete private filesystem paths. Structured failures must contain only sanitized stage/code, retry meaning, delivery certainty, and recoverable-material indicators.

## Frontend tests

React tests cover reducers/view models for overlay phases, settings precedence, multiple named Language Model Configurations with at most one active, Prompt copy naming and shortcut conflict behavior, warning plus explicit confirmation before deleting a Custom Prompt Preset referenced by an Application Profile, reset of affected profiles to the global Active Prompt Preset after confirmed deletion, Application Profile overrides and global fallback, history deletion/recovery actions, and generic failure rendering. Interaction and rendered visual verification cover settings, Recording Overlay, Result Panel, history, empty states, and failures. Tests must prove that React submits commands and renders state without owning session orchestration, that Partial Transcript is not shown during ordinary recording, and that Voxora windows cannot become insertion targets.

## Windows adapter and manual tests

Windows-specific tests and manual scenarios cover common editors, browsers, chat applications, classic Win32 and packaged applications, executable identity matching, focus changes during recording/processing, elevated-target fallback, microphone disappearance, low-volume warning, modifier-only shortcuts and conflicts, clipboard changes during injection, sequence-check races, and unsigned installer/SmartScreen documentation. Native tests stay behind platform ports; portable tests must not require a Windows target.

Model-manager tests cover user-initiated download cancellation/resume as supported, disk exhaustion, corrupt/partial/wrong-version/hash-mismatched artifacts, exact reviewed manifests, deletion, and the absence of automatic update/background checks. No test downloads large model weights.

## CI intent (M2 and later)

The existing CI implements Windows/macOS/Linux formatting plus common-crate compile/lint/tests; a Windows Tauri desktop build; frontend formatting, lint, Vitest, and production build; fail-closed Cargo/npm license and source checks; model-manifest structural/negative tests; and tracked-file secret-pattern checks. M3 now supplies the portable lifecycle suite those common-crate jobs execute. CI makes no paid provider call and downloads no model weight.

## Test evidence and review

Each milestone should report commands, fixture provenance, scenarios covered, and any environment-dependent checks that remain unverified. A passing build alone is insufficient: acceptance requires state, cancellation, timeout, retry, late-response, fallback, recovery, privacy, redaction, dependency, model, and licensing evidence.
