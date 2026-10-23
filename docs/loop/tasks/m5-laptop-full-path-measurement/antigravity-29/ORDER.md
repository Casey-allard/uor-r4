# ORDER: m5-laptop-full-path-measurement, antigravity-29

Lab: antigravity. Item: [m5-laptop-full-path-measurement](../../../../roadmap/items/m5-laptop-full-path-measurement.md). Base: `5f65ed0b8`.

## Goal

Repair all findings from [Claude's re-review (`#3012` comment `6354315779`)](https://github.com/UOR-Foundation/uor-r4/pull/3012#issuecomment-6354315779):
1. **Expand `verify_step_disassembly_text` across the full `uor_r4_integer::stack` serving call graph:** Audit `IntegerStackSession::step` plus all emitted `stack::session::stack_*` mixers, `stack::kernels::stack_*` kernels, and `uor_r4_integer::stack` closures, requiring the full serving call graph (`>= 35` symbols) in `verify_step_objdump_zero_mul_zero_float` so a multiply, divide, or float instruction in any mixer or kernel fails the check.
2. **Eliminate the test-only validator fallback copy so standard CI tests the real crate code:** Move `student_t_0975`, `student_t_975`, `sample_mean_sd_t_ci95`, `ProtocolV3ValidationError`, `validate_protocol_v3_energy_receipt`, `validate_protocol_v3_energy_receipt_with_mode`, `verify_step_disassembly_text`, and `verify_step_objdump_zero_mul_zero_float` into [`crates/uor-r4-integer/src/report_output.rs`](../../../../../crates/uor-r4-integer/src/report_output.rs) (unconditionally compiled in both default CI and `--features serving-wasm`, and re-exported in `uor_r4_integer::stack`) and delete `fallback_v3_validator` from [`crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`](../../../../../crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs).
3. **Fix legacy vs $n \neq 3$ CI95 validation and $n < 2$ refusal:** Support any $n \ge 2$ via `samples` or `(sd, n)` in `validate_protocol_v3_energy_receipt_with_mode`, restrict the 3-point `(min, mean, max)` reconstruction strictly to legacy $n = 3$ receipts that lack `samples` and `sd`, and return `InvalidSampleCount` when $n < 2$.
4. **Restore `antigravity-27/ORDER.md` `Done when` wording and correct the `antigravity-28/ORDER.md` v2 record note:** Revert bullet 2 in [`antigravity-27/ORDER.md`](../antigravity-27/ORDER.md) to its original text, label the synthetic 3-run fixture in `m5_full_path_cost_controls.rs`, and document in [`antigravity-28/REPORT.md`](../antigravity-28/REPORT.md) and [`antigravity-29/REPORT.md`](REPORT.md) that `archive/ops/measurement_report_rnd.json` reports `net_j_per_tok = {"mean": 0.00153, "ci95": 0.000037}` J (`1.530 ± 0.037 mJ`) with no `min`/`max`.
5. **Tighten `uor-r4-stack cost` and `StackParameterBreakdown`:** Compute per-token active learned scalars (`age` window average `H(C+1)/2` and single-row `learned_rank` `H*M` rather than full `H*C` and `H*M^2`), refuse silent `ps` fallback when `kernel_peak_memory_bytes()` is `None` unless `--allow-provisional`, check internal work-rate/latency identity on every `cost` run, run pre-flight refusals before `report_output::claim(out_dir)`, and fix exact `git diff --numstat` counts and Cornish-Fisher order wording.

## Cancel probe (run first)

Injecting `mul x0, x1, x2` into `stack::kernels::stack_gemv_pairs` in disassembly text must fail `verify_step_disassembly_text`.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/compute/energy-measurement-protocol-v3.md`
- `docs/labs/m5-laptop-full-path-measurement/**`
- `docs/loop/tasks/m5-laptop-full-path-measurement/**`

## Done when

- `cargo test -p uor-r4-integer --test m5_full_path_cost_controls` (without and with `--features serving-wasm`) passes with zero duplicated validator code.
- `uor-harness check`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside `scope` must be modified.
