## 8. The unifying theory: what geometry can and cannot replace

This section is the lead reviewer's synthesis after red-team correction.

**Angle convention.** Rotation angles are given as **SO(3) rotation angles**. The corresponding quaternion (S³) angle is half the SO(3) angle.

### 8.1 One object runs through the whole lineage

Every nonzero x ∈ ℝ⁴ ≅ ℍ factors as x = r·u, with r = |x| > 0 and u ∈ S³ ≅ SU(2) (Derived). Each stage of the project used this factorisation, in a different role:

| Era | Role of (r, u) | Status |
|---|---|---|
| TurboQuant-like quantizer | **Compression.** Keep r, quantize u on S³ (gain–shape VQ) | Sound, but already published (HQMQ, 2026). 4-D is a weak rate-distortion point (§2.1, §6.1) |
| Prime/manifold routing | **Address.** (shell, sector) = (quantized r, quantized u) | Hash routing. In the router era's own test, r contributed nothing (INC-0168) |
| Store and recall | **Key.** Addresses select stored records | Real, measured asset (KVAR), not yet integrated |
| (missing) | **Dynamics.** (r, u) as an element of ℝ₊ × SU(2) acting on state by x ↦ r·(u ⊗ x) | Where geometry can replace the state-transition matmul |

Under the dynamics role, radii multiply (decay or gain) and rotations compose non-commutatively. The owner's instinct to "preserve the radial direction" fits *dynamics*: the radius is the retention factor and the direction is the content transform.

The complex analogue already exists: LRU's λ = r·e^{iθ} (Literature 2303.06349). The quaternion version replaces the commuting phase e^{iθ} with a non-commuting u.

### 8.2 What matmul does in a language model

Per token, any autoregressive language model performs four functions (Derived):

1. **State transport.** Fold the new token into a summary of the history.
2. **Content access.** Retrieve specific earlier information.
3. **Feature computation and stored knowledge.** The MLP role, where most parameters live.
4. **Readout.** Map the state to a distribution over the vocabulary.

What geometry can do for each role:

| Role | What geometry offers |
|---|---|
| 1. State transport | **It replaces the transition matrix.** The per-lane transition becomes rotation × radial decay, exact and multiplier-free for snapped rotations (§8.4). Input-to-transition maps (x_t → q_t, r_t, b_t) are per-token projections. At the first layer they can be token-indexed table rows; deeper, they are low-bit additive maps (D0-b). |
| 2. Content access | **It supplies relative transport:** a data-dependent, non-commutative "quaternionic RoPE" (§8.4). The recall products remain unless designed away. Linear-attention recall with a matrix state S_t = r_t S_{t−1} + k̃_t v_tᵀ needs activation×activation outer products and reads, about 0.6M per token at a 23M-parameter scale (red-team C1). Softmax reads need q·k products. These can be (a) **designed away**, with codebook-quantized keys whose scores become table lookups (§3.6; the 600-cell polar code measured about int3-quality at 2.73 bits/dim) and exact addressed memory; or (b) **kept** as integer products, which needs the D0-b ruling in §11.2. |
| 3. Feature computation | **Restructurable, not removable.** Knowledge must be stored in parameters: about 2 bits/parameter, measured with ≥8-bit weights. Post-training int4 falls to 0.7; the authors did not test QAT and suggest it "may be necessary" (Literature 2404.05405). Geometry can share parameters (quaternion/PHM layers have 4× fewer parameters) or *address* them (sparse access), but it cannot remove the bytes. |
| 4. Readout | **Restructurable:** a tied, factored or class-based exact softmax, or an integer lookup table. |

### 8.3 The physics floor

Energy per token ≈ (bits of parameters and state touched per token) × (energy to move a bit) + (instructions × energy per instruction). Arithmetic circuits are a small term (§5).

For models larger than cache, the dominant lever is to **touch fewer parameters per token**. Allen-Zhu and Li found that a 32-expert MoE touching 8.8% of its parameters lost only about 1.3× knowledge capacity. That experiment used a *learned* router (Literature 2404.05405), and a learned router is exactly what the owner excludes.

For cache-resident models (≲20M parameters at ternary), the levers are instruction count and kernel efficiency, and sparse access buys little (arch report). This is why D5 and "no sparse routing" need an explicit owner ruling, scoped by model size (§11.1).

### 8.4 The geometric primitive

**The recurrence** (Derived; Hypothesis as a language-model component):

  h_t^(ℓ) = r_t^(ℓ) · ( q_t^(ℓ) ⊗ h_{t−1}^(ℓ) ) + b_t^(ℓ),  with r ∈ (0,1] and q ∈ SU(2).

**Parallel trainability.**
- The pairs (A, b), with A = r·q, compose associatively: (A₂,b₂)∘(A₁,b₁) = (A₂A₁, A₂b₁ + b₂). An associative scan therefore enables chunk-parallel training.
- Frame form: with U_t = q_t U_{t−1}, the recurrence reduces to a scalar-decay recurrence in the rotated frame, so existing chunked kernels apply (arch F8).
- Accuracy checks: errors are ≤3e-14 in float64. In float32 the relative error is ≤9e-6, and the frame drifts by ≤6e-6 at T=16,384 (red-team check). Float32 training is fine.

