# Orchestrate M5-M8 parallel delivery

## Status

Approved orchestration baseline. This plan changes agent coordination and
delivery sequencing only. It authorizes no M5-M8 product implementation,
dependency, provider request, native integration, model artifact, or real
composition-root wiring by itself.

The previous assumption that one complete Milestone must be implemented and
integrated before the next Milestone begins is retired for M5-M8. The existing
primary/planner/executor/verifier capabilities remain available, but their
instance count, ownership, and call topology are selected per work slice rather
than inherited from the earlier serial Milestones.

## Objective

Run M5, M6, M7, and M8 as independently progressing workstreams after a shared
parallel-readiness gate, while preserving frozen portable contracts, exclusive
write ownership, fail-closed dependency/model review, and the existing
primary-authored Executor Brief and read-only verification lifecycle.

Parallel delivery is not a requirement that all four workstreams start, advance,
or finish simultaneously. A workstream waiting on a model gate, external
protocol fact, Windows-only debugging, or another explicit prerequisite does not
stop independent workstreams. It also gains no authority to cross its ownership
boundary, fill another workstream's scope, alter shared contracts, or claim a
Milestone complete.

## Source decisions

- M5-M8 objectives and acceptance in
  [`../implementation-plan.md`](../implementation-plan.md).
- Canonical domain terms in [`../../CONTEXT.md`](../../CONTEXT.md).
- Product behavior in [`../product.md`](../product.md).
- Dependency direction and trust boundaries in
  [`../architecture.md`](../architecture.md).
- Lifecycle and late-event rules in [`../state-machine.md`](../state-machine.md).
- Test obligations in [`../testing.md`](../testing.md).
- Dependency and model gates in [`../licensing.md`](../licensing.md), ADR 0004,
  ADR 0005, and ADR 0006.
- Agent lifecycle in [`../runbooks/agent-execution.md`](../runbooks/agent-execution.md).
- Git lifecycle in
  [`../runbooks/repository-workflow.md`](../runbooks/repository-workflow.md).

## In scope

- The parallel-readiness gate and frozen-contract baseline.
- Workstream, slice, shared-file, and Integration-stage ownership.
- Agent instantiation and scheduling rules.
- Project subagent concurrency capacity for the four M5-M8 workstreams with
  additional scheduling headroom where the effective runtime limit permits it.
- Shared-contract and shared-file change control.
- Workstream states, gates, handoffs, verification, and completion semantics.
- Stage-end centralized Integration with a narrowly controlled early integration
  check exception.
- Documentation changes needed to make the master plan and generic runbook agree
  with this orchestration.

## Out of scope

- Product code, UI, Tauri commands/events, adapter code, provider protocol code,
  native Windows code, model-manager code, or model manifests.
- Selecting dependencies, SDKs, model artifacts, protocol revisions, or native
  libraries.
- Freezing the actual M5-M8 contracts in this documentation-only task. Contract
  freeze is the first later implementation-stage gate and requires its own plan.
- Modifying the real Tauri composition root or claiming any M5-M8 acceptance.
- Changing product defaults, architecture direction, privacy policy, licensing
  policy, or user-owned merge authority.

## Orchestration invariants

1. **No synchronized finish barrier.** Workstreams progress independently. A
   waiting or blocked workstream does not pause unrelated work.
2. **No scope borrowing.** Spare capacity in one workstream does not authorize it
   to implement another workstream's responsibilities.
3. **No overlapping writers.** Two execution agents never own the same file or
   responsibility concurrently.
4. **Contracts are frozen before fan-out.** Portable ports, correlated events,
   desktop boundary DTOs, and adapter-facing failure meanings receive a named
   baseline before workstream execution begins.
5. **Shared changes are serialized.** Shared contracts, workspace manifests,
   lockfiles, notices, CI, root documentation, and the real composition root use
   a primary-controlled shared-change lane, never opportunistic workstream edits.
