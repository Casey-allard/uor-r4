# Draft: the unifying theory (lead's synthesis; to be reconciled with agent reports)

## T1. One mathematical object runs through the whole lineage
R^4 ≅ ℍ (quaternions). Every nonzero x ∈ ℍ is x = r·u, r = |x| > 0, u ∈ S^3 ≅ SU(2).
- TurboQuant-like era: (r, u) used to COMPRESS vectors (gain–shape quantization; keep r, quantize u on S^3).
- Router era: (shell, sector) = (quantized r, quantized u) used as an ADDRESS (routing).
- Store/recall era: addresses used to STORE and RECALL.
- Missing use: (r, u) as a GROUP ELEMENT acting on state — the similarity group ℝ₊ × SU(2) acting on ℍ by left
  multiplication: x ↦ r·(u ⊗ x). Composition: (r1,u1)(r2,u2) = (r1 r2, u1 u2). Radius composes as decay/gain; u
  composes as non-commutative rotation. This is the object that can replace matmul in the TIME-MIXING path.

## T2. What matmul does in an LM (four roles) and what geometry can replace
1. State transport (fold token into history summary)  -> REPLACEABLE by geometric transport (rotations × radial decay).
2. Content access (retrieve earlier info)              -> REPLACEABLE/RESTRUCTURABLE: codebook-LUT attention scores,
                                                          exact addressed memory; "least-energy retrieval" = hard
                                                          (Hopfield) attention = argmin energy.
3. Feature computation / knowledge (MLP)               -> NOT eliminable; knowledge lives in parameters. Geometry can
                                                          restructure (quaternion/PHM 4x param sharing) and D0-b low-bit
                                                          additive LUT kernels remove the multiplier, but bytes read
                                                          per token remain.
4. Readout (vocabulary distribution)                   -> LUT/hierarchical; tied embeddings.

## T3. Physics floor
Energy/token ≳ (bits of parameters + state touched per token) × (energy per bit moved from where they live)
                + (ops × energy/op). The first term dominates by 2–3 orders of magnitude for DRAM-resident weights
(Horowitz). So "no multiplier" is not the energy lever; bits/param and parameters-touched/token are.
Levers: fewer bits (ternary/4-bit), fewer params touched (sparse access = routing, D5), keep weights on-chip
(small models / cache-resident), recompute-free exact memory.

## T4. The theorem-backed capability geometry uniquely offers
Non-commutative, non-solvable transitions (2I ≅ SL(2,5), quotient A5) give single-lane NC1-complete state tracking
(Barrington). Diagonal SSMs / log-precision constant-depth transformers are in TC0 (Merrill et al.). So a
quaternion-transport recurrence has a provable expressivity separation over Mamba-style diagonal recurrences
(conditional on TC0 ≠ NC1) — and with 2I transitions it executes exactly with integer tables (no multiplies at all).
Caveat: expressivity ≠ learnability ≠ perplexity.

## T5. Exactness ladder for rotations (number theory gives primes a rigorous role)
- 2I (120 units of the icosian ring): exact, closed, bounded integers, but coarse (fixed 120 rotations).
- Golden gates (Parzanchevski–Sarnak 2017, arXiv 1704.02106): icosahedral group C60 + involution
  T = (2+φ)i + j + k of prime norm 7+5φ (N = 59) → words c0 T c1 … T ct almost-cover PU(2) optimally
  (Ramanujan/Deligne), with efficient navigation; |C|²(|C|−1)^{t−1} distinct rotations at T-count t.
  Exact synthesis over Z[φ]/Z[ω] with O(log 1/ε) length is standard in topological quantum compiling
  (Kliuchnikov–Bocharov–Svore 2013, arXiv 1310.4150).
  => a PRIME-indexed hierarchy of exactly-composable, multiplier-free (shift/add in Z[φ]) 4-D rotations,
     with the prime-norm normalization absorbed into the separately-kept radius (the owner's "preserve r").
- Golden rotation is the optimal U(1) covering generator (cited in 1704.02106 via [GVL68]); relevant to phase design
  (zeta phases cannot beat it for coverage).

## T6. Architecture implied
Geometric time-mixing (multi-lane quaternion linear recurrence h_t = r_t·(q_t ⊗ h_{t−1}) + B x_t; r from dyadic/golden
decay table; q from 2I or golden-gate codebook) + low-bit additive channel mixing (ternary/4-bit, LUT kernels) +
optional exact addressed memory + LUT readout. Parallel-scan trainable; integer-exact serving; transformerless;
no MoE; no routing (dense low-bit channel mixing) — but then D5 per-token sparsity is NOT met (owner decision).

## T7. Feasibility honesty
M1 training compute caps from-scratch models at ~10–100M params × ~0.3–2B tokens (confirm with arch agent).
Useful chat/code needs ≥~0.3–1B params trained on ≥~1T tokens, or distillation/transplant from open models.
=> "breakthrough" must be defined as an architecture-level advantage at matched compute + a distilled useful model,
not frontier-from-scratch on a laptop.
