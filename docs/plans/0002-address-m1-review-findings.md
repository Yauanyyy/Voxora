# Address M1 state-machine review findings

## Status

Implemented and independently verified on branch codex/m1-documentation-baseline. Ready for PR review; the user retains merge and review-thread ownership.

## Objective

Correct three confirmed PR #1 review findings so stop gestures cannot cross recording modes, Esc cannot erase delivery uncertainty after irreversible insertion begins, and history recognition retries have a deterministic attempt lifecycle without weakening stale-response rejection.

This is a documentation-only correction to the M1 contract. It adds no product code, dependency, manifest, CI, provider integration, platform integration, model, schema, or migration.

## Source decisions

- docs/implementation-plan.md M3 acceptance for PTT/Toggle behavior, cancellation, retry attempts, late-response rejection, and competing shortcuts.
- docs/plans/0001-m1-documentation-baseline.md fixed state, cancellation, retry, delivery-uncertainty, and identifier semantics.
- docs/state-machine.md five terminal outcomes and structured-event rules.
- CONTEXT.md definitions of Dictation Session, Dictation Record, Recognition Attempt, Final Text, and Insertion Target.
- The three unresolved, non-outdated PR #1 review threads anchored to docs/state-machine.md lines 50, 55, and 65 at reviewed commit 31645ad.

## Review assessment

All three findings are valid:

1. The stop row accepts Push-to-Talk release or Toggle stop unconditionally even though the starting mode is supposed to own the stop gesture.
2. The post-capture Esc row assigns Cancelled even after delivery may have become irreversible, contradicting the DeliveryUncertain race policy.
3. RetryRecognition appends an attempt but does not establish an active retry phase/revision or define how a response after the original terminal session can be valid.

## In scope

- docs/state-machine.md
- docs/testing.md
- this task plan and its final verification record

## Out of scope

- Product code, tests, manifests, dependencies, CI, migrations, schemas, providers, Windows APIs, model artifacts, or UI implementation.
- Changes to the five accepted Dictation Session terminal outcomes.
- Automatic processing, LLM calls, target resolution, or automatic insertion after a history recognition retry.
- GitHub review replies, reactions, thread resolution, review submission, PR merge, or auto-merge.
- New history reprocessing or reinsertion features.

## Ownership

The execution agent has exclusive write ownership of docs/state-machine.md and docs/testing.md. The primary agent owns this plan, validation, commits, push, and PR integration. The verification agent is read-only.

The executor is not alone in the repository. It must preserve all other work and must not edit plans, product/architecture/licensing documents, ADRs, runbooks, AGENTS.md, configuration, or unrelated files.

## Architecture and dependency direction

No executable dependency edge changes. The portable state-machine contract continues to use structured commands/events and session-scoped or attempt-scoped identifiers. The correction must remain provider-, platform-, Tauri-, React-, and Windows-independent.

## Security, privacy, and licensing

- No real Prompt, transcript, Hotword, audio, credential, private endpoint, application identity, or private path may be added.
- History retry must not automatically invoke LLM processing or reuse an old Insertion Target. This avoids an implicit cloud request and unsafe insertion into stale context.
- No dependency, model, native component, asset, or notice change is introduced.

## State and failure behavior

### Mode-owned stop gestures

- StartPushToTalk and StartToggle bind the active Session ID and starting recording mode.
- ReleasePushToTalk stops capture only when both the Session ID matches and the bound starting mode is Push-to-Talk.
- StopToggle stops capture only when both the Session ID matches and the bound starting mode is Toggle.
- A mismatched mode, mismatched Session ID, duplicate stop, or stop after the capture phase is rejected as stale/competing and cannot mutate the active session.

### Esc and irreversible delivery

- Esc after capture yields Cancelled only while remaining work is safely cancellable and delivery has not become irreversible.
- Once clipboard paste or SendInput may have become irreversible, Esc does not roll back, retry, or replace the outcome with Cancelled.
- If delivery cannot be confirmed after irreversible start, the terminal outcome remains DeliveryUncertain and Final Text remains available through the manual recovery path.
- Esc after an already terminal delivery outcome is stale and has no effect.

### History recognition retry