6. **Leaf implementation is not runtime integration.** Accepted workstream code
   may accumulate as scoped commits on the task branch without wiring real
   adapters into the production composition root.
7. **Integration is a separate stage.** Real adapter selection, cross-module
   wiring, production Tauri command/event registration and mappings that replace
   or extend the M5 fake-only boundary, and combined behavior are performed only
   by an Integration Executor Brief after the Integration entry gate.
8. **Verification is layered.** Slice acceptance proves the bounded brief and
   frozen contracts. It does not prove combined product behavior or complete a
   Milestone.
9. **Fail closed.** Missing provider facts, Windows evidence, dependency review,
   or model provenance produce an explicit waiting or blocked state, never a
   placeholder implementation or weakened test.
10. **The primary agent remains authoritative.** Planning agents advise;
    execution agents implement approved briefs; verification agents are
    read-only; the primary agent owns decisions, shared-change approval,
    Integration scope, final validation, Git, and status claims.

## Delivery topology

```text
Primary agent
  |
  +-- Parallel-readiness gate and frozen-contract baseline
  |
  +-- Shared-change lane (serialized for the whole phase)
  |
  +-- M5 desktop UX workstream -------------------+
  +-- M6 Windows platform workstream -------------+--> slice verification
  +-- M7 cloud capability workstream -------------+    and scoped commits
  +-- M8 local ASR/model workstream --------------+
  |
  +-- Integration-stage entry decision
        |
        +-- Integration executor: real composition and cross-module wiring
        +-- Integration verifier: combined acceptance and policy review
        +-- Primary final validation and Milestone status decisions
```

The number of active agents is a scheduling choice, not an architectural fact.
The primary agent may run fewer workstreams than available concurrency slots,
pause one workstream, or reuse an idle agent for a later non-overlapping slice.
No plan may require four execution agents to be active at the same instant.

The project sets `agents.max_concurrent_threads_per_session = 5`. This is a
capacity ceiling, not a requirement to spawn five agents or keep them alive
throughout the phase, and the effective Codex runtime or service may enforce a
lower concurrent limit. Scheduling prioritizes at most one active executor for
each independently ready M5-M8 workstream. Any remaining effective capacity may
run a bounded planning, verification, or serialized shared-change task; otherwise
that task waits for a workstream agent to become quiescent or for the primary to
pause an appropriate writer. The configured fifth thread does not create a fifth
product workstream, weaken exclusive ownership, or allow a verifier to inspect
files while its owning executor is still writing them.

## Parallel-readiness gate

The primary agent must author and approve a dedicated readiness task plan before
delegating M5-M8 product work. Fan-out is allowed only when all of the following
are true:

1. M4 is merged and the checked-out base is current enough for implementation.
2. Existing local and concurrent changes are inventoried and protected.
3. The portable contract inventory covers at least audio capture, shortcuts,
   recognition, processing, target resolution/validation, insertion, Result
   Panel, clipboard fallback, credentials, history, model management,
   cancellation, clocks, identifiers, and correlated events.
4. Any contract changes required for known M5-M8 acceptance are implemented and
   verified through one serialized shared-contract brief.
5. A frozen-contract baseline is identified by a commit and documented version
   or immutable inventory. Workstream briefs name that baseline.
6. Desktop command/event DTOs and frontend-visible state are provider- and
   Windows-type-free and sufficiently specified for M5 contract tests.
7. Adapter conformance suites and synthetic fixture rules are available for M6,
   M7, and M8 leaf implementations.
8. The ownership ledger assigns every planned file or responsibility to exactly
   one active writer or to the shared-change lane.
9. Shared hotspots and their authorized controller are named explicitly.
10. Each initial slice has a primary-authored plan or Executor Brief, entry gate,
    acceptance criteria, exact validation, and non-goals.

If readiness exposes a material product or architecture decision, parallel
execution does not begin until the primary agent resolves it with the user when
required. The readiness task must not pre-create empty adapters merely to make
the directory tree resemble the planned repository.

