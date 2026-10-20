# D4-E8 and D11-COST: Measured Results for Weight Codecs and Stack Engine

September 28, 2026. References #973 under #820.
- **Lab:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Discretization, Serving Kernels & Execution Audits).
- **Lane:** Local M1 cost, zero-multiplier kernels, discrete codecs.
- **Owner Policies:** D0-b, D11, D12.

---

## (A) D4-E8: Float-Reference Measurement of Geometric Weight Codecs

### 1. Protocol & Sealed Root
- **Pre-Registration:** GitHub issue #973 (comment 5879142377).
- **Stopping Rule:** If no arm beats GPTQ (+0.0257 nats) or meets the D4 gate (ΔNLL ≤ +0.0200 nats at ≤ 4.25 bits/weight), do not modify serving formats or kernels. Record honest numbers and classify under D12 as "not yet promoted at this scope".
- **Target Model:** `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model` (SHA-256 `3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a`).
- **Validation Dataset:** `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/valid.u16` (SHA-256 `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8`).
- **Evaluator Protocol:** 512 evenly spaced windows of 256 tokens = 131,072 targets (stride 403).
- **Sealed Report Root:** `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/float-reference-1` (claimed, sealed, and verified via `report_output::{claim, seal, verify}`).

### 2. Measured 5-Arm Summary Table

| Arm | Mean NLL | ΔNLL vs Float (nats) | Top-1 Agreement | Bits / Weight | D4 Gate (≤ +0.0200 nats, ≤ 4.25 bpw) |
|:---|---:|---:|---:|---:|:---:|
| **Float Baseline** | 1.998113 | 0.000000 | 100.00% | 32.0000 | REF |
| **(i) RTN Baseline** | 2.034342 | +0.036230 | 90.65% | 4.2504 | FAIL (ΔNLL > +0.02 nats) |
| **(ii) Online Hadamard + Grouped 4-bit (H+G4)** | 2.025120 | +0.027007 | 91.28% | 4.2725 | FAIL (Over budget: 4.2725 > 4.25 bpw; ΔNLL > +0.02 nats) |
| **(iii) Online Hadamard + E8 Lattice (single-shell, ~2 bpw)** | 4.421591 | +2.423478 | 31.18% | 2.0143 | Single-shell ~2 bpw; matched-bit E8: NOT TESTED |
| **(iv) Online Hadamard + E8 (Finer Head Scale, ~2.2 bpw)** | 4.419581 | +2.421468 | 31.22% | 2.1795 | Single-shell ~2 bpw; matched-bit E8: NOT TESTED |
| **(v) Negative Control (Scrambled Scales)** | 24.173992 | +22.175879 | 0.10% | 4.2500 | FAIL (Damage Detected) |

### 3. Analysis & D12 Disposition
1. **RTN Reproduction:** RTN evaluated to exactly 2.034342 nats (+0.036230 nats), reproducing Lab 1's baseline to 6 decimal places.
2. **Hadamard Grouped 4-bit (H+G4):** Online randomized Walsh-Hadamard transform with grouped 4-bit quantization achieves 2.025120 nats (+0.027007 nats). It is over the bit budget at 4.2725 bits per weight (the gate is 4.25 bpw), as well as above +0.0200 nats.
3. **E8 Lattice Quantization (Single-Shell):** The evaluated E8 arms are single-shell (240 roots + zero vector) at about 2 bits per weight (2.0143 bpw and 2.1795 bpw). They test uncalibrated ~2 bpw compression; **matched-bit E8: NOT TESTED** in this sealed evaluation. (Two-stage residual matched-bit E8 at ~4.0 bpw is now implemented in `crates/` for the next D4 task).
4. **D12 Disposition & Exact Negative Scope:** Under owner decision D12, gates promote, never kill. H+G4 is over the bit budget at 4.2725 bits per weight and above 0.02 nats; the evaluated E8 arms are single-shell at about 2 bits per weight (matched-bit E8: NOT TESTED). Neither candidate is promoted at this scope. In accordance with the pre-registered stopping rule, these negative results are retained at their exact technical scope.

---

## (B) D11-COST: M1 Cost of Merged D11 Stack Engine & Audited Speedup

### 1. Protocol & Target Artifact
- **Target Artifact:** `/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut` (SHA-256 `f14d6c13fd5e6b526d133f1d6aa89ede560ef9326be08d5cd6b43a2727d99c98`).
- **Validation Dataset:** `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout/tokens.u16` (SHA-256 `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`).
- **Stack CLI Binary:** `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/uor-r4-stack` (SHA-256 `854d73368975ea66397745be31c9d33c35bbb3d904f9c5b3e718384df53c30aa`).
- **Evaluation Driver Binary:** `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/examples/geometric-stack` (SHA-256 `92b79f963afd61e59734615c473dd3d3e63ca1aa1cbcc223fb323bd7a0cbd53d`).
- **Windows:** 8 evenly spaced windows = 2,048 targets.
- **Hardware:** Apple M1 (single-threaded serving thread, release build with `opt-level = 3`, thread count: 1).
- **Bit-Identity Oracle:** `uor_r4_integer::stack` must match `uor_r4_lut::stack` NEON logits bit-for-bit (`max_abs_logit_difference == 0`).

