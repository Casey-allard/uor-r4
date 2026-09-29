# D4: Stack Representation Codec Evaluation — Measurements

September 28, 2026. References #973 under #820.
- **Lab:** Lab 3 (Anti-Gravity: Numerical Learning, Codecs, Native Efficiency & Serving Verification).
- **Tool:** [`load-candidate-forward-report`](../../crates/uor-r4-training/src/bin/load-candidate-forward-report.rs).
- **Sealed Root:** `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/evaluation-1/`
  - Manifest SHA-256: `829103162507764bb08e653fb4c2eb36bce1b1c6fcc783b187ccba5c3b16efeb`
  - Sealed at: `1790619945` UTC (September 28, 2026)
  - Attempt Claim: `1790618870` UTC (PID 66261)
- **Evidence:** [`d4-stack-representation-measurements-2026-09-28.json`](../evidence/d4-stack-representation-measurements-2026-09-28.json).
- **Code:** `lab/anti-gravity/stack-representation-d4` (PR #1468).

---

## Executive Summary

1. **Representation experiment with no selected winner.**
   None of the evaluated arms meets the D4 fidelity gate ($\le 0.02$ nats degradation against float at $\le 4.25$ bits/weight):
   - **RTN Baseline:** +0.036229 nats
   - **Hadamard + Grouped 4-bit (H+G4):** +0.063668 nats
   - **Head-Compensated 4-bit:** +0.036229 nats
   The stronger S1.0 GPTQ reference (+0.0257 nats) remains a relevant comparison. This study is recorded as negative and baseline evidence; no candidate is promoted.

2. **Evaluator engine vs. serving audit scope separation.**
   - The evaluation below measures `uor_r4_lut::stack::StackModel`, the D10 NEON multithreaded comparator engine, run on the canonical `uor-r4.lut-stack/1` artifact format.
   - The cited zero-matmul static audit (`scripts/audit_zero_matmul_serving.py`) certifies `libuor_r4_integer.rlib` and `uor-r4-stack` under D11 (0 Class I mul, 0 Class II div, 0 Class III float).
   - An audit of `libuor_r4_integer.rlib` does **not** certify `uor_r4_lut::stack`. The comparator measurements and D11 serving library audit scopes are distinct and tracked separately.

3. **Empirical NLL agreement vs. bitwise arithmetic equality.**
   The measured integer arithmetic delta of **$-4.04 \times 10^{-7}$ nats** demonstrates close empirical agreement in mean NLL on this dataset between float reference execution and integer evaluation. It is **not** bitwise logit equality. (Bit-for-bit logit equality between D10 and D11 integer serving engines is verified separately by `stack_d11_oracle`).

4. **Hadamard negative scope.**
   Applying offline randomized Hadamard transformation to weights followed by grouped 4-bit quantization and inversion back to unrotated parameter space yields +0.063668 nats (+0.0274 nats worse than RTN). This is a scoped negative showing that offline rotation with de-rotation before D11 table quantization incurs double-quantization noise. It is **not** a theorem that all post-training alternatives require online transforms or QAT.

5. **Head-compensated parameter inspection.**
   Inspection of the packed parameters reveals that `head_compensated` selected identical `exp_base` and scales as RTN because the head's scale dynamic range ($e_{\max} - e_{\min} \le 15$) was within bounds. Consequently, the packed weight tensors were 100% bitwise identical to RTN; the 64-byte difference in `model.lut` was solely due to the 26-byte metadata increase in the container JSON header shifting the 64-byte aligned data payload boundary (`data_start` 8960 -> 9024).

6. **Usable codec component delivered.**
   To advance toward D4 without repeating losing arms, Lab 3 implemented and verified `Grouped4BitCodec` in `uor-r4-integer::codec` with exact quantization, dequantization, BLAKE3-checksummed serialization, 4.25 raw parameter bits/weight ($4.0$ code + $0.25$ scale for $G=32$, with container-framed bits/weight honestly reported per matrix shape, e.g. $4.2504$ for $4096 \times 288$ and $4.2587$ for $288 \times 749$, per the exact storage formula $8 \cdot (\lceil RC/2 \rceil + R \lceil C/32 \rceil + 64) / (RC)$), and pluggable `MapCodec` integration for QAT training. Under D12 this result is **not yet promoted at this scope**.

---

## 1. Experimental Setup

- **Model:** `geometric_s1` (reference model, 7,153,860 parameters)
  - Architecture: `rrarra`, Lorentz score, learned rotation.
  - Dimensions: width 288, heads 6, MLP hidden 749, context 256, vocab 4096.
  - Path: `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model/model.safetensors`
  - SHA-256: `3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a`
- **Data:** `valid.u16` (cycle-3 code split, 206,844 tokens)
  - 512 evenly spaced windows of length 256 (stride 403, 131,072 targets).
  - SHA-256: `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8`
- **Comparator Engine:** `uor_r4_lut::stack::StackModel` (2 threads, NEON SIMD backend).
- **Artifact Container:** Canonical `uor-r4.lut-stack/1` format via `uor_r4_training::stack_export`.

---

## 2. Full 512-Window Evaluation Results (131,072 Targets)

| Codec Arm | Float Baseline NLL | Float Ref NLL | Integer Serving NLL | Integer − Float ($\Delta$ nats) | Weight Rounding | Integer Arithmetic | Top-1 Agreement | Decision-Flip Rate | Serving Speed |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Round-to-nearest (RTN Baseline)** | 1.998113 | 2.034342 | 2.034342 | **+0.036229** | +0.036230 | -4.04e-07 | 0.9065 | 0.0935 | 450.9 tok/s |
| **Hadamard + Grouped 4-bit (H+G4)** | 1.998113 | 2.061781 | 2.061780 | **+0.063668** | +0.063668 | -4.49e-07 | 0.8749 | 0.1251 | 478.7 tok/s |
| **Head-Compensated 4-bit** | 1.998113 | 2.034342 | 2.034342 | **+0.036229** | +0.036230 | -4.04e-07 | 0.9065 | 0.0935 | 408.7 tok/s |

### Artifact Characteristics & Exact Hashes

- **RTN Baseline:** 4,969,412 bytes, SHA-256 `e79551baad36f8ddece6b95dfbf9345d362447051e2b2f2ea868d357092e4c97`.
- **Hadamard + Grouped 4-bit:** 4,969,476 bytes, SHA-256 `2d63868d58cd7fa706d2b4d7d3c8ae66ace925fa68000aac708ae5d7254404dc`.
- **Head-Compensated 4-bit:** 4,969,476 bytes, SHA-256 `94ecd4eabe748982b99098fc70d5f856cebe1860680a65c3021db0b1889d5098`.

---

## 3. Localization Analysis and Next Justified Step

### S1.0 Localization Breakdown
According to the S1.0 attribution study (`docs/integration/s1-stack-serving-measurements-2026-09-28.md`):
- **Output Head:** +0.0149 nats (RTN) / +0.0119 nats (GPTQ) — accounts for 41% of total degradation.
- **MLP Maps:** +0.0113 nats (RTN) / +0.0061 nats (GPTQ) — particularly `l0.mlp` (+0.0043) and `l5.mlp` (+0.0037).
- **Mixer Maps:** +0.0059 nats (RTN) / +0.0034 nats (GPTQ).
- **Embedding:** +0.0026 nats.
- **Mixer Scalars:** +0.0013 nats.

### Why Prior Post-Training Interventions Missed D4
1. **Offline Hadamard rotation** fails because the inverse rotation produces continuous values that must be re-quantized into D11 grid tables, causing compound rounding errors. An online Hadamard kernel in serving ($H x$) would avoid this, but requires engine kernel changes.
2. **Head heuristic search** was ineffective because the scale dynamic range in `head` was already within bounds, leaving weights unchanged from RTN.

### Causal Next Action
1. Deliver the canonical `Grouped4BitCodec` in `uor-r4-integer::codec` with exact quantization, dequantization, and pluggable `MapCodec` integration.
2. In QAT training (owned by Claude), plug `Grouped4BitCodec` into the served representation forward hook so gradients adjust the full stack weights against the discrete grid.
3. If an online Hadamard serving kernel is pursued, submit the derivation and interface proposal for Claude's review.

---

## 4. Serving Kernel Verification Scope

Static disassembly analysis via `scripts/audit_zero_matmul_serving.py` targets `libuor_r4_integer.rlib` and `uor-r4-stack` (the D11 integer stack engine):
- **Class I (Hardware Multipliers):** 0 violations.
- **Class II (Hardware Dividers):** 0 violations.
- **Class III (Floating-Point Instructions):** 0 violations.
- **Total checked symbol ranges:** 58 matched ranges (113 call-graph functions).
- **Result:** FULL PASS for `libuor_r4_integer.rlib`.
- **Note:** This audit does not certify `uor_r4_lut::stack` (the D10 comparator engine).
