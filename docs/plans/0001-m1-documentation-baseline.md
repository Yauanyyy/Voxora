# Establish the M1 documentation baseline

## Status

Approved for execution on branch codex/m1-documentation-baseline.

The primary agent owns this plan and the Executor Brief. The execution agent may implement only the bounded work below. A read-only verification agent must accept the result before final integration.

## Objective

Create the product, architecture, lifecycle, testing, roadmap, licensing, and architectural-decision baseline required to judge every later Voxora milestone. The result must accurately describe the accepted product without claiming that unimplemented functionality is available.

This task is documentation-only. It introduces no product code, build workspace, dependency, provider integration, platform integration, model artifact, or continuous-integration workflow.

## Source decisions

The following sources constrain this task, in descending order of task relevance:

- the M1 milestone and Accepted product model in ../implementation-plan.md;
- the architecture, privacy, recovery, licensing, delivery, testing, CI, and risk invariants in ../implementation-plan.md;
- the canonical domain terms in ../../CONTEXT.md;
- the repository rules in ../../AGENTS.md;
- the agent lifecycle in ../runbooks/agent-execution.md;
- the Git and Pull Request rules in ../runbooks/repository-workflow.md;
- the user-confirmed design decisions already consolidated into the master plan.

If these sources conflict, the executor must stop and report the conflict. The executor must not silently reinterpret the product or create a new product decision.

## Deliverables and ownership

The execution agent has exclusive write ownership of these M1 deliverables:

- ../../README.md
- ../../LICENSE
- ../../THIRD_PARTY_NOTICES.md
- ../product.md
- ../architecture.md
- ../state-machine.md
- ../testing.md
- ../roadmap.md
- ../licensing.md
- ../adr/0001-windows-first-portable-core.md
- ../adr/0002-tauri-rust-react-desktop-stack.md
- ../adr/0003-no-project-operated-server.md
- ../adr/0004-gpl-3.0-only.md
- ../adr/0005-ports-and-adapters.md
- ../adr/0006-sherpa-onnx-local-inference.md
- ../adr/0007-credentials-outside-sqlite.md

The executor may also update ../../CONTEXT.md only to add the two already-settled domain terms defined in this plan:

- Dictation Record: one persistent history record for a recorded dictation, relating its Recorded Audio, Recognition Attempts, transcripts, Final Text, outcome, and sanitized failure information.
- Recognition Attempt: one recognition execution over Recorded Audio using one Recognition Configuration; a history retry creates another attempt without replacing earlier attempts.

No other agent has overlapping write ownership during execution. The executor is not alone in the repository and must preserve all unrelated or concurrent changes. It must not edit this task plan, the master plan, repository runbooks, AGENTS.md, or project agent configuration.

## In scope

### Root project baseline

- A README that explains the product vision, current documentation/design-stage status, planned first-release capabilities, explicit non-goals, architecture summary, privacy posture, contribution expectations, license, and links to authoritative documents.
- The canonical, unmodified GNU General Public License version 3 text, identified throughout project documentation with SPDX expression GPL-3.0-only.
- An honest initial THIRD_PARTY_NOTICES.md stating that M1 distributes no third-party runtime component, native binary, asset, or model weight, while defining the record format and maintenance rule for future additions.

### Product specification

docs/product.md must cover every subsection of the Accepted product model:

- Recognition;
- Processing;
- LLM and Prompts;
- Hotwords;
- Recording and UI;
- Targeting and insertion;
- History and storage;
- Application profiles.

It must also state product audience, Windows-first scope, visible settings without standard/expert modes, defaults, first-release scope, post-first-release scope, privacy disclosures, and explicit non-goals. It must distinguish planned behavior from currently implemented behavior.

### Architecture specification

docs/architecture.md must define:

