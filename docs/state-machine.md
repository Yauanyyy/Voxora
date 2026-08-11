# Dictation Lifecycle and Failure Semantics

## Purpose and status

This document defines the deterministic lifecycle contract that future portable implementation and tests must satisfy. It is a design baseline, not an implementation claim. A Dictation Session is one user-initiated attempt from capture through insertion, cancellation, or reported failure. A history retry is another Recognition Attempt in the same Dictation Record, not another recording session.

## Four separate dimensions

The state model must not collapse these meanings into one status field:

1. **Lifecycle phase** — what work is currently expected: `Idle`, `Capturing`, `StoppingCapture`, `Recognizing`, `Processing`, `Delivering`, `Completed`, or `Recovery`.
2. **Terminal outcome** — exactly one of `DeliveredAutomatically`, `ManualDeliveryRequired`, `DeliveryUncertain`, `Cancelled`, or `Failed`. The outcome records delivery, cancellation, or failure semantics only; it never encodes a warning or whether material is durable.
3. **Failure/warning metadata** — sanitized `stage`, `code`, retry meaning, delivery certainty, and persistence/material indicators. It never contains provider bodies, credentials, complete Prompts, transcripts, Hotwords, audio, credential-bearing URLs, or complete private paths.
4. **Recoverable material** — each material has independent availability and durability. Recorded Audio, Partial Transcript (explicitly incomplete), Raw Transcript, Processed Text, Final Text, Result Panel presentation, and clipboard fallback are tracked separately. Available material is `durable` only after `PersistenceSucceeded`; otherwise it is `non-durable`, and absent material has no durability claim.

Warnings and failure metadata are orthogonal to the outcome. For example, when processing fails but Raw Transcript is successfully inserted, the terminal outcome is `DeliveredAutomatically` with a processing-fallback warning, not a terminal recognition or delivery failure. When automatic insertion cannot be confirmed but Final Text is preserved, the outcome is `ManualDeliveryRequired` or `DeliveryUncertain`; text is not considered lost. A `Recovery Artifact` is any retained audio or text material available for recovery; its existence alone does not imply durability.

| Terminal outcome | Assignment |
| --- | --- |
| `DeliveredAutomatically` | Final Text insertion is confirmed, including a processing-fallback warning when Raw Transcript was used. |
| `ManualDeliveryRequired` | Automatic insertion is definitely unavailable, but Final Text is preserved through the Result Panel or clipboard-last-resort. |
| `DeliveryUncertain` | An insertion operation may have become irreversible and success cannot be confirmed; preserve Final Text without rollback or automatic retry. |
| `Cancelled` | The user intentionally stops the session or remaining work with the applicable Esc semantics. |
| `Failed` | Capture, empty audio, recognition, processing-without-delivery, or another terminal failure prevents confirmed delivery and is not an intentional user cancellation. A persistence failure alone preserves the already selected outcome and resolves to `Failed` only when no delivery or cancellation decision exists. |

## Identifiers and structured events

Every command and event carries a Session ID. Recognition work additionally carries a Recognition Attempt ID and the expected phase/attempt revision. IDs are opaque, unique for their scope, and never derived from sensitive content. A conceptual event envelope is:

```text
Event {
  session_id,
  attempt_id?,
  expected_phase?,
  kind,
  sanitized_metadata
}
```

Commands include `StartPushToTalk`, `ReleasePushToTalk`, `StartToggle`, `StopToggle`, `Escape`, and `RetryRecognition`. Internal events include `CaptureStarted`, `AudioLevel`, `CaptureStopped`, `CaptureFailed`, `RecognitionPartial`, `RecognitionFinal`, `RecognitionEmpty`, `RecognitionFailed`, `RecognitionTimedOut`, `RecognitionCancelled`, `ProcessingStepStarted`, `ProcessingStepSucceeded`, `ProcessingStepFailed`, `TargetResolved`, `TargetInvalidated`, `FocusChanged`, `InsertionSucceeded`, `InsertionFailed`, `InsertionUncertain`, `PersistenceSucceeded`, and `PersistenceFailed`.

Events that do not match the active Session ID, Recognition Attempt ID, expected phase, or current attempt revision are rejected as stale and have no user-visible effect.

## Lifecycle rules

