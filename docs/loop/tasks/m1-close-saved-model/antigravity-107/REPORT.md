# REPORT: m1-close-saved-model, antigravity-107

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Order: [ORDER.md](ORDER.md).

References #2837
References #2029

## What did not work

- **`m1_close_pass` remains `false` (`OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`), held only on Bar (e) (`#2841` `d11-scorer fresh-score` on the sealed `Fresh V2` slice):**
  - While `trunk-joint-pointer-v1-d11` now clears **Bar (a)** (clean-room 19-table `UORLUT01` acceptance in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../crates/uor-r4-integer/src/discrete_d11.rs) via `IntegerDiscreteModel::from_bytes`), **Bar (b)** (`0` export gap between `IntegerDiscreteModel::evaluate_prefix` and `uor_r4_discrete::trunk::evaluate_joint_pointer_full_writer` + planted-fault `B4` rejection), **Bar (c)** (`tokens/s` reported on laptop `d11-scorer toks` `time_serving` and GPU `34,325–34,337 context tok/s`), and **Bar (d)** on `ft/dev` across both seeds (`7,702 / 14,287` and `7,701 / 14,287` on `roots0`, `7,386 / 14,287` and `7,374 / 14,287` on `roots_final`, and `2,090 / 3,847` and `2,086 / 3,847` on `P_succ` vs `T2 = 6,300` and `T2_psucc = 1,245`), **Bar (e)** requires running the `#2841` `d11-scorer fresh-score` protocol against the sealed `14,479`-target (`3,594` `P_succ`) `Fresh V2` slice (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` / `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`) using two full exported 19-table `.d11` artifacts.
  - Because `UNAVAILABLE_NOT_RUN` is never `PASS` (`not_run_is_not_pass = true`), `m1_close_pass` in [`docs/labs/m1-close-saved-model/evidence/m1-close-grade.json`](../../../labs/m1-close-saved-model/evidence/m1-close-grade.json) remains `false`, `fresh_v2_seal_intact = true`, and `#2837` / `#2029` stay open.
- **Historical 4-table Stage T `.d11` artifacts (`stage-t-t16-fz`, `stage-t-t16-rel`) still fail Bar (d):**
  - Their 4-table `UORLUT01` writer (`455` and `371 < 6,324` on `ft/dev`) remains capped by pre-rotation shortlist admission $A_W(q_{\text{read}}) = 5,009 < 6,300$.

## What changed

- **Step 1 — D68 scope expansion in [`docs/roadmap/items/m1-close-saved-model.md`](../../../../roadmap/items/m1-close-saved-model.md) and [`DECISION.md`](DECISION.md):**
  - Added `crates/uor-r4-integer/**` and `docs/roadmap/items/m1-close-saved-model.md` to `#2837` `scope:` after fresh-context D68 review (`2d1f0194-35c2-489c-9525-454ef83e2284`, logged at [`#2837#issuecomment-6428847942`](https://github.com/UOR-Foundation/uor-r4/issues/2837#issuecomment-6428847942)).
- **Step 2 — Clean-room 19-table `UORLUT01` integer reader & evaluator in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../crates/uor-r4-integer/src/discrete_d11.rs):**
  - Added `pub const DISCRETE_TRUNK_SCHEMA: &str = "uor-r4.lut-discrete-trunk/1";`, `DiscreteSparseCountTable` (sorted `(u32, u16)` key-token pairs + non-zero `u32` counts with binary-search lookup), `DiscreteTrunkTables`, `pack_4block_roots` (matching `uor_r4_discrete::w2_capacity::pack_code` little-endian nibble order `roots[0] | (roots[1] << 4) | (roots[2] << 8) | (roots[3] << 12)`), and `orbit_root_causal_successor`.
  - Extended `IntegerDiscreteModel::from_bytes` to accept both 4-table `uor-r4.lut-discrete/1` (`tables.len() == 4`) and 19-table `uor-r4.lut-discrete-trunk/1` (`tables.len() == 19`, requiring `s.blocks == 4`), validating contiguous 64-byte-aligned sections, zero padding, zero trailing bytes, `step_candidates` (`64` in `0..16`), `norm_candidates` (`4` in `0..16`), `unigram` (`1 + V` in `0..vocab`), and strictly ascending keys + positive counts in all 12 sparse triple tables.
  - Wired `trunk.apply_joint_pointer_gate(prefix, &mut q_blocks, &self.codebook_roots)` into `IntegerDiscreteModel::evaluate_prefix` when `!self.state_transport_knockout` and added unit test `trunk_d11_19_table_loader_evaluates_joint_pointer_and_rejects_planted_faults`.
- **Step 3 — Cross-crate verification and updated M1 close grading in [`docs/labs/m1-d11-artifact-scorer/instrument/`](../../../labs/m1-d11-artifact-scorer/instrument/):**
  - Updated [`src/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs), [`tests/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/m1_close.rs) (`clean_room_integer_discrete_model_loads_19_table_trunk_d11_with_zero_export_gap_and_serving_toks`), [`docs/labs/m1-close-saved-model/evidence/m1-close-grade.json`](../../../labs/m1-close-saved-model/evidence/m1-close-grade.json), [`docs/labs/m1-close-saved-model/README.md`](../../../labs/m1-close-saved-model/README.md), and [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md).

## Measurements

- **Headline throughput (`CHARTER.md` §3.12):** `34,325`–`34,337 context tok/s` GPU on `ft/dev` (`76,316`–`76,343` TRAIN, `37,838`–`37,853` `fresh_v2`; laptop `d11-scorer toks` `time_serving` verified positive median decode/prefill `tok/s` with `0` errors on 19-table `IntegerDiscreteModel`).
- **Cross-crate `B2` zero export gap & `B4` planted-fault rejection:**
  - `IntegerDiscreteModel::from_bytes` loads `export_trunk_d11` (`is_trunk_d11() == true`) and `IntegerDiscreteModel::evaluate_prefix` matches `uor_r4_discrete::trunk::evaluate_joint_pointer_full_writer` with **`0` export gap** across all prefixes (`[7, 12, 7, 19]`, `[7, 19, 7, 12]`, `[3, 7, 12]`, `[19, 3, 7, 40]`, `[40, 9]`).
  - Planted faults (`step_candidates[0] = 16`, `norm_candidates[0] = 16`, out-of-range `unigram`, unsorted `read_rec` keys, zero-count `read_tok`, non-zero trailing byte) are deterministically rejected (`B4`).
- **Five-bar status for `trunk-joint-pointer-v1-d11` (`m1-close-grade.json`):**
  - `bar_a_pass = true`, `bar_b_pass = true`, `bar_c_pass = true`, `bar_d_pass = true` (`7,702` / `7,701` `roots0` and `7,386` / `7,374` `roots_final` `> 6,324` on `ft/dev`; `P_succ = 2,090` / `2,086 > 1,265`), `bar_e_status = "UNAVAILABLE_NOT_RUN"`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-integer --lib --no-fail-fast`: `56 passed; 0 failed` (`trunk_d11_19_table_loader_evaluates_joint_pointer_and_rejects_planted_faults`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml -j 1 --no-fail-fast`: `21 passed; 0 failed` (`3` `m1_close`, `10` `parity`, `5` `stage1`, `3` `toks`).
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/antigravity-107/ORDER.md`: `PASS`.
- `uor-harness loop gates --full`: `11 / 11` gates `GREEN`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`.

## Still open

- Export the full two-seed 19-table `uor-r4.lut-discrete-trunk/1` `.d11` artifacts and run `#2841` `d11-scorer fresh-score` on the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`) to evaluate **Bar (e)** and close `#2837` / `#2029`.

## Plain summary

We extended the clean-room integer reader and evaluator (`IntegerDiscreteModel::from_bytes` and `IntegerDiscreteModel::evaluate_prefix` in `crates/uor-r4-integer/src/discrete_d11.rs`) to accept and evaluate the 19-table `uor-r4.lut-discrete-trunk/1` schema (`trunk-joint-pointer-v1-d11`), proved `0` cross-crate export gap and planted-fault `B4` rejection against `crates/uor-r4-discrete/src/trunk.rs`, and verified laptop `time_serving` (`tok/s`), so `trunk-joint-pointer-v1-d11` now passes **Bars (a), (b), (c), and (d)** through `crates/uor-r4-integer`, leaving only **Bar (e)** (`#2841` `d11-scorer fresh-score` on the sealed `Fresh V2` slice) to close M1.
