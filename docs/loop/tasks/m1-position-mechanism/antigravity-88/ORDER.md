# ORDER: m1-position-mechanism, antigravity-88

Lab: antigravity. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). Base: `c695ad21a`.

## Goal

The item's done-when reads:

> A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for "helps order" is not yet registered; step 4 registers it.

Complete the remaining pre-registration and scorer prerequisites noted in `kimi-33/REPORT.md` (`#3111`) and `antigravity-62/REPORT.md` (`#3205`):
1. Add `parse_frozen_slice`, `shuffle_prefix_window` (`C_shuf`), and the paired McNemar / admission / fresh-transfer order-test scorer (`score_position_contrast`, `evaluate_position_order_seed`, `grade_position_order_two_seeds`) with finite unit controls in `crates/uor-r4-discrete/src/order_slice.rs`.
2. Update `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` with the `#3111`/`#3205` GPU `flock_vector` transport definition, `#3205` qualification receipts, the owner's `2026-10-24T03:41:41Z` fresh-V2 bar-term ruling (`#2028` comment `6379430093`), the 3-branch outcome decision table, and the `order_slice` paired scorer.
3. Obtain a fresh-context non-author `TEST FITNESS: FIT` and D68 decision review ([`DECISION.md`](DECISION.md)), record the notebook row, and close `#2843` (`status: done`, `needs_gpu: false`).

## Cancel probe (run first)

1. **Run existing `position_mechanism` and `order_slice` tests:**
   - `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib position_mechanism` -> `10 passed; 0 failed`.
   - `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib order_slice` -> `6 passed; 0 failed`.
2. **Inspect remaining gaps (`kimi-33/REPORT.md:16,64`, `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`):**
   - `order_slice.rs` selects and renders the 1,620-row slice (`f76dc5c7…`), but lacks `parse_frozen_slice`, `shuffle_prefix_window`, and the paired McNemar order-test scorer across full DEV (`14,287`), `P_succ` (`3,847`), and the frozen order slice (`1,620`).
   - `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` §1/§3/§4/§6/§7 predates `#3111` (`460d1e413`) and `#3205` (`e6b34e332`), and does not yet cite the landed GPU read definition, `#3205` receipts, or `#2028` comment `6379430093`'s fresh-V2 replication rule ($b - c \ge \lceil 2\sqrt{b+c}\rceil$).
   - Classification (`CHARTER.md §4.2`): **`instrument` (`GO`, not cancelled)**.

## Plan

1. Commit `ORDER.md` and `DECISION.md` first.
2. Extend `crates/uor-r4-discrete/src/order_slice.rs` with `parse_frozen_slice`, `shuffle_prefix_window`, and the paired McNemar scorer + 4 unit tests.
3. Update `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` and `docs/labs/m1-position-mechanism/README.md`.
4. Run fresh-context non-author `TEST FITNESS: FIT` and D68 decision review, post on `#2843`, set `status: done` on `docs/roadmap/items/m1-position-mechanism.md`, write `REPORT.md` and notebook row, run D66 review, and queue `#3256`.

## Files in scope

- `crates/uor-r4-discrete/src/order_slice.rs`
- `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`
- `docs/labs/m1-position-mechanism/README.md`
- `docs/loop/tasks/m1-position-mechanism/antigravity-88/**`

## Done when

`m1-position-mechanism` (`#2843`) meets its `Done when`: `PositionHolonomyV1`, finite CPU/GPU controls, the frozen 1,620-row order slice and paired McNemar scorer pass `cargo test -p uor-r4-discrete`, `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` carries a fresh-context non-author `TEST FITNESS: FIT` and D68 decision review, and `loop gates --full` and `loop pr-check --local` pass.

## Stop and ask if

- Any change would modify `DRIVER_SOURCE_SHA256` without updating `driver_source_pin.rs`.
- Any mechanism or scorer would use floats (`D32`), squashing (`§3.2`), or matmul (`§3.3`).
