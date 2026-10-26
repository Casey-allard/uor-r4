# DECISION (D68): Close `m1-position-mechanism` (`#2843`) on Landed Mechanism, Finite CPU/GPU Controls, Paired Order-Slice Scorer, and Updated `FIT` Sheet

- **Date:** 2026-10-25
- **Lab:** `antigravity` (`antigravity-88`, PR `#3256`)
- **Item / Issue:** [`docs/roadmap/items/m1-position-mechanism.md`](../../../../roadmap/items/m1-position-mechanism.md) (`#2843`, Milestone M1 `#2029`)
- **Authority:** `CHARTER.md` §5.1, §5.10 (D68 decision review, logged on `#2843` for the owner)

---

## 1. Whole-Project Context & Milestone Chain

1. **Role in `PLAN.md` (`D77`, `D79`):**
   - Under `D79`, M1 closes on the saved-model path (`m1-close-saved-model`, `#2837`), while M2 starts with `m1-trunk-assembly-and-training` (`#2844`), which depends on `m1-position-mechanism` (`#2843`), `m1-normalization-mechanism` (`#2842`, `status: done`), and `m1-bvg-k-stage-t-training` (`#2836`, `status: done`).
   - `m1-position-mechanism` (`#2843`) is a `needs_gpu: false` design, finite-control, and pre-registration item whose `Done when` (`docs/roadmap/items/m1-position-mechanism.md:24`) states:
     > *"A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for 'helps order' is not yet registered; step 4 registers it."*
   - Its twin item `m1-normalization-mechanism` (`#2842`, `needs_gpu: false`) has the identical `Done when` structure and closed (`status: done`, PR `#3238`) once its discrete mechanism, finite controls, paired DEV scorer, and updated `TEST FITNESS: FIT` pre-registration sheet were complete on `main`.

2. **What is already delivered on `main` for `#2843`:**
   - **Discrete geometric construction & finite CPU controls (`#3018`, `#3111`):** `PositionHolonomyV1` (`PositionMode::{PrefixHolonomyV1, EndpointControlV1, IdentityControlV1}`, `PositionStepTable`, `relative_holonomy`, `apply_frame_to_root`, `shuffle_prefix_roots`) in `crates/uor-r4-discrete/src/position_mechanism.rs` (`10 / 10` unit controls pass).
   - **Owner-registered bar (`#2028` comment `6356034555`, `2026-10-23T02:36:36Z`, and `#2028` comment `6379430093`, `2026-10-24T03:41:41Z`):** Paired one-sided McNemar $b - c \ge F_I = \max(24, \lceil 2\sqrt{b+c}\rceil)$ with non-inferior admission ($\Delta\text{admitted} \ge 0$) for `H_pos(ordered)` vs `C_match(ordered)` and order-shuffle drop $b - c \ge F_I$ for `H_pos(ordered)` vs `C_shuf` across 2 seeds on DEV (`14,287` full, `3,847` `P_succ`, and the frozen 1,620-row order-sensitive slice), plus fresh-V2 transfer replication $b - c \ge \lceil 2\sqrt{b+c}\rceil$ (no 24-row floor).
   - **Frozen 1,620-row order-sensitive DEV slice (`#3046`, `claude-58`):** `docs/labs/m1-position-mechanism/order-slice-v1.tsv` (`SHA-256 = f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`, `N = 1,620` of `14,287` DEV rows) and `crates/uor-r4-discrete/src/order_slice.rs` (`6 / 6` unit controls pass).
   - **GPU read wiring, finite CUDA qualification, `status 10` refusal, and byte-packed stack compaction (`#3111` `460d1e413` and `#3205` `e6b34e332`):** All post-`#3111` requirements in `#2843` comment `6391161531` and `PLAN.md` line 68 passed on RTX 5090 (`vast-57638243`, `72 / 72` leaf traces matched, `from_snapshot` step `16` refused on CUDA with `status 10`, `ptxas` stack reduced by `-2,896` B to `22,080` B with `0` spills, `305,795` / `295,120 tok/s`).

3. **What this unit (`antigravity-88`, PR `#3256`) completes to close `#2843`:**
   - **Paired order-test scorer & `C_shuf` window helper in `crates/uor-r4-discrete/src/order_slice.rs`:** Adds `parse_frozen_slice` (verifying `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a` and `1,620` ascending ordinals), `shuffle_prefix_window` (`C_shuf` over `Identity` windows), and `score_position_contrast` / `evaluate_position_order_seed` / `grade_position_order_two_seeds` with 4 new unit tests (`10 / 10` `order_slice` tests pass).
   - **Pre-registration sheet synchronization (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`):** Pins the exact `#3111`/`#3205` GPU `flock_vector` transport definition required by `kimi-33/REPORT.md` line 16, cites the `#3205` GPU qualification receipts (`0de22708…`, `9d1072d3…`), adds the `2026-10-24T03:41:41Z` fresh-V2 replication rule ($b - c \ge \lceil 2\sqrt{b+c}\rceil$) and 3-branch outcome decision table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG`, `FAIL_TRANSFER_CLOSE_CONFIG`), and records a fresh-context non-author `TEST FITNESS: FIT` and D68 decision review.
   - **Status transition:** Sets `m1-position-mechanism.md` to `status: done` (`Closes #2843`), unblocking the 6 downstream items on the M1/M2 path (`m1-trunk-assembly-and-training` `#2844`, etc.), while clearly recording in the notebook and `README.md` that the two-seed GPU order training run (`H_pos` vs `C_match` and `C_shuf`) remains `NOT_RUN` (construction, finite CPU/GPU controls, and pre-registration only, per `CHARTER.md` §4.6).
