# Voxora Product Specification

## Status and audience

Voxora is a planned Windows-first desktop voice-input application for ordinary Windows users and open-source enthusiasts. This document records the accepted first-release product contract. M3 implements the portable Dictation Session lifecycle, ports, deterministic fakes, and tests, but the Tauri/React desktop remains the M2 not-yet-implemented shell. There is still no real provider integration, platform adapter, persistence adapter, credential handling, model, UI workflow, or usable voice-input behavior.

The user starts a Dictation Session, speaks, and receives Final Text for insertion into an eligible application outside Voxora. If insertion cannot be performed safely, the product preserves the text for manual delivery. Settings are visible without a standard/expert split.

## Recognition

- A Recognition Provider is an implementation type, such as the planned Doubao cloud service or a local model family. A Recognition Configuration is the complete user-selectable ASR option; there is no separate Recognition Profile layer, and multiple configurations may use one provider.
- The active Recognition Configuration is selected in settings, not by a shortcut. The first cloud provider planned is Doubao streaming ASR. The first local engine planned is sherpa-onnx with one separately reviewed SenseVoice Small artifact; framework selection never approves a model.
- When a user selects and configures a cloud Recognition Configuration, cloud ASR sends Recorded Audio and the stable supported allowed Hotword subset selected for that request directly to that provider. There is no project-operated proxy and no automatic privacy-changing fallback to another recognition configuration.
- Cloud Partial Transcript events may exist, but they are not shown during ordinary recording. If final recognition fails, the last available partial may be stored as explicitly incomplete text.
- A retry from history selects another available Recognition Configuration and creates another Recognition Attempt in the same Dictation Record. It does not overwrite the original failed attempt or create a new recording session.

## Processing

- All settings are visible. Users cannot author regex, scripts, plugins, or executable processing code; local processing consists only of Voxora-maintained built-in rules.
- The initial built-in rules are `Remove Trailing Sentence Punctuation`, which removes sentence-ending periods, question marks, exclamation marks, or ellipses while preserving closing quotation marks, and `Replace Conversational Punctuation With Spaces`, which normalizes common sentence, pause, question, and exclamation punctuation without changing punctuation embedded in decimals, domains, abbreviations, or technical tokens. Both begin disabled to preserve transcription fidelity.
- One global processing order contains the built-in rules and at most one optional LLM step. Built-in rules may appear before or after that step. Application Profiles may inherit, force-enable, or force-disable each built-in rule, but cannot change order or the LLM configuration.
- When LLM processing is disabled or unavailable, its step is skipped and enabled local rules still run in global order. A processing-step failure aborts the transformed result and falls back to the separately retained Raw Transcript.

## LLM and Prompts

- LLM processing is global and default-off. Users may save multiple named Language Model Configuration entries, but at most one is globally active. That active configuration supplies a validated persisted Base URL, opaque credential reference, model, parameters, timeout, and reasoning-mode preference. Without an active configuration, LLM processing is unavailable; Application Profiles cannot choose or disable the provider.
- A persisted Base URL must parse as an absolute URL and may contain only scheme, host, optional port, and path. URL userinfo, including a username or password, plus query and fragment components are rejected before saving and before any request. HTTPS is required for non-loopback endpoints; HTTP is allowed only for loopback endpoints; TLS verification cannot be disabled. Credentials are resolved only from opaque CredentialStore references. Future non-secret provider query parameters use separate validated adapter settings and are never embedded in Base URL. Validation errors and logs use sanitized field/error meaning and never echo credential-bearing URLs.
- First-release requests are stateless and non-streaming. An enabled LLM sends only the current pipeline text, Effective Prompt, and the stable supported allowed Hotword subset selected for that request to the active configured endpoint. Reasoning mode is `provider default`, `disabled`, or `enabled`; adapters map only supported fields, and generic endpoints receive no guessed fields. There is no project-operated proxy or automatic privacy-changing fallback.
- Voxora always has an Active Prompt Preset. Built-in Prompt Presets are immutable and non-deletable but copyable. The planned initial built-ins are original-text cleanup (the default), concise expression, and formal expression.
- A Custom Prompt Preset has a name, content, and optional global shortcut. A Prompt shortcut permanently changes the global Active Prompt Preset. An Application Profile Prompt selection overrides the global Prompt for that application.
- Copying any Prompt creates an editable custom preset named `Original name Copy`, then `Copy 2`, and so on. Content is copied; shortcuts and references are not. The copy does not become active automatically, and the UI opens it directly in its edit view.
- Deleting a Custom Prompt Preset referenced by an Application Profile warns the user and requires explicit confirmation. If deletion proceeds, every affected profile stops selecting that preset and follows the global Active Prompt Preset.
- The Effective Prompt is built at request time using a Voxora-owned immutable wrapper that appends only the stable supported allowed Hotword subset selected for that request as inert reference data. The stored Prompt Preset is never modified.