- the portable domain, port, application-use-case, adapter, and composition-root responsibilities;
- the inward-only dependency direction recorded in the master plan;
- the rule that voice-core has no cfg(windows);
- the Tauri/React boundary and the prohibition on React session orchestration;
- session-scoped coordination without a mutable global singleton or catch-all orchestrator;
- platform and provider isolation, including Windows-only code in platform-windows;
- trust boundaries for local audio/history, provider requests, credentials, logs, model downloads, clipboard access, and external insertion targets;
- high-level recording-to-insertion data flow;
- configuration precedence for global rules, Application Profiles, Prompt Presets, the global Language Model Configuration, and the global Hotword Library;
- persistence boundaries between SQLite metadata/text, audio artifacts, credentials, and future model files;
- non-server operation and manual application/model update boundaries;
- expected future crate and adapter responsibilities without creating those crates.

### State machine and failure semantics

docs/state-machine.md must separate:

1. lifecycle phase;
2. terminal outcome;
3. structured, sanitized failure or warning metadata;
4. recoverable material that currently exists.

It must define structured commands/events and session-scoped or attempt-scoped identifiers. At minimum it must cover:

- Push-to-Talk, Toggle, maximum-duration stop, capture failure, and empty audio;
- the single-active-session rule and competing shortcut events;
- Esc during capture and Esc after capture;
- target and Application Profile resolution at capture end;
- recognition partial, final, empty, timeout, cancellation, retry, and late/stale response;
- transactional processing across built-in rules and the optional LLM step;
- LLM unavailable/disabled as a skip rather than a failure;
- processing failure fallback to the separately retained Raw Transcript;
- insertion success, definite failure, and uncertain delivery;
- target invalidation and focus changes without target reactivation;
- Result Panel and clipboard-last-resort preservation;
- persistence failure and recovery-material preservation;
- history retry as another Recognition Attempt in the same Dictation Record.

The following meanings are fixed for M1:

- If processing fails but Raw Transcript is successfully delivered, the Dictation Session completes with a processing-fallback warning; it is not a terminal recognition or delivery failure.
- If automatic insertion cannot be performed safely but Final Text is preserved in the Result Panel or clipboard, the outcome requires manual delivery; the text is not considered lost.
- A retry from history adds a Recognition Attempt to the existing Dictation Record. It does not overwrite the previous attempt and does not pretend that a new recording session occurred.
- The recording mode that started the active session owns its stop gesture. Another recording-start gesture cannot create or take over a second session.
- Esc during capture deletes the intentionally cancelled audio and creates no history. Esc after capture stops work that remains safely cancellable and preserves Recorded Audio and available results in history.
- Once an insertion operation may have become irreversible, cancellation does not roll it back or automatically retry it. The result becomes delivery-uncertain when success cannot be confirmed, preventing duplicate text.
- Events received after cancellation, timeout, superseding retry, or terminal completion are rejected when their Session ID, Recognition Attempt ID, or expected phase no longer matches.

Structured errors must contain only sanitized stage/code, retry meaning, delivery certainty, and recoverable-material indicators. They must not contain provider response bodies, credentials, complete Prompts, complete transcripts, Hotword content, credential-bearing URLs, audio, or complete private filesystem paths.

### Testing strategy

docs/testing.md must turn documented behavior into future test obligations:

- deterministic state-transition and invalid-event tables;
- FakeAudioCapture, FakeRecognitionEngine, FakeTextProcessor, FakeTextInjector, target resolver, history, credential, model-manager, shortcut, and clock coverage as applicable;
- cancellation, timeout, retry, stale/late response, processing fallback, recovery, and injection-uncertainty scenarios;
- shared provider/adapter contract tests using synthetic fixtures;
- persistence, retention, deletion, redaction, and credential-serialization tests;
- React reducer/view-model and interaction tests without core orchestration;
- Windows manual and adapter tests for focus, packaged/classic app identity, elevation, microphone loss, modifier-only shortcuts, and clipboard races;
- CI intent from the master plan, while making clear that CI implementation belongs to M2;
- a rule that CI never makes paid provider calls or downloads large model weights.

### Licensing policy and checklists

docs/licensing.md must define a fail-closed review for:

- Rust and JavaScript source dependencies;
- native libraries and bundled binaries;
- images, fonts, sample media, and other assets;
- provider SDKs or protocol references;
- inference frameworks;
- model weights and every accompanying tokenizer, vocabulary, configuration, or preprocessing file.

