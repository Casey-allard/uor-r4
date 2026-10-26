# ORDER: r-vocab-4096-vs-16384, antigravity-91

Lab: antigravity. Item: [r-vocab-4096-vs-16384](../../../../roadmap/items/r-vocab-4096-vs-16384.md). Base: `5a104c6f8`.

## Goal

Complete `r-vocab-4096-vs-16384` (`#2887`, "Compare 4,096 and 16,384 tokenizer pieces", unlocked by `#2840` `m1-faster-writer` in `5a104c6f8`). The item's done-when reads:

> Matched runs at 4,096 and 16,384 pieces with the same registered bars, recorded as notebook rows.

## Cancel probe (run first)

1. **Verify existing matched `V = 4,096` vs `V = 16,384` runs and tests before any new GPU spend (`CHARTER.md` §3.5, §4.2):**
   - **Matched GPU writer & serving-cost runs (`#2840`, `5a104c6f8`):** Under `docs/harness/prep/m1-faster-writer-2026-10-26.md` (`TEST FITNESS: FIT`, `D68`, `D67`), `writer-bucket-qualify` ran matched GPU evaluations at `V = 4,096` and `V = 16,384` (`B = 4`, `K in {1, 8, 16}`, 3 seeds x 32 rows = 96 rows per `(V, K)` plus uniform `16` vs `64` tokens/cell at `K = 8`) across `Build A` (`qualification-build-a.json`, `37419bdc…`) and `Build B` (`qualification-build-b.json`, `df583e45…`) on `1x RTX 5090` (`vast-57796936`), recorded in `docs/labs/m1-faster-writer/README.md` and `docs/research/notebook/m1-discrete-attention.md` row 79. Re-verify finite controls with `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls --no-fail-fast`.
   - **Matched BPE tokenizer & numeric-boundary runs (`#2872`, `number-aware-tokenizer`):** Run `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-tokenizer --no-fail-fast` (`23/23` pass, including `number_aware_three_arm_comparison_and_digit_piece_census` at `V = 4,096` and `V = 16,384` across `ctrl1`, `ctrl2`, `trt` on `170,176` B corpus `05f21705…`).
   - **Discrete-driver (`B = 4`) capacity audit (`B3`):** Verify the 5 `V <= 4,096` / `8,192` guards in `address_cells.rs:322,462`, `lib.rs:83`, `prime_zeta.rs:108`, `discrete.cu:633`, and `d11_export.rs:149,386` / `discrete_d11.rs:667`.
   - **Result:** `23 / 23` `uor-r4-tokenizer` and `6 / 6` `writer_bucket_controls` tests pass (`classification: mechanism` on matched `4,096` vs `16,384` GPU writer & BPE comparison; `classification: instrument` on the 5 discrete-driver `V <= 4,096` / `8,192` guards).

## Files in scope

- `docs/harness/prep/r-vocab-4096-vs-16384*`
- `docs/labs/r-vocab-4096-vs-16384/**`
- `docs/loop/tasks/r-vocab-4096-vs-16384/**`
- `docs/roadmap/items/r-vocab-4096-vs-16384.md` (status line)
- `docs/roadmap/README.md` (rendered via `xtask roadmap render`)

## Done when

1. Cancel probe passes (`23/23` `uor-r4-tokenizer`, `6/6` `writer_bucket_controls`).
2. `docs/harness/prep/r-vocab-4096-vs-16384-2026-10-26.md` and `DECISION.md` pass fresh-context `TEST FITNESS: FIT` and `D68` decision review, with `[antigravity] owner log:` posted on `#2887`.
3. `docs/labs/r-vocab-4096-vs-16384/README.md` and `REPORT.md` record the matched `4,096` vs `16,384` runs and notebook rows, `r-vocab-4096-vs-16384.md` is set to `status: done`, and all gates pass.

## Stop and ask if

- Any check or review requires touching files outside the declared scope, or changing a `CHARTER.md` §3 constraint or GPU cap.
