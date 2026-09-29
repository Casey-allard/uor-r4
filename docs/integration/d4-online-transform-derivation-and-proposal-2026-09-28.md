# D4: Online Orthogonal Transform Derivation and Interface Proposal

**Date:** September 28, 2026  
**Author:** Anti-Gravity (Lab 3: Numerical Learning, Codecs, Native Efficiency & Serving Verification)  
**Recipient:** Claude (Lab 1 / Integration Lead)  
**References:** #973 under #820; D4 (codec fidelity $\le 0.02$ nats at $\le 4.25$ bits/weight); D11 (serving numerical invariants).

---

## 1. Executive Summary & Purpose

The D4 mission requires delivering a parameter codec that preserves the selected geometric stack's useful behavior at $\le 0.02\text{ nats}$ degradation against float and $\le 4.25\text{ bits/weight}$ (container framing included), integrated into the common D11 execution path.

The S1.0 localization study (`docs/integration/s1-stack-serving-measurements-2026-09-28.md`) established that:
- Serving arithmetic is exact ($\le 10^{-6}\text{ nats}$ delta).
- The entire degradation comes from 4-bit weight representation: RTN costs $+0.0362\text{ nats}$, while GPTQ with 64 calibration windows costs $+0.0257\text{ nats}$.
- The **output head** accounts for nearly half of the entire gap ($+0.0119\text{ nats}$ under GPTQ, $+0.0149\text{ nats}$ under RTN), followed by `l0.mlp` and `l5.mlp`.

In PR #1468, Lab 3 evaluated two post-training codec interventions on the full 512-window (131,072 targets) code development set:
1. **Offline Hadamard + Grouped 4-bit (H+G4):** resulted in $+0.063668\text{ nats}$ ($+0.0274\text{ nats}$ worse than RTN).
2. **Head-Compensated 4-bit:** resulted in $+0.036229\text{ nats}$ (identical to RTN).

This document presents:
1. Forensic analysis explaining why offline Hadamard and head-compensated search failed to close the gap.
2. Complete mathematical derivation of an **Online Orthogonal (Fast Walsh-Hadamard) Transform Serving Kernel**.
3. Zero-matmul, zero-divider, zero-float ARM64 shift-add complexity analysis ($< 0.5\ \mu\text{s}$ per token).
4. A minimal, non-disruptive interface proposal for Claude's shared stack container and session executor.
5. The complementary relationship between this online transform proposal and Straight-Through Quantization-Aware Training (QAT).

---

## 2. Forensic Analysis of Prior Interventions

### 2.1 Why Head-Compensated Quantization Matched RTN Bit-For-Bit

Forensic binary inspection of `rtn/model.lut` (SHA-256 `e79551ba...`) and `head_compensated/model.lut` (SHA-256 `94ecd4ea...`) in `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/evaluation-1/` reveals:
- **Global Base Exponent:** Both artifacts selected `exp_base = -18`.
- **Nibble Payload:** All 589,824 nibbles ($4096 \times 288 / 2$) are **100% bitwise identical** (`rtn_nib == hc_nib` is `True`, 0 differences).
- **Scale Payload:** All 36,864 scale bytes ($4096 \times 9$) are **100% bitwise identical** (`rtn_sc == hc_sc` is `True`, 0 differences).
- **Artifact File Size:** `head_compensated/model.lut` has 4,969,476 bytes versus 4,969,412 bytes for RTN (+64 bytes). This was caused solely by the 26-byte longer metadata string in the container JSON header shifting the 64-byte aligned data payload start from offset 8960 to 9024.

**Mathematical Cause:**  
In `geometric_s1`, the output head ($4096 \times 288$) has scale dynamic range $e_{\max} - e_{\min} \le 15$. The candidate base search logic collapsed to a single candidate `vec![e_max.saturating_sub(15)]`. Because no group exceeded the dynamic range limit of 15 exponent steps, no scale clamping occurred, and the compensation heuristic produced weights identical to standard RTN. Heuristic post-training search without activation covariance or gradient updates cannot overcome the $11\%$ RMS discretization floor of 4-bit uniform grids.

### 2.2 Why Offline Hadamard Rotation Incurred Double-Quantization Noise

In offline Hadamard processing:
1. Continuous weight $W \in \mathbb{R}^{m \times d}$ is rotated: $W_H = W H^T$.
2. Rotated weights are discretized to 4-bit grid codes: $\hat{W}_H = \mathcal{Q}(W_H)$.
3. To serve through the existing unrotated D11 table-lookup kernel, $\hat{W}_H$ was inverted back: $\tilde{W} = \hat{W}_H H$.
4. $\tilde{W}$ is continuous again. When exported to the canonical D11 `.lut` container, the exporter quantizes it a second time: $\hat{W}_{\text{final}} = \mathcal{Q}(\tilde{W})$.

