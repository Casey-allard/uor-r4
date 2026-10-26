# ORDER: m1-within-cell-writer, antigravity-94

Lab: antigravity. Item: [m1-within-cell-writer](../../../../roadmap/items/m1-within-cell-writer.md). Base: `441a2eda6`.

## Goal

Repair the `prep.sheet_shape` and `TEST FITNESS: FIT` verdict-line format in [`docs/harness/prep/m1-within-cell-writer-2026-10-26.md`](../../../harness/prep/m1-within-cell-writer-2026-10-26.md) flagged by `harness.yml` (`prep::tests::the_real_sheets_are_readable` and `preprules::tests::the_real_sheets_pass_the_shape_rule`) on PR `#3280`, while preserving all completed deliverables from `antigravity-93` (`6c0f8a22e4568d3245872650c336d5e5d4c873ac` and `a52bbf6c2881cfecbfe7d68f50702813037656f5`).

Finished steps from `antigravity-93`:
- Built and verified the ahead-of-time `nvcc` CUDA module (`within_cell_writer_source.rs`), device wrapper (`within_cell_writer_device.rs`), evaluator (`within_cell_writer.rs`), and CLI runner (`within-cell-writer-run.rs`) at `6c0f8a22e4568d3245872650c336d5e5d4c873ac`.
- Ran `qualify`, `decompose-stage-t`, and `b0` on `1x RTX 5090` (`vast-57824058`) across two independent builds, archived all outputs to `caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/` (`within-cell-writer-20261026.tar.gz` `480916cb96ee86bc2d36091be7c62863e09bea6c9418904ca0eb8879ce1dbfbd`), tore down the pod, and committed the sealed receipts, mechanism brief, notebook row, roadmap item status (`done`), and `antigravity-93/REPORT.md` at `a52bbf6c2881cfecbfe7d68f50702813037656f5`.

## Cancel probe (run first)

1. Run `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness -- prep` in `.worktrees/antigravity-m1-within-cell-writer`: reproduces `prep::tests::the_real_sheets_are_readable` (`docs/harness/prep/m1-within-cell-writer-2026-10-26.md: no verdict line found`) and `preprules::tests::the_real_sheets_pass_the_shape_rule` (`missing block(s): Base artifact, Data split, Steps, Stop rule, Cost, Minimum test power, TEST FITNESS line`).
2. Classification: `instrument` — pre-registration sheet heading and verdict-line formatting (`prep.sheet_shape`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-within-cell-writer*`
- `docs/labs/m1-within-cell-writer/**`
- `docs/loop/tasks/m1-within-cell-writer/**`
- `docs/mechanisms/discrete-learning.md`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-within-cell-writer.md`
- `docs/roadmap/README.md`

## Done when

- `docs/harness/prep/m1-within-cell-writer-2026-10-26.md` carries all required `prep::REQUIRED_BLOCKS` headings (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`) and the `TEST FITNESS: FIT` verdict block.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness` passes in `.worktrees/antigravity-m1-within-cell-writer`.
- `docs/loop/tasks/m1-within-cell-writer/antigravity-94/REPORT.md` passes `uor-harness loop report --check`, `uor-harness loop pr-check --local`, `uor-harness loop gates --full`, and a fresh-context D66 exact-head review.

## Stop and ask if

- Never wait on Casey; resolve any scope/bar/classification question by a fresh-context D68 decision review logged on `#3267`.
