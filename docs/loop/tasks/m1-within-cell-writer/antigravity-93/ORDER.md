# ORDER: m1-within-cell-writer, antigravity-93

Lab: antigravity. Item: [m1-within-cell-writer](../../../../roadmap/items/m1-within-cell-writer.md). Base: `441a2eda6`.

## Goal

The item's done-when reads:

> - Saved Stage T outputs decompose the 455 versus 2,140 gap into admission, top-8 candidate coverage, within-cell L1 ranking and DEV read-code coverage, including a TRAIN-only within-cell reranker control.
> - B0 compares current single-pass, nearest-endpoint single-pass, two-pass, frozen/permuted frames, a 16→16 frame remap, lookup controls and the within-cell reranker on the same re-derived epoch-0 FullVectorV1 table under OuterL1V1 and frozen DEV rows.
> - The primary comparison uses the strict paired rule `d > 24 && d^2 > 4(b+c)` on eight-bit interior-key accuracy over the pre-registered intersection of admitted rows. Total top-1, intersection-admitted top-1, admission, code coverage, non-inertness, comparison counts, resident bytes and throughput are reported separately.
> - New-law controls pin bounded arithmetic, negative gates, multiple active levels, `s[12]=1/2`, direct copy succession, frame selection on TRAIN/validation, and old-lineage parity. Every negative gate is shown failing before its implementation passes.
> - #3128 and #3129 close only when their acceptance controls and the relevant matched receipts are sealed. A negative closes only the tested configuration. Training requires a positive B0 plus a new sheet, component ablations, throughput projection and distinct data-order or initialization replication. D11 is a separate later unit.

## Cancel probe (run first)

1. Verify that `m1-bvg-k-stage-t-training` (`#2836`) is `done` and its sealed Stage T archive (`caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz`, `sha256 = c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`) is present.
2. Run a host decomposition probe on `T16_fz` (`epoch = 420`) and `T16_rel` (`epoch = 678`):
   - On `T16_fz` (`5,009` admitted DEV rows): `OuterL1V1` ranks the target in `top-8` on `1,484` rows (`3,525` outside `top-8`) and `top-1` on `455` rows (`1,029` in `top-2..8`), whereas `T_restr(A_W)` scores `2,140` `top-1` (`4,006` `top-8`) and `G_restr(A_W)` scores `3,084` `top-1`.
   - On `T16_rel` (`3,827` admitted DEV rows): `OuterL1V1` scores `371` `top-1` (`1,124` `top-8`, `2,703` outside `top-8`), against `1,602` for `T_restr(A_W)` and `2,364` for `G_restr(A_W)`.
3. Classification: `mechanism` — the `2,140 - 455 = 1,685` gap sits in within-shortlist / within-cell token selection (`#3267`, `#3128`, `#3129`).

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

- Saved Stage T outputs (`T16_fz` and `T16_rel`) decompose the `455` vs `2,140` gap into admission, `top-8` coverage, within-cell vs cross-cell L1 ranking, DEV read-code coverage, and TRAIN-only within-cell reranker controls (`R_cell_code`, `R_cell_code_perm`, `R_cell_bigram`, `R_cell_unigram`).
- Ahead-of-time `nvcc` CUDA module (`within_cell_writer_source.rs`) and evaluator (`within_cell_writer.rs`, `within-cell-writer-run`) implement `ReadLawV2` (`#3128`) and frame-source modes (`#3129`) without touching `discrete.cu` (`driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`).
- All 6 negative gates fail when set aside and pass when active, and finite controls `C1`–`C6` pass on CUDA.
- Pre-registration sheet `docs/harness/prep/m1-within-cell-writer-2026-10-26.md` receives non-author `TEST FITNESS: FIT` and `D67 PRE-RUN REVIEW: PASS` on `#3267`.
- `B0` runs on GPU across two builds on `FullVectorV1` (`2c886736...`) under `OuterL1V1`, archives to `caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/`, tears down the pod, and records `REPORT.md` + fresh-context D66 review.

## Stop and ask if

- Never wait on Casey; resolve any scope/bar/classification question by a fresh-context D68 decision review logged on `#3267`.
- If any finite GPU control fails, stop and repair as `INSTRUMENT` under D31.
- If two-pass fails to beat nearest-endpoint single-pass by `d > 24 && d^2 > 4(b+c)` on 8-bit interior-key accuracy over the admitted intersection, stop the epoch-0 two-pass framing configuration under D22 and record the exact-memory within-cell lookup/reranker successor.
