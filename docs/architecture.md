# Voxora Architecture

## Status and intent

This is the planned architecture for a documentation-stage project. It describes boundaries and dependency direction without creating crates, adapters, manifests, or runtime behavior. The first release is Windows-first, but portable business logic is designed independently of Windows and provider APIs.

## Layer responsibilities

```text
React UI
    ↓ commands and state events
Tauri desktop composition root
    ↓
voice-application
    ↓
voice-ports  ← adapter implementations
    ↓             ├─ history-sqlite
voice-core        ├─ provider-doubao
                   ├─ provider-openai-compatible
                   ├─ local-asr-sherpa
                   └─ platform-windows
```

- `voice-core` owns domain values, identifiers, deterministic transitions, and provider/platform-independent error meaning. It must contain no `cfg(windows)` and must not depend on Tauri, React, Windows APIs, UI Automation types, or provider SDKs.
- `voice-ports` owns capability contracts and portable request/result types and depends only on `voice-core`. Planned ports cover audio capture, shortcuts, recognition, processing, insertion, target resolution, credentials, history, model management, and clock behavior.
- `voice-application` owns bounded use cases and session-scoped coordination. It depends only on core and ports; it is not a catch-all orchestrator.
- Adapters implement ports and depend inward on ports/core. Inward layers never depend on an adapter. The Tauri crate is the only production composition root and selects the adapters used by a desktop build.
- React receives mapped state and submits commands at the Tauri boundary. It never imports Rust/provider concepts directly and never orchestrates a Dictation Session.

## Coordination and state ownership

Every command, event, recognition attempt, cancellation, timeout, and late response carries a stable Session ID or Recognition Attempt ID. Mutable session state is owned by the active use-case instance or an explicit session record, never by a global mutable singleton. Capture, recognition, processing, targeting, history, recovery, and insertion remain separable responsibilities. The complete event and race contract is in [`state-machine.md`](state-machine.md).

## Platform and provider isolation

Windows-only microphone, global shortcuts, target resolution, clipboard/SendInput injection, credential-store access, packaged-application identity, and native integration belong in `platform-windows`. Cloud and local recognition, LLM processing, and protocol parsing belong in provider adapters behind ports. Provider response formats and Windows types must not cross into portable core types; adapters translate them into structured, sanitized meanings.

## Trust boundaries

| Boundary | Data and control | Required rule |
| --- | --- | --- |
| User and local capture | Microphone samples and recording controls | Audio is sensitive; low-volume analysis remains local and warning-only. |
| Local history/storage | Audio artifacts, transcripts, Prompts, Hotwords, application identities | SQLite metadata/text and audio artifacts have separate retention/deletion controls. Encryption at rest is not promised; rely on per-user filesystem protection. |
| Credential store | Opaque credential references and provider secrets | Secrets use the platform credential store and never ordinary SQLite, JSON, logs, fixtures, crash reports, exports, or plaintext backups. |
| Cloud ASR provider | Recorded Audio and the supported allowed Hotword subset | Sent directly only when the user selects and configures a cloud Recognition Configuration. There is no project proxy or automatic privacy-changing fallback. |
| LLM endpoint | Current pipeline text, Effective Prompt, and the allowed Hotword subset | Sent directly only when an Active Language Model Configuration exists and processing is enabled. No project proxy or automatic privacy-changing fallback; logs omit complete sensitive content and credential-bearing URLs. |
| Model acquisition | User-initiated model files and metadata | Every exact artifact is independently reviewed, hash-verified, and kept outside the application package. No automatic update or background check. |
| Clipboard/insertion target | Final Text and external application focus | Preserve user clipboard data best effort with sequence checks; do not steal focus or self-elevate. |
| Diagnostics | Stage/code, retry meaning, delivery certainty, recoverable-material indicators | Never include complete Prompt, transcript, audio, provider response, Hotword list, private path, or credential. |

External applications are untrusted insertion targets. Voxora captures the target at recording stop, never reactivates it during processing, and routes Final Text to the Result Panel or clipboard when safe delivery cannot be confirmed.

## Recording-to-insertion flow

1. A configured shortcut starts one Dictation Session and binds its stop gesture to the starting mode.
2. Audio capture emits local level information and stops on the user's gesture, Esc, capture failure, or the configured maximum.
3. At capture end, the current focused eligible target and executable identity are resolved once for insertion and Application Profile matching.
4. Recognition runs using the selected Recognition Configuration. Partial results are internal; a final result or an explicitly incomplete available partial is retained with the attempt.
5. The processing pipeline works transactionally on a copy of Raw Transcript. Enabled local rules and, when configured, one LLM step produce Processed Text; any step failure discards the transformed copy and falls back to Raw Transcript.
6. Final Text is selected, then inserted only if the captured target remains valid and focused. Success, definite failure, or delivery uncertainty is recorded without silently dropping text.
7. History persists the Dictation Record, attempts, material-availability/durability flags, and sanitized outcome. If persistence fails, existing Recovery Artifacts and in-memory text remain non-durable, the user receives a generic unsaved-history warning, and Final Text follows the Result Panel/clipboard-last-resort path unless already confirmed delivered. Recovery material remains available according to retention and deletion settings only after it is durable.

## Configuration precedence

- Global Default Processing Rules define one ordered list of built-in rules. An Application Profile may inherit, force-enable, or force-disable each rule; it cannot change order or the global Language Model Configuration.
- The global Active Prompt Preset is always present. A matched Application Profile may select a Prompt Preset for that application; otherwise the global Active Prompt Preset applies. A Prompt shortcut changes the global selection persistently.
- Users may save multiple named Language Model Configuration entries, but at most one global Active Language Model Configuration supplies LLM endpoint/settings. Profiles cannot choose or disable the LLM provider. If no active configuration exists, the LLM step is unavailable and is skipped, not failed.
- One global Hotword Library supplies enabled groups. Provider/token limits select a stable allowed subset and report counts; profiles do not own Hotwords.

## Persistence boundaries

SQLite is planned for settings, Prompt Presets, Hotwords, Application Profiles, Dictation Record metadata, recognition attempts, transcripts, outcomes, retention, durability, and sanitized failure details. Recorded Audio is stored as separate artifacts, not SQLite blobs. Credential values remain in the platform credential store and are referenced only opaquely. Future model files are separate reviewed artifacts with source, version, size, SHA-256, license, and provenance metadata. Deletion and retention services must remove linked artifacts consistently. A persistence failure never erases existing Recovery Artifacts or in-memory text; available material remains non-durable until persistence succeeds, and non-durable audio is not claimed to survive exit or crash.

## Operation without a project server

Voxora is a local desktop application. It does not require a project-operated server, account, cloud sync, team service, telemetry endpoint, or server self-hosting component. Cloud providers are optional user-configured destinations reached directly by the relevant adapter; there is no project proxy or automatic privacy-changing fallback. Application and model updates are manual/user-initiated; there is no application auto-update or background model check.

## Expected future repository responsibilities

The master plan records the expected crate tree. `voice-core`, `voice-ports`, and `voice-application` are the portable foundation; `history-sqlite` owns persistence; provider adapters own Doubao, OpenAI-compatible processing, and sherpa-onnx integration; `platform-windows` owns native capture, shortcuts, targeting, credentials, and insertion; the Tauri crate composes the desktop. M1 intentionally creates none of these directories or manifests.

See ADRs [0001](adr/0001-windows-first-portable-core.md), [0002](adr/0002-tauri-rust-react-desktop-stack.md), and [0005](adr/0005-ports-and-adapters.md) for the accepted architectural choices.
