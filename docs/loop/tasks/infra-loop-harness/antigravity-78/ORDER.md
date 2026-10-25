# ORDER: infra-loop-harness, antigravity-78 (reconcile read-binding-on-lawful-read prep sheet headings so uor-harness check passes)

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file and prep-sheet gate maintenance). Base: `616384cacf190b6c174d45b5e79d0b176649b925`.

## Goal

Fix the repository-wide `uor-harness check` failure (`error prep.sheet_shape`) on `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` (`missing block(s): Base artifact, Data split, Steps, Cost, Minimum test power`) and `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` (`missing block(s): Stop rule` under `## 6. Cost, abort criteria, and dispatch gate`) that currently causes the GitHub Actions workflow `"harness rules and tests"` (`harness.yml`) to fail on every PR and causes `uor-harness loop next` to classify open PRs as `GO repair #<pr> — check "harness rules and tests" is FAILURE`:

1. Add `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` to `scope:` in `docs/roadmap/items/infra-loop-harness.md` (`#2904`).
2. Reconcile the section headings of `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` (written in `antigravity-60` for `#2889`) with `REQUIRED_BLOCKS` in `crates/uor-r4-harness/src/prep.rs:30-39` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS: FIT`) without altering any registered arm, bar, control, or stop rule.
3. Accept `"abort"` alongside `"stop"` for the `"stop"` heading needle in `crates/uor-r4-harness/src/prep.rs` and add `the_real_sheets_pass_the_shape_rule` in `crates/uor-r4-harness/src/preprules.rs` so `cargo test -p uor-r4-harness` guards all committed sheets in `docs/harness/prep/` against `prep.sheet_shape` regressions.

## Cancel probe (run first)

Class: `instrument` (CI gate / `prep.sheet_shape` check at `616384cacf190b6c174d45b5e79d0b176649b925`):
1. `uor-harness loop claim infra-loop-harness --lab-topic fix-read-binding-sheet` refused solely because `PR #3169` (`codex-1`, idle draft) holds the `infra-loop-harness` umbrella. Verified via `gh pr list --state open --json number,files` that neither `#3169` nor `#3225` touches `docs/roadmap/items/infra-loop-harness.md`, `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`, `crates/uor-r4-harness/src/prep.rs`, or `crates/uor-r4-harness/src/preprules.rs` (`0` overlapping files). Claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella exception.
2. Verified via `uor-harness check` at `616384cacf190b6c174d45b5e79d0b176649b925` that `error prep.sheet_shape` is the only `error` across all 258 rules (`258 rules, 227 failed (0 error, 226 warn)` once fixed).

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md`
- `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`
- `crates/uor-r4-harness/src/prep.rs`
- `crates/uor-r4-harness/src/preprules.rs`
- `docs/loop/tasks/infra-loop-harness/antigravity-78/**`

## Done when

- `uor-harness check` reports `0 error` across all 213 rules (`prep.sheet_shape` passes on all 64 pre-registration sheets);
- `~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass (`References #2904`, `References #2889`).

## Stop and ask if

- Any file outside the declared scope is needed, or any gate in `uor-harness loop gates --full` fails.