- RetryRecognition is allowed for a durable Dictation Record that retains usable Recorded Audio and when no live Dictation Session or other retry attempt is active.
- It does not create a new Dictation Session or Session ID. It keeps the record and originating Session ID for correlation, creates a fresh Recognition Attempt ID, increments the attempt revision, and activates a record-scoped retry context in Recognizing.
- Retry events are accepted only when Dictation Record ID, originating Session ID, new Recognition Attempt ID, attempt revision, and expected retry phase all match.
- The original Dictation Session remains terminal and immutable. Its terminal outcome, previous attempts, Raw Transcript, Processed Text, Final Text, target, and Application Profile resolution are not overwritten.
- A retry final result appends the new attempt-scoped Raw Transcript and marks that Recognition Attempt succeeded. It is persisted in the same Dictation Record and made available in history for manual use.
- The retry does not automatically run the Processing Pipeline, call an LLM, resolve/reuse an Insertion Target, show an insertion Result Panel, or inject text. Those would require a separate explicitly designed command.
- A retry empty result, timeout, cancellation, or provider failure marks only the new Recognition Attempt failed with sanitized metadata and preserves Recorded Audio and earlier attempts/results.
- Completion or failure closes the retry context and advances the attempt revision. Responses from earlier or closed attempts are stale and cannot mutate the record.

## Implementation steps

1. Split the unconditional stop row into mode- and Session-ID-guarded PTT and Toggle transitions.
2. Narrow post-capture Esc to safely cancellable phases and document the irreversible-delivery exception.
3. Define the record-scoped history retry context, correlation identifiers, phase transition, success/failure completion, and non-processing/non-insertion behavior.
4. Update the event-envelope and stale-event language only as needed to support the retry context without weakening live-session checks.
5. Add matching required scenarios and testing obligations for cross-mode stops, Esc during irreversible delivery, valid retry responses, stale prior attempts, and retry non-processing/non-insertion.
6. Run all validation below and report exact results.

## Tests and validation

Run from the repository root:

    git status --short --branch
    git diff --check
    git diff --name-status
    git diff --stat

Also verify:

- only this plan, docs/state-machine.md, and docs/testing.md change;
- the state document still defines exactly five Dictation Session terminal outcomes;
- every stop transition requires the correct starting mode and Session ID;
- Esc before irreversible delivery and Esc after irreversible delivery have distinct deterministic effects;
- RetryRecognition has a new attempt ID/revision and an active retry phase without a new Dictation Session;
- original session outcome/results remain immutable;
- retry success/failure and stale-response behavior are deterministic;
- history retry never automatically processes, calls an LLM, resolves/reuses a target, or inserts;
- corrected unfinished-marker, incorrect-license-declaration, relative-link, sensitive-content, and private-path checks remain clean;
- no code, manifest, dependency, CI, binary, asset, model, schema, or migration is added.

No runtime build, provider call, platform test, paid API call, or model download is appropriate for this documentation-only correction.

## Acceptance criteria

- All three review findings are demonstrably resolved in the state contract and future test obligations.
- A competing shortcut cannot stop a session started by another recording mode.
- Esc cannot change an irreversible or uncertain delivery into Cancelled.
- A history retry accepts only events for its fresh Attempt ID/revision and closes deterministically.
- A history retry does not overwrite the original Dictation Session or automatically process/deliver text.
- Existing privacy, recovery, licensing, architecture, and five-outcome guarantees remain unchanged.
- A read-only verification agent returns ACCEPT.

## Rollback and recovery

The correction changes Markdown only and has no runtime, data, credential, migration, or model effect. Before merge it can be corrected with ordinary additive commits or left unmerged. Shared history must not be rewritten.

## Verification record

The execution agent changed only docs/state-machine.md and docs/testing.md. The first read-only verification pass found one remaining contradiction in the testing outcome matrix for post-capture Esc. After that row was split into pre-irreversible and post-irreversible cases, the verifier returned ACCEPT with no remaining findings.

Verified evidence covers mode- and Session-ID-owned stop guards, irreversible-delivery Esc behavior, the record-scoped history retry lifecycle, exact five-outcome consistency, stale-event rejection, matching test obligations, scope allowlisting, relative links, whitespace, sensitive content, private paths, license invariants, and absence of executable or dependency changes. Runtime/provider/platform/model checks were not run because this task is documentation-only.

## Executor Brief

Implement docs/plans/0002-address-m1-review-findings.md exactly on codex/m1-documentation-baseline.

Read AGENTS.md, CONTEXT.md, docs/implementation-plan.md, docs/state-machine.md, docs/testing.md, docs/plans/0001-m1-documentation-baseline.md, and this complete plan before editing.

You have exclusive write ownership only of docs/state-machine.md and docs/testing.md. You are not alone in the repository: preserve all unrelated edits and stop on any conflict with authoritative sources.

Correct all three confirmed review findings using the exact state semantics in this plan. Do not create a new Dictation Session for history retry; use a fresh Recognition Attempt ID/revision in a record-scoped retry context, preserve the original terminal session, and keep retry recognition-only with no automatic processing, LLM call, target reuse, Result Panel, or insertion.

Do not edit any other file. Do not add code, dependencies, manifests, CI, schemas, migrations, provider/platform integrations, models, assets, or new product features. Do not stage, commit, push, reply to GitHub, resolve threads, submit a review, or merge.

Run the plan validation and report changed files, exact results, remaining risks, and anything unverified.
