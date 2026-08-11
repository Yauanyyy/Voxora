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