| Situation | Phase/outcome rule | Material effect |
| --- | --- | --- |
| Push-to-Talk press or Toggle start while `Idle` | Begin `Capturing`; bind the stop gesture to the mode that started the session. | Create a new Session ID; no second session may start. |
| Competing start gesture while active | Ignore/reject as a competing command. | Existing capture and its stop gesture remain unchanged. |
| Push-to-Talk release or Toggle stop | Enter `StoppingCapture`, then `Recognizing` when capture ends. | Preserve Recorded Audio if any. |
| Maximum duration reached | Stop capture automatically and continue recognition. | Record a deadline warning; do not treat it as cancellation. |
| Capture device failure | End with terminal outcome `Failed` and sanitized capture-failure metadata. | Preserve any actual Recorded Audio as available recovery material; its durability depends on `PersistenceSucceeded`. |
| Empty audio | End with terminal outcome `Failed`; do not call recognition with no meaningful samples. | No zero-length artifact is treated as recoverable audio; report sanitized `capture/empty-audio`. |
| Esc during `Capturing` | End with terminal outcome `Cancelled`. | Delete intentionally cancelled audio and create no history or Recovery Artifact. |
| Esc after capture | Stop remaining safely cancellable work and end with terminal outcome `Cancelled`. | Preserve Recorded Audio and all available results; mark them durable only after persistence succeeds. |
| Target resolution at capture end | Resolve the currently focused eligible target once; also resolve Application Profile identity. | Never use a previously valid target if current focus is unrelated or ineligible. |

The Recording Overlay is not focusable or an insertion target. During capture it may show elapsed seconds, local amplitude, low-volume warnings, and 30-second/final-ten-second deadline warnings. After capture it shows only generic processing/failure state; detailed reasons belong in history.

## Recognition and attempts

1. `Recognizing` accepts Partial Transcript events only for the active attempt. Partials are not displayed in ordinary recording UI and may be retained only as explicitly incomplete recovery text if final recognition fails.
2. A matching `RecognitionFinal` stores Raw Transcript separately. An empty final result ends with terminal outcome `Failed` and sanitized recognition-empty metadata; it is not silently successful empty text.
3. A recognition timeout or provider failure ends with terminal outcome `Failed`, preserving Recorded Audio and any available or explicitly incomplete transcript as recovery material. An internal recognition cancellation without a higher-level user cancellation also ends with `Failed`; an explicit Esc after capture remains terminal outcome `Cancelled` while its cancellation token stops recognition.
4. `RetryRecognition` from history appends a new Recognition Attempt to the existing Dictation Record. It never overwrites earlier attempts and does not recreate capture or target resolution.
5. Responses arriving after cancellation, timeout, terminal completion, or a superseding retry are stale. They are rejected by ID/revision checks and cannot replace Raw Transcript, Processed Text, Final Text, or outcome.

## Transactional processing

Processing starts only with a retained Raw Transcript. Each enabled built-in rule and the optional LLM step runs against a working copy in the global order. The optional LLM step is skipped when disabled or when no Active Language Model Configuration exists; skip is not a failure. A failure in any step discards the transformed working copy, retains Raw Transcript separately, and selects Raw Transcript as Final Text. If that text is inserted successfully, the terminal outcome is `DeliveredAutomatically` with a processing-fallback warning. If delivery is definitely unavailable or uncertain, the terminal outcome is respectively `ManualDeliveryRequired` or `DeliveryUncertain`, with the Raw/Final Text material preserved.

## Targeting and delivery

Target validity and focus are checked at delivery time. Voxora never forcibly reactivates a target or steals focus during recognition/processing. Delivery outcomes are:

- `InsertionSucceeded` — the injector reports confirmed insertion into the still-valid captured target; the terminal outcome is `DeliveredAutomatically`.
- `InsertionFailed` — safe insertion is definitely unavailable. Preserve Final Text in a non-focus-stealing Result Panel; if that panel cannot appear, write Final Text to the clipboard and notify the user. This ends as `ManualDeliveryRequired`.
- `InsertionUncertain` — an operation may have become irreversible (for example, clipboard paste or SendInput began but confirmation was lost). Do not cancel-roll back or automatically retry; preserve Final Text and mark delivery uncertain to prevent duplicate text. The Result Panel/clipboard remains the manual recovery path.