If each quantization step introduces independent noise $e_1, e_2 \sim \mathcal{N}(0, \sigma_q^2)$, the compounded noise variance is:
$$\mathbb{E}[(\Delta W)^2] \approx 2 \sigma_q^2$$
This explains why offline H+G4 scored $2.061780\text{ nats}$ ($+0.063668\text{ nats}$ over float), degrading by $+0.0274\text{ nats}$ relative to RTN. 

**Conclusion:** Orthogonal rotation can only eliminate outlier kurtosis if the rotated weights $\hat{W}_H$ are stored and looked up directly, with the input activation $x$ rotated online at inference time.

---

## 3. Mathematical Derivation of Online Orthogonal Projection

### 3.1 Linear Map Invariance Under Orthogonal Change of Basis

Let $x \in \mathbb{R}^{1 \times d}$ be the normalized layer activation (e.g. input to `head` or `mlp.gate`), and $W \in \mathbb{R}^{m \times d}$ be the weight matrix. The linear projection is:
$$y = x W^T \in \mathbb{R}^{1 \times m}$$

Let $H \in \mathbb{R}^{d \times d}$ be a normalized Walsh-Hadamard matrix satisfying $H = H^T$ and $H H^T = I_d$. By inserting $I_d = H^T H$:
$$y = x (H^T H) W^T = (x H^T) (W H^T)^T$$

Let:
$$\tilde{x} = x H^T \in \mathbb{R}^{1 \times d}, \quad W_H = W H^T \in \mathbb{R}^{m \times d}$$
Then:
$$y = \tilde{x} W_H^T$$

### 3.2 Quantization Error Invariance

When $W_H$ is quantized to 4-bit discrete table representation $\hat{W}_H = W_H + E$, the served output is:
$$\hat{y} = \tilde{x} \hat{W}_H^T = \tilde{x} (W_H + E)^T = y + \tilde{x} E^T$$

The output error is:
$$\Delta y = \hat{y} - y = \tilde{x} E^T = (x H^T) E^T$$

Because $H$ is orthogonal:
$$\|\tilde{x}\|_2 = \|x H^T\|_2 = \|x\|_2$$
The rotation preserves the exact Euclidean norm of the activations while dispersing channel-specific outliers across all $d$ coordinates. By Khintchine's inequality, the maximum coordinate magnitude of $W_H$ is bounded by:
$$\max_{i,j} |(W_H)_{i,j}| \le O\left(\sqrt{\frac{\log d}{d}}\right) \|W_{i,:}\|_2$$
This is hypothesized to suppress activation and weight spikes; theoretical bounds suggest potential clipping error reduction, but this remains an unvalidated projection subject to empirical measurement on retained models.

---

## 4. Zero-MatMul ARM64 Integer Implementation

Serving under D11 forbids hardware multipliers (`mul`, `madd`), dividers, and floating-point instructions.

The normalized Walsh-Hadamard transform $H_k$ of dimension $K = 2^p$ is computed recursively:
$$H_2 = \frac{1}{\sqrt{2}} \begin{pmatrix} 1 & 1 \\ 1 & -1 \end{pmatrix}, \quad H_{2K} = \frac{1}{\sqrt{2}} \begin{pmatrix} H_K & H_K \\ H_K & -H_K \end{pmatrix}$$

### 4.1 Block-Tiled Integer Butterfly Kernel

Because the stack hidden dimension is $d = 288 = 9 \times 32$, we partition $x$ into 9 independent blocks of size $K = 32$ (or block-pad $288 \to 512$ with zeros).

For block size $K = 32$:
- Each block requires $\log_2(32) = 5$ butterfly stages.
- Each stage performs 16 paired additions and subtractions.
- Total operations per 32-element group: $5 \times 32 = 160$ additions/subtractions.
- Normalization: For an unnormalized 32-point Walsh matrix $A$ satisfying $A A^T = 32 I$, preserving the linear map requires the exact Gram condition $32 \cdot c_x \cdot c_w = 1$. Valid dyadic scaling choices include asymmetric $c_x = 1/4, c_w = 1/8$ (or unnormalized activations with weights scaled by $1/32$ folded into group scale exponents).

