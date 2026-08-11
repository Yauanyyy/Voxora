# Repository Collaboration Workflow

This document is the source of truth for Voxora Git preflight, branches, commits, pushes, Pull Requests, review handling, and merge ownership. `AGENTS.md` contains only the invariants that must always remain visible.

## Repository bootstrap

Before the first commit exists, there is no meaningful base branch for a task branch or Pull Request. The initial repository bootstrap may therefore be prepared on the unborn `main` branch as a one-time exception.

The bootstrap commit should contain only reviewed repository foundations, such as governance, licensing, architecture documents, workspace manifests, baseline tests, and CI. After that commit, all task work follows the branch workflow below.

Creating or pushing the bootstrap commit is allowed only after the relevant validation succeeds. If no GitHub remote is configured, complete the safe local work and report that push and PR creation were unavailable.

## Preflight before changes

At the repository root, always inspect local state first:

```powershell
git status --short --branch
git remote -v
```

When a remote exists, fetch without modifying the worktree:

```powershell
git fetch --all --prune
```

When the current branch has an upstream, also inspect divergence:

```powershell
git rev-list --left-right --count 'HEAD...@{upstream}'
```

Apply these rules:

- Review fetched changes that affect the task before editing.
- If the branch is only behind, the worktree is clean, and a fast-forward is unambiguous, update with fast-forward only and reread affected guidance.
- If fetch fails, no upstream exists, branches diverge, or coordination is ambiguous, do not merge, rebase, overwrite, or start risky edits. Report the state.
- Inspect existing changes and treat them as user or concurrent-agent work unless proven otherwise.
- Never stash, move, overwrite, delete, commit, or mix unrelated changes merely to make the worktree convenient.
- Check changed and untracked files for credentials before staging anything.

Read-only investigation may continue when fetch or upstream checks are unavailable, but its report must state that the checked-out baseline may be stale.

## Task branches

After the bootstrap commit, start new work from a clean, current `main` branch and create a dedicated branch before editing:

```powershell
git switch -c codex/<short-task-name>
```

Rules:

- Codex-created branches use the `codex/` prefix and a short kebab-case task name.
- Continue an existing task branch only when its branch, upstream, Pull Request, and scope match the current task.
- Never put unrelated changes into an existing task PR.
- Never modify, commit, or push feature work directly on `main`.
- Never force-push or rewrite shared history.

## Planning and implementation

Material changes should have a bounded plan or executor brief that identifies:

- user-visible outcome and acceptance criteria;
- in-scope and explicitly out-of-scope work;
- affected ports, adapters, domain objects, state transitions, files, and tests;
- dependency, native binary, model, and license implications;
- security, privacy, concurrency, recovery, and cross-platform risks;
- validation commands and rollback considerations.

Implement the smallest coherent vertical slice. Do not broaden a task with speculative providers, models, rules, UI surfaces, or abstraction layers.

## Validation and sensitive-data review

Before committing:

1. Run the tests and checks required by `AGENTS.md`, affected manifests, and component documentation.
2. Review the complete diff and untracked files.
3. Verify that no credential, token, complete Prompt, transcript, audio, private endpoint, credential-bearing URL, full private path, or sensitive fixture is tracked.
4. Verify dependency and model-license changes and update `THIRD_PARTY_NOTICES.md` when required.
5. Confirm documentation, tests, schemas, migrations, and behavior describe the same contract.
6. Record any check that could not run, including the reason and the expected follow-up environment.

UI changes require screenshots or equivalent visual verification. Screenshots must hide API keys, account identifiers, private endpoints, Prompt content, transcripts, hotwords, application identity details, and other unnecessary user data.

## Commits

Stage only files belonging to the current task. Use a concise imperative commit message with an optional scope, for example:

```text
docs: define repository workflow
core: model dictation cancellation
windows: preserve clipboard during insertion
```

A commit must not claim broader functionality or validation than the diff actually provides.

## Push and Pull Request

After validation and commit:

1. Fetch again and inspect the task branch, its upstream, and `main`.
2. If remote history advanced or coordination is ambiguous, stop and report instead of rebasing, merging, or force-pushing automatically.
3. When safe, push the task branch.
4. Open a **Ready for review** Pull Request. Do not open a speculative PR before the implementation is coherent and validated.

Codex may perform commit, push, and PR creation autonomously after successful validation unless the user opts out for the current task. Lack of a remote, authentication, branch permissions, required checks, or repository metadata is a blocker to the unavailable step only; report it precisely and preserve the safe local result.

Every PR must describe:

- objective and user-visible behavior;
- scope and major files or modules;
- architecture, dependency-direction, and state-machine impact;
- security, privacy, credential, history, and recovery impact;
- dependencies, models, licenses, and `THIRD_PARTY_NOTICES.md` changes;
- commands executed and their results;
- UI evidence when applicable;
- rollback or recovery approach;
- residual risks and anything not verified.

## Review handling

Codex Review is opt-in. It may be requested only when the user explicitly asks for it before implementation or explicitly authorizes it for the current PR.

For any human, automated, or Codex review finding:

1. Present the severity, location, observable risk, and smallest proposed correction.
2. Obtain user approval before implementing a finding that changes the accepted design or task scope.
3. Re-run affected validation after approved corrections.
4. Do not automatically reply, resolve threads, dismiss findings, or treat a review as merge authorization.

Straightforward in-scope CI fixes may be applied without redesigning the feature, but they still require a new validation report.

## Merge ownership

Codex must never merge a Pull Request, enable auto-merge, bypass a branch protection rule, or treat PR creation as permission to merge.

The user performs every merge after deciding that required CI, approvals, review findings, release implications, and branch protection requirements are satisfied. If any blocker remains, leave the PR open and report it.