Clipboard restoration is best effort for safe common formats and uses sequence checks so Voxora does not overwrite a clipboard changed by the user. Elevated or disappearing targets fall back safely without self-elevation.

## Persistence and recovery

History persistence records the Dictation Record, attempts, Raw/Processed/Final text availability, Recorded Audio reference, terminal outcome, durability flags, and sanitized failure/warning metadata. On `PersistenceFailed`, never erase an existing Recovery Artifact or in-memory transcript/Final Text. Keep every available material marked `non-durable` until a later `PersistenceSucceeded`; do not claim that non-durable audio survives process exit or crash. Immediately show a generic unsaved-history warning because history cannot be assumed writable. If Final Text is not already confirmed delivered, present it through the Result Panel and then use clipboard-last-resort if the panel cannot appear. The existing delivery/cancel/failure decision remains the terminal outcome (`DeliveredAutomatically`, `ManualDeliveryRequired`, `DeliveryUncertain`, `Cancelled`, or `Failed`) with a persistence warning; persistence failure never creates a sixth outcome. Failed sessions create recovery records when persistence succeeds even when ordinary text/audio history is disabled, while a failed persistence attempt leaves only the explicitly retained non-durable material. Retention and deletion may later remove durable records according to user policy.

When `PersistenceSucceeded` arrives, the retained material is marked durable according to the persisted record and artifact results. A persistence failure after confirmed automatic insertion preserves `DeliveredAutomatically` plus a persistence warning; a failure before confirmed delivery still follows the Result Panel/clipboard path and retains the applicable manual, uncertain, cancelled, or failed outcome.

Orphaned temporary audio after a crash is deleted on the next normal startup; crash recovery is not a product guarantee.

## Deterministic race policy

Cancellation, timeout, terminal completion, and retry advance the expected phase/attempt revision. Every asynchronous event must match the active IDs and expected phase before it can mutate state. A late partial, final, processing result, or insertion callback is therefore a no-op with sanitized diagnostic evidence only. Once delivery may be irreversible, cancellation never triggers rollback or an automatic duplicate attempt.

## Required scenario table

Future tests must cover at least:

| Scenario | Required observable result |
| --- | --- |
| PTT press/release | One capture, mode-owned stop, then recognition. |
| Toggle start/stop | One capture; another start cannot take it over. |
| Maximum duration | Capture stops and recognition continues with a warning. |
| Capture failure | Terminal outcome `Failed`; any actual audio remains available with durability determined by persistence. |
| Empty audio | Terminal outcome `Failed`; no provider call and no recoverable zero-length audio artifact. |
| Esc during capture | Terminal outcome `Cancelled`; audio is intentionally deleted and no history or Recovery Artifact is created. |
| Esc after capture | Terminal outcome `Cancelled`; remaining work stops safely and audio/results remain available, durable only after persistence succeeds. |
| Partial then final | Partial is hidden in normal UI; final Raw Transcript is retained. |
| Partial then recognition failure | Last partial is explicitly incomplete recovery text. |
| Recognition empty/timeout/provider failure | Terminal outcome `Failed`; matching stale response cannot mutate the record and available audio/partial material remains recoverable. |
| Recognition cancellation without higher-level user cancellation | Terminal outcome `Failed`; the attempt stops without replacing earlier attempts. |
| Late response | Matching stale response cannot mutate the record. |
| Processing failure with confirmed insertion | Raw Transcript remains separate; terminal outcome is `DeliveredAutomatically` with a processing-fallback warning. |
| LLM unavailable/disabled | LLM step is skipped; enabled local rules still run. |
| Insertion success | Terminal outcome `DeliveredAutomatically`; confirmed Final Text insertion is recorded. |
| Definite insertion failure | Terminal outcome `ManualDeliveryRequired`; Result Panel or clipboard preserves Final Text. |
| Insertion uncertainty | Terminal outcome `DeliveryUncertain`; no automatic retry and delivery uncertainty prevents duplicate text. |
| Target invalidation/focus change | No reactivation; Result Panel or clipboard preserves Final Text under the applicable manual/uncertain outcome. |
| Persistence failure | Existing outcome is preserved with a persistence warning; material remains available but non-durable until success, and unsaved-history warning/manual delivery rules apply. |
| History retry | New Recognition Attempt in the same Dictation Record; prior attempt remains. |
