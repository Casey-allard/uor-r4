# REPORT: m1-close-saved-model, antigravity-106

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Order: [ORDER.md](ORDER.md).

References #2837

## What did not work

- **`m1_close_pass` remains `false` (`OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`), held at Bar (a) clean-room `uor-r4-integer` 19-table reader:**
  - Although Path A (`trunk-joint-pointer-v1-d11` in [`crates/uor-r4-discrete/src/trunk.rs`](../../../../crates/uor-r4-discrete/src/trunk.rs)) now clears **Bar (b)** (`0` export gap via `export_trunk_d11` / `import_trunk_d11`), **Bar (c)** (`34,325`–`34,337 context tok/s` GPU on `ft/dev`, `76,316`–`76,343` TRAIN, `37,838`–`37,853` `fresh_v2`), and **Bar (d)** (`7,702` / `7,701` `roots0` and `7,386` / `7,374` `roots_final` `>= 6,324`, `P_succ = 2,090` / `2,086 >= 1,265` across both seeds), clean-room `IntegerDiscreteModel::from_bytes` in `crates/uor-r4-integer/src/discrete_d11.rs:702` (outside `#2837`'s file scope) currently enforces `header.tables.len() == 4` and rejects the 19-table `uor-r4.lut-discrete-trunk/1` container (`bar_a_d11_self_contained = false`).
  - Consequently, `d11-scorer fresh-score` (`#2841`) remains gated (`UNAVAILABLE_NOT_RUN`, `bar_e_fresh_transfer_two_seeds = false`), and `#2837` / `#2029` remain open (`NOT_RUN` is never `PASS`).
- **Codebook collision penalty on `roots_final` (`classification: mechanism` configuration finding):**
  - In Stage T `T16_fz` `final-checkpoint.json` (`roots_final`), `88` codebook vector collisions (`4,008 / 4,096` distinct 32-int vectors) cause `OuterL1V1` `token_id asc` tie-breaking to drop collided higher-ID targets even after exact `DiscreteHamiltonGateV1` rotation (`6,943 -> 6,674` / `6,665` on joint-hierarchy, `7,702` / `7,701 -> 7,386` / `7,374` on joint-pointer), though both `roots0` (`4,096 / 4,096` distinct vectors) and `roots_final` clear `T2 = 6,300` by `+365` to `+1,402` rows.
- **Prior `DEV_PSUCC_T2_TOP1 = 1,729` instrument bug (`classification: instrument`):**
  - `DEV_PSUCC_T2_TOP1` in [`docs/labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs) had copied `#2844`'s `fresh_v2` `Level 1_global` count (`1,729`). On `ft/dev` at `context = 192` (`3,847` `P_succ` rows), `Unigram` is `194 / 3,847` and `T2` (`query_table`) is **`1,245 / 3,847`** (`1,375 / 3,932` on `fresh_v2`). `trunk-joint-pointer-v1-d11` (`2,090` / `2,086`) beats both `1,245` (`+845` / `+841`) and `1,729` (`+361` / `+357`).

## What changed

- **Step 1 — 4-block $\mathcal{O}_{16}^4$ orbit-root causal pointer + 19-table `UORLUT01` container in [`crates/uor-r4-discrete/src/trunk.rs`](../../../../crates/uor-r4-discrete/src/trunk.rs):**
  - Added `SparseCountTable::to_triples` / `from_triples`, `orbit_root_causal_successor`, `TrunkTables::level3_joint_pointer_score` (`(c_qr << 6) + (c_qc << 5) + (c_rr << 3) + (c_r << 1) + c_q + (ptr_match << 8)`), `select_level3_joint_pointer_global`, `evaluate_joint_pointer_full_writer`, and `export_trunk_d11` / `import_trunk_d11` (`TRUNK_LUT_SCHEMA = "uor-r4.lut-discrete-trunk/1"`, 19 sections, 64-byte aligned, preserving `DRIVER_SOURCE_SHA256 = "04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d"`).
- **Step 2 — Corrected `DEV_PSUCC_T2_TOP1` and added `trunk-joint-pointer-v1-d11` in [`docs/labs/m1-d11-artifact-scorer/instrument/`](../../../labs/m1-d11-artifact-scorer/instrument/):**
  - Updated [`src/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs), [`tests/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/m1_close.rs), [`docs/labs/m1-close-saved-model/evidence/m1-close-grade.json`](../../../labs/m1-close-saved-model/evidence/m1-close-grade.json), [`docs/harness/prep/m1-close-saved-model-2026-10-26.md`](../../../harness/prep/m1-close-saved-model-2026-10-26.md), [`docs/labs/m1-close-saved-model/README.md`](../../../labs/m1-close-saved-model/README.md), and [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md).

## Measurements

- **Headline throughput (`CHARTER.md` §3.12):** `34,325`–`34,337 context tok/s` GPU on `ft/dev` (`76,316`–`76,343` TRAIN, `37,838`–`37,853` `fresh_v2`; `12,630` / `12,523 tok/s` laptop base D11 decode).
- **Two-seed stored-stream evaluation (`1,049,856` TRAIN, `14,287` `ft/dev`, `14,287` `fresh_v2`):**
  - `select_level3_joint_hierarchy_global` + post-rotation `OuterL1V1`: `ft/dev` **`6,943` / `6,943` (`+643` over `T2 = 6,300`)** on `roots0`, **`6,674` / `6,665` (`+374` / `+365`)** on `roots_final`, and **`6,875` / `6,875` (`+110` over `T2_fresh = 6,765`)** on stored `fresh_v2` streams.
  - `select_level3_joint_pointer_global` + post-rotation `OuterL1V1` (`trunk-joint-pointer-v1-d11`): `ft/dev` **`7,702` / `7,701` (`+1,402` / `+1,401` over `T2 = 6,300`)** on `roots0`, **`7,386` / `7,374` (`+1,086` / `+1,074` over `T2 = 6,300`)** on `roots_final`, `ft/dev P_succ` **`2,090` / `2,086` (`+845` / `+841` over `T2_psucc = 1,245`)**, and stored `fresh_v2` **`7,703` / `7,722` (`+938` / `+957` over `T2_fresh = 6,765`, `P_succ = 2,408` / `2,413 > 1,375`)** with `0` export gap in `import_trunk_d11`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --no-fail-fast`: all unit and integration tests passed (`7 / 7` `trunk::tests`, `1 / 1` `driver_source_sha256_matches_the_pin`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --no-fail-fast`: `20 / 20` passed (`2` `m1_close`, `10` `parity`, `5` `stage1`, `3` `toks`).
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/antigravity-106/ORDER.md`: `PASS`.
- `uor-harness loop gates --full`: `11 / 11` gates `GREEN`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`.

## Still open

- Extend `IntegerDiscreteModel::from_bytes` in `crates/uor-r4-integer/src/discrete_d11.rs` (outside `#2837`'s file scope) to accept the 19-table `uor-r4.lut-discrete-trunk/1` schema (`Bar (a)`), run `d11-scorer fresh-score` (`#2841`) on the sealed `Fresh V2` slice (`Bar (e)`), and close `#2837` / `#2029`.

## Plain summary

We implemented and tested the Path A 4-block $\mathcal{O}_{16}^4$ orbit-root causal pointer selector (`select_level3_joint_pointer_global` + `evaluate_joint_pointer_full_writer`) and 19-table `UORLUT01` container (`export_trunk_d11` / `import_trunk_d11`) in `crates/uor-r4-discrete/src/trunk.rs`, clearing Bar (b) (`0` export gap), Bar (c) (`34,325`–`34,337 context tok/s`), and Bar (d) (`7,702` / `7,701` `roots0` and `7,386` / `7,374` `roots_final` `> 6,300` `T2`, `P_succ = 2,090` / `2,086 > 1,245`, `fresh_v2 = 7,703` / `7,722 > 6,765`) across both seeds, leaving only the clean-room `uor-r4-integer` 19-table reader (`Bar (a)`) and `#2841` `d11-scorer fresh-score` (`Bar (e)`) to close M1.