## Workstreams and exclusive ownership

Ownership below is responsibility-based. Each later task plan must convert it to
exact files before delegation. Newly discovered shared files remain in the
shared-change lane until the primary agent explicitly reclassifies them.

### M5 desktop UX workstream

Owns:

- React views, reducers/view models, interaction behavior, styles, and frontend
  tests for settings, history, Recording Overlay, Result Panel, and fake flows;
- the fake-only Tauri command/event boundary, portable DTO mappings, fake adapter
  composition, test/dev registration, and associated Rust/frontend contract
  tests when exact desktop files are assigned by its later Executor Brief;
- frontend fake transport or test harnesses that implement the frozen desktop
  contract without selecting production M6-M8 adapters;
- rendered visual evidence with sensitive content excluded.

Does not own:

- Rust session orchestration, portable state transitions, Windows/provider
  adapters, production Tauri adapter selection, provider- or Windows-specific
  DTO definitions, or real production-adapter composition-root wiring.

M5 may implement and run the complete fake-only Tauri boundary needed by its
Milestone. This is not production Integration: selecting M6-M8 adapters,
replacing fakes with real implementations, and wiring cross-workstream runtime
configuration remain exclusive Integration-stage responsibilities.

M5 may split into sequential or non-overlapping slices such as desktop state and
transport contract, settings/history surfaces, overlay/Result Panel behavior,
and visual/accessibility verification. All UI for M6-M8 capabilities remains M5
ownership; platform/provider executors provide contract evidence rather than
editing React.

### M6 Windows platform workstream

Owns:

- Windows audio capture and microphone-selection adapter modules;
- Windows shortcut registration, modifier-only semantics, and conflict handling;
- Windows current-focus target resolution and application identity mapping;
- clipboard/SendInput insertion, target validation, and safe fallback modules;
- Windows adapter tests and documented manual-debug evidence.

Does not own:

- portable core semantics, React UI, cloud/local recognition, provider payloads,
  production composition, or unrelated Credential Manager behavior already
  delivered by M4.

M6 may subdivide audio/shortcuts and targeting/insertion only when their file and
native-resource ownership is independent. Windows debugging delays one slice
without granting it access to another workstream.

### M7 cloud capability workstream

Owns:

- the Doubao recognition adapter and synthetic protocol fixtures;
- the OpenAI-compatible processing adapter and synthetic payload fixtures;
- provider-local request validation, capability negotiation, response parsing,
  cancellation/timeout mapping, and sanitized failures;
- provider-specific dependency/protocol review records assigned by the primary.

Does not own:

- portable configuration semantics already accepted in M4, credential storage,
  UI, native Windows behavior, local ASR, real composition, or changes to the
  frozen generic recognition/processing contracts.

Doubao and OpenAI-compatible slices may progress independently when their exact
ownership is disjoint. Unresolved Doubao protocol facts do not block a separately
briefed OpenAI-compatible slice, and the reverse is also true.

### M8 local ASR and model workstream

Before the M8 artifact gate, owns only:

- exact sherpa-onnx framework/integration review;
- exact SenseVoice Small artifact provenance, contents, size, SHA-256, license,
  commercial-use, redistribution, and accompanying-file review;
- quarantined review-only retrieval of the exact candidate artifact when a
  primary-approved bounded review brief requires the actual files to establish
  their contents, byte sizes, SHA-256 values, provenance, or governing terms;
- a proposed manifest and notice impact for primary approval.

Gate-review retrieval is evidence collection, not product acquisition or model
approval. The review brief must name the exact authoritative source and expected
revision, use an isolated non-product location, record retrieval method/date and
hash evidence, and define cleanup or retention of review evidence. Retrieved
weights and accompanying files must not be committed, bundled, copied into an
application/model installation directory, activated, loaded for inference, or
made available to product code. Only non-sensitive review records and proposed
metadata may enter the repository. A successful download or matching hash does
not by itself approve the artifact; every license, commercial-use,
redistribution, accompanying-file, provenance, security, and integration check
must still pass.

