# REPORT: r-vocab-4096-vs-16384, antigravity-91

Lab: antigravity. Order: [ORDER.md](ORDER.md).
Closes #2887.

## What did not work

1. **Expanding BPE vocabulary size alone (`V = 4,096` $\to$ `V = 16,384`) did not fix number splitting (`classification: mechanism` negative on `V` alone under `Digits(true)` and plain `ByteLevel`):**
   - Under `ctrl1` (`Digits(true)+ByteLevel`, matching pinned `d36d3e87…`), quadrupling the vocabulary from `4,096` (`blake3:ed3f625d…`) to `16,384` (`blake3:311a1e7c…`) leaves `pure_multi_digit = 0` (`0 / 100` two-digit strings `"00".."99"` formed) because `digits_individual = true` splits every ASCII digit before BPE merges run.
   - Under `ctrl2` (plain `ByteLevel`), quadrupling the vocabulary from `4,096` (`blake3:81522e79…`) to `16,384` (`blake3:ec4c2c66…`) leaves `pure_multi_digit = 5` (`5 / 0 / 0`, `5 / 100` two-digit strings) and `space_prefixed_digit = 110` vs `109` (`" 2026"` absorbed as `[Ġ20, 26]` while bare `"2026"` splits differently).
   - Conversely, `trt` (`Digits(false)+ByteLevel`) achieves `pure_multi_digit = 109` (`100 / 100` `"00".."99"`, `6` three-digit, `3` four+-digit) and `space_prefixed_digit = 0` identically at **both** `V = 4,096` (`blake3:c38dabf5…`) and `V = 16,384` (`blake3:411c366d…`). Pre-tokenization (`Digits(false)`), not vocabulary size, is the lever for number tokenization.
2. **Legacy `FullScan` writer scaling at `V = 16,384` (`classification: mechanism` on `FullScan`, resolved by `m1-faster-writer` `#2840`):**
   - Under the legacy `O(V · B)` full-vocabulary scan, quadrupling $V$ from `4,096` to `16,384` multiplied per-row primitive evaluations by **`3.83x`** (`39,268 -> 150,243` at `K = 8`) and reduced GPU writer throughput by **`3.90x`** (`855 -> 219 tok/s` on `1x RTX 5090`).
3. **Full discrete-driver (`PreparedDriver` / `Device` / `D11`) execution at `V = 16,384` (`classification: instrument` / architectural boundary, `B3`):**
   - Five explicit code-level guards in `crates/uor-r4-discrete` and `crates/uor-r4-integer` bound `V` in the full `B = 4` discrete driver:
     1. `address_cells::build` (`crates/uor-r4-discrete/src/address_cells.rs:322, 462`): line `322` enforces `quota = v / CELLS > 16` (`V > 4,096`), and `StructuredV1` (`line 462`) ties all interior blocks `1..B-1` to a single orbit root `ROOTS[k]` (`k in 0..16`), giving `256 x 16 = 4,096` distinct `B = 4` vectors (`16` tokens/cell). Supporting `V = 16,384` (`64` tokens/cell) at `B = 4` requires `FullVectorV1`-style independent interior-block indexing `(k / 16, k % 16)` in `0..256` (as in `address_cells.rs:383-460`) and raising line `322`'s `v / CELLS <= 16` bound to `64`.
     2. `Spec::validate()` (`crates/uor-r4-discrete/src/lib.rs:83`): `!(2..=8192).contains(&self.vocabulary)`.
     3. `first_n_primes` (`crates/uor-r4-discrete/src/prime_zeta.rs:108`): `count > 8192`.
     4. `discrete_count_score` (`crates/uor-r4-discrete/src/discrete.cu:633`) and `Device::count_reference` (`crates/uor-r4-discrete/src/device.rs:1890`): `vocabulary > 8192`.
     5. `d11_export::validate_and_extract_cells` (`crates/uor-r4-discrete/src/d11_export.rs:149`), `import_d11` (`d11_export.rs:386`), and `IntegerDiscreteModel::from_bytes` (`crates/uor-r4-integer/src/discrete_d11.rs:667`): `!(2..=8192).contains(&spec.vocabulary)`.
   - Per `CHARTER.md` §3.5 (*"Spend GPU only where a run can change a decision"*), no redundant GPU pod was rented because the matched `V = 4,096` vs `V = 16,384` GPU writer evaluations (`#2840`) are already sealed across two independent `sm_120` builds (`Build A` `37419bdc…` and `Build B` `df583e45…`).

