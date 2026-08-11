# Establish the M2 workspace and CI skeleton

## Status

Implemented and locally verified on branch `codex/m2-workspace-ci-skeleton`. Delivery to `main` remains subject to the Ready for review Pull Request checks and the user's merge decision.

The primary agent owns this plan, the dependency review, implementation, validation, and Git integration. The current session does not authorize subagent delegation, so the primary agent will execute the bounded Executor Brief directly while preserving the role and scope constraints in the repository runbooks.

## Objective

Create the smallest buildable cross-platform Voxora workspace: three portable Rust foundation crates, a Tauri 2 desktop composition root, and a React + TypeScript + Vite frontend. Add deterministic baseline tests and CI gates for formatting, linting, builds, tests, dependency licenses, tracked-file secrets, and future model manifests without implementing any M3 or adapter behavior.

The desktop shell must state honestly that product functionality is not implemented. M2 proves build boundaries and automation only.

## Source decisions

- M2 and the CI design in [`../implementation-plan.md`](../implementation-plan.md).
- Product scope and non-goals in [`../product.md`](../product.md).
- Dependency direction, composition-root rules, and portable-core invariants in [`../architecture.md`](../architecture.md).
- Future test obligations in [`../testing.md`](../testing.md).
- Fail-closed dependency and model policy in [`../licensing.md`](../licensing.md).
- ADR 0001, ADR 0002, ADR 0004, and ADR 0005 in [`../adr/`](../adr/).
- Canonical terms in [`../../CONTEXT.md`](../../CONTEXT.md).
- Planning, validation, Git, and merge rules in the repository runbooks.

If an authoritative source conflicts with this plan, implementation stops until the conflict is resolved explicitly.

## In scope

- Root Cargo workspace metadata, a pinned Rust stable toolchain, Cargo lockfile, Rust formatting/lint configuration, and `cargo-deny` policy.
- `crates/voice-core`, `crates/voice-ports`, and `crates/voice-application` as portable libraries with inward-only manifest dependencies, crate-level boundary documentation, and non-speculative metadata/build tests.
- `apps/desktop` as a minimal npm workspace containing React, TypeScript, Vite, Vitest, ESLint, and exact npm lock data.
- `apps/desktop/src-tauri` as the only production composition root, depending inward on `voice-application` and using Tauri 2 without provider, persistence, model, or Windows-adapter integration.
- A minimal project-authored SVG and generated Windows ICO required by Tauri's Windows resource build; it is a placeholder build asset, not a settled product logo.
- A simple frontend page that identifies the M2 skeleton and does not imply dictation features are implemented.
- A model-manifest schema/policy and dependency-free validator with synthetic pass/fail tests. No actual model manifest or model artifact is approved or added.
- A dependency-license review record, updated third-party notices, and fail-closed automated Cargo/npm license checks.
- GitHub Actions for portable Rust checks on Windows, macOS, and Linux; Windows desktop build; frontend build/lint/test; license/model-manifest checks; and tracked-file secret-pattern checks.
- README and applicable status documentation updates needed to describe the implemented M2 baseline truthfully.

## Out of scope

- M3 domain values, identifiers, state machine, ports, fake adapters, session services, or lifecycle behavior.
- Provider, local-ASR, persistence, credential, model download, platform, Windows API, audio, shortcut, target, clipboard, injection, tray, overlay, history, settings, or application-profile functionality.
- Empty future provider, history, local-ASR, or platform crates.
- Tauri commands/events beyond the framework bootstrap; React-owned orchestration; global mutable session state.
- Model weights, model downloads, native third-party binaries committed to the repository, third-party/product-branded icons, fonts, sample media, or product assets beyond the required project-authored M2 build icon.
- Packaging, signing, installer generation, publishing, paid provider calls, or application auto-update.
- Changes to accepted product defaults, architecture direction, licensing policy, milestone order, or merge ownership.

## Ownership

The primary agent has exclusive write ownership for the new M2 workspace, source, test, configuration, CI, policy, review, and lock files, plus the narrowly required README, third-party-notice, testing/licensing-policy, master-plan status, and task-plan updates.

No concurrent writer is assigned. Existing unrelated files and user changes must be preserved. The branch must contain only M2 work.

