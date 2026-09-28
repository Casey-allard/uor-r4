# Stack Mission Port Design (Track T3): Zero-MatMul R2 Kernel Mapping

**Author**: Lab 3 (Anti-Gravity)  
**Date**: 2026-09-27 (Revised)  
**Scope**: Track T3 Mission Runtime (`ROADMAP.md` §4.3, References #973, #820, #963, #964).  
**Invariant Contract**: D0-b / Rules R1–R5 (Strictly 0 Class I multipliers, 0 Class II dividers, 0 Class III floats in served numerical kernels). Read-only design pending T1(a) exposure gate report (~07:00 UTC 09-28).

---

## 1. Executive Summary & Architectural Scope

The geometric stack (`crates/uor-r4-training/src/geometric_stack.rs`, Cycle 4) demonstrates competitive scaling using an interleaved `rrarra` architecture: **4 recurrence (`r`) layers** and **2 read/attention (`a`) layers** (6 layers total). However, the existing serving comparator (`crates/uor-r4-lut/src/stack.rs` under D10) violates Mission Rule R1/R2 by executing runtime hardware multiplications on activations, gate products, Hamilton products, and attention value mixing.

Under Track T3, the stack serving engine will be ported **directly as an extension of `uor-r4-integer`**, strictly enforcing the zero-multiplier serving contract (D0-b):
1. **Recurrence State Structure**: The stack's recurrence has an additive drive and a decay gate ($s_{t+1} = \lambda s_t + b$). Consequently, its state is **not a group element**. The binary icosian group $2I$ finite automaton does **not** apply to the stack's `r` layers; it belongs exclusively to Lab 1's T1(b) separate tracking lanes.
2. **Runtime $\times$ Runtime Product Emulation**: For every runtime product (Hamilton products, decay/keep injection, SwiGLU gating, $q \cdot k$ read scores, value mixing, and Lorentz inner products), an explicit zero-multiplier method is designated: shift-add emulation, a quarter-square lookup table, or 4-bit quantization of one operand for a product table.
3. **Fidelity Gate**: Every quantized/emulated operator joins the mandatory fidelity gate: $\le 0.02$ nats degradation against float at equal inputs, with the decision-flip rate reported.
4. **Learned Convolution Taps**: The causal convolution taps are learned parameters (not fixed dyadic constants) and are quantized to $\le 4$ bits.
5. **No Fast Inverse Square Root**: "Fast inverse square root" approximations and polynomial evaluations require hardware multiplications; they are replaced with table lookups with shift normalization, CORDIC, or digit-by-digit `isqrt`.
6. **Hardware Cache Figures**: The Apple Silicon M1 Firestorm P-core L1 data cache is **128 KiB** (E-core L1D is 64 KiB). The $2I$ Cayley table ($120 \times 120$ bytes) is **~14.06 KiB** (~14–15 KB), not 1,920 B.
7. **R3 Dense Access Statement**: The geometric stack requires approximately **3.2–3.6 MB of dense parameter reads per token**, which is an interim serving profile and is **non-compliant with D5** (which requires sparse/sub-linear addressed geometric routing).
8. **Disassembly Census**: The source-level multiply count is superseded by an exact static disassembly census of `uor-r4-lut` using the Track T3 audit tool.

---

## 2. Op-by-Op Mapping: `geometric_stack.rs` $\to$ `uor-r4-integer` R2 Kernels

| Operation | `geometric_stack.rs` / `uor-r4-lut` Op | `uor-r4-integer` R2 Kernel Mapping | Method for Runtime $\times$ Runtime Products | Memory & Table Cost |
|---|---|---|---|---|
| **Linear Projections** | `gemv` 4-bit nibbles with hardware multiply | `IntegerModel::affine_direct_into` / `affine_wide_into` with 4-row blocked product table reuse (PR #1436) | Parameter $\times$ runtime: 16-entry precomputed product table | 32 B L1 register table; 0.5 B/weight parameter read |
| **RMSNorm & Rsqrt** | Float `rms_norm` / `isqrt` with division | Exact integer squared norm via `math::sum_squares`; reciprocal square root via **table lookup with shift normalization** or digit-by-digit `math::isqrt` | No multiplies; shift normalization + 256-entry Q16 rsqrt LUT | 512 B table; 288 B read/write |
| **Quaternion Recurrence Scan** | Hamilton product $a \otimes b$ (16 multiplies) | Quaternion state scan $s_{t+1} = \lambda s_t + b$; additive drive and decay gate (not a group element; $2I$ automaton belongs to T1(b)) | **Shift-add emulation** or **4-bit quantization of one operand** for a 16-entry product table | 0 B extra table; 9.2 KiB state across 4 layers |
| **Recurrence Decay & Keep** | $\lambda = \exp(-\text{rate} \cdot r)$, $\text{keep} = \sqrt{1 - \lambda^2}$ | Dual-channel 256-entry Q31 lookup table `decay_keep_lut[rate_idx]` returning $(\lambda, \text{keep})$ | Precomputed LUT read; product with state via **shift-add emulation** | 2,048 B table (256 $\times$ 8 B); 0 hardware multiplies |
| **Causal Convolution** | $c_t = \sum \text{taps} \cdot h$ via `grid_apply` | **Learned taps quantized to $\le 4$ bits**; applied into channel ring buffer | $\le 4$-bit parameter: 16-entry product table or shift-add | 0 B table; 13.8 KiB ring buffer across 4 layers |
| **SwiGLU MLP Gating** | Elementwise product $\text{silu}(g) \cdot u$ | Grouped 4-bit/8-bit product table lookup `low_bit_dot` or quarter-square table | **4-bit quantization of $\text{silu}(g)$** for 16-entry product table, or **quarter-square table** $\frac{1}{4}((a+b)^2 - (a-b)^2)$ | 1 KiB SiLU table + 256 B quarter-square table; 768 channels |
| **Read Scores (Dot)** | $\langle q, k \rangle / \sqrt{d}$ | Multi-head dot attention; scaling by $1/\sqrt{d}$ via dyadic shift | **Quarter-square table** or **4-bit quantization of key operand** for product table | Shared with projection table; $2 \times 48$ B per head |
| **Read Scores (Lorentz)** | $-\beta (d_L(q, k) - \text{offset})$ with `lift` | `LorentzRead::score` with `lorentz::excess_q32`, tabulated `arcosh1p_q24`, and `math::scale_pow2` | Coordinate products via **shift-add emulation**; distance tabulated | 4.1 KiB `arcosh1p` table (1,025 $\times$ 4 B); 0 multipliers |
| **Softmax & Normalization** | Softmax exp and wide division by sum | `lorentz::exp_q32` (Taylor series via `checked_mul`) or 256-entry Q32 negative exp LUT; ticket draw / `math::div_round` | Exponentiation via tabulated LUT; normalization via `math::div_round` | 1 KiB exp table; 256 entries |
| **Value Mixing** | $y = \sum w_j v_j$ with multiplies | Grouped LUT accumulation (T-MAC style) over quantized value matrices; zero runtime multiplications | **4-bit quantization of attention weights $w_j$** for table accumulation, or **shift-add emulation** | In-place shift-accumulation; $2 \times d$ bytes/pos |
| **Vocab Head & Sampling** | Dense unquantized head & float sampler | `IntegerModel::project_vocab_with_products_into` + `sampling::Sampler` (`exact_min_p_threshold`) | Parameter $\times$ runtime: reused 4-row blocked product table | Product table reused from affine; 0 Class I/II/III ops |

*Fidelity Gate Enforcement*: Every runtime operand quantization ($\text{silu}(g)$ to 4-bit, attention weights to 4-bit, Hamilton/decay operand quantization) must individually satisfy $\le 0.02$ nats degradation against float at equal inputs, with the decision-flip rate reported.

---

## 3. Disassembly Census: `uor-r4-lut` Stack Engine vs R2 Zero-Multiplier Target

Static disassembly of `libuor_r4_lut.rlib` using the Track T3 disassembler audit tool (`scripts/audit_zero_matmul_serving.py`) reveals the true assembly-level instruction distribution across the stack serving engine:

### 3.1 Static Disassembly Findings in `uor_r4_lut::stack`

```
Total uor_r4_lut::stack symbols: 45
Total Class I (Hardware Multipliers): 192 instructions
Total Class II (Hardware Dividers)  : 2 instructions
Total Class III (Floating-Point)    : 24 instructions
```

The violations are heavily concentrated in the inner serving loop `<uor_r4_lut::stack::StackSession>::advance`:
- **153 Class I Multipliers**:
  - `smull` (signed multiply long 32-bit $\to$ 64-bit): used in Hamilton product, SwiGLU gating, and norm calculations.
  - `madd` / `msub` (multiply-accumulate / multiply-subtract): used in quaternion coordinates and vector projections.
  - `mul` / `umulh` (64-bit and high-half multiplies): used in wide accumulation and scaling.
- **2 Class II Dividers**:
  - `udiv` instructions emitted in reciprocal normalization and softmax scaling.
- **14 Class III Floating-Point Instructions**:
  - `fmov` register transfers between GPR and SIMD/FP registers during intermediate state handling.

### 3.2 Elimination Strategy in `uor-r4-integer`

In the Track T3 port, all 192 multipliers, 2 dividers, and 24 FP/SIMD instructions are replaced by verified R2 primitives:
1. **Hamilton Products (`smull`, `madd`)**: Replaced by `math::checked_mul` (shift-and-add exact integer multiplication) or 4-bit quantized operand product table lookups.
2. **SwiGLU Gating (`smull`)**: Replaced by quarter-square lookup table $\frac{1}{4}((a+b)^2 - (a-b)^2)$ or 4-bit quantized product table.
3. **Hyperbolic / Coordinate Norms (`smull`, `madd`)**: Replaced by `math::sum_squares` (shift-and-add vector norm).
4. **Dividers (`udiv`)**: Replaced by integer reciprocal table lookups combined with dyadic power-of-two rescaling (`math::scale_pow2`) or `math::div_round`.
5. **Register Transfers (`fmov`)**: Completely eliminated; all intermediate states reside in GPR integer registers (`x0`–`x30`) and integer stack slots.

---

## 4. Hardware Footprint & Bytes-Per-Token Budget

### 4.1 Lookup Table Footprint (M1 L1 Cache Resident)
All static tables required for zero-multiplier stack serving fit comfortably inside the **128 KiB L1 data cache** of the Apple Silicon M1 Firestorm P-core:
- **Signed-4 Blocked Product Table**: 32 bytes (16 $\times$ i16) per thread.
- **Arcosh1p Q24 Table**: 4,100 bytes (1,025 $\times$ u32).
- **SiLU Q16 Table**: 1,024 bytes (512 $\times$ i16).
- **Exp_Q32 / Negative Exp Table**: 1,024 bytes (256 $\times$ u32).
- **Decay & Keep Factor Table**: 2,048 bytes (256 $\times$ u64).
- **Quarter-Square Multiplication Table**: 256 bytes (64 $\times$ u32).
- **Total Static Table Footprint**: **~8.5 KiB** ($\mathbf{15.0\times}$ margin under 128 KiB P-core L1D).

*(Clarification on the $2I$ Cayley Table)*: The binary icosian group $2I$ has 120 elements. Its full Cayley table is $120 \times 120 = 14,400$ bytes (**~14.06 KiB**), not 1,920 B. As established in Section 1, this table belongs to Lab 1's T1(b) tracking lanes and is not required for the stack's `r` layers.

### 4.2 State and Per-Token Memory Movement
For width $d = 288$, 6 layers (`rrarra`: **4 recurrence layers, 2 read layers**), context $C = 256$:

- **Persistent Recurrence State ($O(1)$ memory across 4 recurrence layers)**:
  - 4 recurrence layers $\times$ 72 lanes $\times$ 4 coordinates $\times$ 8 bytes (i64) = **9,216 bytes**.
  - Convolution tap history: 4 layers $\times$ 3 history steps $\times$ 288 channels $\times$ 4 bytes (i32) = **13,824 bytes**.
  - Total recurrent state: **23,040 bytes (22.5 KiB)** (persists across turns; zero heap reallocations).

- **Read KV Cache ($O(N)$ memory across 2 read layers)**:
  - The KV cache holds **both keys and values**: $2 \times 288 \times 2\text{ B}$ per read layer at INT16 = **1,152 bytes / token / read layer**.
  - Across 2 read layers: $2 \times 1,152\text{ B} = \mathbf{2,304\text{ bytes / token}}$.
  - At full context ($N = 256$): $256 \times 2,304\text{ B} = \mathbf{589,824\text{ bytes (576 KiB)}}$ total KV cache footprint.

- **Per-Token Operation Counts by Class**:
  - **Weight-table reads**: $\approx 6.5\text{M} - 7.2\text{M}$ 4-bit nibbles ($\approx 3.2 - 3.6$ MB of dense parameter reads).
  - **Emulated products**: $\approx 1,152$ Hamilton coordinate products, $\approx 576$ SwiGLU gating products, $\approx 576$ attention score products, $\approx 576$ value-mixing accumulations per token.
  - **Fixed-table reads**: $\approx 288$ decay-keep lookups, $\approx 576$ SiLU lookups, $\approx 512$ attention exp lookups, $\approx 288$ norm/rsqrt lookups per token.

- **Parameter Traffic & R3 Statement**:
  - Weight access per token (4-bit weights packed 2 per byte):
    - 4 Recurrence layers: $4 \times (2 \times d \times d + 2 \times d \times d + 3 \times d \times \text{mlp}) / 2 \approx 1.99$ MB.
    - 2 Read layers: $2 \times (3 \times d \times d + d \times d + 3 \times d \times \text{mlp}) / 2 \approx 1.16$ MB.
    - Vocab projection head: $(d \times 4096) / 2 \approx 0.59$ MB.
    - Total dense parameter reads: **~3.2–3.6 MB per token**.
  - **R3 Statement**: *The geometric stack's requirement of ~3.2–3.6 MB of dense parameter reads per token is an interim implementation and is non-compliant with D5. D5 mandates sub-linear parameter access through exact prime/zeta/R4 geometric addressing rather than dense parameter streaming.*
  - At M1 sustained memory bandwidth (~50 GB/s), 3.4 MB/token establishes a theoretical memory bandwidth floor of **~0.068 ms / token** (~14,700 tok/s ceiling).

---

## 5. Implementation Roadmap & Guardrails

1. **Architecture Containment**: All new serving code extends `crates/uor-r4-integer`. No new crates, binaries, or engines will be created; `uor-r4-lut` remains frozen as an unmaintained D10 comparator.
2. **Phase Gate Alignment**: This design remains strictly read-only until Lab 1 (Claude) reports T1(a) exposure results (~07:00 UTC on 09-28):
   - **If the stack passes the $\le 0.03$ nats gate vs transformer control**: Begin port with **one recurrence layer** behind the fidelity gate ($\le 0.02$ nats degradation vs float at equal inputs, with decision-flip rate reported).
   - **If the stack is not viable**: Park this design and continue focusing on native geometric model serving and energy optimization.
   - **Do not start the port earlier**.
3. **Disassembly Audit Enforcement**: Any newly compiled symbols in `uor-r4-integer` will be added to `scripts/audit_zero_matmul_serving.py` and validated for 0 Class I/II/III instructions prior to merging.