## What changed

- **Pre-registration & comparison sheet (`docs/harness/prep/r-vocab-4096-vs-16384-2026-10-26.md`):** Registered the matched `V = 4,096` vs `V = 16,384` bars (`B1` GPU writer parity & primitive evaluation scaling across `K in {1, 8, 16}` and uniform occupancy; `B2` 3-arm BPE tokenizer & digit-piece invariants; `B3` discrete `B = 4` codebook capacity boundary) with fresh-context `TEST FITNESS: FIT`.
- **D68 Decision Card (`docs/loop/tasks/r-vocab-4096-vs-16384/antigravity-91/DECISION.md`):** Recorded the whole-project D68 decision retaining `V = 4,096` for M1–M3 and closing `#2887` on the matched `V = 4,096` vs `V = 16,384` GPU writer (`#2840`), BPE tokenizer (`#2872`), and discrete-carrier capacity (`B3`) evidence.
- **Lab Record (`docs/labs/r-vocab-4096-vs-16384/README.md`):** Tabulated the complete matched `V = 4,096` vs `V = 16,384` results (leading with `tokens/s`), the 5-site `B = 4` discrete capacity checklist, and the notebook records (`docs/research/notebook/m1-discrete-attention.md` row 79 and the `docs/research/notebook/data-tokenizer.md` row specification).
- **Roadmap (`docs/roadmap/items/r-vocab-4096-vs-16384.md` & `docs/roadmap/README.md`):** Set `r-vocab-4096-vs-16384` to `status: done` and re-rendered `docs/roadmap/README.md`.
- **Size (`uor-harness loop pr-check --local`):** `tests 0 file(s) +0/-0, code 0 file(s) +0/-0, docs 6 file(s) +343/-1, generated 1 file(s) +3/-3, other 0 file(s) +0/-0` (no size estimate declared on `r-vocab-4096-vs-16384.md`).

## Measurements

