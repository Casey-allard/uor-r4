# REPORT: m1-close-saved-model, antigravity-108

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Issue: `#2837` (Milestone M1 `#2029`).

## What did not work

- **Before this unit (`instrument` classification on `efcbd7065`), `d11-scorer score` and `d11-scorer fresh-score` (`#2841`) evaluated only the 4-table pre-rotation read vector in `d11_score_rows` (`src/kernel_core.h`), failing `reference_check` on 19-table `uor-r4.lut-discrete-trunk/1` artifacts:**
  - `ArtifactTables::from_bytes` (`docs/labs/m1-d11-artifact-scorer/instrument/src/artifact.rs`) unpacked only `discrete.table`, `discrete.cells`, `discrete.routes`, and `discrete.gates`, omitting the 15 `discrete.trunk.*` sections (`step_candidates`, `norm_candidates`, `unigram`, and the 12 sparse count tables).
  - Consequently, `d11_evaluate_row` (`src/kernel_core.h`) ranked tokens directly from the unrotated 4-table read vector instead of applying the 19-table `DiscreteTrunkTables::apply_joint_pointer_gate` rotation before post-rotation `OuterL1V1` admission (`A_W(q_trunk)`). Verified failing-first in `tests/m1_close.rs`: setting `tables_no_trunk.trunk = vec![0]` produces `0 / 5` agreement (`ref_no_trunk["agreed"] == 0`) against `IntegerDiscreteModel::observe_target`.
- **Self-path in `docs/roadmap/items/m1-close-saved-model.md`'s `scope:` caused a false glob overlap with the `infra-loop-harness` umbrella (`docs/roadmap/items/*.md`, `PR #3309`):**
  - Dropped `docs/roadmap/items/m1-close-saved-model.md` from `docs/roadmap/items/m1-close-saved-model.md`'s `scope:`.
- **M1 Bar (e) (`d11-scorer fresh-score` on the sealed `Fresh V2` slice `0b7e754c…` / `a1cda529…`) remains `UNAVAILABLE_NOT_RUN` (`NOT_RUN` is not `PASS`):**
  - With the 19-table `uor-r4.lut-discrete-trunk/1` kernel path now wired into `ArtifactTables`, `d11_score_rows`, and `DeviceScorer`, `#2841` (`m1-fresh-panel-transfer`) is unblocked to score the two-seed 19-table `trunk-joint-pointer-v1-d11` `.d11` artifacts on the GPU. Until `#2841` executes `d11-scorer fresh-score`, `#2837` and `#2029` remain open (`m1_close_pass: false`).

## What changed

1. **Packed 19-table `uor-r4.lut-discrete-trunk/1` layout in `IntegerDiscreteModel` and `ArtifactTables` (`crates/uor-r4-integer/src/discrete_d11.rs`, `docs/labs/m1-d11-artifact-scorer/instrument/src/artifact.rs`):**
   - Added `IntegerDiscreteModel::packed_codebook_roots(&self) -> Vec<u32>` returning the `V` packed 16-bit 4-block canonical orbit-root codes (`r0 | (r1 << 4) | (r2 << 8) | (r3 << 12)`).
   - Added `pub trunk: Vec<u32>` to `ArtifactTables`. When `model.is_trunk_d11()` is `true`, `ArtifactTables::from_bytes` packs `trunk[0] = 1`, `step_candidates[64]`, `norm_candidates[4]`, `unigram[1 + V]`, `packed_codebook_roots[V]`, a 24-word `(offset, count)` directory, and lexicographically sorted `(key, tok, count)` triples for all 12 sparse tables (`read_table` through `query_rec_unit_table`). For 4-table `uor-r4.lut-discrete/1` artifacts, `trunk` is `vec![0u32]`.
2. **19-table joint-pointer Hamilton gate in `kernel_core.h`, `scorer.cu`, `kernel.rs`, and `device.rs` (`docs/labs/m1-d11-artifact-scorer/instrument/src/{kernel_core.h,scorer.cu,kernel.rs,device.rs}`):**
   - Added `const uint32_t* trunk;` to `D11Model` (`src/kernel_core.h`), `host::CModel` (`src/kernel.rs`), `d11_score_rows` (`src/scorer.cu`), and `DeviceScorer` (`src/device.rs`).
   - Implemented `d11_cl06_inverse`, `d11_trunk_sparse_get`, `d11_trunk_sparse_key_range`, `d11_trunk_consider_candidate`, and `d11_trunk_apply_joint_pointer_gate` in `src/kernel_core.h`, matching `DiscreteTrunkTables::apply_joint_pointer_gate` (`crates/uor-r4-integer/src/discrete_d11.rs:514–618`) in exact 32-bit/64-bit integer arithmetic (`ActiveHolonomyV1` quaternion recurrence, 4-block $\mathcal{O}_{16}^4$ orbit-root causal pointer `(ptr_match << 8)`, joint-first count hierarchy `(c_qr << 6) + (c_qc << 5) + (c_rr << 3) + (c_r << 1) + c_q`, `BFS` `DiscreteHamiltonGateV1` rotation, and `d11_snap(..., 1)` orbit projection) before post-rotation `OuterL1V1` cell admission (`A_W(q_trunk)`) and token ranking.
