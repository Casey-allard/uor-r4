# #788 — infra: local register gates poisoned by stale-worktree builds — resolve the repo root at runtime; audit-limits must fail on an empty walk

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:45Z
- **Updated:** 2026-08-18T18:40:45Z
- **Closed:** 2026-08-18T18:40:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/788
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The local register gates can silently lie after worktree work, found the hard way by the 2026-08-18 baseline audit (AUD-VER-001/-002). With rlibs in the shared `target/` compiled inside a since-deleted worktree (`.worktrees/i704attention`):

- R1 `check-model` and R4 `audit-deferral` **errored** (`reading /Users/…/.worktrees/i704attention/model/ledger.toml: No such file or directory`),
- R2/R3 `repo-conformance::registered` **failed 29/29** (`features/suites` NotFound — the directory exists in the real tree),
- R5 `audit-limits` **"passed" in 0 s against a repository root that did not exist** — a walk over nothing reporting success,
- and because `cargo test` is **fail-fast at the test-binary level**, the one poisoned `registered` binary silently prevented every later test binary (including the whole core suite and BDD) from running in the workspace pass.

Root cause: `crates/repo-model/src/lib.rs:164` and `crates/repo-conformance/tests/registered.rs:13` resolve the repo root from **compile-time** `env!("CARGO_MANIFEST_DIR")`, which a cached artifact carries from wherever it was built. `cargo clean -p repo-model -p repo-conformance -p xtask` clears it (note: cleaning xtask alone is NOT enough — the baked path lives in the repo-model rlib). CI is unaffected (fresh checkouts); every local batch-flow worktree session risks it.

## Scope

1. Resolve the repo root at **runtime**: `std::env::var("CARGO_MANIFEST_DIR")` (cargo sets it for `cargo run`/`cargo test` at runtime) with a `current_dir` fallback for direct binary invocation — in repo-model, `registered.rs`, and any xtask path resolution.
2. `audit-limits` (and any sibling walker) must **fail on an empty scan set** — count files walked, refuse zero. "No violations" and "no files" must not share a representation.
3. Gate-script hygiene: use `--no-fail-fast` on workspace test runs in local gate scripts/docs so one poisoned binary cannot hide the rest; add the interim `cargo clean` note to AGENTS.md "Things that bite".

## DoD

Gates pass from a fresh worktree AND from main with deliberately-stale sibling artifacts present; a seeded empty-walk demo fails; AGENTS.md note landed.

**Next action:** patch the repo-model root function + add the walked-file-count guard (one small PR).


## Comments (1)

### comment 5332507847 — 2026-08-18T18:40:44Z — Casey-allard

Closing with records: PR #795 merged (2026-08-18T18:38Z).

- Runtime repo-root resolution: `repo_model::repo_root()` walks up from the process CARGO_MANIFEST_DIR (cwd fallback for direct invocation, loud failure otherwise); repo-conformance's `registered.rs` + `bdd.rs` now route through it; xtask inherited the fix (its baked path lived in the repo-model rlib, as the audit found).
- Empty-walk refusal in both audit walkers (R5 `shipped_sources`, R4 deferral gather) with seeded falsifier tests — the pre-fix behavior on the same existing-but-empty fixture root was a 0-second vacuous PASS (AUD-VER-001).
- AGENTS.md "Things that bite": stale-worktree rlib hazard + `cargo clean -p repo-model -p repo-conformance -p xtask` recovery + `--no-fail-fast` on local workspace runs (AUD-VER-002).

Verified: all register gates green via cargo and via direct binary invocation from the repo root; direct invocation from an unrelated directory fails loudly; falsifiers fail red on the seeded empty walk. DoD's stale-sibling-artifact leg is structurally closed (no compile-time path capture remains in the gate crates); the fixture-loading tests that still use compile-time env!() are recorded in AGENTS.md as the same hazard class (out of this issue's scope). Unassigning.


---
_Comment/review ids in this file: 5332507847_