1. **Bar `B1` — Matched GPU Writer & Serving Cost at `V = 4,096` vs `V = 16,384` (`1x NVIDIA GeForce RTX 5090`, `vast-57796936`, driver `595.84`, `Build A` `37419bdc303b79ac4beac0fab0ea292d74dc34ac9623754457e9107d89a40fd0` & `Build B` `df583e45e1ca857cb698faf23c5a266fdc6abd3073be5f897662e432c82262bb`, `0` metric diffs):**
   - **`K = 1` (seeded, `96` rows per `V`):**
     - `V = 4,096`: `FullScan` `2,951` / `2,943 tok/s` (`8,721` evals/row) $\to$ `IdOrder` **`115,523` / `111,627 tok/s`** (`39.1x` / `37.9x` speedup, **`271`** evals/row, **`32.08x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`118,081` / `118,081 tok/s`** (`40.0x` / `40.1x` speedup, **`271`** evals/row, **`32.08x`** reduction, `0` mismatches).
     - `V = 16,384`: `FullScan` `756` / `754 tok/s` (`33,344` evals/row) $\to$ `IdOrder` **`84,063` / `83,478 tok/s`** (`111.2x` / `110.7x` speedup, **`318`** evals/row, **`104.54x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`86,486` / `85,637 tok/s`** (`114.4x` / `113.6x` speedup, **`318`** evals/row, **`104.54x`** reduction, `0` mismatches).
     - `V = 16,384 / V = 4,096` ratio at `K = 1`: **`1.17x`** primitive evaluations (`318 / 271`) vs `3.82x` under `FullScan`.
   - **`K = 8` (seeded, `96` rows per `V`):**
     - `V = 4,096`: `FullScan` `855` / `853 tok/s` (`39,268` evals/row) $\to$ `IdOrder` **`47,904` / `47,151 tok/s`** (`56.0x` / `55.3x` speedup, **`382`** evals/row, **`102.74x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`51,282` / `51,336 tok/s`** (`60.0x` / `60.2x` speedup, **`365`** evals/row, **`107.53x`** reduction, `0` mismatches).
     - `V = 16,384`: `FullScan` `219` / `218 tok/s` (`150,243` evals/row) $\to$ `IdOrder` **`22,977` / `22,884 tok/s`** (**`104.9x` / `105.0x` speedup**, **`765`** evals/row, **`196.15x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`26,936` / `26,845 tok/s`** (**`123.0x` / `123.1x` speedup**, **`592`** evals/row, **`253.50x`** reduction, `0` mismatches).
     - `V = 16,384 / V = 4,096` ratio at `K = 8`: **`2.00x`** primitive evaluations under `IdOrder` (`765 / 382`) and **`1.62x`** under `ServingPruned` (`592 / 365`) vs `3.83x` under `FullScan`.
   - **`K = 16` (seeded, `96` rows per `V`):**
     - `V = 4,096`: `FullScan` `479` / `477 tok/s` (`74,128` evals/row) $\to$ `IdOrder` **`28,941` / `28,751 tok/s`** (`60.4x` / `60.3x` speedup, **`518`** evals/row, **`143.10x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`31,840` / `31,714 tok/s`** (`66.5x` / `66.5x` speedup, **`443`** evals/row, **`167.06x`** reduction, `0` mismatches).
     - `V = 16,384`: `FullScan` `122` / `122 tok/s` (`283,788` evals/row) $\to$ `IdOrder` **`13,038` / `12,979 tok/s`** (`106.9x` / `106.4x` speedup, **`1,282`** evals/row, **`221.28x`** reduction, `0` mismatches) $\to$ `ServingPruned` **`16,689` / `16,655 tok/s`** (**`136.8x` / `136.5x` speedup**, **`666`** evals/row, **`425.53x`** reduction, `0` mismatches).
     - `V = 16,384 / V = 4,096` ratio at `K = 16`: **`1.50x`** primitive evaluations under `ServingPruned` (`666 / 443`) vs `3.83x` under `FullScan`.
   - **Uniform Occupancy (`K = 8`, `32` rows/case):**
     - `V = 4,096` (`16` tokens/cell): `39,270` $\to$ **`384`** evals/row (`0 + 256 + 128`, **`102.26x`** reduction, `0` mismatches).
     - `V = 16,384` (`64` tokens/cell): `150,246` $\to$ **`768`** evals/row (`0 + 256 + 512`, **`195.63x`** reduction, `0` mismatches — exact **`2.00x`** work ratio vs `V = 4,096`).
2. **Bar `B2` — Matched BPE Tokenizer & Digit-Piece Census (`crates/uor-r4-tokenizer/tests/training.rs`, `23 / 23` tests pass on `170,176` B corpus `05f21705…`):**
   - `ctrl1` (`Digits(true)+ByteLevel`): `V = 4,096` (`blake3:ed3f625d…`) and `V = 16,384` (`blake3:311a1e7c…`) both have `pure_single_digit = 10`, `pure_multi_digit = 0`, `space_prefixed_digit = 0`.
   - `ctrl2` (plain `ByteLevel`): `V = 4,096` (`blake3:81522e79…`) and `V = 16,384` (`blake3:ec4c2c66…`) have `pure_single_digit = 10`, `pure_multi_digit = 5` (`5 / 0 / 0`), `space_prefixed_digit = 110` vs `109`.
   - `trt` (`Digits(false)+ByteLevel`): `V = 4,096` (`blake3:c38dabf5…`) and `V = 16,384` (`blake3:411c366d…`) both have `pure_single_digit = 10`, **`pure_multi_digit = 109` (`100 / 100` `"00".."99"`, `6` 3-digit, `3` 4+-digit)**, and **`space_prefixed_digit = 0`**, with byte-identical determinism across 2 training runs per arm (`12 / 12` runs).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-tokenizer --no-fail-fast`: `23 / 23 PASS` (`13` lib + `2` `train_bpe_cli` + `8` `training`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls --no-fail-fast`: `6 / 6 PASS`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap render` & `roadmap check`: `PASS`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --locked -p xtask -- links`: `PASS`
- `uor-harness check`: `42 rules, 0 failed (0 error, 0 warn)`
- `uor-harness loop report --check --order docs/loop/tasks/r-vocab-4096-vs-16384/antigravity-91/ORDER.md`: `PASS`
- `uor-harness loop pr-check --local`: `verdict: PASS` (`10 / 10` local rules `PASS`, `0` warnings)
- `uor-harness loop gates --full`: `11 / 11 GREEN` (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `loop-next-sources`, `loop-next-wait`, `loop-next-owner-direction`, `git-hooks`, `loop-migrate`, `loop-review-changes-blocks`)

## Skills used

- `uor-research-architect`: Enforced CHARTER §3.1–§3.5 (geometry only, no squashing/matmul, GPU-only research compute, no redundant GPU spend when sealed two-build RTX 5090 receipts already answer the question) and §4.1–§4.5 (`classification: mechanism` vs `classification: instrument`).
- `solution-integrity`: Verified the exact 5-site code-level capacity boundary (`address_cells.rs:322,462`, `lib.rs:83`, `prime_zeta.rs:108`, `discrete.cu:633`, `d11_export.rs:149,386`, `discrete_d11.rs:667`) explaining why `StructuredV1` (`B = 4` tied interior roots) saturates at `256 x 16 = 4,096` tokens.
- `scope-guard`: Kept all changes strictly inside the item's declared scope (`docs/harness/prep/r-vocab-4096-vs-16384*`, `docs/labs/r-vocab-4096-vs-16384/**`, `docs/loop/tasks/r-vocab-4096-vs-16384/**`, status-only edit on `r-vocab-4096-vs-16384.md`, and rendered `docs/roadmap/README.md`).

## Still open

- If a later milestone (`M6` coding/reasoning) widens the discrete vocabulary from `4,096` to `16,384`, apply the 5-file `B3` checklist in `docs/labs/r-vocab-4096-vs-16384/README.md` §4 (independent interior-block indexing `(k / 16, k % 16)` in `address_cells.rs` so each of the `256` cells holds up to `64` injective tokens at `B = 4`, plus raising the `8,192` validator/prime-table cap to `16,384`).

## Plain summary

This unit completes `r-vocab-4096-vs-16384` (`#2887`), which compares a `4,096`-piece tokenizer against a `16,384`-piece (`4x` larger) tokenizer across three registered bars (`B1` GPU writer speed and per-row comparison count, `B2` BPE number tokenization, and `B3` discrete `B = 4` vector-table capacity):

1. **GPU writer speed and per-row work (`B1`, measured on an RTX 5090 GPU in `#2840`):** In the old `FullScan` writer (which checked every vocabulary token on every row), quadrupling the vocabulary from `4,096` to `16,384` pieces made the writer `3.90x` slower (`855 -> 219 tokens/s` at shortlist width `K = 8`, `39,268 -> 150,243` primitive comparisons per row). With the new CSR cell-bucketed writer (`#2840`), the `256` cell-shortlist comparisons are fixed regardless of vocabulary size and only tokens inside the `K` shortlisted cells are compared. As a result, quadrupling the vocabulary from `4,096` to `16,384` increases per-row comparisons by only `1.17x` at `K = 1` (`271 -> 318`), `2.00x` at `K = 8` (`384 -> 768` uniform / `382 -> 765` seeded), and `1.62x` at `K = 8` with distance pruning (`365 -> 592`), running at **`26,936 tokens/s`** at `V = 16,384, K = 8` (`123.0x` faster than `FullScan`).
2. **Number tokenization (`B2`, measured in `#2872`):** Quadrupling the vocabulary from `4,096` to `16,384` under the old `Digits(true)` rule (`ctrl1`) still produces **`0` multi-digit pieces**, and under plain `ByteLevel` (`ctrl2`) still produces `110` / `109` space-prefixed digit tokens (`" 2026"` split differently from `"2026"`). Switching the pre-tokenizer to `Digits(false)+ByteLevel` (`trt`) forms all `100` two-digit numbers (`"00".."99"`) and `109` pure multi-digit pieces with `0` space-prefixed digit tokens at **both** `4,096` and `16,384` pieces.
3. **Discrete `B = 4` vector capacity (`B3`):** In the current `B = 4` discrete model, `StructuredV1` (`address_cells.rs`) ties both interior blocks (`1` and `2`) to the same single 16-root orbit index `ROOTS[k]`, which caps collision-free capacity at `256 cells x 16 roots = 4,096` tokens (alongside `8,192` validator caps in `lib.rs`, `prime_zeta.rs`, `discrete.cu`, `d11_export.rs`, and `discrete_d11.rs`). We therefore retain `V = 4,096` for M1–M3 and document the exact 5-file `(k / 16, k % 16)` interior-block indexing change needed whenever a later milestone widens `V` to `16,384`.
