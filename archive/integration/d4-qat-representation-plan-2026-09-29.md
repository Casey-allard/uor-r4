# D4 QAT Representation Plan: Gap Reduction via Quantization-Aware Training

- **Date:** 2026-09-29
- **Author:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Native Execution, and Measured Efficiency)
- **Coordinated With:** Claude main (Lab 1: Director & Architecture Owner)
- **Status:** PROPOSED RESEARCH PLAN & ADAPTER DESIGN (subject to Lab 1 review and owner approval)
- **Referenced Decisions:** D0-b, D4, D5, D9, D11, D12
- **Owning Issue:** #973 (under #820)

---

## 1. Executive Summary & Objective

The objective of Track D4 is to deliver a weight representation and codec that preserves the learned behavior of the selected geometric language model at:
$$\Delta\text{NLL} \le +0.0200\text{ nats degradation against float at equal inputs}$$
$$\text{Storage Rate} \le 4.25\text{ bits per weight (including all group scales, framing, and padding against the true unpadded weight count)}$$

Our empirical float-reference study on `geometric_s1` (PR #1479, report root `float-reference-1`) established the exact baseline:
- Float Reference: 1.998113 nats (100.00% top-1 agreement).
- S1.0 GPTQ Reference: +0.0257 nats (flip rate 8.17%).
- RTN Baseline: +0.036230 nats (90.65% agreement).
- Online Hadamard + Grouped 4-bit (H+G4): +0.027007 nats (91.28% agreement, 4.2725 bits/weight).
- Online Hadamard + E8 Lattice (H+E8): +2.423478 nats (31.18% agreement, 2.0143 bits/weight).

Under owner decision D12, neither post-training arm met the D4 gate; both remain unpromoted at this scope. Under D0-b/D11, the integer serving arithmetic is bit-identical to D10 NEON (`max_abs_logit_difference == 0` across 2,048 targets, $\le 10^{-6}$ nats). **The entire degradation is representation error.**

This plan specifies the proposed numerical mechanisms, exact storage accounting against unpadded denominators, and QAT integration hooks to close the remaining gap (+0.0270 nats $\rightarrow \le +0.0200$ nats) via Lab 1's existing Quantization-Aware Training interface.

---

## 2. Forensic Distortion Breakdown

In the S1.0c layerwise attribution (`docs/integration/s1-stack-serving-measurements-2026-09-28.md`), the representation loss from single-group perturbations on `geometric_s1` is localized as follows:

| Parameter Group (S1.0c) | GPTQ $\Delta\text{NLL}$ | Top-1 Agr. | RTN $\Delta\text{NLL}$ | Top-1 Agr. | Causal Observation (Empirical Hypotheses) |
|:---|---:|---:|---:|---:|:---|
| **Output Head (`head`)** | **+0.0119 nats** | 0.9467 | **+0.0149 nats** | 0.9395 | Largest single group (46.3% of GPTQ gap). Embedding folded with final RMSNorm gain ($W_{\text{head}} = W_{\text{embed}} \cdot g_{\text{norm}}$). Logit margin explanations ($\Delta z$) are hypotheses; 4-bit grouping inverts top-1 decisions. |
| **`l0.mlp`** | **+0.0027 nats** | 0.9772 | **+0.0043 nats** | 0.9718 | First MLP layer. Non-power-of-two width (749 padded to 768). |
| **`l5.mlp`** | **+0.0021 nats** | 0.9745 | **+0.0037 nats** | 0.9695 | Final MLP layer. |
| **`l5.mixer_maps`** | **+0.0018 nats** | 0.9817 | **+0.0032 nats** | 0.9744 | Final layer read/mixer projections. |
| **`l5.mixer_scalars`** | **+0.0008 nats** | 0.9949 | **+0.0008 nats** | 0.9949 | Final layer conv taps and decay rates via 8-bit grid codes. |
| **Layer 5 Total (Sum of 3 groups)** | **+0.0047 nats** | — | **+0.0077 nats** | — | Sum of `l5.mixer_maps` (+0.0018), `l5.mlp` (+0.0021), and `l5.mixer_scalars` (+0.0008) under GPTQ. |
| **Input Embedding (`embed`)** | **+0.0026 nats** | 0.9742 | **+0.0026 nats** | 0.9742 | Unfolded discrete lookup table. |
| **`l1`–`l4.mlp` Combined** | **+0.0014 nats** | — | **+0.0033 nats** | — | Intermediate MLP layers (`l1`: +0.0003, `l2`: +0.0004, `l3`: +0.0004, `l4`: +0.0003 under GPTQ). |
| **`l0`–`l4.mixer_maps` Combined** | **+0.0016 nats** | — | **+0.0027 nats** | — | Intermediate mixer projections (`l0`: +0.0004, `l1`: +0.0004, `l2`: +0.0001, `l3`: +0.0004, `l4`: +0.0003 under GPTQ). |
| **`l0`–`l4.mixer_scalars` Combined** | **+0.0005 nats** | — | **+0.0005 nats** | — | Intermediate layer conv/decay scalar grid codes. |
| **All MLPs (`l0`–`l5.mlp`)** | **+0.0061 nats** | — | **+0.0113 nats** | — | Aggregate MLP perturbation share across all 6 layers. |
| **Mixer Maps (`l0`–`l5.mixer_maps`)** | **+0.0034 nats** | — | **+0.0059 nats** | — | Aggregate mixer map perturbation share. |
| **Mixer Scalars (`all_scalars`)** | **+0.0013 nats** | 0.9919 | **+0.0013 nats** | 0.9919 | All scalar grid codes together. |
| **Simultaneous Perturbation (`all`)** | **+0.0257 nats** | 0.9184 | **+0.0362 nats** | 0.9065 | Full export model evaluated against float reference. |

> [!IMPORTANT]
> **Non-Additive Perturbation Caveat:**
> Single-group perturbation measurements (evaluating one quantized group while keeping all other parameters at float) are **non-additive hypotheses**, not an exact additive causal partition.
> While the linear sum of single-group perturbations (+0.0254 nats under GPTQ) happens to align closely with the simultaneous perturbation (+0.0257 nats) on `geometric_s1`, non-linear neural networks exhibit cross-layer interaction effects.
> Therefore, attributing the representation gap to individual groups serves as an empirical hypothesis to prioritize QAT investigation, not a decoupled mathematical partition.

**Finding:** The output head alone accounts for almost half of the total degradation. Any post-hoc quantizer (RTN, GPTQ, or FWHT) treats the weights as fixed and attempts minimum MSE reconstruction $\min_W \|W - \hat{W}\|_F^2$. However, minimum Frobenius weight error does NOT necessarily minimize cross-entropy loss $\mathcal{L}_{\text{CE}}$, because logit ranking is non-linear around the top-1 boundary.

---

## 3. Causal Interventions via Lab 1 QAT Hook

The owner decision of September 28 at 17:00 UTC ([DECISIONS.md](DECISIONS.md)) directed:
> "Lab 1 adds QAT, and D4 targets the same gap. The D11 port, bundle and audit continue."

This mandate authorizes developing QAT-compatible representation adapters and loss hooks; it is not approval of new model widths, training doses, or predicted gain claims. All training doses and gain numbers described below are **proposed hypotheses submitted for Lab 1 review and owner approval**.

Lab 1 implemented the QAT interface in `crates/uor-r4-training/src/geometric_stack.rs`:
```rust
pub trait MapCodec: Send + Sync {
    fn name(&self) -> &str;
    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>>;
}
```
In served training mode (`StackModel::set_served_representation`), the forward pass evaluates through `codec.round_trip()`, while the backward pass uses a Straight-Through Estimator (STE):
$$\frac{\partial \mathcal{L}}{\partial W} \approx \frac{\partial \mathcal{L}}{\partial W_{\text{served}}}$$

### Intervention A: Straight-Through Estimator (STE) Head Adaptation (Proposed Plan)
- **Hypothesis:** By evaluating the output head through `HeadCompensatedMapCodec` or `Grouped4BitCodec` during candidate training steps (e.g. proposed 256–512 steps), the float master weights adjust to separate pre-quantization logit margins.
- **Predicted Effect (Hypothesis):** Aims to reduce output head degradation from +0.0119 nats toward $\le +0.0040$ nats. This is an empirical target to be tested, not a guaranteed result.
- **Matched Control:** Float checkpoint fine-tuned for identical steps without `set_served_representation`.

### Intervention B: Structural Group-32 Dimension Alignment & Total Storage Accounting
- **Total Storage Rate Accounting:**
  Under D4, storage rate must count all container framing (headers, table descriptors, checksums), group scales, and zero-padding against the **true original unpadded weight denominator**:
  $$\text{Total Stored Rate (bpw)} = \frac{\text{Total Payload Bytes (weights + scales + framing + padding)} \times 8}{\text{Total Original Unpadded Weights}}$$
  The aligned code-plus-scale rate ($4.0 + 8.0/32 = 4.25$ bpw) is the raw parameter rate, not the total stored rate.
- **Problem in `geometric_s1`:** In `geometric_s1`, $d_{\text{mlp}} = 749$. When padded to whole groups of 32 ($768$), $19$ zero units per row are stored:
  $$\text{Storage} = 24 \text{ groups} \times (16\text{ B nibbles} + 1\text{ B scale}) = 408\text{ B} = 3,264\text{ bits}$$
  $$\text{Bits / Original Weight} = \frac{3,264}{749} = 4.3578\text{ bits/weight}$$
  Across the full model, this structural padding pushed total storage to **4.2725 bits/weight**, exceeding the $\le 4.25$ D4 gate.
- **Invariant:** The frozen `geometric_s1` architecture must NOT be modified retroactively to disguise a codec miss.
- **Proposed Future Options (Subject to Lab 1 Architectural Direction):**
  1. For future stack architectures, Lab 1 may consider sizing dimensions such that $d \equiv 0 \pmod{32}$ (e.g. $d_{\text{mlp}} = 768$ or $736$).
  2. Alternatively, partial-group packing may be explored where trailing weights use smaller group structures if supported by the D11 integer serving kernel.

### Intervention C: Conway-Sloane E8 Lattice Codebook STE (Proposed Research)
- **Problem:** H+E8 exhibited degradation (+2.4235 nats) because post-training single-shell VQ mapped 8-dimensional vectors to 240 minimal roots on a single sphere ($r = \sqrt{2}$), collapsing radial dynamics.
- **Proposed Investigation:** Under QAT, two-stage residual matched-bit E8 lattice quantization ($b \to \text{E8}_1 + \text{E8}_2(\text{residual})$ at ~4.0 bpw, now implemented in `crates/uor-r4-integer/src/codec.rs`) can be tested through the `MapCodec` interface.
- **Status:** Unpromoted negative result under D12 at single-shell scope; matched-bit E8 remains a research arm for subsequent evaluation.

---

## 4. Bounded Experiment Protocol & Machine Safeguards

1. **Machine Slot Discipline:**
   - `/Volumes/UOR-Workspace/locks/model-slot.json` is currently held by OpenCode (Lab 2) until ~04:02 UTC.
   - Strictly **zero training runs or heavy model fits** will be launched by Lab 3 while the lock is active.
2. **Implementation Scope for Lab 3:**
   - Propose production `MapCodec` adapters in a separate Class C proposal (`lab/anti-gravity/d4-qat-adapters`):
     - `D4Grouped4BitAdapter` (wrapping `Grouped4BitCodec` in RTN and MinMSE modes)
     - `HeadCompensatedMapCodec` (compensated base and boundary scale search for output head)
   - Maintain comprehensive unit and regression coverage in `crates/uor-r4-training/tests/d4_map_codec_adapter.rs`.
   - Ensure complete export compatibility: weights trained or saved with these adapters round-trip into `StackArtifact` without requiring runtime multiplier or divider instructions.
   - Deliver #1479 carrying the D4 negative result, D11 cost records, dequantize_block Result, matched-bit E8 primitive, and ILP kernel (landed). Deliver QAT adapters through PR #1490 for Lab 1 to decide on shared interfaces.
3. **Downstream Consumer:**
   - Lab 1 (Claude main) consumes `D4Grouped4BitAdapter` and `HeadCompensatedMapCodec` via `StackModel::set_served_representation` during the S1 QAT fine-tuning run.