## Architecture and dependency direction

The allowed production dependency direction is:

```text
React UI -> Tauri desktop composition root -> voice-application -> voice-ports -> voice-core
```

- `voice-core` has no dependency on another workspace crate and contains no `cfg(windows)`.
- `voice-ports` depends only on `voice-core`.
- `voice-application` depends only on `voice-core` and `voice-ports`.
- `apps/desktop/src-tauri` may depend on `voice-application` and Tauri as the composition root.
- No portable crate depends on Tauri, React, a provider SDK, a platform API, or an adapter.
- React only renders the static M2 state. It owns no Dictation Session state or workflow.
- The M2 crates expose no speculative domain or port API; crate-level documentation and metadata tests are sufficient to establish the build boundary.

## Security, privacy, and licensing

- No fixture, UI text, source, workflow, or test may contain a real credential, endpoint, Prompt, transcript, Hotword, audio sample, account identifier, application identity, or private path.
- No network call exists in product code. CI only retrieves toolchains and locked public dependencies.
- Direct Rust dependencies are limited to Tauri and its build helper. Direct npm dependencies are limited to React plus the build, type-check, lint, test, and Tauri CLI toolchain needed for the shell.
- Exact resolved versions and integrity hashes are committed in `Cargo.lock` and `apps/desktop/package-lock.json`.
- `cargo-deny` must deny unlicensed, copyleft-disallowed-by-policy, unknown registry/source, yanked, and explicitly denied licenses; only reviewed GPL-3.0-only-compatible license expressions may pass.
- The npm license validator reads every installed package's declared license, allows only explicitly reviewed compatible SPDX expressions, and fails on missing, unknown, custom, malformed, or denied declarations.
- The dependency review records authoritative project sources, versions, selected license branches, build/runtime role, redistribution posture, review date, and invalidation conditions. Automated inventory is evidence and does not replace this direct review.
- `THIRD_PARTY_NOTICES.md` must distinguish source/build dependencies fetched from public registries from artifacts actually committed or bundled. M2 commits no model or native runtime binary.
- Model-manifest validation requires exact identity/version/source/license/size/SHA-256/distribution fields and rejects unknown fields or placeholder hashes. The absence of model manifests remains valid because M8 approval has not occurred.

## State and failure behavior

M2 implements no Dictation Session state or provider/platform failure behavior. Its failure contract is limited to build automation:

- malformed source, lint violations, type errors, failed tests, or desktop build errors fail their job;
- unknown, missing, or denied dependency-license data fails closed;
- a malformed or incomplete future model manifest fails closed;
- paid provider calls and large model downloads are absent;
- failures do not create or mutate user data because no product storage or capture path exists.

## Dependency review baseline

The initial direct-version choices are fixed for reproducibility and will be checked against the generated lockfiles before acceptance:

| Component | Version | Role | Declared license/source reviewed 2026-08-12 |
| --- | --- | --- | --- |
| Rust stable | 1.97.1 | compiler, formatter, Clippy | Rust project distribution metadata from `static.rust-lang.org`; toolchain only, not bundled |
| `tauri` | 2.11.5 | desktop composition root | Apache-2.0 OR MIT; `github.com/tauri-apps/tauri` |
| `tauri-build` | 2.6.3 | Tauri build script | Apache-2.0 OR MIT; `github.com/tauri-apps/tauri` |
| `react` / `react-dom` | 19.2.8 | static shell rendering | MIT; `github.com/facebook/react` |
| `@tauri-apps/cli` | 2.11.4 | desktop development/build CLI | Apache-2.0 OR MIT; `github.com/tauri-apps/tauri` |
| `vite` | 8.2.1 | frontend build/dev server | MIT; `github.com/vitejs/vite` |
| `@vitejs/plugin-react` | 6.0.5 | React transform | MIT; `github.com/vitejs/vite-plugin-react` |
| `typescript` | 6.0.3 | frontend type checking | Apache-2.0; `github.com/microsoft/TypeScript` |
| `vitest` | 4.1.10 | frontend tests | MIT; `github.com/vitest-dev/vitest` |
| `eslint` / `@eslint/js` | 10.8.1 / 10.0.1 | frontend lint | MIT; `github.com/eslint/eslint` |
| `typescript-eslint` | 8.67.0 | TypeScript ESLint integration | MIT; `github.com/typescript-eslint/typescript-eslint` |
| `globals` | 17.9.0 | ESLint environment data | MIT; `github.com/sindresorhus/globals` |
| React type packages | 19.2.18 / 19.2.4 | TypeScript declarations | MIT; `github.com/DefinitelyTyped/DefinitelyTyped` |
| `@types/node` | 24.13.3 | Node/CI TypeScript declarations | MIT; `github.com/DefinitelyTyped/DefinitelyTyped` |
| `prettier` | 3.9.6 | frontend/script/workflow formatting | MIT; `github.com/prettier/prettier` |
| `cargo-deny` | 0.20.2 | Rust license/source/advisory policy | Apache-2.0 OR MIT; `github.com/EmbarkStudios/cargo-deny` |
| GitHub `checkout` / `setup-node` | pinned v6 tag commits | CI source checkout/Node setup | MIT; official `github.com/actions/*` actions, commit-pinned |