No M8 product implementation begins until the complete gate is accepted. After
the gate, later briefs may assign:

- the local sherpa-onnx recognition adapter;
- model acquisition, verification, activation, deletion, and import modules;
- synthetic small-file model-manager tests and CPU/manual benchmark evidence.

M8 does not own cloud providers, React UI, Windows delivery, portable contract
redesign, production composition, or unreviewed model/framework artifacts. A
failed or incomplete artifact gate freezes M8 product implementation without
blocking M5, M6, or M7.

## Shared-change lane

The following are shared by default:

- `crates/voice-core/`, `crates/voice-ports/`, and cross-workstream portions of
  `crates/voice-application/`;
- root `Cargo.toml`, `Cargo.lock`, `deny.toml`, npm lockfiles, and workspace-level
  scripts or schemas;
- `THIRD_PARTY_NOTICES.md`, common dependency/model review indexes, and shared CI;
- Tauri manifests, capabilities, window/tray configuration, root command/event
  registration, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, and the real
  production composition root, except for exact fake-only boundary and desktop
  files explicitly assigned to an M5 slice; production adapter selection and
  real cross-workstream wiring always remain shared Integration ownership;
- authoritative product, architecture, state-machine, testing, implementation,
  roadmap, runbook, and status documentation.

A workstream needing one of these files submits a Shared Change Request containing:

1. the frozen baseline and requesting slice;
2. the missing capability or exact dependency/manifest need;
3. why the request cannot remain local to the workstream;
4. affected workstreams and contract compatibility;
5. security, privacy, licensing, and notice impact;
6. proposed tests and migration/rollback impact;
7. whether the requesting slice can continue independently while the request is
   evaluated.

The primary agent then rejects the request, revises the requesting brief, or
authors a separate shared-change brief. A temporary execution agent may implement
that brief only with exclusive shared-file ownership while affected writers are
quiescent. The primary agent verifies the new baseline, records its effect on
every active slice, and explicitly resumes affected work. Workstream executors
never make an incidental shared-contract change because it is small.

Dependency additions follow the same lane. A leaf adapter may own its local
manifest only when the primary-assigned file list says so; root workspace
membership, lockfile regeneration, shared CI, reviews, and notices remain
serialized and must agree as one change set.

## Slice lifecycle and agent use

For every slice, the primary agent:

1. confirms the workstream entry gate and frozen baseline;
2. writes the authoritative plan and final Executor Brief;
3. assigns exact exclusive files and responsibilities;
4. names concurrent work and prohibited shared files;
5. delegates to a `luna_executor` instance;
6. receives a quiescent handoff with changed files and validation evidence;
7. sends the approved brief and scoped result to a read-only `sol_verifier`;
8. resolves findings through a revised bounded brief when necessary;
9. marks the slice `accepted` only after the verifier returns `ACCEPT` and the
   primary confirms scope, sensitive-data, dependency, and ownership compliance;
10. may create a scoped checkpoint commit without wiring production adapters.

`sol_planner` may investigate an uncertain dependency, contract, or acceptance
question, but it is not permanently assigned to a workstream and does not author
the authoritative plan. Executor and verifier instances are created per bounded
slice; the earlier one-executor/one-verifier serial topology is not a template
for the parallel phase.

During concurrent work, global formatting or repository-wide checks can observe
unrelated in-progress files. Slice briefs therefore distinguish:

- local checks that are safe while other writers are active;
- a quiescent scoped check before slice verification;
- repository-wide checks reserved for shared-change checkpoints and Integration.

A verifier reviews only the named slice and frozen contract. It reports any
cross-workstream risk but does not reject a correct leaf slice merely because a
different workstream is unfinished.

## Workstream states

The coordination record uses these meanings:

