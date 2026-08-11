# Testing Strategy

## Status and test philosophy

M1 defines future verification obligations; it does not add a test harness or claim that tests currently pass. Tests must prove the documented contract with deterministic fakes before native capture, provider calls, model downloads, or real insertion are introduced. No fixture may contain a real credential, endpoint, Prompt, transcript, Hotword, account identifier, audio sample, or private path.

## Portable state and domain tests

Use deterministic transition tables and, where useful, property tests for invalid event ordering and stale identifiers. The tables must cover:

- Push-to-Talk and Toggle start/stop, mode-owned stop gestures, competing shortcuts, one-active-session enforcement, maximum-duration stop, capture failure, and empty audio;
- Esc during capture versus Esc after capture, including deletion/no-history and preservation/recovery differences;
- Partial Transcript acceptance, hidden normal UI behavior, final recognition, empty final, timeout, cancellation, provider failure, retry attempts, and late/stale response rejection;
- transactional local-rule/LLM processing, unavailable or disabled LLM skip, Raw Transcript fallback, and `DeliveredAutomatically` with a processing-fallback warning when fallback text is inserted;
- target/profile resolution at capture end, target invalidation, focus changes without reactivation, insertion success, definite failure, delivery uncertainty, Result Panel, and clipboard-last-resort paths;
- persistence failure, recoverable-material flags, retention/deletion decisions, and retry as another Recognition Attempt in the same Dictation Record.

Use a fake clock to drive deadlines and cancellation without sleeping. Validate serialization compatibility for portable identifiers, phases, the exact terminal outcomes, orthogonal warning/failure metadata, recoverable-material availability, durability flags, and sanitized errors.

## Exact outcome and durability obligations

Every state-transition table must assert one exact terminal outcome and independently assert warning/failure metadata plus material availability/durability:

| Scenario | Exact outcome | Required material and metadata |
| --- | --- | --- |
| Capture failure | `Failed` | Any actual Recorded Audio remains available; durability depends on persistence. |
| Empty audio | `Failed` | No provider call and no recoverable zero-length audio artifact; sanitized empty-audio metadata. |
| Esc during capture | `Cancelled` | Audio is deleted and no history or Recovery Artifact is created. |
| Esc after capture | `Cancelled` | Recorded Audio and available results remain available, durable only after persistence succeeds. |
| Recognition empty, timeout, or provider failure | `Failed` | Audio and any explicitly incomplete partial remain recoverable; warning/failure metadata is sanitized. |
| Recognition cancellation without higher-level user cancellation | `Failed` | The attempt stops without replacing prior attempts; stale responses cannot mutate the record. |
| Processing fallback followed by confirmed insertion | `DeliveredAutomatically` | Raw Transcript remains separately retained and a processing-fallback warning is present. |
| Confirmed insertion | `DeliveredAutomatically` | Final Text delivery is confirmed. |
| Definite insertion failure | `ManualDeliveryRequired` | Result Panel then clipboard-last-resort preserves Final Text. |
| Insertion uncertainty | `DeliveryUncertain` | No automatic retry; Final Text remains available to prevent duplicate delivery. |
| Persistence failure | Preserve the existing `DeliveredAutomatically`, `ManualDeliveryRequired`, `DeliveryUncertain`, `Cancelled`, or `Failed` outcome | Existing Recovery Artifacts and in-memory text are not erased; all available material is non-durable until `PersistenceSucceeded`; show a generic unsaved-history warning and use Result Panel/clipboard-last-resort if Final Text is not confirmed delivered. Do not claim non-durable audio survives exit or crash. |

The matrix must also prove that a persistence warning never becomes a sixth terminal outcome and that late responses after cancellation, timeout, retry, or terminal completion cannot alter the selected outcome or durability flags.

## Fake ports and contract coverage

The portable application tests should use fakes for each applicable port:

| Fake | Obligations |
| --- | --- |
| `FakeAudioCapture` | Start/stop, amplitude warning, maximum duration, device failure, empty audio, cancellation. |
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

- a configured cloud ASR Recognition Configuration sends only Recorded Audio and the supported allowed Hotword subset directly to that provider, never through a project proxy and never with an automatic privacy-changing fallback;
- an enabled LLM request sends only current pipeline text, the Effective Prompt, and the allowed Hotword subset to the one globally active Language Model Configuration endpoint, never audio and never through an automatic privacy-changing fallback.

## Persistence, privacy, and redaction tests

Future persistence tests must verify independent text/audio retention, record and artifact deletion, orphan temporary-audio cleanup on normal startup, failed-session recovery even when ordinary history is disabled, and independent Raw/Processed/Final text storage. Credential serialization tests must assert that secrets do not appear in SQLite, JSON, logs, exports, backups, or crash reports.

Log-redaction tests must reject complete Prompts, transcripts, provider response bodies, audio, Hotword content, credential-bearing URLs, account identifiers, and complete private filesystem paths. Structured failures must contain only sanitized stage/code, retry meaning, delivery certainty, and recoverable-material indicators.

## Frontend tests

React tests cover reducers/view models for overlay phases, settings precedence, multiple named Language Model Configurations with at most one active, Prompt copy naming and shortcut conflict behavior, warning plus explicit confirmation before deleting a Custom Prompt Preset referenced by an Application Profile, reset of affected profiles to the global Active Prompt Preset after confirmed deletion, Application Profile overrides and global fallback, history deletion/recovery actions, and generic failure rendering. Interaction and rendered visual verification cover settings, Recording Overlay, Result Panel, history, empty states, and failures. Tests must prove that React submits commands and renders state without owning session orchestration, that Partial Transcript is not shown during ordinary recording, and that Voxora windows cannot become insertion targets.

## Windows adapter and manual tests

Windows-specific tests and manual scenarios cover common editors, browsers, chat applications, classic Win32 and packaged applications, executable identity matching, focus changes during recording/processing, elevated-target fallback, microphone disappearance, low-volume warning, modifier-only shortcuts and conflicts, clipboard changes during injection, sequence-check races, and unsigned installer/SmartScreen documentation. Native tests stay behind platform ports; portable tests must not require a Windows target.

Model-manager tests cover user-initiated download cancellation/resume as supported, disk exhaustion, corrupt/partial/wrong-version/hash-mismatched artifacts, exact reviewed manifests, deletion, and the absence of automatic update/background checks. No test downloads large model weights.

## CI intent (M2 and later)

The master plan intends Windows/macOS/Linux formatting, common-crate compile, lint, and tests; Windows desktop/adapter tests; frontend build/lint/Vitest; dependency/license/model-manifest checks; and tracked-file secret-pattern checks. M1 only records this intent. CI must never make paid provider calls or download large model weights. M2 owns the CI implementation and workspace setup.

## Test evidence and review

Each milestone should report commands, fixture provenance, scenarios covered, and any environment-dependent checks that remain unverified. A passing build alone is insufficient: acceptance requires state, cancellation, timeout, retry, late-response, fallback, recovery, privacy, redaction, dependency, model, and licensing evidence.