### 2. Measured M1 Cost Comparison

| Engine / Configuration | NLL | Step Latency (ms/token) | Throughput (tokens/s) | Speedup vs Baseline | Bit-Identity (`max_abs_diff`) | Zero-Matmul Audit | Sealed Report Root |
|:---|---:|---:|---:|---:|:---:|:---:|:---|
| **D10 Reference (NEON)** | 2.956329 | 1.110 | 900.92 | — | 0 | (Excluded: contains NEON/floats) | `s2-d11-release-optimized-1` |
| **D11 Baseline** | 2.956329 | 6.061 | 164.99 | 1.00x | 0 | FULL PASS | `s2-d11-release-baseline-1` |
| **D11 Audited ILP Speedup** | 2.956329 | 5.448 | 183.55 | no measured speedup: Lab 1's interleaved A/B, six base/ILP pairs, −0.3% (within noise), bit-identical (`claude-1479-rerun/interleaved-1/`) | **0** | **FULL PASS** | `s2-d11-release-optimized-1` |

- **Execution Context:** Single-threaded execution (1 thread).
- **Interleaved A/B Outcome & Qualified Step Cost:** Lab 1's quiet-machine interleaved A/B across six alternating base/ILP pairs showed −0.3% throughput difference (within noise) and bit-identical logits (`claude-1479-rerun/interleaved-1/`), establishing no measured speedup from the ILP unrolling. Qualified step cost: about 5.40 ms/token at 1 thread on the S2 artifact, quiet machine, from the same roots. Process RSS and whole-system energy stay unqualified.
- **Numerical Parity Gate:** Fully verified across all 2,048 evaluation positions: `max_abs_logit_difference == 0`, `top1_agreement == 1.000000`, `positions_with_a_difference == 0`.
- **Weights Traversed Per Token (Analytical Count):** 7,238,304 weights/token.
- **Cold Load Time:** 20.6 ms.

### 3. Substance of the Audited Speedup
In `crates/uor-r4-integer/src/stack/kernels.rs`:
1. `stack_gemv_pairs`: Replaced sequential accumulation across zipped slices with 4-way unrolling into independent accumulators `(a0, a1, a2, a3)`. This breaks serial data dependencies on Apple Silicon M1's 8-wide out-of-order execution pipeline, allowing load instructions from table lookups to issue concurrently.
2. `stack_gemv`: Replaced serial pair traversal with 4-way parallel accumulation for projection maps (`.down` and `.out`).
3. **Audit Verification:** `python3 scripts/audit_zero_matmul_serving.py /Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/uor-r4-stack --stack` yields **FULL PASS**:
   - Class I (hardware multipliers): 0
   - Class II (hardware dividers): 0
   - Class III (floating-point instructions / vector registers): 0
   - Checked across all 31 mandatory symbol ranges and 34 reachable call-graph functions.
4. **Oracle Verification:** `cargo test -p uor-r4-training --test stack_d11_oracle --offline` yields 6 passed, 0 failed. Note: `stack_d11_oracle` tests small synthetic stack fixtures, numerical corner cases, and error-handling wrap semantics in the release binary; it is an integration and bounds unit test, **not an evaluation of the 2,048-target S2 model benchmark**. Likewise, unit adapter tests (`d4_map_codec_adapter`) test arithmetic and round-trip mechanics on synthetic matrices, which does not constitute proof of trained model fidelity on full evaluation sets.

### 4. Comprehensive M1 Cost Profile (Sealed Root `d11-cost-profile-1`)

> [!WARNING]
> **Provenance and Lineage Disclosure:**
> As documented in [Evidence Lineage and Resealing Correction](d11-cost-profile-1-lineage-correction.md), `d11-cost-profile-1` is an **annotated report**, not the raw immutable run record.
> We formally withdraw any assertion that git commit history or source tracking proves bitwise identity of the overwritten pre-reseal raw disk bytes.
> The exact post-reseal bytes are preserved read-only at `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1.snapshot-preserved` (`chmod -R a-w`), and authentic gating qualification awaits Lab 1's re-run into a fresh, distinct report root.

