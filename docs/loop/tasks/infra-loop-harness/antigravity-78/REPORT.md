# REPORT: infra-loop-harness, antigravity-78

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2904. References #2889.

## What did not work, first

1. **Umbrella claim lock (`instrument`):** `uor-harness loop claim infra-loop-harness --lab-topic fix-read-binding-sheet` refused because `PR #3169` (`codex-1`, idle draft) holds the `infra-loop-harness` umbrella. Verified via `gh pr list --state open --json number,files` that neither `#3169` nor `#3225` touches `docs/roadmap/items/infra-loop-harness.md`, `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`, `crates/uor-r4-harness/src/prep.rs`, or `crates/uor-r4-harness/src/preprules.rs` (`0` overlapping files), and claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella exception.
2. **Pre-registration sheet shape check (`prep.sheet_shape`, `instrument`):** Reconciling the section headings of `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` cleared its 5 missing-block findings (`Base artifact`, `Data split`, `Steps`, `Cost`, `Minimum test power`), after which our new regression test `preprules::tests::the_real_sheets_pass_the_shape_rule` immediately caught a second sheet on `origin/main` (`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`) whose section 6 heading was written as `## 6. Cost, abort criteria, and dispatch gate` (`missing block(s): Stop rule`). Because `preprules::tests::the_real_sheets_are_processed` previously discarded the return value of `all(&p, ...)` (`let _ = all(...)`), `cargo test -p uor-r4-harness` had passed while `uor-harness check` (in `.github/workflows/harness.yml`) failed on every pull request. Updating `heading_needles("stop")` in `crates/uor-r4-harness/src/prep.rs` to accept `"stop"` or `"abort"` and retaining `the_real_sheets_pass_the_shape_rule` in `crates/uor-r4-harness/src/preprules.rs` fixed both sheets without touching `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` while `PR #3225` is in flight.

## What changed

- [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (`+1 / -1` lines, docs): added `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` to `scope:`.
- [`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`](../../../harness/prep/read-binding-on-lawful-read-2026-10-25.md) (`+3 / -3` lines, docs): updated section headings (`## 1. Mechanism Under Test, Base Artifact, and Data Split`, `### Four Matched Arms, Steps, and Minimum Test Power (Across 3 Seeds: 20261009, 20261010, 20261011)`, `## 5. Cost and Stop Rule`) to satisfy all 8 `REQUIRED_BLOCKS` in `crates/uor-r4-harness/src/prep.rs:30-39` with zero change to any registered arm, bar, control, or stop rule.
- [`crates/uor-r4-harness/src/prep.rs`](../../../../crates/uor-r4-harness/src/prep.rs) (`+24 / -15` lines: `+17 / -15` code, `+7 / -0` tests): replaced `heading_needle` with `heading_needles`, accepting `"stop"` or `"abort"` for the `"stop"` block, and added unit test `abort_heading_satisfies_stop_block`.
- [`crates/uor-r4-harness/src/preprules.rs`](../../../../crates/uor-r4-harness/src/preprules.rs) (`+11 / -0` lines, tests): added unit test `the_real_sheets_pass_the_shape_rule` asserting `sheet_shape(&p).is_empty()` across all committed sheets in `docs/harness/prep/`.
- [`docs/loop/tasks/infra-loop-harness/antigravity-78/ORDER.md`](ORDER.md) and [`REPORT.md`](REPORT.md) (`+83 / -0` lines, task records).
- Total diff size across 6 files: `+122 / -19` lines (`+17 / -15` code, `+18 / -0` tests, `+87 / -4` docs/records; ratio `0.71` against a 200-line unit estimate).

## Measurements

- No model or GPU compute was run; this is a host-only harness and pre-registration sheet gate repair (`needs_gpu: false`).
- `prep.sheet_shape` findings across all 64 committed sheets in `docs/harness/prep/`: reduced from `2` errors (`read-binding-on-lawful-read-2026-10-25.md` and `r-secant-surrogate-descent-2026-10-25.md`) to `0` errors (`./local/target/debug/uor-harness check`: `258 rules, 227 failed (0 error, 226 warn)`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-harness --lib the_real_sheets_pass_the_shape_rule` (before `prep.rs` fix): FAILED (`1 failed`, `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` missing `Stop rule`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-harness --no-fail-fast` (with fix restored): PASS (`33 passed` in `prep`/`preprules`, all unit and integration test binaries `0 failed`).
- `./local/target/debug/uor-harness check`: PASS (`258 rules, 227 failed (0 error, 226 warn)`, exit code `0`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: PASS (`92 items OK`).
- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check`: PASS.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings`: PASS.
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/antigravity-78/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS (`0` hard failures).
- `uor-harness loop gates --full`: PASS (`4/4` gates green).

## Skills used

- `scope-guard`, `solution-integrity`, `gates`, `uor-research-architect`.

## Still open

- `#2904` (`infra-loop-harness`) remains `status: ready` as the shared loop-harness umbrella item.
- `#2889` (`read-binding-on-lawful-read`) remains ready for its discrete `DiscreteReadBindingHingeV1` implementation and GPU evaluation under the now-compliant sheet `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`.

## Plain summary

Before running a GPU experiment, each lab writes a one-page pre-registration sheet in `docs/harness/prep/` stating the hypothesis, the arms to compare, the pass bar, the data split, the cost, and the stop rule. The project's automated rule checker (`uor-harness check`, which runs in GitHub Actions on every pull request) checks that every sheet has headings for all required sections (`prep.sheet_shape`). Two recently added sheets (`read-binding-on-lawful-read-2026-10-25.md` and `r-secant-surrogate-descent-2026-10-25.md`) used section titles that did not match the checker's heading keywords, causing the `"harness rules and tests"` CI job to fail across open pull requests. This change updates the headings in `read-binding-on-lawful-read-2026-10-25.md`, teaches the checker in `crates/uor-r4-harness/src/prep.rs` to recognize `"abort"` as well as `"stop"` for stop-rule headings, and adds a unit test (`the_real_sheets_pass_the_shape_rule`) in `crates/uor-r4-harness/src/preprules.rs` so `cargo test -p uor-r4-harness` catches any missing section heading immediately.