The final review must inspect every resolved transitive license and source through the lockfiles and fail-closed checks. Any unexpected license or source pauses acceptance until documented.

## Implementation steps

1. Commit this authoritative M2 task plan before product-code changes.
2. Create the root Cargo workspace, pin Rust 1.97.1 with rustfmt/Clippy, and configure workspace package/lint metadata.
3. Add the three portable library crates with only boundary documentation, inward manifest dependencies, and metadata/build baseline tests.
4. Create the minimal Tauri 2 composition root and configuration without commands, plugins, platform adapters, or packaging; add only the project-authored SVG/ICO required by the Windows resource build.
5. Create the React/TypeScript/Vite shell, Vitest render test, ESLint configuration, exact npm engines, and deterministic npm lockfile.
6. Add the model-manifest schema, validator, synthetic validator tests, and policy documentation.
7. Add fail-closed npm and Cargo license policies, resolve the exact lock graphs, review all unexpected licenses/sources, and record the accepted results.
8. Add commit-pinned GitHub Actions for portable Rust, Windows desktop, frontend, licensing/model manifests, and tracked-file secret patterns.
9. Update README, THIRD_PARTY_NOTICES, testing/licensing documentation, and M2 status without claiming later functionality.
10. Install the pinned local Rust toolchain when needed, run the complete validation matrix, fix in-scope failures, and review the complete diff for scope, privacy, architecture, and licensing.
11. Commit the coherent validated implementation, fetch and recheck divergence, push the task branch, and open a Ready for review Pull Request. Never merge it.

## Tests and validation

Run from the repository root unless a working directory is stated:

```text
git status --short --branch
git diff --check
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo deny check
npm ci --ignore-scripts        (apps/desktop)
npm run format:check           (apps/desktop)
npm run lint                   (apps/desktop)
npm run test                   (apps/desktop)
npm run build                  (apps/desktop)
npm run licenses:check         (apps/desktop)
npm run models:check           (apps/desktop)
npm run tauri build -- --no-bundle  (apps/desktop, Windows)
```

Also validate:

- `cargo tree` contains only inward workspace edges and reviewed registry dependencies;
- `git grep -n 'cfg *(windows)' -- crates/voice-core` returns no matches;
- no future adapter/provider/platform crate or model manifest/artifact exists;
- the npm and Cargo validators are exercised with synthetic missing/unknown/denied license or malformed-manifest cases and fail non-zero;
- workflows contain no paid provider call, model download, secret value, mutable third-party action tag, or automatic release/publish step;
- exact lockfiles are committed and `npm ci`/`cargo --locked` paths are used in CI;
- all relative documentation links resolve;
- the complete diff contains no credential, private endpoint, complete Prompt/transcript/Hotword/audio, account identifier, private application identity, or complete private path;
- `THIRD_PARTY_NOTICES.md` matches the resolved dependency review and does not imply that a model is approved;
- Windows desktop build evidence is recorded separately if the local host cannot provide a required native prerequisite.

## Acceptance criteria