The dependency checklist must record identity/version, authoritative source, provenance, selected license branch, exact license text, compatibility rationale, feature use, binary/source redistribution duties, notice duties, source-offer or Corresponding Source implications, security/advisory evidence, and reviewer/date.

The model checklist must independently record exact artifact and revision, publisher and official source, every distributed or downloaded file, format, size, SHA-256, exact license text, commercial-use rights, redistribution rights, conversion/derivative terms, accompanying-file licenses, provenance evidence, distribution path, notices, review status/date, and invalidation conditions.

The policy must reject by project policy AGPL, SSPL, non-commercial, research-only, field-of-use-restricted, source-unclear, or otherwise unacceptable artifacts unless the user explicitly changes the policy after documented review. It must not misstate every policy rejection as a legal GPL incompatibility.

The documentation must state all of the following:

- GPL-3.0-only is not GPL-3.0-or-later.
- Independent implementation prohibits copying, rewriting, porting, translating, or deriving SayIt or other GPL/AGPL project source.
- A scanner is evidence, not a substitute for source, license-text, distribution, and notice review.
- An inference framework license does not approve a model.
- sherpa-onnx selection does not approve any SenseVoice artifact.
- Converted ONNX weights may remain governed by the source-weight terms.
- User-initiated download does not automatically remove project distribution or facilitation obligations.
- M8 remains blocked until one exact SenseVoice Small artifact passes the model checklist.

### Roadmap and ADRs

docs/roadmap.md must summarize M0 through M10 and link to the master plan as the sole delivery authority. It must not become a second, independently maintained implementation plan.

Each ADR must remain concise and capture the accepted choice, why it was chosen, material rejected alternatives, and important consequences without duplicating the architecture specification:

1. Windows-first delivery with a portable core.
2. Tauri 2, Rust, React, and TypeScript for the desktop stack.
3. No project-operated server or server self-hosting component.
4. GPL-3.0-only project licensing and independent implementation.
5. Ports-and-adapters architecture with inward dependencies.
6. sherpa-onnx as the planned local inference framework, separately gated from model approval.
7. Platform credential storage with cloud credentials excluded from SQLite and ordinary files.

## Traceability matrix

| Accepted decision group | Primary M1 authority | Supporting documents |
| --- | --- | --- |
| Product outcome and non-goals | docs/product.md | README.md, ADR 0003 |
| Recognition | docs/product.md | docs/architecture.md, docs/state-machine.md, docs/testing.md |
| Processing | docs/product.md | docs/architecture.md, docs/state-machine.md, docs/testing.md |
| LLM and Prompts | docs/product.md | docs/architecture.md, docs/testing.md |
| Hotwords | docs/product.md | docs/architecture.md, docs/testing.md |
| Recording and UI | docs/product.md | docs/state-machine.md, docs/testing.md |
| Targeting and insertion | docs/product.md | docs/architecture.md, docs/state-machine.md, docs/testing.md |
| History and storage | docs/product.md | docs/architecture.md, docs/state-machine.md, docs/testing.md, ADR 0007 |
| Application Profiles | docs/product.md | docs/architecture.md, docs/testing.md |
| Dependency direction | docs/architecture.md | ADR 0001, ADR 0002, ADR 0005 |
| State and error meaning | docs/state-machine.md | docs/testing.md |
| License and model gates | docs/licensing.md | LICENSE, THIRD_PARTY_NOTICES.md, ADR 0004, ADR 0006 |
| Milestone order | docs/implementation-plan.md | docs/roadmap.md |

## Out of scope

The executor must not add or implement:

- Rust, Tauri, React, TypeScript, SQLite, or CI manifests;
- source code, tests, fixtures, generated types, schemas, migrations, or lockfiles;
- GitHub Actions or other automation;
- Doubao, OpenAI-compatible, sherpa-onnx, SenseVoice, Windows API, hotkey, audio, injection, credential, or database integration;
- any dependency, package-manager operation, SDK, native library, model, sample audio, image, font, binary, installer, or asset;
- empty future crate, adapter, application, workflow, or model directory scaffolding;
- a project server, account system, cloud synchronization, telemetry, device identity, usage analytics, team management, application auto-update, or server self-hosting project;
- implementation claims, benchmark claims, compatibility claims, provider capability claims, or license approvals not supported by reviewed evidence;
- changes to accepted product defaults, milestone order, Git policy, agent roles, or merge ownership.

