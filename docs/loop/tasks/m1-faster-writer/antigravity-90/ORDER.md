# ORDER: m1-faster-writer, antigravity-90

Lab: antigravity. Item: [m1-faster-writer](../../../../roadmap/items/m1-faster-writer.md). Base: `6f38dd393865a0b4320a2e28bee3be675130dc16`.

## Goal

The item's done-when reads:

> A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. How far below the scan counts as enough is not yet registered; register it in step 4.

Complete steps 4 and 5 of `m1-faster-writer` (`#2840`):
1. Register the step-4 `B1`/`B2` bar under `D68`, finalize `docs/harness/prep/m1-faster-writer-2026-10-26.md` (superseding `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`) with all open choices pinned (`V = 4,096` and `16,384` seeded tables + 14,287-row `DEV` real-read panel on `A8` `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` and `PrimeZeta` `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`), and obtain fresh-context `FIT`, `D68`, and `D67` reviews.
2. Implement the ahead-of-time CUDA CSR bucketed writer (`bucket_source.rs`, `writer_bucket_device.rs`, `writer_bucket_qualification.rs`, `src/bin/writer-bucket-qualify.rs`) without altering `discrete.cu` or `driver_source_sha256`.
3. Run two-build CUDA qualification on `1x RTX 5090`, archive to `caseyallard/uor-r4-store:results/antigravity/m1-faster-writer-2026-10-26/`, record receipts and mechanism/notebook updates, and mark `m1-faster-writer` `done` (`Closes #2840`).

## Cancel probe (run first)

Ran on `/Users/casey.allard/uor-r4/.worktrees/antigravity-m1-faster-writer` (`47115b060952a3b1facc2aa43f6e3b956be6158d`):
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls`: 6 passed, 0 failed (`claude-34` host CSR index and finite controls intact).
- `ls crates/uor-r4-discrete/src/bucket_source.rs crates/uor-r4-discrete/src/bin/writer-bucket-qualify.rs`: both absent.
- Classification: **mechanism** work remaining (step 4 bar/sheet + step 5 CUDA writer and qualification). Proceeding.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-faster-writer*`
- `docs/mechanisms/admission.md`
- `docs/labs/m1-faster-writer/**`
- `docs/loop/tasks/m1-faster-writer/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-faster-writer.md`

## Done when

- `docs/harness/prep/m1-faster-writer-2026-10-26.md` passes `uor-harness check` (`prep.sheet_shape`) and holds a fresh-context `TEST FITNESS: FIT`, `D68` bar decision review (`[antigravity] owner log:` on `#2840`), and `D67` pre-run review.
- `crates/uor-r4-discrete` compiles and passes all host tests (`--no-fail-fast`) and stub-`nvcc` `--features cuda --all-targets` check with `driver_source_sha256` unchanged.
- Two independent nvcc builds (`Build A` and `Build B`) of `writer-bucket-qualify` on `1x RTX 5090` produce identical `B1` (`0` mismatches across seeded `V ∈ {64, 256, 4096, 16384}`, edge cases, 64-step move sequence, and 14,287-row `DEV` real-read panel on `A8` and `PrimeZeta`), `B2` (`token_cell_calls_per_row == 0`, `cell_distance_evals_per_row <= 257`, `token_distance_evals_per_row <= K * max_c |B[c]| + 2`, and `>= 10x` reduction vs full scan at `V = 4,096` and `V = 16,384`), and 4/4 planted-fault detections.
- Results are archived in `caseyallard/uor-r4-store:results/antigravity/m1-faster-writer-2026-10-26/`, documented in `docs/labs/m1-faster-writer/README.md`, `docs/mechanisms/admission.md`, `docs/research/notebook/m1-discrete-attention.md`, and `docs/roadmap/items/m1-faster-writer.md` (`status: done`).

## Stop and ask if

- Any parity control (`B1`) shows a non-zero mismatch between the CSR bucketed CUDA writer and the full-scan `discrete.cu` writer (`writer` at lines `375-417`). Classify instrument vs mechanism first; never count an instrument bug as a mechanism negative.
- Pod spend approaches the `$2` task ceiling or the cluster-wide `4 pods` / `$8/h` caps.
