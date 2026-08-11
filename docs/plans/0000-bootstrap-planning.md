# Establish the master implementation plan

## Objective

Commit Voxora's shared domain language, repository governance, Git/PR policy, project-scoped agent configuration, authoritative master implementation plan, and delegation protocol before product implementation begins.

## Source decisions

- User-confirmed product design captured in `CONTEXT.md` and the planning interview.
- `AGENTS.md` architecture, privacy, licensing, and Git invariants.
- `docs/runbooks/repository-workflow.md` bootstrap and user-owned merge policy.

## In scope

- Governance and planning Markdown files.
- Existing `.codex/config.toml` and `.codex/agents/*.toml` project agent definitions.
- A minimal `.gitignore` protecting future build, secret, model, database, and audio artifacts.
- The first local Git commit on the unborn `main` branch.

## Out of scope

- Product code, generated workspace files, runtime dependencies, CI, provider integration, model download, Windows APIs, and GitHub push/PR creation.

## Ownership

The primary agent owns all files in this bootstrap task. No execution subagent edits files. A verification subagent may perform read-only review after the plan is written.

## Architecture and dependency direction

No product dependency edge is created. The master plan records the future inward dependency direction.

## Security, privacy, and licensing

- No credentials, private Prompt, transcript, audio, endpoint, or user-specific application identity may enter the commit.
- GPL-3.0-only is recorded as policy; the full license file is intentionally scheduled for M1.
- No third-party runtime dependency or model is introduced.

## State and failure behavior

No runtime behavior changes.

## Implementation steps

1. Write the authoritative implementation plan.
2. Write the agent delegation and verification runbook.
3. Link both from `AGENTS.md`.
4. Add a protective `.gitignore`.
5. Inspect all tracked candidates and Git diff for sensitive or unintended content.
6. Run Markdown/whitespace checks available without adding dependencies.
7. Obtain read-only verifier feedback and correct any in-scope defects.
8. Create the initial local Git commit.

## Tests and validation

```powershell
git diff --check
git status --short --branch
git diff --cached --check
git diff --cached --stat
```

Also inspect the complete staged file list and search tracked candidates for obvious secret patterns.

## Acceptance criteria

- The plan covers milestones, dependencies, state/failure testing, licensing, privacy, CI, risks, and completion conditions.
- The agent runbook requires a primary-authored brief, bounded execution, read-only verification, and primary integration.
- `AGENTS.md` links both documents.
- The commit contains no product implementation or third-party dependency.
- The first local commit succeeds on `main`; no push or PR is attempted without a remote.

## Rollback and recovery

Before push, the initial commit remains local and can be amended by a later explicitly approved task. Files are plain Markdown and configuration with no migration or user data impact.

## Executor Brief

Primary-agent task: finalize the planning and governance files, validate the staged content, obtain read-only verification, and create the first local commit. Do not add product code, dependencies, CI, provider integrations, models, or platform implementation.
