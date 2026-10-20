# Frontier Native Geometric Model: Cross-Lab Radial Transfer & Zero-MatMul Serving Synthesis

Updated September 27, 2026. **Pre-alpha; experimental autoregressive geometric state model. No useful general-language, coding, frontier, geometric-advantage or full-path energy qualification is established.**

## 1. Executive Summary & Cross-Track Convergence

The four concurrent research tracks converge on an integrated empirical verification:

1. **Track 1 (Serving Engine & Memory)**: Delivered the zero-matmul streaming CLI `uor-chat`, signed 4-bit lookup tables (`low_bit_dot`), and hierarchical prime memory indexing ($K > 256$, tested up to $K = 3000+$) with 32 persistent slots, 224 L1 dialogue slots, and 64 L2 paging slots.
2. **Track 2 (Lab 2 Lorentz)**: Delivered the Minkowski hyperboloid geometry in 64 spatial coordinates (read width 64), discrete CORDIC/arcosh lookups, and packed serving models (`qat-lorentzflat_s1`, `qat-dot_s1`).
3. **Track 3 (Lab 3 Radial Read Control & Transfer)**: Delivered explicit learned parameter inheritance from the continuous Dot parent (`step-15,672`), affine-in-$z$ control changing the distance-to-score law (distinguishing nonlinear score spacing within that representation, while noting that the matched Dot reset control is required for practical transferred comparison), and fixed 1,024-update adaptation comparison tooling.
4. **Track 4 (Lab 4 Diagnostics & Protocol)**: Delivered selection-policy diagnostics proving greedy departures vs ranking/emission bottleneck, literal-role integer token serialization, and cross-lab receipt verification. PR #1418 merged, supplying the exact-token native conversation adapter.

## 2. Empirical Radial Transfer Startup Witness

The explicit parameter transfer (`joint_transfer.rs`) inherits all 21 shared tensors (1,678,466 scalars) from the retained continuous Dot parent:
- **Parent Checkpoint**: `.uor-models/investigations/language-continuation-20260925/fit-quaternion-6/checkpoint-final`
- **Parent Exposure**: 64,192,512 historical target visits, optimizer step 15,672, continuous F32 Dot, quaternion state width 256, read width 64, context 256, vocabulary 4,096.
- **Lineage Integrity**: All 21 shared arrays copied exactly; optimizer moments and per-parameter clocks cleanly reset to zero (zero moment pollution); fresh sampling seed 240927.

On the four fixed full-256 canonical development windows (blocks 0, 21, 42, 63; 1,024 targets, 1,020 non-trivial history positions), the empirical zero-update transfer witness measures:

| Reader Variant | Initial Next-Token NLL (nats) | Mean Causal Read Mass | Mean Conditional Read Entropy (nats) | Zero-Read Positions (/1,020) | Required Gradients Finite & Nonzero |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Dot Parent (Reset Control)** | **1.996717** | N/A (Standard Dot) | N/A | N/A | Preserved |
| **Lorentz Radial Reader** | **2.389537** | 0.120842 | 4.203201 | 0/1,020 (0.0%) | 6/6 (100.0%) |
| **LorentzAffine Radial Reader** | **2.400457** | 0.117484 | 4.206362 | 0/1,020 (0.0%) | 6/6 (100.0%) |

### Observations:
- **Initial Reader Disturbance**: Both radial arms show an immediate loss disturbance of +0.3928 nats (Lorentz) and +0.4037 nats (LorentzAffine) relative to the Dot reset control (1.996717 nats). This confirms that changing the reader score law immediately perturbs learned language representations before adaptation.
- **Signal Connectivity**: 6/6 required gradient families (`read.query.weight`, `read.key.weight`, `read.value.weight`, `read.no_read.weight`, `read.lorentz_log_beta`, `read.lorentz_offset`) possess finite, non-zero coordinates with non-vanishing L2 norms ($0.0209$ to $0.2029$).
- **Bit-Identical Post-Token0 States**: At position 1, prior to divergent causal trajectories, post-token0 states are bit-identical between Lorentz and Affine. The single-key read log-odds difference observes the score law difference directly (+0.06778 nats).
- **Eight-Token Continuations**:
  - Dot Reset: `toys and read them all day.`
  - Lorentz: `toys. One day, he found a`
  - LorentzAffine: `toys. One day, he found a`