| State | Meaning | May claim Milestone complete? |
| --- | --- | --- |
| `planned` | Scope exists but entry conditions are not yet satisfied. | No |
| `ready` | Entry conditions and an approved brief exist. | No |
| `active` | An executor is working within the approved ownership. | No |
| `waiting` | Progress awaits a known gate, protocol fact, environment, or debug result with a stated resume condition. | No |
| `blocked` | The primary cannot make meaningful in-scope progress without new authority or external-state change. | No |
| `slice ready` | Executor handoff and required scoped checks are complete; read-only verification is pending. | No |
| `slice accepted` | The slice passed its brief and verifier, but combined wiring is not proven. | No |
| `ready for integration` | All workstream slices required by its Milestone are accepted and its integration contract is complete. | No |
| `integrated candidate` | Centralized wiring and combined tests include the workstream, pending final acceptance/status review. | No |
| `milestone complete` | Every Milestone acceptance condition, centralized integration obligation, required environment evidence, policy check, and final verifier condition has passed. | Yes |

State changes are evidence-based. `waiting`, `blocked`, a checkpoint commit, a
Ready PR, or any number of accepted slices cannot be relabeled as Milestone
completion.

## Stage-end centralized Integration

Accepted slices do not modify the real production composition root by default.
This removes the old wording dependency that M5 had to finish before M6-M8 code
could exist: M5 validates frozen fake-adapter behavior while M6-M8 independently
implement contract-conformant leaves, and only Integration replaces fakes in the
running product.
The primary agent starts a separate Integration stage only after the intended
integration cohort is ready. The default cohort is M5, M6, M7, and M8; a
workstream may reach `ready for integration` earlier and wait without expanding
scope. A blocked workstream delays final cohort Integration and completion but
does not stop independent leaf progress in other workstreams.

Changing the cohort, removing a first-release capability, or integrating a
partial product as if it satisfied M5-M8 requires an explicit product-scope
decision. The primary agent cannot silently redefine completion to avoid a gate.

The primary-authored Integration plan owns:

- production Tauri composition-root changes and adapter selection;
- production command/event registration and frontend-to-application mappings
  that select real adapters or replace/extend the M5 fake-only boundary;
- cross-module configuration and credential resolution;
- real capture-to-recognition-to-processing-to-delivery wiring;
- selection between cloud and local Recognition Configurations without automatic
  privacy-changing fallback;
- shared manifest, lockfile, CI, notice, and authoritative status updates;
- combined fake and real-adapter contract tests, Windows manual evidence, UI
  visual evidence, redaction review, and complete validation.

The Integration executor has exclusive ownership of all real wiring and shared
files for that stage. Workstream executors are quiescent unless the primary sends
a bounded correction brief with non-overlapping ownership. After implementation,
an Integration verifier performs a read-only cross-workstream review against all
accepted slice plans and Milestone acceptance conditions.

### Controlled early integration check

The primary agent may approve an early integration check only when a named risk
cannot be adequately tested through frozen contracts, adapter conformance tests,
or local fakes. The exception requires a short plan specifying:

- the exact unresolved risk and why local evidence is insufficient;
- the minimal temporary or test-only wiring;
- exclusive ownership and affected workstream pauses;
- validation and cleanup/retention of the check;
- confirmation that it does not establish production Integration or Milestone
  completion.

An early integration check does not change the default stage-end centralized
Integration strategy and must not become an unreviewed production composition
path.

## Verification responsibilities

### Slice verifier

Checks the bounded brief, frozen contracts, local behavior, cancellation,
timeouts, stale responses, fallback/recovery, redaction, dependency direction,
fixture provenance, and workstream scope. It does not edit or claim combined
runtime behavior.

### Shared-change verifier

Checks compatibility for every affected workstream, inward dependency direction,
contract-test updates, lockfile/review/notice consistency, and whether all paused
workstreams have an explicit resume baseline.

### Integration verifier

