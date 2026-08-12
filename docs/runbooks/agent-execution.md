# Agent Planning, Execution, and Verification

This runbook defines how the primary agent delegates implementation while retaining responsibility for architecture, product decisions, validation, and Git integration.

## Roles

### Primary agent

The primary agent owns:

- product interpretation and unresolved decisions;
- `docs/implementation-plan.md` and task-level plans;
- architecture and dependency direction;
- task scope, file ownership, acceptance criteria, and validation commands;
- deciding whether work is safe to delegate;
- reviewing executor and verifier reports;
- final diff, validation, commit, push, and PR handling.

The primary agent must not delegate a material unresolved decision and then accept whichever design an executor invents.

### Planning agent

`sol_planner` is a read-only advisor. It may inspect repository facts and propose a bounded implementation approach, but it does not edit, commit, push, alter accepted decisions, or author the authoritative task plan.

The primary agent must personally synthesize the evidence, write and finalize the task plan, and author the final Executor Brief. A planning-agent proposal cannot be forwarded unchanged merely because it looks complete.

### Execution agent

`luna_executor` implements one approved Executor Brief. It owns only the files or responsibility named in the brief. It must stop on a material ambiguity rather than redesigning the project.

### Verification agent

`sol_verifier` is read-only. It checks the result against the approved brief, architecture, tests, security, privacy, licensing, and repository workflow. It leads with actionable findings and returns an accept/fix-required verdict.

## Task plan location

Material tasks receive a committed plan under:

```text
docs/plans/NNNN-short-task-name.md
```

Use sequential four-digit numbering. Small mechanical fixes may use an in-thread brief when they do not affect architecture, behavior, dependencies, persistence, security, privacy, or licensing.

## Required task-plan format

```markdown
# Task title

## Objective

Concrete outcome and user-visible behavior.

## Source decisions

Links to the master-plan milestone, product section, architecture section, ADRs, and glossary terms that constrain the task.

## In scope

Exact behavior and files or modules that may change.

## Out of scope

Related work that must not be added.

## Ownership

The execution agent's exclusive file or responsibility ownership. Note any files concurrently owned by others and prohibit reverting them.

## Architecture and dependency direction

Ports, adapters, domain types, state transitions, and allowed dependency edges.

## Security, privacy, and licensing

Credential handling, sensitive data, log rules, dependency/model review, and notice changes.

## State and failure behavior

Success, cancellation, timeout, retry, late response, fallback, recovery, and persistence effects relevant to the task.

## Implementation steps

Ordered steps small enough for one executor turn or a deliberately bounded continuation.

## Tests and validation

Exact commands plus required success/failure scenarios.

## Acceptance criteria

Observable pass/fail statements.

## Rollback and recovery

How to revert or disable the change without losing user data.

## Executor Brief

Concise imperative instructions that can be forwarded verbatim to the execution agent.
```

## Lifecycle

### 1. Preflight

The primary agent:

1. follows `docs/runbooks/repository-workflow.md`;
2. reads applicable guidance and the master plan;
3. inspects current code, tests, manifests, and working-tree changes;
4. verifies that prerequisite milestones and decisions are complete;
5. distinguishes facts, assumptions, and user decisions.

### 2. Plan

The primary agent personally writes and finalizes the task plan. When planning facts can be investigated independently, a read-only planning or exploration agent may gather evidence and propose options, but the primary agent must synthesize those results and author the final brief.

### 3. Delegate

The primary agent sends only the approved Executor Brief to `luna_executor`, together with explicit ownership and a warning that other work may exist in the repository.

Do not run two execution agents with overlapping files or responsibilities. Parallel work is allowed only when ownership and validation are independent.

### Parallel execution

Parallel execution is a scheduling mode inside this lifecycle, not permission to
skip planning, ownership, verification, or Git controls. When a task plan
authorizes parallel work, the primary agent must additionally define:

- a stable baseline and any contract/readiness gate that must pass before fan-out;
- independently verifiable workstreams and slices rather than assuming existing
  Milestone or agent boundaries are safe to reuse;