**Expressivity.**
- 2I ≅ SL(2,5) is non-solvable; its quotient is A5. One lane with input-selected q ∈ 2I solves the A5 word problem, which is NC¹-complete by Barrington's theorem.
- Diagonal, complex-diagonal and non-gated SSMs, and log-precision constant-depth transformers, are in TC⁰ (Literature 2404.08819).
- This is a separation from **diagonal** recurrences, conditional on TC⁰≠NC¹.
- It is **not unique**. Two learned reflections (DeltaProduct), PaTH, RWKV-7 and permutation gathers have the same power, and so does the current nonlinear cell in principle.
- **Limits.**
  - Among non-abelian simple groups, rotation-only 4-D lanes can carry only A5. S5 needs reflections (state-tracking report).
  - The binary dihedral groups are non-abelian and arbitrarily fine, but solvable. So only *non-solvable* structure forces 2I (§4.3).

**Learnability, measured** (§6.2).
- Quaternion lanes learned A5 exactly and extrapolated 16×.
  - With a 3-generator alphabet and 8 lanes, 5 of 7 runs succeeded, and outcomes were bimodal.
  - With the full 60-letter alphabet, 4 lanes with a curriculum succeeded.
- Every commutative model stayed at chance.
- The repo's near-identity parameterisation q = normalize(e0 + 0.1·raw) never learned it. Rotations near the identity commute to first order.
- Learning is fragile: only 1–2 of 8 lanes converge. A single lane, or training without a curriculum, failed.

**Exact serving and where exactness ends.**
- *2I (60 rotations in SO(3); 120 unit quaternions).*
  - State can be a group index: 1 byte, with composition by a 120-row table.
  - Snapped lanes served this way stayed at 100% to length 4,096, where the float32 models drifted to 0.62–0.74 (§6.2).
  - Acting on a vector in doubled ℤ[φ] coordinates costs at most 24 add/sub and 8 shifts per 4-D rotation. Coefficients stay bounded.
  - The resolution is coarse: the smallest non-identity rotation is 72°.
- *Golden gates (Parzanchevski–Sarnak).* Words c₀Tc₁…Tc_t over the icosahedral group, with T = (2+φ)i + j + k of prime norm 7+5φ, give a prime-indexed hierarchy with asymptotically optimal *almost*-covering and efficient navigation (Literature 1704.02106).
  - The theorem also predicts rare "big holes". At practical sizes this review measured covering *no better than random codebooks*, plus a hole around the identity (§6.4).
  - After scaling by √(7+5φ), the coordinates are exact in ½ℤ[φ]. Composition is exact but the norm grows, so practical serving must renormalise and round.
  - Their value is algebraic, not better compression.
- *Integer small rotations.* q ∝ p = (2^k, a, b, c) with a, b, c ∈ {−1, 0, 1}: 157 rotations, the finest at 7.15°.
  - Applying p is shifts and adds, but the normaliser 1/|p| is almost never dyadic. Only 15 of 157 codewords have rational |p|, and only the 24 Hurwitz units have dyadic unit coordinates (red-team, Derived).
  - Each fine lane therefore needs a constant multiply by r/|p|. That takes about 6 shift-adds per coordinate for ≤1% drift at 4k tokens, i.e. about 36 operations per lane-step versus 16 hardware multiplies (red-team, Measured). With 3 terms, an isometric lane's norm drifted 827× by T = 4,096.
- **The no-go result** (math report §2.4).
  - Any forgetting factor 0 < |λ| < 1 in ℤ[φ] is an expansion in the Galois-conjugate embedding. With λ = φ⁻¹, exact coefficients grow by log₂φ ≈ 0.694 bits per step.
  - A dyadic factor 2^−k fails in the same way, because denominators grow.
  - Therefore **every decay or forget gate must round the state each step**, as any fixed-point recurrence does. Exactness is available only for pure group-index lanes (2I, and cyclic phase lanes).

**Design principle** (Hypothesis, grounded in Krohn–Rhodes). Every finite automaton decomposes into a cascade of simple groups and reset (flip-flop) components. A principled geometric state is therefore a **mixture of lane types**, each validated on its own probe before language:

| Lane type | Algebra | Role |
|---|---|---|
| Group-index lanes | Exact 2I | The non-abelian simple factor A5 |
| Phase lanes | Cyclic C_N, modular add | Abelian counters |
| Gated reset and decay lanes | Shift-subtract, rounded | Flip-flops and memory |
| Reflection lanes (optional) | O(4) or Householder | S_n |

**Separation.** Tracking lanes must be *separate* group-index lanes (h_t = q_t·h_{t−1}, with no additive input and a readout by index embedding). They cannot sit inside an additive matrix state. The "no additive input" evidence (2609.18966, a single-author, repository-pre-registered study) is itself described by its author as "a probe … not a training recipe".

### 8.5 What follows

The coherent geometric language model has four parts:
- **Geometric state transport:** exact group-index lanes, rounded decay lanes, and relative transport.
- **Low-bit channel mixing,** executed as lookup tables (T-MAC-class).
- **Contextual access** by exact addressed memory and/or codebook-keyed lookup-table attention.
- **An integer readout.**

Every role-1 and role-2 multiplication is then either replaced by geometry (exact rotations and codebook score tables) or reduced to low-bit table lookups. The remaining exception is linear-attention or softmax products over *unquantized* activations, which need the §11.2 ruling.

The design is transformerless, has no MoE and no learned router, and is dense in role 3. So D5 is unmet unless the owner allows deterministic addressing of role-3 parameters for models larger than cache (§11.1).
