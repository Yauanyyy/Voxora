# Fix history test temporary-directory collisions

## Status

Approved for implementation on `codex/fix-history-test-temp-collision`.

## Objective

Make `history-sqlite` tests allocate collision-resistant temporary roots when
Rust runs tests concurrently on platforms whose wall-clock resolution can return
the same timestamp to multiple tests. Preserve all production persistence and
artifact behavior.

## Source decisions

- Cross-platform persistence validation in
  [`../implementation-plan.md`](../implementation-plan.md) and
  [`../testing.md`](../testing.md).
- Adapter and recovery boundaries in [`../architecture.md`](../architecture.md).
- Agent and Git lifecycles in the repository runbooks.
- CI evidence from PR #5: the macOS `Rust portable crates` job failed at
  `HistorySqlite::open` in `capture_boundary_keeps_only_supplied_nonempty_audio`,
  while the same matrix passed on Ubuntu and Windows and the PR changed no Rust
  source or workflows.

## In scope

- Replace the test-only timestamp-only `temp_root` name with a name containing
  the process ID, wall-clock nanoseconds, and an atomic per-process sequence.
- Add a focused concurrent test proving generated roots are distinct.
- Run formatting, linting, tests, and diff validation appropriate to
  `history-sqlite`.

## Out of scope

- Production SQLite, migration, retention, audio-artifact, or error behavior.
- CI path filtering or workflow changes.
- Retrying tests to hide failures.
- New crates or development dependencies.
- Refactoring test cleanup into a general temporary-directory framework.

## Ownership

The execution agent exclusively owns
`crates/history-sqlite/src/lib.rs` for this implementation. The primary agent
owns this plan, verification, status, Git, push, and Pull Request. The executor
is not alone in the repository and must preserve every unrelated change.

## Architecture and dependency direction

The change stays inside the `#[cfg(test)]` module. It adds no production API or
dependency edge and does not alter `voice-core`, `voice-ports`,
`voice-application`, SQLite schema, or adapter contracts.

## Security, privacy, and licensing

Use only standard-library process and atomic identifiers. Add no dependency,
native component, model, network access, notice entry, sensitive fixture, or
complete private path to logs or tracked evidence.

## State and failure behavior

Each `temp_root` call in one test process must be unique even when calls observe
the same clock value concurrently. Process ID and wall-clock nanoseconds reduce
cross-process/restart collision risk; the atomic sequence guarantees uniqueness
within the process. Existing cleanup remains best-effort and test-local.

## Implementation steps

1. Add one test-only static `AtomicU64` sequence.
2. Include process ID, nanoseconds, and the incremented sequence in `temp_root`.
3. Add a focused concurrent uniqueness test without creating persistent files.
4. Run the exact validation and inspect the diff for production-code changes.

## Tests and validation

Run from the repository root:

```text
cargo fmt --all -- --check
cargo test --locked -p history-sqlite --lib
cargo clippy --locked -p history-sqlite --all-targets --all-features -- -D warnings
git diff --check
git status --short --branch
```

The final CI evidence remains the Windows/macOS/Linux `Rust portable crates`
matrix on the Ready Pull Request.

## Acceptance criteria

- Concurrent `temp_root` calls cannot return the same path within one process.
- Generated names retain a time component and add process/sequence components.
- The previously failing capture-boundary test and all other `history-sqlite`
  library tests pass locally.
- No production code path, dependency, manifest, lockfile, CI workflow, or notice
  changes.
- Required local checks pass and remote macOS evidence is reported accurately.

## Rollback and recovery

Reverting the test-only commit restores the prior helper and has no user-data or
runtime migration impact. Test cleanup continues to remove only each generated
root.

## Executor Brief

Implement only the approved test collision fix in
`crates/history-sqlite/src/lib.rs`. You exclusively own that file for this task
and must preserve unrelated work. In the `#[cfg(test)]` module, add a standard-
library `AtomicU64` counter and make `temp_root` include process ID, current
nanoseconds, and a monotonically incremented per-process sequence. Add a focused
test that invokes `temp_root` concurrently and proves all returned paths are
distinct. Do not change production behavior, introduce a dependency, edit CI,
or broaden cleanup behavior. Run every validation command above and report the
changed file, results, residual risks, and unverified macOS evidence. Do not
commit, push, open a Pull Request, or resolve review threads.
