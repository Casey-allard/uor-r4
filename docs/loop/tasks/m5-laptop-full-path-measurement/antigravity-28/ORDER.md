# ORDER: m5-laptop-full-path-measurement, antigravity-28

Lab: antigravity. Item: [m5-laptop-full-path-measurement](../../../../roadmap/items/m5-laptop-full-path-measurement.md). Base: `29cec774f`.

## Goal

Repair all blocking and starred findings from [Claude's review (`#3012` comment `6353748795`)](https://github.com/UOR-Foundation/uor-r4/pull/3012#issuecomment-6353748795) and [DeepSeek's review (`#3012` comment `6353777438`)](https://github.com/UOR-Foundation/uor-r4/pull/3012#issuecomment-6353777438) on PR `#3012`:
1. **Fix the 4 broken links (CI `docs archive and links`):** point `docs/compute/energy-measurement-protocol-v3.md` and `docs/labs/m5-laptop-full-path-measurement/README.md` to `archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`, and point `docs/loop/tasks/m5-laptop-full-path-measurement/antigravity-27/REPORT.md` to `docs/roadmap/items/m4-export-uorlut01-cli.md` (`#2859`).
2. **Replace wrong upstream references (`#2861` / `#2857` / `m5-route-select-active-subset`)** with the actual upstream dependency `m4-export-uorlut01-cli` (`#2859`) across `antigravity-27/ORDER.md`, `antigravity-27/REPORT.md`, `README.md`, and the staged notebook row, and state the `--features serving-wasm` test command and CI-coverage note.
3. **Move Student-$t$ CI (`student_t_0975`, `sample_mean_sd_t_ci95`), `work_rate_w == gross_j_per_tok * tokens_per_s` validation (`validate_protocol_v3_energy_receipt`), and `objdump` zero-multiply/zero-float disassembly check (`verify_step_objdump_zero_mul_zero_float`) into `crates/uor-r4-integer/src/` (`src/stack/session.rs` and `src/bin/uor-r4-stack.rs`)** so `cost` computes unbiased sample SD + Student-$t$ 95% CIs, reports `total_ns` (`measured_total_step_us`) alongside `phase_sum_us` and `unaccounted_overhead_us`, verifies `objdump` on `step`, and refuses on logits/state drift, `objdump` violations, invalid `--energy-receipt`, or unexpected external daemons (unless `--allow-provisional`).
4. **Fix Protocol v3's description of `archive/ops/measurement_report_rnd.json`:** state accurately that `ci95` (`0.037 mJ`) belongs to `net_j_per_tok` (`min/mean/max = [3.128, 3.165, 3.192] mJ`), `[3.135, 3.172, 3.199] mJ` are `gross_j_per_tok`'s `min/mean/max`, and Protocol v2 §4 calls the `40.5 W` vs `54.53 W` gap unexplained.
   - *Correction note (recorded in [`REPORT.md`](REPORT.md), [`../antigravity-29/REPORT.md`](../antigravity-29/REPORT.md), and [`../antigravity-30/REPORT.md`](../antigravity-30/REPORT.md)):* `archive/ops/measurement_report_rnd.json` records `serving.energy.net_j_per_tok = {"mean": 0.00153, "ci95": 3.7e-05}` J (`1.530 ± 0.037 mJ`) with no `min`/`max`; `[3.128, 3.165, 3.192] mJ` in item 4 above was the synthetic 3-run fixture in `m5_full_path_cost_controls.rs`, not a field of `measurement_report_rnd.json`.

## Cancel probe (run first)

Passing `archive/ops/measurement_report_rnd.json` to `uor-r4-stack cost --energy-receipt` or `validate_protocol_v3_energy_receipt` in `crates/uor-r4-integer/src/stack/session.rs` fails with `PowerThroughputInconsistency` and `InvalidStudentTCi95`.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/compute/energy-measurement-protocol-v3.md`
- `docs/labs/m5-laptop-full-path-measurement/**`
- `docs/loop/tasks/m5-laptop-full-path-measurement/**`

## Done when

- Zero broken links in `uor-harness check --only links`.
- `student_t_0975`, `work_rate_w`, and `objdump` checks live in `crates/uor-r4-integer/src/` and are enforced by `uor-r4-stack cost`.
- All 3 tests in `m5_full_path_cost_controls` (`--features serving-wasm`) and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside `scope` must be modified.
