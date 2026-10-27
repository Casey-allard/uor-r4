# ORDER: m1-close-saved-model, antigravity-110

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `a842d455f`.

## Goal

The item's done-when reads:

> One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

## Cancel probe (run first)

1. Check `export_and_verify_trunk_artifact` in `crates/uor-r4-discrete/src/trunk.rs:2012-2048` to verify whether it emits the 19-table `uor-r4.lut-discrete-trunk/1` container (`export_trunk_d11`) or the 4-table Stage 2 container (`export_d11`).
2. Check `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` in `docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs:51-61` and `fresh_rows` in `src/lib.rs:183-193` against the stored `w2-inputs/2026-10-25/driver-data/` files and `codex-attention-dose1-fresh-v2-preservation.tar`'s `report.json` (`df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`).

**Cancel probe result (`classification: instrument`):**
1. `export_and_verify_trunk_artifact` (`trunk.rs:2021`) still called 4-table `export_d11(&base_snap)` instead of 19-table `export_trunk_d11(&base_snap, step_table, norm_frames, tables)`.
2. `fresh_transfer.rs:51-61` carried hand-expanded trailing hex chars on `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` instead of the exact digests from `report.json` (`df5d074a…`) and `w2-inputs/2026-10-25/driver-data/` (`0919e90f953e…`, `a838bb1a67e9…`, `3dee858f84b7…`, `b5e7352d67d6…`), and `fresh_rows` (`lib.rs:189`) called `config.validate()` without normalizing `Split::Fresh` to `Split::Dev`. Repaired with failing-first tests before launching the GPU pod.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-close-saved-model*`
- `docs/labs/m1-close-saved-model/**`
- `docs/loop/tasks/m1-close-saved-model/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-close-saved-model.md`
- `docs/roadmap/README.md`
- `ROADMAP.md`

## Done when

1. `export_and_verify_trunk_artifact` and `trunk-run repack-d11` deterministically export and round-trip verify the two-seed 19-table `uor-r4.lut-discrete-trunk/1` artifacts (`t16-frozen-zero-seed1-trunk.d11` and `t16-frozen-zero-seed2-trunk.d11`) from the sealed Stage T `T16_fz` checkpoints and GPU-captured `mw-balp/train` streams (`Gate R` verified against `#2844`).
2. `fresh_transfer.rs` pins the exact `mw-balp/train` and `ft/dev` SHA-256 digests (`3dee858f84b7…`, `b5e7352d67d6…`, `0919e90f953e…`, `a838bb1a67e9…`), refuses all fresh slices on `dev_rows` / `rows_from_corpus`, and accepts `Split::Fresh` or `Split::Dev` on `fresh_rows` once `validate_fresh_transfer_gate` passes.
3. Laptop `d11-scorer toks` (`B3`) and two-build GPU `d11-scorer score --reference` (`ft/dev`, `14,287` rows) + `d11-scorer fresh-score` (`fresh_v2`, `14,479` rows, `0b7e754c…` / `a1cda529…`) run on 1x RTX 5090 after fresh-context FIT and D67 pre-run review, with all receipts archived to `caseyallard/uor-r4-store` and recorded in `m1_close.rs`, `docs/labs/m1-close-saved-model/`, and `docs/research/notebook/m1-discrete-attention.md`.

## Stop and ask if

- Never stop to wait; if any GPU integrity check (`refused_rows > 0`, `reference_mismatches > 0`, or `b4_pass == false`) fails, classify as `instrument` and repair before calling any capability outcome; if `G-Fresh-Full` or `G-Fresh-Psucc` misses on the GPU, record the exact outcome (`FAIL_BAR_E_CLOSE_CONFIGURATION` or `PASS_BAR_E_FRESH_TRANSFER`) honestly in `m1_close.rs` and `REPORT.md`.