## Hotwords

- Voxora has one global Hotword Library containing named groups that are globally enabled or disabled. Each Hotword is only its text; it has no weight, pronunciation, alias, provider field, or application-specific selection.
- Enabled Hotwords are offered to recognition providers that support them and to an enabled LLM request. Provider or token limits must not cause silent omission: Voxora selects a stable supported allowed Hotword subset for each request and displays `used N of M`; history stores counts rather than complete Hotword content.
- Hotword Candidate analysis is a post-first-release, local-only, default-off feature. It never auto-adds terms and is not part of first-release acceptance.

## Recording and UI

- Push-to-Talk and Toggle bindings may coexist. Toggle defaults to `Ctrl+Shift+Space`; Push-to-Talk is unbound until configured. Both modes use one configurable one-to-five-minute maximum, default five minutes. Reaching the limit stops capture and continues recognition.
- Only one Dictation Session may be active. The mode that started it owns its stop gesture; another recording-start gesture cannot create or take over a second session. Modifier-only bindings have explicit conflict handling and never suppress arbitrary OS input.
- Esc during capture intentionally cancels, deletes the recorded audio, and creates no history. Esc after capture stops remaining safely cancellable work while preserving Recorded Audio and available results in history.
- The Recording Overlay is non-focusable and never an insertion target. During capture it shows elapsed seconds, input amplitude, a low-volume warning, and time-limit warnings. After capture it shows only a generic processing state; failure details are not shown there. Low volume warns but never pauses or ends recording. The UI warns at 30 seconds remaining and shows a final ten-second countdown.
- Failure UI is generic and directs the user to history for sanitized stage and reason details. Start at login is configurable and default-off.

## Targeting and insertion

- At capture end, Voxora resolves the currently focused eligible Insertion Target. It does not fall back to a previously valid target when focus is now an unrelated non-input control. The capture-end target also determines Application Profile matching.
- Recognition and processing never steal focus. Automatic insertion occurs only if the captured target remains valid and focused. Voxora's own windows and overlays are always excluded.
- The first Windows injector is planned to use clipboard paste with a SendInput fallback. Clipboard preservation is best effort for safe common formats and uses sequence checks so a clipboard changed by the user is not overwritten. Voxora does not elevate itself or install a privileged helper; elevated targets fall back safely.
- If insertion is unavailable or unsafe, a non-focus-stealing Result Panel presents Final Text and a Copy action. If the panel cannot appear, Final Text is written to the clipboard and the user is notified.

## History and storage

- A Dictation Record relates Recorded Audio, recognition attempts, Raw Transcript, Processed Text, Final Text, status, and sanitized failure information. Text history and audio history are independently configurable, both default-on with a default 30-day retention period; retention is user-adjustable.
- Failed sessions create recovery records even when ordinary history is disabled, so recorded material is not silently discarded. Users can play stored audio, delete it, and retry recognition with another configuration. Direct audio export is post-first-release. Users can delete one record or all records.
- Orphaned temporary audio after a crash is deleted on the next normal startup; crash recovery is not a product guarantee. SQLite transcript/history content is not promised to be encrypted at rest; the product relies on per-user filesystem protection and exposes deletion/retention controls.

## Application Profiles

- Application Profiles are default-off and match executable identity only. Classic applications use a canonical executable path stored locally and redacted from logs. Packaged applications use Package Family Name/AUMID-compatible identity.
- Window titles are never collected, persisted, logged, or uploaded. A matched profile may override built-in-rule enablement and select a Prompt Preset. With no match, global rules and the global Active Prompt Preset apply.

## Privacy, non-goals, and release boundaries

Audio, transcripts, Prompt content, Hotwords, application identities, and history are sensitive local data. When the user selects a configured cloud Recognition Configuration, Recorded Audio and the stable supported allowed Hotword subset selected for that request go directly to that provider. When LLM processing is enabled, only current pipeline text, the Effective Prompt, and the stable supported allowed Hotword subset selected for that request go directly to the active endpoint. Cloud credentials are intended for the platform credential store and never ordinary SQLite, JSON, logs, fixtures, crash reports, exports, or plaintext backups. Voxora operates no project proxy and performs no automatic privacy-changing fallback. Logs contain sanitized stages and codes, not complete Prompts, transcripts, audio, provider responses, credential-bearing URLs, or complete private paths. No project server, account system, cloud sync, telemetry, device identifier, usage-statistic upload, team management, or application auto-update is planned.

The planned first-release scope is Windows-first and follows the M0–M9 milestones. Post-first-release candidates are local Hotword Candidate analysis, a second local ASR model, direct audio export, macOS/Linux adapters and packaging, additional cloud ASR providers, and additional safe built-in rules. See [`docs/implementation-plan.md`](implementation-plan.md) for the sole delivery authority and [`docs/state-machine.md`](state-machine.md) for deterministic failure semantics.