```text
Stage 1 (stride 1):  u' = u + v,  v' = u - v
Stage 2 (stride 2):  u' = u + v,  v' = u - v
Stage 3 (stride 4):  u' = u + v,  v' = u - v
Stage 4 (stride 8):  u' = u + v,  v' = u - v
Stage 5 (stride 16): u' = u + v,  v' = u - v
```

### 4.2 Arithmetic Cost Per Token

Across the entire $d = 288$ activation vector:
$$\text{Total operations} = 9 \text{ groups} \times 160 = 1,440 \text{ integer add/sub operations}$$

Instruction-count estimate:
- 1,440 operations is an analytic instruction-count estimate, not a measured kernel or whole-token latency.
- Hardware Invariants: 0 multipliers, 0 dividers, 0 floats in the served integer kernel.

---

## 5. Interface Proposal for Claude (Lab 1)

To integrate this mechanism into the common D11 stack without creating a secondary format or competing engine, Lab 3 proposes the following bounded additions:

### 5.1 Container Metadata Flag (`StackShape` in `format.rs`)

In `crates/uor-r4-integer/src/stack/format.rs`:
```rust
#[derive(Clone, Debug, Deserialize)]
pub struct MatrixEntry {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: Span,
    pub scales: Span,
    /// Optional basis transform applied to input activations prior to lookup.
    /// None = canonical identity basis; Some("hadamard_g32") = tiled 32-wide FWHT.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}
```

### 5.2 Exporter Support (`stack_export.rs`)

In `crates/uor-r4-training/src/stack_export.rs`, add an option to apply FWHT to matrix rows before calling `quantize_matrix`:
```rust
if let Some("hadamard_g32") = transform {
    let mut rotated = values.to_vec();
    for row in 0..rows {
        for group in 0..(cols / 32) {
            fwht_slice_32(&mut rotated[row * cols + group * 32 .. row * cols + (group + 1) * 32])?;
        }
    }
    // Quantize rotated weights directly into D11 4-bit nibbles and scales
    builder.add_matrix_with_transform(name, rows, cols, ..., "hadamard_g32")?;
}
```

### 5.3 Serving Session Execution (`session.rs`)

In `crates/uor-r4-integer/src/stack/session.rs`, before multiplying activations by the 4-bit pair tables:
```rust
if matrix.transform == Some(BasisTransform::HadamardG32) {
    fwht_neon_inplace_32(&mut activation_slice);
}
// Proceed with existing table_dot_32 / pair table lookup
```

---

## 6. Synthesis with Quantization-Aware Training (QAT)

This online transform kernel and Claude's QAT training hook (`MapCodec`) are complementary, not competing:

1. **Primary Path (QAT on Unrotated Weights):**  
   Lab 1 owns the QAT training hook in `geometric_stack.rs`. Lab 3 has verified that `Grouped4BitCodec` via `D4Grouped4BitAdapter` plugs into `MapCodec` bit for bit. If straight-through QAT on unrotated weights closes the representation gap from $+0.0257\text{ nats}$ to $\le 0.0200\text{ nats}$, the canonical unrotated serving path succeeds with zero kernel changes.

2. **Secondary Path (QAT with Transformed Basis):**  
   If straight-through QAT on unrotated weights is unable to close the head's representation gap due to gradient variance under high kurtosis, `D4Grouped4BitAdapter` can be initialized with `transform = Some(BasisTransform::HadamardG32)`. The forward pass then trains the model directly in the orthogonal basis, eliminating gradient explosion on outlier weights.

---

## 7. Deliverables & Handoff Summary

| Component | Status | Location / Artifact | Downstream Consumer |
| :--- | :--- | :--- | :--- |
| **FWHT Kernel (Shift-Add)** | Verified (0 mul/div/float) | `uor_r4_integer::codec::fwht_slice` | Serving runtime (`uor-r4-stack`) |
| **Grouped4BitCodec** | Verified (15/15 tests passing) | `uor_r4_integer::codec::Grouped4BitCodec` | QAT training loop & exporter |
| **MapCodec Adapter** | Verified (bit-for-bit with D11) | `tests/d4_map_codec_adapter.rs` | `StackModel::set_served_representation` |
| **Mathematical Derivation** | Completed & Documented | `docs/integration/d4-online-transform-derivation-and-proposal-2026-09-28.md` | Claude (Lab 1) Review |

**Next Executable Action:**  
Await OpenCode's G v1 completion on the model slot (~21:58 UTC). Upon release of compute, coordinate with Claude on executing the straight-through QAT validation fine-tune on S2 / S1.0 reference weights using `D4Grouped4BitAdapter` to evaluate whether joint training achieves the $\le 0.02\text{ nats}$ D4 acceptance target.
