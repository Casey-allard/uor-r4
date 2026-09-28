## 4. Mathematics stress test

Sources: the mathematics report (nine numerical experiments, e1–e9), the state-tracking report, and the literature and architecture reports.

### 4.1 Verdict on each mathematical claim

| Claim | Verdict | Evidence |
|---|---|---|
| The quaternion arm is a left Hamilton product and preserves norm | **Correct.** Quantizing without renormalizing is harmless at T=256 (norm-ratio std 6.3e-4) | Source; verify F6; e2 |
| The Householder pair is the "ordinary", non-geometric control | **Wrong framing.** H(v)H(e0)x = v·x·v is itself a non-commutative quaternion map. Its compositions reach SO(4) | Derived; verified numerically to 1.3e-15 |
| The 120 H4 roots form the group 2I, with an exact table | **Correct.** Closed; all 1,728,000 triples associative; order census matches 2I | e1; independent rebuild |
| 2I is non-solvable (A5 ≅ 2I/±1; 2I ≅ SL(2,5)) | **Correct** | e1 |
| "E8 = H4 ⊕ φH4" via icosians | **Correct.** The icosians form an even unimodular rank-8 lattice with 240 roots. formal_vocabulary.md calls this an "Assumption"; it is an isometry theorem. Irrelevant to model capacity | e1 |
| `learner/embedding.rs` implements "H4 ⊕ φH4" | **Mislabelled.** The companion is an independent learned quaternion with no Galois coupling, which contradicts AGENTS.md | Source |
| Exact ℤ[φ] replaces multipliers | **Correct only for isometries.** A 2I rotation costs ≤24 add/sub plus 8 shifts. Any forgetting gate forces coefficient growth of 0.694 bits/step (no-go theorem, §8.4) | e1; state-tracking kernel check |
| Non-commutative group state gives state tracking beyond transformers and diagonal SSMs | **Correct as a conditional theorem** (Barrington; Merrill et al., assuming TC⁰≠NC¹, fixed depth, log precision). **Not unique**: two learned reflections, PaTH and permutation gathers match it. **Learnable, but fragile** (§6.2) | Literature; e6; state-tracking |
| Quaternion lanes can track symmetric groups | **Only A5 among non-abelian simple groups.** S5 cannot be tracked by any number of rotation-only 4-D lanes; reflections are needed. Measured S5 accuracy equals the parity-only level | Derived; state-tracking |
| Zeta phases are useful multiscale coordinates | **Decorative** (§4.2) | e3, e3b |
| RH is needed | **No.** The code uses 512 zeros verified against mpmath to 4e-10. The Lean files take `hRH` as a hypothesis and contain no `sorry` | e3; source |
| "Prime least-energy Riemann manifold routing" | **Not one well-defined object** (§2.2) | Derived |
| Hopf sectors and "phase transport" | **The formula is correct; the transport claim is overstated.** The code evaluates the Hopf connection pointwise, which is a gauge choice rather than parallel transport. Cells degenerate near the singular circles | Derived |
| The H⁴ sinh³r shell law balances routing capacity | **The formula is correct; the capacity claim is overstated.** Load depends on the embedding's density | Derived |
| "H⁴ × H⁴ coupled field" | **Undefined as stated.** The name also collides with the Coxeter group H4 | Derived |
| 600-cell optimality | **The theorem is correct but tangential** (it concerns energy minimization, not MSE). Measured, the 600-cell *is* the best 120-point S³ code tested: MSE 0.0705, vs 0.0718 for a Lloyd code, 0.0800 for a Hopf grid and 0.105 for random | e4 |
| Cayley–Dickson "endomorphic routing" | **Mislabelled.** It is a multiplicative hash plus cyclic minors | e8 |

### 4.2 Zeta-zero phases, measured

- **As token-identity codes (the repo's use).** With 8 channels, 36.8% of byte tokens and 95.6% of 4,096 tokens have a near-twin (cosine > 0.99). A hash gives 0%. **A zeta code is a worse identity code than a hash.**
- **As RoPE-style frequencies.** At N=32 they sit at the 28th percentile of random sets. At N=128 they alias *worse than 99% of random sets* (quasi-linear spacing).
- **Discrepancy.** No better than random: 0.036–0.080 vs 0.039. That is 10–20× worse than the golden Weyl sequence (0.0036), which is the provably optimal U(1) covering generator.
- **At log-primes.** They carry Landau's bias (mean cosine −0.12 to −0.19, matching theory), so they encode *primality*, not language.

### 4.3 Where exact geometry is forced, and where it is merely convenient

- **Classification.** The finite subgroups of SU(2) are the cyclic groups, the binary dihedral groups, 2T, 2O and 2I.
  - The binary dihedral groups are non-abelian and arbitrarily fine.
  - **Non-solvable** structure in a quaternion lane forces 2I.
- **Consequence for training.** When training converges to an exact A5 tracker in an SU(2) lane, the learned elements *must* be conjugate to 2I. The measured "rediscovery" of the 600-cell (§6.2) is therefore a confirmation that the solution was found and snaps cleanly. It is not a surprise.
- **Closure is not exact before snapping.** At tolerances tighter than 0.05, the learned closure is not finite (red-team check). Snapping is what makes the lane exact.

### 4.4 The mathematically strongest version of the project

**(A) Finite non-solvable group transport as data-dependent multiplicative state and relative position.**

Design and serving:
- Each lane's action a_t ∈ 2I is chosen per token.
- Serving costs one byte-table read per lane per step, plus one more for relative transport G_iG_j⁻¹.
- Attention logits take the PaTH form ⟨G_j k_j, G_i q_i⟩ with multiplier-free rotation.

Training: a continuous SU(2) relaxation with a *non*-near-identity parameterisation and a length curriculum, followed by snapping.

Guarantees:
- Exact for unbounded length (e9; state-tracking automaton to length 4,096).
- NC¹-complete tracking in one layer, under the stated assumptions.

Its edge over DeltaProduct, PaTH or gathers is **serving cost and exactness, not capability**.

**(B) E8/icosian and D4 lattice codebooks for 2–4-bit weights and KV.**
- The gains are real but bounded: E8 is 0.66 dB better than the cubic lattice.
- Decoding is multiplier-free (the QuIP# E8P precedent).
- This is the honest home of "E8 = H4 ⊕ φH4" and of the owner's 4-D polar idea.

**(C) Exact identity and addressing (UOR).** This is engineering rather than geometry, and it is real.

**Decorative for a language model:**
- zeta phases;
- primes as semantics;
- Hopf sectors and "phase transport";
- sinh³ shells;
- H⁴×H⁴;
- "least energy" as physics;
- the Cayley–Dickson routing;
- the Galois companion.

Keep these as identity or provenance only, or drop them.

**What the mathematics cannot supply is general prose quality.** Language modelling is density estimation, and its binding constraints are parameter and data scale and learnability.