## Architecture and dependency direction

M1 creates documentation only and therefore adds no executable dependency edge. The documents must consistently prescribe this future direction:

    React UI
        -> Tauri desktop composition root
        -> voice-application
        -> voice-ports
        -> voice-core

Adapters implement ports and depend inward. Inward crates never depend on adapters. The production Tauri crate composes selected adapters. Windows and provider types do not cross their adapter boundaries.

## Security, privacy, and licensing

- No real credential, token, account identifier, private endpoint, Prompt, transcript, Hotword list, audio, application identity, or private user path may appear in documentation or examples.
- Cloud ASR and LLM secrets are referenced by opaque credential identifiers and belong in Windows Credential Manager, never ordinary SQLite, JSON, logs, fixtures, crash reports, exports, or plaintext backups.
- SQLite transcript/history is not promised to be encrypted at rest. Documentation must disclose reliance on per-user filesystem protection.
- Logs and structured diagnostics use sanitized stages/codes and exclude complete sensitive content.
- M1 adds no third-party runtime dependency or model and must say so truthfully in THIRD_PARTY_NOTICES.md.
- LICENSE must be copied from a trusted canonical GPL version 3 source without edits.
- Model weights remain separate distribution artifacts with their own review and hash requirements.

## Implementation steps

1. Read all source decisions and restate the bounded objective and validation commands.
2. Create docs/product.md and map each Accepted product model subsection to explicit requirements, defaults, fallback behavior, and non-goals.
3. Create docs/architecture.md with responsibilities, dependency direction, trust boundaries, data/configuration flow, persistence boundaries, and platform/provider isolation.
4. Create docs/state-machine.md with phase, outcome, failure metadata, recoverable-material model, event ordering, race handling, and the fixed semantics in this plan.
5. Create docs/testing.md so every critical state, recovery, privacy, redaction, adapter, UI, and platform rule has a future verification strategy.
6. Create docs/licensing.md with independent dependency, native component, asset, framework, and model acceptance checklists.
7. Create docs/roadmap.md as a concise milestone index pointing back to the master plan.
8. Create the seven ADRs with the exact numbering and decisions listed above.
9. Add Dictation Record and Recognition Attempt to CONTEXT.md using glossary-only definitions and avoided terms; do not add implementation details.
10. Create README.md with an honest design-stage status and links to the authoritative documentation.
11. Copy the canonical GNU GPL version 3 text into LICENSE and use GPL-3.0-only consistently elsewhere.
12. Create the truthful empty-baseline THIRD_PARTY_NOTICES.md and its future record template.
13. Cross-check all documents against the traceability matrix, canonical glossary, master plan, and each other.
14. Run every validation below and report exact results and any unverified condition.

## Tests and validation

Run from the repository root:

    git status --short --branch
    git diff --check
    git diff --name-status HEAD
    git diff --stat HEAD
    git grep -n -E 'TODO|TBD|FIXME' -- README.md THIRD_PARTY_NOTICES.md CONTEXT.md docs ':!docs/plans'
    git grep -n -i -E 'SPDX-License-Identifier:[[:space:]]*GPL-3\.0-or-later|licensed under[[:space:]]+GPL-3\.0-or-later|License:[[:space:]]*GPL-3\.0-or-later' -- .

Both searches must return no matches. Excluding task plans from the unfinished-marker search prevents the validation instructions from matching themselves. The license search rejects an incorrect project-license declaration while allowing explanatory text that contrasts GPL-3.0-only with GPL-3.0-or-later. Also perform and report:

