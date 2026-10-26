# REPORT: m1-position-mechanism, antigravity-88

- Lab: `antigravity`
- Item: [`m1-position-mechanism`](../../../../roadmap/items/m1-position-mechanism.md) (#2843)
- PR: #3256
- Base: `c695ad21a72c5b91def22458eebbe796e07d5392`

Closes #2843

## What did not work

- In `crates/uor-r4-discrete/src/order_slice.rs`, the initial assertion in `committed_frozen_order_slice_matches_pinned_sha256_and_1620_rows` checked `(8, 37)` instead of `(0, 41)` as the first `(row_ordinal, target_position)` pair of `docs/labs/m1-position-mechanism/order-slice-v1.tsv` (`instrument` test-expectation fix). Updating the assertion to `(0, 41)` matched the committed TSV (`SHA-256 = f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`, `1,620` rows) on the first re-run.
- In `docs/loop/tasks/m1-position-mechanism/antigravity-88/ORDER.md`, the initial cancel-probe draft was `4,659` bytes (`> 4,096` B advisory `read-path` budget). Trimming `ORDER.md` to `3,108` bytes resolved the budget check cleanly.

## What changed

- **Size:** 9 files changed (+815 / -52 lines across `origin/main...HEAD`: +559 / -3 Rust code and unit-test lines in `crates/uor-r4-discrete/src/order_slice.rs`, +256 / -49 documentation, pre-registration sheet, notebook, roadmap, and task lines).
- **`crates/uor-r4-discrete/src/order_slice.rs`:**
  1. Added `FROZEN_ORDER_SLICE_SHA256` (`f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`), `FROZEN_ORDER_SLICE_ROWS` (`1,620`), `PINNED_DEV_ELIGIBLE_ROWS` (`14,287`), `PINNED_DEV_COPY_SUCCESSOR_ROWS` (`3,847`), `MCNEMAR_MIN_FLOOR` (`24`), and `parse_frozen_slice(tsv: &str) -> Result<Vec<(u64, u64)>>`.
  2. Added `shuffle_prefix_window(window: &[Identity], seed: u64) -> Result<Vec<Identity>>`, applying the exact splitmix64 Fisher-Yates permutation of `position_mechanism::shuffle_prefix_roots` to prefix `Identity` entries `0..L-1` while holding the query token `window[L - 1]` and prefix token multiset fixed for the `C_shuf` control (`kimi-33/REPORT.md:64`).
  3. Added `integer_ceil_two_sqrt`, `PositionRowObservation`, `PositionPairedCount`, `PositionSeedEvaluation`, `PositionOrderVerdict`, `score_position_contrast`, `evaluate_position_order_seed`, and `grade_position_order_two_seeds` — a pure-integer paired one-sided McNemar, target-admission, and fresh-transfer scorer (zero floats, `D32`) covering `H_pos(ordered)` vs `C_match(ordered)` ($b - c \ge F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$, $\Delta\text{admitted} \ge 0$), `H_pos(ordered)` vs `C_shuf` ($b - c \ge F_I$), and the owner's `2026-10-24T03:41:41Z` fresh-V2 replication rule ($b - c \ge \lceil 2\sqrt{b + c}\rceil$, no 24-row floor, `#2028` comment `6379430093`).
  4. Added 4 unit tests (`committed_frozen_order_slice_matches_pinned_sha256_and_1620_rows`, `shuffle_prefix_window_preserves_query_multiset_and_matches_root_shuffle`, `integer_ceil_two_sqrt_and_paired_mcnemar_scorer_verify_floor_and_admission`, `evaluate_seed_and_two_seed_verdict_enforces_sealed_fresh_v2_rule`), bringing `order_slice::tests` to `10 / 10` passing while keeping `DRIVER_SOURCE_SHA256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` unchanged.
- **`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` & `docs/labs/m1-position-mechanism/README.md`:**
  1. Pinned the exact `#3111`/`#3205` GPU `flock_vector` final block-$b$ transport definition in §1 (`last[node]` payload-block root anchor `tr = root_id(rv, g)`, immutable `query_payload`, internal tree aggregation on `g[BFS + frame[child]*16 + frame[c0]]`, and byte-packed `unsigned char` stack arrays), resolving `kimi-33/REPORT.md` line 16.
  2. Cited the `#3205` RTX 5090 finite GPU qualification receipts (`position-qualify-build{1,2}.json`, `0de22708…` / `9d1072d3…`), the `2026-10-24T03:41:41Z` fresh-V2 replication rule ($b - c \ge \lceil 2\sqrt{b + c}\rceil$), the 3-branch outcome decision table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG` keeping fresh V2 sealed, `FAIL_TRANSFER_CLOSE_CONFIG`), the single A8 proxy-trunk configuration (`c695ad21a`), and the fresh-context non-author `TEST FITNESS: FIT` and D68 decision review (`b49b8f8b-3a24-4729-89fe-58cc393ac27e`).
- **`docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-position-mechanism.md`, and `docs/roadmap/README.md`:** Added the `2026-10-25` notebook row and marked `m1-position-mechanism` `done` (`93 items OK: ready 20, held 15, decision 1, done 57`).

## Measurements

- **Finite position mechanism controls (`crates/uor-r4-discrete`, host CPU, 2026-10-25):**
  - Command: `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --no-fail-fast -p uor-r4-discrete --lib position_mechanism`
  - Result: `10 passed; 0 failed`.
- **Frozen order-slice, `C_shuf` window helper & paired McNemar scorer controls (`crates/uor-r4-discrete`, host CPU, 2026-10-25):**
  - Command: `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --no-fail-fast -p uor-r4-discrete --lib order_slice`
  - Result: `10 passed; 0 failed` (`order-slice-v1.tsv` SHA-256 `f76dc5c7…` and `1,620` ascending ordinals verified; `shuffle_prefix_window` matches `shuffle_prefix_roots`; exact integer `ceil(2 * sqrt(b + c))` and paired McNemar / admission / sealed fresh-V2 rules verified).
- **Retained RTX 5090 finite CUDA qualification (`#3205`, `docs/labs/m1-position-mechanism/position-qualify-build{1,2}.json`):**
  - `305,795` / `295,120 tok/s` forward and `269,974` / `261,672 tok/s` evaluate (`Flock` / `Relation`, `PrefixHolonomyV1`), `72 / 72` leaf traces matched, `22,080` B stack (`-2,896` B, `0` spills), CUDA `status 10` refusal verified.
- **GPU order training status:** `NOT_RUN` in this unit (`needs_gpu: false` design, finite-control, and pre-registration item).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --no-fail-fast -p uor-r4-discrete --lib order_slice` — `10 passed; 0 failed`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --no-fail-fast -p uor-r4-discrete --lib position_mechanism` — `10 passed; 0 failed`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --no-fail-fast -p uor-r4-discrete --lib driver_source_pin` — `1 passed; 0 failed`
- `uor-harness prep list` — `FIT` on `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap render && CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap check` — `93 items OK`
- `uor-harness loop report --check --order docs/loop/tasks/m1-position-mechanism/antigravity-88/ORDER.md` — `PASS`
- `uor-harness loop pr-check --local` — `PASS`
- `uor-harness loop gates --full` — `11/11 GREEN`

## Skills used

- `scope-guard`: Kept all edits strictly within `m1-position-mechanism.md`'s declared scope (`crates/uor-r4-discrete/src/order_slice.rs`, `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`, `docs/labs/m1-position-mechanism/README.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-position-mechanism.md`, `docs/roadmap/README.md`, and `docs/loop/tasks/m1-position-mechanism/antigravity-88/**`).
- `solution-integrity`: Placed the paired order-test scorer and `C_shuf` window helper in `crates/uor-r4-discrete/src/order_slice.rs` so `DRIVER_SOURCE_SHA256` remains unchanged (`04c51654…`), and verified the sheet and D68 decision card with a fresh-context reviewer (`b49b8f8b-3a24-4729-89fe-58cc393ac27e`).

## Still open

- **Two-seed GPU order test (`H_pos` vs `C_match` and `C_shuf`):** Dispatched under its own D67 pre-run review inside the global GPU caps ($\le 4$ pods, $\le \$8.00/\text{h}$) or as part of `m1-trunk-assembly-and-training` (`#2844`, M2 under D79).

## Plain summary

To predict the next word accurately, a language model needs to know not just which words appeared earlier in the sentence, but the order in which they appeared. Earlier versions of the discrete model compared only the starting token and the current query token ("endpoints"), which meant swapping the order of words in the middle of a sentence did not change how an earlier word was carried forward. `PositionHolonomyV1` fixes this using only whole-number lookup tables: as the model steps through a sentence, each token's 8-number block root selects one of 16 discrete geometric rotations in a 64-element group ($\mathrm{Cl}(0,6)$), and the cumulative rotation from position $s$ to position $t$ depends on the exact order of all tokens between $s$ and $t$. This PR adds the integer-only window-shuffle helper (`C_shuf`), frozen 1,620-row order-slice verifier, and paired McNemar scorer in `crates/uor-r4-discrete/src/order_slice.rs` (`10 / 10` tests pass), updates the pre-registration sheet `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` with the GPU read definition, RTX 5090 qualification receipts, and fresh-context `TEST FITNESS: FIT` + D68 decision review, logs the notebook row, and closes `#2843`.
