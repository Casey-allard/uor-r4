# M5 Laptop Full-Path Cost and Energy Measurement (`m5-laptop-full-path-measurement`)

**Item:** [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../../roadmap/items/m5-laptop-full-path-measurement.md) ([`#2862`](https://github.com/UOR-Foundation/uor-r4/issues/2862) / [`#2033`](https://github.com/UOR-Foundation/uor-r4/issues/2033))
**Protocol:** [`docs/compute/energy-measurement-protocol-v3.md`](../../compute/energy-measurement-protocol-v3.md)
**Provisional rehearsal receipt:** [`provisional-synthetic-stack-receipt.json`](provisional-synthetic-stack-receipt.json) (`sha256:eb257d403cd629c9cf04b012cfa5839e7574e0d426b7939748b3138b4d735958`)
**Finite host controls:** [`crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`](../../../crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs)

---

## 1. Classification and Scope Boundary

- **Status:** Host instrumentation, Protocol v3 specification, finite control suite, and provisional Apple M1 rehearsal on `mk-synthetic-stack` (`seed = 5`) complete. **M5 milestone acceptance (`status: done` on `m5-laptop-full-path-measurement`) is NOT claimed**, because the upstream M4 `UORLUT01` model export (`m4-export-uorlut01-cli`, [`#2859`](https://github.com/UOR-Foundation/uor-r4/issues/2859), [`docs/roadmap/items/m4-export-uorlut01-cli.md`](../../roadmap/items/m4-export-uorlut01-cli.md)) does not exist yet (`0 / 8` milestones complete).
- **Zero-multiply disassembly audit:** [`IntegerStackSession::step`](../../../crates/uor-r4-integer/src/stack/session.rs) and `stack_step` remain untouched. Running `python3 scripts/audit_zero_matmul_serving.py local/antigravity/target/release/uor-r4-stack --stack` passes all `44 / 44` mandatory symbol ranges and `87` reachable call-graph functions with `0` Class I (multiplier), `0` Class II (divider), and `0` Class III (floating-point / SIMD-to-general transfer) instructions, and `uor-r4-stack cost` runs `verify_step_objdump_zero_mul_zero_float` across the transitive call graph from `IntegerStackSession::step` and `stack::kernels::stack_argmax` (`107` symbols / `13,801` instructions on the `--release` binary; `114` symbols / `14,473` instructions on `CARGO_BIN_EXE_uor-r4-stack`, requiring all 43 mandatory `uor_r4_integer::stack` symbols with 0 missing, auditing stack closure instantiations and reachable non-allowlisted `core`/`alloc`/`compiler_builtins` callees including `alloc::vec::*` and `alloc::slice::*`, rejecting indirect calls `blr`/`call *` outside `rayon_core` work-stealing / TLS wrappers, permitting 64-bit `d0..d31` loads/stores solely for AArch64 NEON 2x`i32` integer SIMD `sqadd.2s` while banning `h0..h31`/`s0..s31` scalar float loads/stores and all float/multiply/divide opcodes, and disclosing in `disassembly_audit.legacy_ratchet_note` that `stack_exp_neg` and `stack_sigmoid_q31` are D11 sealed integer-table kernels tracked as legacy sites by the D23 ratchet).
- **Test command and CI coverage:** Run the full 3-test control suite (including the `serving-wasm`-gated `IntegerStackModel` and `uor-r4-stack cost` CLI tests) with:
  ```sh
  CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test m5_full_path_cost_controls
  ```
  (Standard CI runs `cargo test -p uor-r4-integer` without `--features serving-wasm`, which executes the Protocol v3 receipt validator and `verify_step_disassembly_text` directly from [`crates/uor-r4-integer/src/report_output.rs`](../../../crates/uor-r4-integer/src/report_output.rs) with zero duplicated fallback code, while `--features serving-wasm` executes all `3 / 3` tests.)

---

## 2. Protocol v2 Defects Closed by Protocol v3

[`archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`](../../../archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md) (§4) identified two defects in [`archive/ops/measurement_report_rnd.json`](../../../archive/ops/measurement_report_rnd.json):

| Defect | In [`archive/ops/measurement_report_rnd.json`](../../../archive/ops/measurement_report_rnd.json) | Exact Value | Relative Error | Protocol v3 Enforcement in [`crates/uor-r4-integer/src/report_output.rs`](../../../crates/uor-r4-integer/src/report_output.rs) |
|---|---|---|---|---|
| 1. Power / throughput inconsistency | `serving.energy.work_rate_w = 40.5 W` vs `gross_j_per_tok.mean (0.003172) * tokens_per_s.mean (17,190)` | `54.52668 W` | `25.72%` (`> 2.0%` limit) | `ProtocolV3ValidationError::PowerThroughputInconsistency` rejects any receipt where `\|work_rate_w - gross_j_per_tok * tokens_per_s\| / (gross_j_per_tok * tokens_per_s) > 0.02` |
| 2. Normal $z_{0.975} = 1.96$ instead of Student's $t_{0.975, n-1}$ (plus missing net `samples`/`sd`) | `serving.energy.net_j_per_tok` omits `samples`/`sd` (`{"mean": 0.00153, "ci95": 3.7e-5}` J = `1.530 ± 0.037 mJ`) across $n = 3$ runs (`gross_j_per_tok` min/mean/max `[3.135, 3.172, 3.199]` mJ $\implies$ exact 3 observations `[3.135, 3.182, 3.199]` mJ via $x_2 = 3\bar{x} - x_{\min} - x_{\max}$, $s = 0.033151\text{ mJ}$, $\text{SE} = 0.019140\text{ mJ}$, $1.96 \times \text{SE} = 0.037514\text{ mJ}$) | $t_{0.975, 2} \cdot \text{SE} = 4.302653 \times 0.019140 = 0.082352\text{ mJ}$ (`0.079815 mJ` if `[min, mean, max]` were treated as samples) | `55.07%` (`> 2.0%` limit) | `ProtocolV3ValidationError::InvalidSampleCount` rejects any `ci95` lacking its own `samples` ($n \ge 2$) or `(sd, n >= 2)`, and `InvalidStudentTCi95` rejects any `ci95` differing from $t_{0.975, n-1} \cdot s / \sqrt{n}$ by more than `2.0%` |
| 3. Non-quiescent host / non-positive net power | `provisional-synthetic-stack-receipt.json`: `external_inference_daemons_detected = ["ollama"]`, `work_rate_w = 23.319 W < idle_rate_w = 24.255 W` | `net_mj_per_tok = 0.0` | Refused in strict mode | `validate_protocol_v3_energy_receipt` (strict mode) and `uor-r4-stack cost` (without `--allow-provisional`) reject `ExternalInferenceDaemonsDetected` and `NonPositiveNetPower` |

---

## 3. Four-Phase Parameter and Latency Breakdown on `mk-synthetic-stack` (`seed = 5`)

Measured on Apple M1 (`Darwin 27.0.0 arm64`) over `mk-synthetic-stack` (`seed = 5`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `vocab = 40, width = 32, heads = 2, mlp = 32, pattern = "ra", context = 64`, `8` windows = `512` steps, `logits_sha256 = 10ff4339c329138e8352cd96c646580c0de082ac3924efb8df366063653524ba`, `phase_instrumentation_logits_match = true`):

| Phase | Active 4-Bit Weights / Token | Total 4-Bit Weights | Active Weight+Scale Bytes / Token | Active / Total Learned Scalars (Bytes) | Mean Latency ($\mu\text{s}$) | p50 ($\mu\text{s}$) | p90 ($\mu\text{s}$) | p99 ($\mu\text{s}$) | Share of Step |
|---|---|---|---|---|---|---|---|---|---|
| 1. `ingestion` (`embed`) | `32` | `1,280` | `17 B` (`/ 680 B`) | `0 / 0` (`0 B`) | `0.305` | `0.416` | `0.459` | `0.625` | `0.23%` |
| 2. `routing` (`l0` `r` + MLP) | `7,424` | `7,424` | `3,944 B` | `208 / 208` (`560 B`) | `73.134` | `51.167` | `54.750` | `1,702.541` | `56.18%` |
| 3. `memory_lookup` (`l1` `a` + MLP) | `7,232` | `7,232` | `3,842 B` | `71 / 134` (`532 B`) | `53.284` | `31.625` | `77.708` | `135.458` | `40.93%` |
| 4. `emission` (`head`) | `1,280` | `1,280` | `680 B` | `0 / 0` (`0 B`) | `3.449` | `4.375` | `4.667` | `5.042` | `2.65%` |
| **Total** | **`15,968`** | **`17,216`** (`92.7509%` active) | **`8,483 B`** (`/ 9,146 B`) | **`279 / 342`** (`1,092 B`) | **`130.172`** (unphased mean `103.842` $\mu\text{s}$, p50 `100.041` $\mu\text{s}$, p90 `139.417` $\mu\text{s}$, p99 `350.041` $\mu\text{s}$) | — | — | — | **`100.00%`** |

### Memory and Process Isolation Summary (`provisional-synthetic-stack-receipt.json`)

- **Cold load:** `3.426 ms` (`0.003426 s`).
- **Kernel high-water-mark footprint (`vmmap -summary` `Physical footprint (peak)`):** `3,784,704 B` (`3.609 MiB`).
- **Multi-stage `ps -o rss=` checkpoints:** `initial_rss_mb = 1.969 MiB`, `post_load_rss_mb = 3.250 MiB`, `post_session_alloc_rss_mb = 4.516 MiB`, `post_eval_rss_mb = 4.578 MiB`, `peak_rss_mb = 4.578 MiB`.
- **Analytical bytes touched per token:** `341,531 B/tok` (shape-derived analytical calculation, labelled distinctly from hardware performance-counter traffic).
- **Process isolation & machine state:** `external_inference_daemons_detected = ["ollama"]` (detected `/Applications/Ollama.app` running on the host) and `load_average_before = "{ 31.69 34.64 32.30 }"` (concurrent five-lab loop builds active). Because the host was not quiescent and the artifact is the `17,558`-parameter synthetic fixture (`92.75%` active 4-bit weights per token), this receipt is explicitly marked `"qualification_status": "PROVISIONAL_SYNTHETIC_FIXTURE_NOT_M5_ACCEPTANCE"`.
- **Whole-system power & energy rehearsal (`macmon pipe -s 0 -i 100`, `sys_power`, $n = 3$ trials of `32,768` tokens):**
  - `idle_rate_w = 24.255221 W` (`43` samples over `5.005 s` under concurrent host load).
  - `work_rate_w = 23.319112 W`, `tokens_per_s = 21,858.645 tok/s`, `gross_j_per_tok = 0.001066814 J/tok` (`0.001066814 * 21858.645 = 23.319112 W`, `0.00%` power/throughput discrepancy).
  - `gross_mj_per_tok`: samples `[0.988340, 0.964506, 1.247596]` mJ/tok, `mean = 1.066814 mJ/tok`, `sd = 0.157015 mJ/tok`, Student-$t_{0.975, 2}$ `ci95 = 0.390046 mJ/tok`.

---

## 4. Staged Notebook Row (for Backfill Once M5 Candidate Runs)

| Date | Line | Question / Hypothesis | Artifact / Commit | Result | Classification |
|---|---|---|---|---|---|
| `2026-10-22` | `m4-serving` | Does `uor-r4-stack cost` + Protocol v3 (`docs/compute/energy-measurement-protocol-v3.md`) measure dynamic per-phase parameter counts (`ingestion`, `routing`, `memory_lookup`, `emission`), per-phase latency (`p50`/`p90`/`p99` + `measured_total_step_us` and `unaccounted_overhead_us`) outside the zero-multiply `step` call graph, kernel high-water-mark memory (`vmmap` / `VmHWM`), and Student-$t_{0.975, n-1}$ energy confidence intervals while rejecting Protocol v2's `measurement_report_rnd.json` defects and non-quiescent hosts? | `crates/uor-r4-integer/src/report_output.rs`, `crates/uor-r4-integer/src/stack/session.rs`, `crates/uor-r4-integer/src/bin/uor-r4-stack.rs`, `docs/labs/m5-laptop-full-path-measurement/provisional-synthetic-stack-receipt.json` (`sha256:eb257d403cd629c9cf04b012cfa5839e7574e0d426b7939748b3138b4d735958`) | On `mk-synthetic-stack` (`seed = 5`), `parameter_breakdown` reports `15,968 / 17,216` active 4-bit weights/tok (`92.7509%`, `32 + 7,424 + 7,232 + 1,280`) and `279 / 342` active learned scalars/tok (`16,247 / 17,558` active learned parameters/tok); `measure_step_phases` matches `step` logits bit-for-bit (`10ff4339...`); `audit_zero_matmul_serving.py --stack` and `verify_step_objdump_zero_mul_zero_float` pass all 43 mandatory symbols and transitive call graph (`107` symbols / `13,801` instructions on `--release`, `114` symbols / `14,473` instructions on `CARGO_BIN_EXE_uor-r4-stack`, `0` violations); Protocol v3 validator catches both `measurement_report_rnd.json` defects (`25.72%` power/throughput mismatch and `55.07%` $z$-vs-$t_2$ CI95 understatement plus missing net samples/sd) and strict mode refuses non-quiescent receipts. | `instrument ready` (provisional synthetic fixture only; M5 acceptance awaits `m4-export-uorlut01-cli` `#2859` and owner bar registration on `#2862` / `#2033`) |