Checks real composition, cross-workstream state and failure semantics, UI command
ownership, provider/platform isolation, credential and payload boundaries,
recorded-audio and Final Text preservation, target safety, model integrity,
complete licenses/notices, required environment evidence, and the combined test
matrix. It returns one final `ACCEPT` or `FIX REQUIRED` verdict for the Integration
plan; it does not automatically mark individual Milestones complete.

### Primary final review

The primary agent decides each Milestone status separately from evidence. M5 may
not be complete merely because the UI works against fakes; M6-M8 may not be
complete merely because their adapters pass contract tests. Each status requires
its original acceptance conditions plus applicable Integration and environment
evidence.

## Security, privacy, dependencies, and licensing

- Provider and model work remains opt-in and sends no real request in CI.
- Workstream fixtures contain no complete private transcript, Prompt, Hotword
  list, endpoint, account, key, audio, application identity, or private path.
- Credentials remain behind the platform credential port and opaque references.
- M7 repeats Base URL validation immediately before a request and logs no provider
  bodies or sensitive payloads.
- Before the exact artifact gate passes, M8 adds no product model-download
  behavior, approved manifest claim, framework/model dependency, installation,
  activation, inference, bundling, or committed artifact. A primary-approved
  bounded review brief may retrieve the exact candidate only into quarantine for
  direct contents/size/hash/license/provenance evidence under the M8 rules above.
- M6 keeps all Windows types and native behavior in `platform-windows`.
- Every dependency, native component, protocol-derived artifact, asset, framework,
  and model change uses the shared-change lane with a complete review and
  synchronized `THIRD_PARTY_NOTICES.md` update.
- No workstream adds telemetry, accounts, sync, a project server, updater,
  privileged helper, or automatic privacy-changing fallback.

## Validation of this orchestration change

This documentation task requires:

```text
git diff --check
git status --short --branch
```

The primary agent must also inspect the complete documentation diff and confirm:

- the implementation plan no longer requires M5, then M6, then M7, then M8
  implementation;
- the generic runbook preserves primary-authored plans and read-only verification;
- shared write ownership is serialized;
- centralized Integration is the default and early checks are exceptional;
- waiting/blocked work does not halt independent work or imply completion;
- no product code, dependency, secret, or sensitive fixture was added.

## Acceptance criteria

- M5-M8 are defined as independently progressing workstreams behind one
  parallel-readiness gate and one later centralized Integration stage.
- The plan contains no requirement that all four workstreams be active or finish
  simultaneously.
- Project configuration caps spawned subagents at five, while scheduling remains
  valid under a lower effective runtime limit and never treats extra capacity as
  a mandatory fifth product workstream.
- M5 explicitly owns an approved fake-only Tauri command/event boundary while
  production adapter selection and cross-workstream wiring remain Integration
  ownership.
- Each workstream has explicit responsibilities and prohibited scope.
- Shared contracts/files have one serialized change protocol and cannot be edited
  incidentally by leaf executors.
- The M8 artifact gate blocks M8 implementation only, unless its absence later
  prevents the default complete Integration cohort.
- Slice acceptance, Integration readiness, and Milestone completion are distinct.
- The early integration check exception is primary-approved, minimal, and cannot
  establish completion.
- Existing agent roles remain governed by the repository runbook without fixing
  their old instance count or serial call topology.
- The user retains every merge decision.

## Rollback and recovery

This change is documentation-only. Before implementation begins, it can be
reverted without user-data or runtime migration impact. After parallel work has
begun, replacing the orchestration requires the primary agent to stop new briefs,
inventory active ownership and accepted slices, preserve all concurrent changes,
and write a superseding plan. Never erase a workstream's accepted evidence or
silently move its files to another owner.

## Executor Brief

No product implementation is authorized by this plan alone. For every later
readiness, workstream, shared-change, early-check, or Integration task, the
primary agent must write a new bounded plan and final Executor Brief that names
the frozen baseline, exact files, exclusive responsibility, concurrent work,
entry gate, validation, acceptance criteria, and non-goals.