3. **Failing-first + restored parity and planted-fault `B4` test (`docs/labs/m1-d11-artifact-scorer/instrument/tests/m1_close.rs`):**
   - Extended `clean_room_integer_discrete_model_loads_19_table_trunk_d11_with_zero_export_gap_and_serving_toks` to verify that `host_score` with `tables_no_trunk.trunk = vec![0]` fails `reference_check` (`agreed == 0` / `5`), and with `tables.trunk[0] == 1` passes `reference_check` (`agreed == 5 / 5`, `0` mismatches on `read_vector`, `admitted_cells`, `top8`, `prediction`, and target hinge losses) and passes `planted_fault_control` (`b4_pass == true`).
4. **Updated lab write-up, notebook, and item scope (`docs/labs/m1-close-saved-model/README.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-close-saved-model.md`):**
   - Documented `antigravity-108` in `docs/labs/m1-close-saved-model/README.md` and `docs/research/notebook/m1-discrete-attention.md`, and removed `docs/roadmap/items/m1-close-saved-model.md` from `docs/roadmap/items/m1-close-saved-model.md`'s `scope:`.

## Measurements

- **Headline throughput (`CHARTER.md` §3.12):**
  - **Path A `trunk-joint-pointer-v1-d11` (`19`-table `UORLUT01` `uor-r4.lut-discrete-trunk/1`):** `34,325`–`34,337 context tok/s` GPU on `ft/dev` (`76,316`–`76,343` TRAIN, `37,838`–`37,853` `fresh_v2`), plus clean-room `IntegerDiscreteModel::evaluate_prefix` laptop `d11-scorer toks` (`time_serving`, `12,630` / `12,523 tok/s` base D11 decode).
- **19-table `d11_score_rows` (`kernel_core.h`) vs `IntegerDiscreteModel::observe_target` (`tests/m1_close.rs`):**
  - Failing-first (`tables_no_trunk.trunk = vec![0]`): `checked = 5`, `agreed = 0` (`FAIL` when set aside).
  - Restored (`tables.trunk[0] == 1`): `checked = 5`, `agreed = 5` (`0` mismatches on `read_vector`, `admitted_cells`, `top8`, `prediction`, `token_hinge`, `address_hinge`, `rank_hinge`, `loss_ta`, `loss_tr`).
  - Planted-fault control (`B4`): `b4_pass = true` (`container_cells_tamper_rejected = true`, `container_routes_lsb_hash_changed = true`, `container_routes_oob_rejected = true`, `cells_mismatched_rows > 0`, `route_or_gate_mismatched_rows > 0`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml -j 1 --no-fail-fast`: **`21 / 21` passed** (`3` `m1_close`, `10` `parity`, `5` `stage1`, `3` `toks`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-integer --lib --no-fail-fast`: **`56 / 56` passed**.
- `NVCC=local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --offline --features cuda --all-targets`: **passed (`0` warnings)**.
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/antigravity-108/ORDER.md`: **`OK`**.
- `uor-harness loop pr-check --local`: **`OK`**.
- `uor-harness loop gates --full`: **`11 / 11 GREEN`**.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`.

## Still open

- Execute `#2841` `d11-scorer fresh-score` (`Bar (e)`) on the two-seed 19-table `trunk-joint-pointer-v1-d11` `.d11` artifacts against the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`). References `#2837`.

## Plain summary

`d11-scorer score` and `d11-scorer fresh-score` (`docs/labs/m1-d11-artifact-scorer/instrument/`) now pack and evaluate all 19 sections of a `uor-r4.lut-discrete-trunk/1` `.d11` artifact (`trunk-joint-pointer-v1-d11`) inside `d11_score_rows` (`kernel_core.h`, both host C++ and CUDA `DeviceScorer`), matching `IntegerDiscreteModel::observe_target` with `0` mismatches and passing `planted_fault_control` (`B4`).
