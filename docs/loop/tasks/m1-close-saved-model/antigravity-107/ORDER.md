# ORDER: m1-close-saved-model, antigravity-107

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `bb38c1e93`.

## Goal

The item's done-when reads:

> One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

In `antigravity-106` (`#3304`, `bb38c1e93bf8cb1d0e22028fe47113a57214079e`), `trunk-joint-pointer-v1-d11` (`export_trunk_d11` / `import_trunk_d11` in `crates/uor-r4-discrete/src/trunk.rs`) cleared Bars (b), (c), and (d) on two seeds (`7,386 / 14,287` and `7,374 / 14,287` on `roots_final`; `7,702 / 14,287` and `7,701 / 14,287` on `roots0`, vs `T2 = 6,300`), while Bar (a) remained blocked solely because `IntegerDiscreteModel::from_bytes` in `crates/uor-r4-integer/src/discrete_d11.rs:702` rejected any `UORLUT01` container with `header.tables.len() != 4`.

Per the approved D68 decision card ([`DECISION.md`](DECISION.md)):
1. Extend `IntegerDiscreteModel::from_bytes` and `IntegerDiscreteModel::evaluate_prefix` in `crates/uor-r4-integer/src/discrete_d11.rs` to accept and evaluate the 19-table `uor-r4.lut-discrete-trunk/1` schema under `#![forbid(unsafe_code)]` and `D11 R1–R5` (`0` floats, `0` squashing/softmax, `0` dense matmul).
2. Prove `0` export gap and planted-fault `B4` rejection between `uor-r4-discrete::trunk::export_trunk_d11` and `uor-r4-integer::discrete_d11::IntegerDiscreteModel`, and verify laptop `time_serving` (`d11-scorer toks`).
3. Update `m1_close.rs`, `m1-close-grade.json`, `docs/labs/m1-close-saved-model/README.md`, and `docs/research/notebook/m1-discrete-attention.md` to record that `trunk-joint-pointer-v1-d11` clears **Bars (a), (b), (c), and (d)** through `crates/uor-r4-integer`, leaving only **Bar (e)** (`#2841` `d11-scorer fresh-score`).

## Cancel probe (run first)

Run `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-integer --no-fail-fast` on `bb38c1e93bf8cb1d0e22028fe47113a57214079e` and verify `IntegerDiscreteModel::from_bytes` rejects 19-table `uor-r4.lut-discrete-trunk/1` at `crates/uor-r4-integer/src/discrete_d11.rs:647–704`.
- **Result (2026-10-26):** Confirmed (`25 passed`; `discrete_d11.rs:647` and `:702` reject `uor-r4.lut-discrete-trunk/1` / `tables.len() != 4`). Classified as **instrument** (clean-room `UORLUT01` reader schema support gap, repaired in this unit).

## Files in scope

- `crates/uor-r4-discrete/**`
- `crates/uor-r4-integer/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-close-saved-model*`
- `docs/labs/m1-close-saved-model/**`
- `docs/loop/tasks/m1-close-saved-model/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-close-saved-model.md`

## Done when

1. `IntegerDiscreteModel::from_bytes` and `IntegerDiscreteModel::evaluate_prefix` in `crates/uor-r4-integer/src/discrete_d11.rs` accept and evaluate the 19-table `uor-r4.lut-discrete-trunk/1` container with `0` export gap against `uor-r4-discrete::trunk::evaluate_joint_pointer_full_writer`, reject planted faults (`B4`), and support `d11-scorer toks` (`time_serving`).
2. `m1_close.rs` and `m1-close-grade.json` verify `clean_room_19table_d11_accepted = true`, `bar_a_pass = true`, `bar_b_pass = true`, `bar_c_pass = true`, and `bar_d_pass = true` for `trunk-joint-pointer-v1-d11` through `crates/uor-r4-integer`.
3. All unit/integration tests and `uor-harness loop gates --full` pass cleanly.

## Stop and ask if

- Any change would violate `CHARTER.md` §3 or touch an owner-only path (`docs/harness/owner-paths.json`).