- exact exclusive files and responsibilities for every active executor;
- a shared-file lane for contracts, lockfiles, workspace manifests, notices, CI,
  authoritative documentation, and production composition roots;
- workstream entry, waiting, blocked, acceptance, and integration conditions;
- which checks are safe during concurrent writes and which require a quiescent
  repository checkpoint;
- a later integration owner and cross-workstream verification scope.

Parallel does not mean simultaneous completion. A workstream waiting on an
external protocol, platform environment, dependency review, or model gate does
not stop independent workstreams. It also must not broaden its scope, edit
another workstream's files, or claim completion while waiting.

The primary agent may instantiate planning, execution, and verification agents
per bounded slice. Earlier agent counts, permanent Milestone assignments, and
serial call relationships are not reusable defaults. `sol_planner` remains a
read-only advisor, every `luna_executor` still requires a primary-authored brief,
and every `sol_verifier` remains read-only.

#### Frozen contracts and shared changes

When parallel slices consume common ports, events, DTOs, schemas, manifests, or
composition boundaries, the primary agent freezes an explicit baseline before
fan-out. An executor that discovers an insufficient shared contract stops the
affected work and reports a change request; it does not extend the contract
incidentally.

The primary agent evaluates the impact, resolves any material decision, and, if
approved, authors a separate shared-change brief. Shared changes are serialized
under one exclusive owner while affected writers are quiescent. After scoped
verification, the primary agent records the new baseline and tells every
affected workstream whether to resume, revise, or remain waiting.

#### Slice acceptance and integration

A parallel slice is accepted only after its executor handoff, required scoped
validation, read-only verifier `ACCEPT`, and primary scope/policy review. Slice
acceptance proves only the bounded brief against the named baseline. It does not
prove production composition, complete a Milestone, or authorize another
workstream's scope.

When a plan selects stage-end centralized integration, accepted leaf slices do
not modify the real production composition root by default. The primary agent
later authors an independent Integration plan with exclusive ownership of real
adapter selection, wiring, shared manifests, and combined validation. A
controlled early integration check is allowed only through a separate
primary-approved brief when a named risk cannot be tested adequately through
contracts or fakes; it remains an exception and cannot establish completion.

Waiting, blocked, slice-accepted, and integration-ready states are not synonyms
for task or Milestone completion. Completion requires the original acceptance
criteria, applicable integrated behavior, required environment evidence, final
verification, and primary status review.

### 4. Execute

The execution agent:

- rereads applicable guidance and named files;
- restates the objective and validation commands;
- makes the smallest coherent implementation;
- preserves unrelated changes;
- runs the required validation;
- reports changed files, results, remaining risks, and unverified items;
- does not commit, push, publish, or open a PR unless the brief explicitly delegates that action.

### 5. Verify

The primary agent sends the approved plan and implementation result to `sol_verifier`. The verifier does not edit. It checks:

- correctness and acceptance gaps;
- state-machine, cancellation, timeout, retry, and late-response behavior;
- credential, privacy, history, recovery, and redaction guarantees;
- platform/provider leakage into portable code;
- dependency/model license compliance;
- tests, documentation, migration safety, and scope control.

### 6. Resolve findings

The primary agent decides whether a finding is:

- an in-scope defect to send back through a revised Executor Brief;
- a new product or architecture decision requiring the user;
- a false positive with documented evidence;
- deferred work that must be recorded without pretending the task is complete.

### 7. Integrate

After verification passes, the primary agent:

1. runs final validation appropriate to the complete diff;
2. checks sensitive data and license notices;
3. updates the master plan and task status;
4. follows the repository workflow for commit, push, and Ready PR;
5. never merges the PR.

## Change-control rules

- An executor may not change an accepted ADR, master-plan constraint, or product default unless the brief explicitly authorizes it.
- A verifier finding does not authorize edits by itself.
- New dependencies, native binaries, model artifacts, migrations, credential storage, log content, or network destinations require explicit brief coverage.
- A task is not complete because code compiles; its acceptance, failure, privacy, recovery, and license conditions must pass.
- If a required check cannot run, report the exact reason and leave the result unverified rather than weakening the gate.