Sealed root: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1` (claimed, sealed, and verified via `report_output::{claim, seal, verify}`).
Execution configuration: 1 thread, release build with `opt-level = 3`.
Full Provenance Identities:
- **Binary Identity (`uor-r4-stack`):** SHA-256 `854d73368975ea66397745be31c9d33c35bbb3d904f9c5b3e718384df53c30aa`
- **Model Identity (`model.lut`):** SHA-256 `f14d6c13fd5e6b526d133f1d6aa89ede560ef9326be08d5cd6b43a2727d99c98`
- **Dataset Identity (`tokens.u16`):** SHA-256 `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`
- **Thread Count:** 1 thread (`threads: 1` in machine record)

| Metric | Measured Value | Field Classification & Notes |
|:---|---:|:---|
| **Cold Load Time** | 16.34 ms | Reads and parses 4.97 MB `.lut` artifact |
| **Prompt Ingestion Latency** | 5.26 ms/token | 190.07 tokens/s (64 prompt tokens in 0.337 s) |
| **Input-Driven Forward Throughput (1 thread)** | 184.58 tokens/s | Forward step rate stepping through 2,048 tokens across 8 windows of context 256 (not free-running autoregressive generation) |
| **Step Latency (Min)** | 4.81 ms | |
| **Step Latency (p50)** | 5.40 ms | Median per-token forward step latency |
| **Step Latency (Mean ± StdDev)** | 5.42 ± 0.45 ms | |
| **Step Latency (p90)** | 5.75 ms | 90th percentile step latency |
| **Step Latency (p95)** | 5.93 ms | 95th percentile step latency |
| **Step Latency (p99)** | 6.72 ms | 99th percentile step latency |
| **Step Latency (Max)** | 15.47 ms | Single-step outlier |
| **Process RSS (Initial)** | 2.05 MB | Point-in-time `ps` sample before loading artifact |
| **Process RSS (Post-Load)** | 12.14 MB | Point-in-time `ps` sample after model resident load |
| **Process RSS (Post-Evaluation Sample)** | 49.22 MB | Point-in-time `ps` sample after evaluation completion (not a high-water mark peak) |
| **Weights Traversed Per Token (Analytical Count)** | 7,238,304 | Shape-based count: 3,845,349 bytes (weights + scales) |
| **Active Working Set (Analytical Estimate)** | 12,373,333 bytes | Analytical sum derived from model parameter shapes, lookup tables, scratch buffers, and session states (not instrumented hardware memory bus traffic) |

**Cost Field Definitions & Clarifications:**
- **Throughput Mode:** 184.58 tokens/s reflects input-driven forward evaluation where token inputs are fed sequentially and logits computed per position (`input_driven_forward_throughput_tokens_per_second`). It does not measure free-running autoregressive generation with categorical or min-p sampling.
- **Process RSS:** The 49.22 MB figure was captured via a single `ps` RSS query immediately following evaluation completion (`post_eval_rss_mb`). It is a point-in-time snapshot, not a continuous high-water mark peak RSS.
- **Memory Working Set:** The 12,373,333 bytes/token figure (`analytical_bytes_touched_per_token_estimate`) is an analytical model-structure calculation aggregating parameter storage, table buffers, context buffers, and scratch arrays. It is not an instrumented hardware memory bus transaction measurement (e.g. from hardware performance counters).
- **Weights Read:** The 7,238,304 count (`analytical_weights_read_per_token`) is a structural shape calculation of weights evaluated per forward step.

### 5. Whole-System Energy Measurements (`macmon` $\Delta E / \Delta N$)
Measured using `/opt/homebrew/bin/macmon` (sys_power) via `scripts/energy_per_token.py` following ROADMAP §4.3 protocol with visible idle floor subtraction across two run lengths:

| Run Length | Tokens | Wall Time (s) | Throughput (tok/s) | Idle Baseline Power (mW) | Workload Power (mW) | Gross Energy (J) | Gross J/token | Net Energy ($\Delta E$) | Net J/token ($\Delta E / \Delta N$) | Status |
|:---|---:|---:|---:|---:|---:|---:|---:|---:|---:|:---|
| **Run Length 1** | 4,096 | 23.143 s | 176.98 | 13,248.4 ± 3,656.3 | 14,259.1 ± 2,618.4 | 330.003 J | 0.0806 J/tok (80.6 mJ) | 23.389 J | **0.0057 J/tok (5.7 mJ)** | Unvalidated (Truncated Capture) |
| **Run Length 2** | 10,240 | 56.321 s | 181.82 | 12,144.8 ± 1,412.0 | 15,958.0 ± 5,042.2 | 898.766 J | 0.0878 J/tok (87.8 mJ) | 214.766 J | **0.0210 J/tok (21.0 mJ)** | Unvalidated (Truncated Capture) |

**Energy Capture Limitations & Recorder Repair:**
- **Capture Truncation Defect:** Both raw captures (`energy-4096.raw` and `energy-10240.raw`) were found to be exactly 65,563 bytes in length, matching the Darwin 64 KiB OS pipe buffer limit. In the inherited harness, `macmon` stdout was not concurrently drained during execution; the sampler process blocked on `write()` after filling the pipe buffer, capturing only ~3.7 s of data for Run Length 1 (instead of 23.1 s) and ~3.6 s for Run Length 2 (instead of 56.3 s).
- **Status of Reported Figures:** The 5.7 mJ/token and 21.0 mJ/token numbers are unvalidated historical attempts resulting from incomplete sampling coverage. The raw files are preserved in `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/` as unvalidated historical records.
- **Harness Resolution:** `scripts/energy_per_token.py` has been repaired to:
  1. Concurrently drain sampler stdout and stderr via dedicated background reader threads during execution.
  2. Preserve both idle baseline and workload samples in `--raw-out`.
  3. Formally validate timestamp coverage across both intervals, rejecting runs that fail to span the full execution duration.