## 3. Hardware & Architectural Invariants & Evidence Boundaries

Serving execution adheres strictly to owner-adopted D0-b and Milestone M1–M5 criteria on consumer Apple Silicon (M1), with clear evidence boundaries:

1. **Strictly Zero Transformers & Zero Hardware MatMul in Analyzed Serving Symbols**:
   - Live integer serving kernel contains **0 soft attention matrices** and **0 dense MLPs**.
   - Verified via `scripts/audit_zero_matmul_serving.py` across static disassembly of all 24 mandatory numerical serving symbols in `libuor_r4_integer.rlib`:
     - **Class I (Multipliers)**: 0 forbidden instructions (`mul`, `madd`, `smull`, etc.)
     - **Class II (Dividers)**: 0 forbidden instructions (`sdiv`, `udiv`)
     - **Class III (Floats)**: 0 forbidden instructions (`fmul`, `fmov`, `fadd`, etc.)
   - *Evidence Scope*: The static audit applies to the 24 analyzed numerical serving symbols in `libuor_r4_integer.rlib`; it does not certify the complete application call graph by association. Continuous training/adaptation candidates (`step-15,672` parent, Lorentz, LorentzAffine) are continuous F32 models and have not been converted to signed 4-bit integer weights or served in these benchmarks. `LorentzAffine` currently has no admitted integer serving implementation.
2. **Process Memory Footprint**:
   - Measured peak RSS during live integer streaming and benchmarks: **6.70 MB – 15.86 MB**, strictly below the 35.0 MB ceiling and well within the 25.0 MB target.
   - Net RSS growth across sample sequences is flat; note that flat RSS over a sample run reflects low process memory footprint rather than zero dynamic allocator churn across all code paths.
3. **Inference Latency**:
   - Average single-token latency across 128–520 tokens on M1 CPU: **2.33 ms – 3.12 ms/token**, meeting the $\le 4.0\text{ ms/tok}$ target.
   - Pure CPU execution: **0 GPU / Metal / CUDA linkages** (`test_m5_apple_silicon_zero_gpu_cpu_only_invariants`).
4. **Causal Memory Contrast**:
   - Long-horizon recall benchmarks (20 turns, $K = 2500+$ tokens) achieve **10/10 (100.0%) recall** with up to **508.3x perplexity inflation** and **6.231 nats delta NLL** under NoRead ablation (`conversational_benchmarks.rs`). This measures exact key-value needle retrieval and slot preservation under synthetic multi-turn scenarios; per fourth-lab integration findings, it does not establish open conversational fluency or broad language capability.
   - Cross-lab Lorentz model evaluation achieves **5.8450 nats delta NLL** advantage on repeated sequence recall (`cross_lab_lorentz_frontier.rs`).

## 4. Lineage and Artifact Provenance

- **Continuous Parent**: `investigations/language-continuation-20260925/fit-quaternion-6/checkpoint-final` (SHA256: `490922decefee3e6036331dd0bc112746f24662d4bddfbe540b59c1d00820571`)
- **Radial Transfer Receipt**: `docs/evidence/radial-parameter-transfer-validation-2026-09-27.json`
- **Dot Reset Receipt**: `docs/evidence/dot-reset-validation-2026-09-27.json`
- **Study Comparator & Evaluation**: `docs/evidence/reader-study-tools-validation-2026-09-27.json`
- **Disassembly Audit Tool**: `scripts/audit_zero_matmul_serving.py`
- **Frontier Verification Test**: `crates/uor-r4-integer/tests/cross_lab_lorentz_frontier.rs`
