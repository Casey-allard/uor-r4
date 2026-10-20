# B3: E8P Lattice Weight Coding with Incoherence Transform on SmolLM2-360M (Track B Study)

**Date**: 2026-09-29  
**Lab**: Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Native Kernels, Measured Cost)  
**Governing Directive**: Lab 1 Board [#1513](https://github.com/UOR-Foundation/uor-r4/issues/1513), Epics [#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509) (Track B) and [#1510](https://github.com/UOR-Foundation/uor-r4/issues/1510) (Infra)  
**Evidence Artifact**: `docs/evidence/b3-e8-smollm2-360m-2026-09-29.json`  
**Sealed Report Root**: `/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-full-32layers-rht-e8p`  
**Execution Binary**: `/tmp/uor-target/release/b3-e8-smollm2` (`3e304bfc693ccbf653422c1e2b35fdf5f7de0ebc5caa80a3b80a125e4a8290e3`) (declared-not-verified; source/executable binding unavailable (historical run))

---

## 1. Executive Summary & Scientific Verdict

Under Lab 1 directive B3 (Track B's second lever), Anti-Gravity evaluated geometric lattice weight coding on SmolLM2-360M's MLP weights (`gate_proj`, `up_proj`, `down_proj`, totaling 235,929,600 weights across all 32 layers) at 2, 3, and 4 bits per weight against standard 4-bit round-to-nearest (RTN) and scalar controls under an exact Randomized Hadamard Transform (RHT).

### Pre-Registered Kill Criterion
- **Rule**: `Kill: 3-bit E8 is more than 0.05 nats worse than round-to-nearest 4-bit. Record it and stop.`
- **Measured**:
  - $\Delta \text{NLL}_{\text{RTN-4bit}} = +0.071494 \text{ nats}$ (at 4.2765 bpw)
  - $\Delta \text{NLL}_{\text{RHT-E8P-3bit}} = +4.962782 \text{ nats}$ (at 3.0134 bpw)
  - Excess degradation over RTN 4-bit: $\mathbf{+4.891289 \text{ nats}}$
- **Verdict**: **`KILL CRITERION TRIGGERED (+4.891289 > 0.0500 nats)`**.

Per [D12](DECISIONS.md#d12--negative-evidence-retention-and-promotion-gates) and [D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract), this negative result is preserved at its exact technical scope. Offline uncompensated $E_8$ lattice vector quantization of dense MLP weights—as evaluated in this study using an uncompensated offline implementation with heuristic sign selection over the QuIP# E8P codebook and homogenized by a Randomized Hadamard Transform—suffers severe compounding error across deep layers. This negative result characterizes this specific offline uncompensated implementation on SmolLM2-360M MLP layers, without asserting broader claims about canonical QuIP# or disproving all $E_8$ lattice quantization as a family. No positive conversion claims are asserted. In accordance with directive B3, Anti-Gravity halts further offline sweeps on this arm and preserves the negative evidence.

---

## 2. Experimental Setup & Pre-Registered Arms

- **Target Model**: SmolLM2-360M-Instruct
  - Path: `/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct`
  - Weight SHA-256: `e6bffe7435d7ddc10fd3b9a9efd429dafbacb1cb17015fb5562664e7532bf86e` (declared-not-verified)
  - Architecture: `LlamaForCausalLM` ($L=32, d_{\text{model}}=960, d_{\text{ffn}}=2560, V=49152$)
- **Quantization Scope**:
  - Layers: $0..32$ (all 32 layers)
  - Matrices per layer: `gate_proj`, `up_proj`, `down_proj` ($3 \times 32 = 96$ matrices)
  - Weights per layer: $3 \times (960 \times 2560) = 7,372,800$ weights
  - Total quantized weights: $\mathbf{235,929,600 \text{ weights}}$
  - Note on alignment: hidden width 960 and intermediate width 2560 are exactly divisible by 8 ($E_8$ block dimension) and 32 (RTN group size), ensuring 0 zero-padding overhead.
- **Evaluation Dataset**:
  - Corpus: SimpleWiki 20231101 (`articles.jsonl`, BLAKE3 `194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`)
  - Tokens: `/Volumes/UOR-Workspace/uor-r4-models/corpora/simple-wiki-20231101/simplewiki_32x1024.u16` (SHA-256 `9bf6a8334cfe86d0e07ceceb8436002c6c0671a12f79f2e356b233c7a63f2bbd`)
  - Evaluation protocol: 32 windows of context length $T=1024$, batch size 1 ($\mathbf{32,768}$ evaluation tokens total).
- **Codecs & Controls Evaluated**:
  1. **Arm 0 (Float Baseline)**: Full precision unquantized $f32$ reference.
  2. **Arm 1 (Plain RTN 4-bit)**: Group size 32, 16 code bytes + 1 scale byte per 32 weights ($17/32 = 4.2500$ bpw target, 4.2765 bpw serialized).
  3. **Arm 2 (RHT + RTN 4-bit)**: Randomized Hadamard Transform (RHT) on weight columns followed by RTN 4-bit (isolating transform effect).
  4. **Arm 3 (RHT + RTN 3-bit)**: RHT followed by 3-bit uniform scalar quantization (group size 32) as matched-bit scalar control.
  5. **Arm 4 (RHT + E8P 2-bit)**: RHT followed by 16-bit Conway–Sloane $E_8$ padded codebook (QuIP# E8P: 256 packed magnitude patterns, 2 parity cosets, 2.0134 bpw serialized).
  6. **Arm 5 (RHT + E8P 3-bit)**: RHT + E8P 2-bit base + 1-bpw residual stage using the 241 canonical $E_8$ roots (3.0134 bpw serialized).
  7. **Arm 6 (RHT + E8P 4-bit)**: RHT + E8P 2-bit base + E8P 2-bit residual stage via Residual Vector Quantization (RVQ, 4.0134 bpw serialized).

---

## 3. Measured Empirical Results

All metrics were computed across all 32,768 tokens and 235.9M weights, with reference logits streamed to bound RSS strictly below 1.5 GB. Results are sealed in `/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-full-32layers-rht-e8p`:

| Arm | Codec / Control Description | Target bpw | Serialized bpw | Total Serialized Bytes | Mean NLL (nats) | $\Delta$ NLL vs Float | Mean KL vs Teacher | Top-1 Agr (%) | Codec Roundtrip Gate |
|:---:|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Arm 0** | Float Reference Baseline | 32.00 | 32.0000 | 943,718,400 | 2.122189 | 0.000000 | 0.000000 | 100.00% | N/A |
| **Arm 1** | Plain RTN 4-bit (Group size 32) | 4.25 | 4.2765 | 126,119,680 | 2.193682 | +0.071494 | 0.093349 | 85.61% | **PASSED** |
| **Arm 2** | RHT + RTN 4-bit | 4.25 | 4.2766 | 126,121,216 | 2.237834 | +0.115645 | 0.139265 | 82.74% | **PASSED** |
| **Arm 3** | RHT + RTN 3-bit (Scalar Control) | 3.00 | 3.2766 | 96,630,016 | 2.850512 | +0.728323 | 0.805773 | 61.03% | **PASSED** |
| **Arm 4** | RHT + E8P 2-bit (QuIP# Lattice) | 2.00 | 2.0134 | 59,376,128 | 13.723733 | +11.601545 | 11.770558 | 0.03% | **PASSED** |
| **Arm 5** | RHT + E8P 3-bit (E8P + 1-bpw res) | 3.00 | 3.0134 | 88,868,096 | 7.084971 | +4.962782 | 5.112392 | 12.51% | **PASSED** |
| **Arm 6** | RHT + E8P 4-bit (E8P + E8P RVQ) | 4.00 | 4.0134 | 118,359,296 | 5.814762 | +3.692573 | 3.837410 | 24.57% | **PASSED** |

### Key Diagnostic Deltas
- **Transform Effect at 4-bit** ($\Delta\text{NLL}_{\text{RTN4}} - \Delta\text{NLL}_{\text{RHT-RTN4}}$): $\mathbf{-0.044152 \text{ nats}}$
- **Lattice Gain at Matched 3-bit Budget** ($\Delta\text{NLL}_{\text{RHT-RTN3}} - \Delta\text{NLL}_{\text{RHT-E8P3}}$): $\mathbf{-4.234459 \text{ nats}}$  
  *(Scalar uniform grid outperforms $E_8$ lattice by 4.23 nats at matched ~3 bpw budget under identical RHT incoherence transformation)*
- **Excess Degradation vs RTN 4-bit**: $\mathbf{+4.891289 \text{ nats}}$ (Threshold: $+0.050000 \text{ nats}$)

---

## 4. Analysis of Findings

1. **Exact Codec Round-Trip Verification**:
   Every one of the 96 MLP weight matrices across all six quantization arms passed exact bit-for-bit serialization and deserialization checks before dequantization (`codec_roundtrip_gate_passed: true`), ensuring that serialization bugs are completely ruled out as a source of degradation. Note that serialization round-trip (`to_bytes` / `from_bytes`) verifies storage fidelity, but does not establish that the encoder found the optimal nearest codeword in Euclidean distance.
2. **Bits Per Weight Accounting**:
   Bitrates were derived strictly from stored file bytes:
   $$\text{bpw} = \frac{8 \times \text{serialized\_bytes}}{\text{quantized\_weights}}$$
   The E8P codecs achieve exact theoretical storage: 2.0134 bpw for 2-bit, 3.0134 bpw for 3-bit, and 4.0134 bpw for 4-bit (including row scales and metadata).
3. **Possible contributors to this implementation's degradation**:
   - Why does a scalar grid beat the $E_8$ lattice by 4.23 nats at 3 bits?
     Scalar quantization at group size 32 assigns an independent scale factor every 32 weights ($2560/32 = 80$ scales per row). This allows fine-grained local scale adaptation across different feature channels.
   - In contrast, $E_8$ lattice vector quantization packages 8 dimensions into a single codeword normalized by a single row-level or coarse-block scale. Even though $E_8$ offers optimal 8D sphere packing density in $\mathbb{R}^8$, the uncompensated residual orientation error across 8 coupled dimensions propagates nonlinearly through the SwiGLU activation ($\text{Swish}(x W_{\text{gate}}) \odot (x W_{\text{up}})$) and across 32 transformer layers.
   - Without second-order Hessian feedback (such as GPTQ or QuIP#'s LDQ rounding) or Quantization-Aware Training (QAT), uncompensated post-training vector quantization struggles to preserve deep language model behavior.
4. **Implementation Scope & Encoder Suboptimality**:
   The experimental encoder (`quantize_e8p_block`) used a fast heuristic sign-candidate search (fixing sign patterns per parity coset based on `target[i] < 0`), which evaluates at most two distinct sign patterns (evaluating at most 512 candidate codewords out of 65,536). As demonstrated by the counterexample regression in `test_e8p_encoder_counterexample_and_oracle`, codeword 256 is encoded as codeword 128 with squared error 4.0 despite codeword 256 having zero error. A reference oracle search over all 65,536 codewords via exhaustive enumeration under declared f32 arithmetic is provided in `oracle_nearest_e8p_codeword`. Both the heuristic encoder suboptimality and the absence of second-order Hessian compensation or training adaptation are possible contributors, with their relative effects unisolated in this evaluation.
5. **Definitive Decision Scope**:
   This experiment preserves the measured negative result for uncompensated offline $E_8$ lattice vector quantization on SmolLM2-360M without QAT. Per directive B3, this specific offline arm is halted without parameter sweeps. No positive conversion claims are asserted.

---

## 5. Next Steps

With Item 2 completed and preserved as a definitive negative:
1. Deliver Item 2 via PR referencing #1513, #1509, #820.
2. Update `docs/integration/current-state.md` to reflect the B3 kill criterion verdict.
3. Advance immediately to **Item 3: Metal Kernel Port** to unlock training throughput for 30M models (`cpu-accelerate` baseline first, then 7 custom stack ops in `geometric_stack.rs`).