- exact presence of the three root deliverables, six topic documents, and seven ADRs;
- comparison of LICENSE with the trusted canonical GNU GPL version 3 text;
- complete diff review for scope, misleading implementation claims, terminology drift, and broken relative links;
- allowlist review proving that only the named M1 documentation files and the two glossary additions changed after this task-plan commit;
- review for credentials, private endpoints, complete Prompt/transcript/Hotword content, audio, account identifiers, and private absolute paths;
- review that no manifest, lockfile, source code, binary, model, asset, empty implementation directory, or CI workflow was added;
- review that THIRD_PARTY_NOTICES.md does not list planned components as currently distributed;
- review that docs/roadmap.md defers authority to docs/implementation-plan.md;
- review that ADR 0006 separates sherpa-onnx selection from SenseVoice artifact approval.

No paid provider call, package installation, network integration test, model download, platform API test, or application build is appropriate for M1.

## Acceptance criteria

- README.md, LICENSE, THIRD_PARTY_NOTICES.md, six topic documents, and seven ADRs exist with the required scope.
- README states that Voxora is in the documentation/design stage and does not present planned features as implemented.
- Every Accepted product model subsection is traceable through this plan to docs/product.md and, where applicable, architecture, state-machine, testing, licensing, or ADR documentation.
- Product defaults, fallback behavior, history/recovery behavior, privacy boundaries, and non-goals match the master plan without contradiction.
- Architecture documentation makes all portable-core, inward-dependency, adapter, Tauri/React, Windows-boundary, and session-state invariants unambiguous.
- State-machine documentation deterministically answers cancellation, timeout, retry, late/stale response, processing fallback, target invalidation, insertion uncertainty, persistence failure, and recovery scenarios.
- The two settled glossary additions are concise domain definitions with no implementation details.
- LICENSE exactly matches the canonical GNU GPL version 3 terms, while project policy consistently says GPL-3.0-only.
- Licensing documentation contains separate fail-closed dependency and model acceptance checklists.
- sherpa-onnx is not represented as approving SenseVoice or any model artifact.
- THIRD_PARTY_NOTICES.md truthfully records that M1 has no distributed third-party runtime component or model.
- No unreviewed dependency, model, code, manifest, workflow, binary, asset, or speculative scaffolding is introduced.
- All validation passes, or the executor reports the exact blocked check without claiming acceptance.
- The read-only verifier returns ACCEPT after any in-scope findings are resolved.

## Rollback and recovery

M1 changes only Markdown and license text. It has no schema, migration, dependency, runtime, user-data, credential, or model-state effect. Before merge, corrections are made with ordinary additive commits on the task branch; shared history is not rewritten. If M1 must be abandoned, leave the branch and Pull Request unmerged for the user to decide. Never delete or rewrite unrelated repository history.

## Executor Brief

Implement M1 on codex/m1-documentation-baseline exactly as specified in docs/plans/0001-m1-documentation-baseline.md.

First read AGENTS.md, CONTEXT.md, docs/implementation-plan.md, docs/runbooks/agent-execution.md, docs/runbooks/repository-workflow.md, and the entire approved M1 task plan. Restate the bounded objective and validation commands before editing.

You have exclusive ownership only of README.md, LICENSE, THIRD_PARTY_NOTICES.md, docs/product.md, docs/architecture.md, docs/state-machine.md, docs/testing.md, docs/roadmap.md, docs/licensing.md, the seven specifically named docs/adr files, and the two exact glossary additions authorized for CONTEXT.md. You are not alone in the repository: preserve other edits, do not revert unrelated work, and stop if an authoritative source conflicts with the approved plan.

Create a coherent documentation baseline that traces every Accepted product model subsection, fixes the prescribed state/failure semantics, defines future test obligations, and establishes fail-closed dependency/model review. Keep ADRs concise. State honestly that no product implementation or current third-party runtime/model distribution exists. Copy an unmodified canonical GNU GPL version 3 text into LICENSE and use GPL-3.0-only as the project SPDX expression.

Do not add code, manifests, lockfiles, dependencies, CI, binaries, assets, models, provider/platform integration, empty implementation scaffolding, or new product decisions. Do not edit the task plan, master plan, runbooks, AGENTS.md, or agent configuration. Do not commit, push, open a Pull Request, or merge.

Run every validation listed in the plan. Report changed files, command results, the LICENSE comparison source/result, traceability and sensitive-data review, remaining risks, and anything not verified.