- The Cargo workspace contains exactly the three portable foundation crates plus the Tauri desktop member; no speculative provider/platform/history/local-ASR crate exists.
- All three portable crates compile, lint, and test without Windows conditionals in `voice-core` or outward dependency edges.
- The desktop shell builds on Windows and displays only an honest M2 skeleton state.
- Frontend type-check/build, lint, formatting, and Vitest pass from an exact npm lockfile.
- Rust formatting, Clippy, check, and tests pass from an exact Cargo lockfile.
- GitHub Actions cover common Rust crates on Windows/macOS/Linux, Windows desktop build, frontend checks, dependency-license checks, model-manifest validation, and tracked-file secret patterns.
- Dependency checks fail closed for denied, unknown, missing, or unreviewed license/source declarations, and all resolved exceptions are documented rather than silently allowed.
- The model-manifest validator rejects incomplete, unknown-field, placeholder-hash, or invalid-license manifests and approves no model by default.
- README, testing/licensing docs, dependency review, and third-party notices truthfully describe the M2 state and dependency set.
- No credentials, sensitive content, provider calls, model weights, paid calls, telemetry, auto-update, server component, or M3 behavior is introduced.
- Required validation passes, or any environment-blocked check is reported precisely without claiming the affected acceptance condition.

## Rollback and recovery

M2 introduces only source, manifests, lockfiles, local development configuration, and CI. It creates no schema, migration, credential, model, audio, transcript, or user-data state. Before merge, corrections use additive commits on the task branch without rewriting shared history. Abandoning M2 means leaving the branch and Pull Request unmerged for the user; it does not require data migration or cleanup.

## Verification record

Local verification completed on Windows on 2026-08-12 with Rust 1.97.1, Node.js 24.15.0, npm 11, and cargo-deny 0.20.2.

- `git diff --check`, Rust formatting, locked workspace check, Clippy with warnings denied, and locked workspace tests passed.
- `cargo deny check` passed for the reviewed `x86_64-pc-windows-msvc` graph; the synthetic unknown-license case failed closed as required.
- A clean `npm ci --ignore-scripts` followed by Prettier, ESLint, Vitest, the production frontend build, npm license/source/integrity checks, and model-manifest checks passed.
- Tracked and untracked source secret-pattern tests/scanning passed, and all relative Markdown links and local anchors resolved.
- `cargo tree` confirmed the exact inward workspace edges and no external dependency in the three portable crates beyond their declared workspace edges.
- `npm run tauri build -- --no-bundle` produced the Windows desktop executable. A launched-build visual check confirmed the title, honest M2-only message, no-session state, and expected accessible document structure; the application was then closed.
- `npm audit` reported zero known vulnerabilities. No model manifest/artifact, provider/platform/history/local-ASR crate, paid call, credential path, telemetry, packaging, publishing, or M3 behavior was introduced.
- Review follow-up on 2026-08-12 replaced model-license substring blocking with an exact allowlist initially containing only `Apache-2.0`; synthetic AGPL and proprietary/research-only expressions now fail closed. Repository-relative validation now applies both POSIX and Windows path semantics to artifact and review-evidence paths, with Windows absolute-path regressions covered by tests.

## Executor Brief

Implement M2 on `codex/m2-workspace-ci-skeleton` exactly as specified in this plan.

Read the repository guidance and all source decisions named above. Preserve unrelated work and stop on any authoritative conflict. Create only `voice-core`, `voice-ports`, `voice-application`, and the Tauri desktop member. Keep the portable crates free of Tauri, React, Windows/platform/provider types, `cfg(windows)`, speculative domain APIs, and global mutable state. The desktop composition root and React page must remain a truthful build skeleton with no dictation orchestration or product behavior.

Pin exact tool/dependency versions in lockfiles. Add fail-closed Rust/npm license and model-manifest gates, record the dependency review, update notices, and use commit-pinned official GitHub Actions. Do not add providers, adapters, history, credentials, models, assets, packaging, publishing, paid calls, telemetry, server behavior, auto-update, or M3 state-machine work.

Run every validation in the plan, including negative tests for unknown/denied licenses and malformed model manifests. Report exact command results, changed files, dependency/license findings, desktop-build evidence, residual risks, and anything unverified. Do not merge a Pull Request.
