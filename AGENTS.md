# Repository Guidelines

## Purpose and sources of truth

Voxora is an independently implemented GPL-3.0-only desktop voice-input application. Do not copy, rewrite, port, or derive source code from SayIt or other GPL/AGPL projects. Public feature concepts may inform product requirements, but all implementation must be original.

Read these files before material work:

- `CONTEXT.md` for canonical domain language only;
- `docs/implementation-plan.md` for ordered milestones, gates, accepted behavior, and completion conditions;
- `docs/product.md` for product behavior and scope;
- `docs/architecture.md` for boundaries and dependency direction;
- `docs/adr/` for accepted hard-to-reverse decisions;
- `docs/runbooks/agent-execution.md` for primary-agent plans, executor briefs, delegated implementation, and verification;
- `docs/runbooks/repository-workflow.md` for Git, commit, push, PR, review, and merge rules.

When sources conflict, stop and surface the conflict. Do not silently choose whichever source is convenient.

## Architecture invariants

- Portable business logic must not depend on Tauri, React, Windows APIs, UI Automation types, or provider SDKs.
- `voice-core` must not contain `cfg(windows)`.
- Platform and provider capabilities enter through explicit ports and adapters.
- Windows-only injection, target resolution, hotkeys, credentials, and native integration stay in `platform-windows`.
- React components render state and submit commands; they do not orchestrate dictation sessions.
- Do not store mutable session state in global singletons.
- Avoid a catch-all orchestrator. Keep capture, recognition, processing, targeting, history, recovery, and insertion responsibilities separate.
- Session transitions, cancellation, timeout, retry, and late responses use structured events and session-scoped identifiers.
- Failures must not silently lose recorded audio, available transcripts, or final text.

## Security and privacy

- Cloud ASR and LLM credentials belong in the platform credential store. Never persist them in ordinary SQLite, JSON, logs, fixtures, crash reports, or plaintext backups.
- Logs must not contain complete Prompts, transcripts, API responses, audio, credential-bearing URLs, or complete filesystem paths.
- Use sanitized error stages and codes in diagnostics. User-readable failure details belong in history without leaking secrets.
- Do not add telemetry, device identifiers, usage-statistic uploads, accounts, cloud sync, a project-operated server, or application auto-update.
- Treat audio, transcripts, Prompt content, hotwords, application identities, and history as sensitive local data.
- Never put real credentials or private user content in tests or examples.

## Dependencies, models, and licensing

- Check code license compatibility, source provenance, feature requirements, and redistribution conditions before adding a dependency.
- Reject AGPL, SSPL, non-commercial, research-only, field-of-use-restricted, source-unclear, or otherwise incompatible components unless the user explicitly changes the policy after a documented review.
- Model weights are separate distribution artifacts. Verify each model's source, exact version, size, SHA-256, license, commercial-use terms, and download/redistribution conditions independently of its inference framework.
- Keep `THIRD_PARTY_NOTICES.md` current whenever a dependency, native component, asset, or model manifest changes.
- Do not add a model or provider integration merely because its framework license is acceptable.

## Agent workflow

Project-scoped subagents are configured under `.codex/agents/` and enabled in `.codex/config.toml`. When multi-agent work is authorized and useful:

- use `sol_planner` for read-only architecture evidence, planning proposals, acceptance-criteria analysis, and draft brief advice;
- use `luna_executor` only for a bounded, approved executor brief;
- use `sol_verifier` for final read-only verification against the approved plan, tests, security, privacy, and licensing constraints.

For every material implementation step, the primary agent must personally synthesize, write, and finalize the task plan and Executor Brief before delegation. Planning subagents are read-only advisors and cannot be the authoritative author. Follow `docs/runbooks/agent-execution.md`; execution agents do not invent product or architecture decisions.

Do not assign concurrent agents overlapping write ownership. Preserve edits made by the user or other agents, and never revert unrelated work.

Before editing:

1. Read applicable repository guidance and relevant domain/architecture documents.
2. Follow the Git preflight in `docs/runbooks/repository-workflow.md`.
3. Inspect the affected code, tests, manifests, and current working-tree changes.
4. Separate verified facts from assumptions and unresolved product decisions.

## Implementation and validation

- Prefer a narrow vertical slice with tests over broad placeholder scaffolding.
- Use fake ports to test success, cancellation, timeout, provider failure, processing fallback, late response, recovery, and injection failure without platform APIs.
- Add tests whenever behavior, state transitions, persistence, security boundaries, or provider contracts change.
- Once manifests exist, run the affected subset of Rust formatting, linting, tests, TypeScript build/lint/tests, and dependency-license checks before committing.
- Review the final diff for scope expansion, sensitive data, license changes, platform leakage, and misleading documentation.
- Record checks that could not run and why; never claim unexecuted validation passed.

## Git and Pull Requests

Follow `docs/runbooks/repository-workflow.md`.

Key invariants:

- After the bootstrap commit, never modify or push directly to `main`; use a dedicated `codex/` task branch.
- Never force-push, rewrite shared history, or mix unrelated work.
- After scoped work passes validation, Codex may commit, push, and open a Ready for review PR unless the user opts out for that task.
- Codex must never merge a PR. The user owns every merge decision.
- Codex Review is opt-in and does not authorize automatic fixes, replies, thread resolution, or merge.
