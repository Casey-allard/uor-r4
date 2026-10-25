# ORDER: m1-w2-code-utilization, claude-95 (build and run the code-utilization instrument)

Lab: claude. Item: [m1-w2-code-utilization](../../../../roadmap/items/m1-w2-code-utilization.md) (#3179). Base: `61d6cb0f3`.
deps: the new standalone `docs/labs/m1-w2-code-utilization/instrument/Cargo.lock`. Its dependencies are the path crate `uor-r4-discrete` and the `serde`, `serde_json` and `sha2` crates the workspace already uses. This follows the item rule from claude-93 (#3184, review record of 0915d3ed6) and the precedent `docs/labs/m2-fresh-heldout-panel/instrument/`.

## Goal

`loop next` gives GO on this item (M1 path, unblocks 14). This unit runs the registered sheet `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md` (FIT, round 2) once, on the stored W2 capture.

1. **Build the host instrument.** It is the standalone crate `docs/labs/m1-w2-code-utilization/instrument/`:
   - a detached `[workspace]`, with a read-only path dependency on `uor-r4-discrete` built without `cuda`;
   - exact integers only, with no float type anywhere, tests and timers included.
   - It reuses `w2_capacity`'s public API and reimplements the private pieces, as the item lists.
2. **Run it on the authenticated capture.** The inputs are `run-b.tar` `6aa0a766…` → `w2-out-b/`, and `record.json` `69717971…`.
   - Precedence 0 reproduces every W2 number before any band is read.
   - QA, QB and QC each record one band, and the descriptive arms are reported.
   - Arm R is NOT_RUN as a missing input: the Q1–Q3 checkpoints `a5a22dce…`, `dde889aa…` and `102beed4…` are in neither the HF store's `local-archive/2026-10-20/uor-r4-local.files.txt` nor `archive/storage/`.
3. **Seal and record.**
   - The record `uor-r4.w2-code-utilization/1` goes in the HF store under `results/claude/w2-utilization-2026-10-25/`, with checksums.
   - The lab write-up is `docs/labs/m1-w2-code-utilization/README.md`.
   - One row goes in `docs/research/notebook/m1-discrete-attention.md`.
   - G1 is recorded as registered or declined, per QB.

No GPU, no pod, and no edit under `crates/`.

## Cancel probe (run first)

Class: measurement (host only). At the base:
- the item's scope has no `crates/` glob, and the sheet's last `TEST FITNESS` line is FIT;
- `run-b.tar` and `record.json` authenticate against their pins;
- `docs/labs/m1-w2-code-utilization/` does not exist.

## Files in scope

- `docs/labs/m1-w2-code-utilization/**` (the instrument crate and the write-up)
- `docs/research/notebook/m1-discrete-attention.md` (one row; notebook rows are always allowed)
- `docs/roadmap/items/m1-w2-code-utilization.md` (its status line only, which is always allowed)
- this task folder

## Done when

- **Tests.** The instrument's tests pass (`cargo test` in its folder, with a scratch `CARGO_TARGET_DIR`). They include synthetic-fixture tests for each arm, the floor rule, leave-one-out and each band, plus a real-capture test of precedence 0.
- **The run.** It ran once and reproduced every W2 number, or an instrument fault is recorded instead. Its record states QA, QB and QC bands and is sealed with its sha256 in the HF store.
- **The record.** The notebook row and the lab README are written. If the bands are read, the REPORT says `Closes #3179` and the item reads `status: done`. On an instrument fault, the REPORT says `References #3179`.
- **Checks.** `loop report --check` and `loop pr-check --local` pass, and a fresh-context review counts at the exact head.

## Stop and ask if

- The instrument needs a change to `uor-r4-discrete`. That is a separate item.
- A precedence-0 number fails to reproduce after the instrument is ruled out.
- A result would change a bar, status or gate beyond this item's own status.
