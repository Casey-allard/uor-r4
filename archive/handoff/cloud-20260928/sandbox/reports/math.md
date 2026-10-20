# UOR-R4 mathematics stress test

Author: mathematics specialist, 2026-09-25. Repo `/home/user/uor-r4` @ `413a32f` (read-only). Experiment code and raw
outputs: `scratchpad/exp/math/` (`e1`–`e9`, each `*_output.txt`). Labels: `[SOURCE file:line]`, `[MEASURED]` (script +
output file named), `[LITERATURE url]` (retrieved this session), `[DERIVED]` (shown here), `[HYPOTHESIS]`.

## 1. Executive verdict

1. **The exact mathematics is correct, and almost none of it is in the current model.**
   - Independently verified: 2I is a closed group (associativity on all 1,728,000 triples); it is perfect; 2I/{±1} is
     simple of order 60 (A5). The icosian Z-span is even unimodular of rank 8 with 240 roots, i.e. E8. The zeta table
     matches mpmath to 4e-10.
   - The D8 model path uses none of 2I/H4/Z[φ]/zeta/primes ("no hemisphere folding or H4 codebook", "not exact Z[phi]"
     `[SOURCE crates/uor-r4-training/src/joint_model.rs:1642,1652]`).
   - Its only geometry is a quantized continuous unit-quaternion multiply on 64 four-dimensional lanes.
2. **The transport is correct; the "ordinary" control is mislabelled.**
   - The quaternion arm is an exact left Hamilton product in SU(2)_L ⊂ SO(4) (3-dim, left-isoclinic).
   - The control H(u)H(e0) is a plane *rotation* through e0, and its compositions reach all of SO(4) (6-dim).
   - No transport-free control exists.
   - Quantizing without renormalization is harmless: 256-step norm std 6.3e-4 `[MEASURED e2]`.
3. **The one theorem-backed asset is non-solvable group transport, and it is not quaternion-specific.**
   - A 4-D linear lane with 2I transitions computes the A5 word problem, which is NC¹-complete.
   - Diagonal SSMs and transformers cannot do this at constant depth unless TC⁰=NC¹.
   - DeltaProduct/PaTH-style reflection products do the same.
   - The current nonlinear cell has this power already. The LM shows no quaternion benefit (2.110 vs 2.085 nats for the
     control).
4. **Measured A5 state tracking (tiny scale, E6).**
   - Quaternion lanes trained by gradient descent learn the A5 word problem *exactly*: 100% accuracy at 16× the training
     length.
   - With 4 lanes this holds for the full 60-letter alphabet, including at the repo's α=0.1.
   - The repo's Householder control cannot do the full alphabet, because it pins one reflection at e0.
   - Diagonal lanes fail, as the TC⁰ theory predicts.
   - Two *learned* reflections (DeltaProduct-style) do exactly as well as quaternions. The capability is therefore
     rotation transport in general, not quaternions.
   - A single lane can fail even when a solution exists; representability is not learnability.
   - From random initialization, gradient descent **rediscovered 2I**. Two of the four learned lanes have exactly A5's
     class-angle histogram, and their closure is a 120-element group with a 35.8° minimum angle. Swapping a lane for an
     exact 120×120 table lookup keeps 100% accuracy at 1,024 steps.
   - Precision is not the limit: under the project's grids, exact 2I tracking survived 200,000 steps in 512 of 512 runs
     `[MEASURED e6, e9]`.
5. **Exact Z[φ] arithmetic works only for isometric dynamics.**
   - A multiplier-free 2I rotation costs ≤24 integer add/sub plus 8 shifts.
   - Isometric accumulation grows coefficients only slowly: 5.7 bits at T=4096.
   - Any forgetting factor forces 0.694 bits/step of growth (711 bits at T=1024), and this is provably unavoidable.
   - Exactness and forgetting are incompatible `[MEASURED e1, DERIVED]`.
6. **Zeta phases add nothing measurable.**
   - As token codes: 37% of byte tokens and 96% of 4,096 tokens have a near-twin (cosine > 0.99); a hash has 0%.
   - As RoPE-style frequencies: on par with random sets at N=32, worse than 99% of random sets at N=128.
   - Discrepancy: no better than random.
   - At log-primes they carry the Landau bias (mean cos −0.12…−0.19), which encodes primality, not language
     `[MEASURED e3]`.
7. **"Prime least-energy Riemann manifold routing" is not one defined object.**
   - Least energy on a Riemannian manifold means geodesic nearest-neighbour, i.e. argmax of an inner product (hard
     attention). The primes are an enumeration hash.
   - The implemented energy (the 2I word metric) takes 4–9 values, which is why ties were recorded.
   - The router's "phase transport" is a gauge choice of the Hopf connection.
   - (shell, sector) addressing is hash routing. This is a tension with "no sparse routing" and with D5 (flagged, not
     resolved).
8. **The quantization geometry is correct, and its gains are real but bounded.**
   - The 600-cell beats a Hopf/PolarQuant-style grid by 0.55 dB at 6.9 bits; E8 beats the cubic lattice by 0.66 dB
     `[MEASURED e4]`.
   - The owner's original idea is essentially PolarQuant at depth 2, whose angles are Hopf coordinates. TurboQuant and
     PolarQuant both keep the radius.
   - The best practical use is E8 low-bit weight codebooks (QuIP#).
9. **No RH dependency.** The ML uses verified numerical zeros. The Lean files assume `hRH` and contain 0 `sorry`.
10. **Drift verdict.**
    - The mathematics doing the work in the current model is ordinary (gated RNN, softmax read, pointer mixture).
    - The geometric vocabulary is mostly decorative.
    - The one real theorem (group state tracking) is unexploited: no discrete 2I, no scan, no tracking objective.
    - Nothing in this mathematics addresses the actual bottleneck: learnability and scale for language.

## 2. Findings

### 2.1 Claim → verdict → evidence

| # | Claim (where) | Verdict | Evidence |
|---|---|---|---|
| 1 | Quaternion arm is a left Hamilton product, norm-preserving | **Correct & modestly useful** | Term-by-term match to (qx)ᵢ formulas `[SOURCE joint_model.rs:1742-1770; uor-r4-integer/src/model.rs:486-492]`; \|qx\|=\|q\|\|x\| `[DERIVED]`; 256-step drift std 2.8e-4 (units only), 6.3e-4 (units+state), worst 0.22% `[MEASURED e2]` |
| 2 | Householder pair is the "matched ordinary control" `[SOURCE docs/integration/joint-recurrent-campaign-2026-09-24.md:28-33]` | **Mislabelled** | It is an equally geometric rotation family: a rotation by 2∠(u,e0) in span{e0,u}, eigenvalues 1,1,e^{±2iψ}. It is a 3-parameter non-closed family whose products reach SO(4) (dim 6) ⊋ SU(2)_L (dim 3) `[DERIVED §2.2]`. Its pinned reflection is what fails full-alphabet A5 `[MEASURED e6]`. There is no truly ordinary arm: `Transport` has only these two variants `[SOURCE crates/uor-r4-integer/src/config.rs:12-15]`, so neither "no transport" nor an unconstrained 4×4 map is tested |
| 3 | α/√2 matches the two arms | **Correct** | ‖L_q−I‖_F ≈ 2θ = 0.2\|r\| for Q; ‖R−I‖_F ≈ 2√2ψ = 0.2\|r\| for H `[DERIVED]` |
| 4 | Quantized units need no renormalization | **Correct at T=256** | See #1. The quaternion norm is multiplicative, so drift is an additive random walk in log-norm: per step 1.8e-5, over 256 steps 2.9e-4 `[MEASURED e2]` |
| 5 | 120 H4 roots = 2I; exact product table | **Correct & useful (serving)** | Closure, all-triples associativity, orders {3:20, 4:30, 5:24, 6:20, 10:24}, 9 classes `[MEASURED e1]`; repo tests `[SOURCE group_table.rs:257-270]` |
| 6 | 2I is non-solvable (A5/SL(2,5)) | **Correct** | Derived subgroup = 2I (perfect); quotient class sizes 1,12,12,15,20, and its only normal class-unions have size 1 and 60 `[MEASURED e1]`. 2I ≅ SL(2,5) because both are perfect central Z/2 extensions of A5 `[DERIVED from standard facts]` |
| 7 | "E8 = H4 ⊕ φH4" via icosians `[SOURCE docs/formal_vocabulary.md:99]` | **Correct but irrelevant to LM capacity.** The row is labelled "Assumption" and says "identified as Z-modules", which is trivially true of any two rank-8 lattices. The substantive, true statement is an *isometry* | Z-span rank 8; with the form 2Q, where N(x)=a+bφ ⇒ Q=a+b, the Gram matrix has integer entries, det 1, even diagonal, and there are exactly 240 vectors with 2Q=2, so the lattice is E8 `[MEASURED e1]` |
| 8 | "Continuous embeddings in H4 ⊕ φH4" `[SOURCE learner/embedding.rs:1,139]` | **Mislabelled** | The companion is an independent learned quaternion initialized at root (7i+13) mod 120 with its own gradient `[SOURCE embedding.rs:173,186; jepa_trainer.rs:1999]`. There is no φ/Galois coupling, which contradicts AGENTS.md ("companion is not independent learned state") |
| 9 | Exact Z[φ] replaces multipliers | **Correct only for group actions and additive accumulation** | ≤24 add/sub + 8 shifts per 2I rotation (type-1 elements are signed permutations). Growth is bounded if isometric and exponential if contracting `[MEASURED e1, DERIVED §2.4]` |
| 10 | Integer serving is "multiplier-free" | **Syntactically true, energetically misleading** | General products are emulated by ≤128-iteration shift-add loops `[SOURCE uor-r4-integer/src/math.rs:59-78]`; the quaternion transport alone makes 16×64 = 1,024 such products per token `[DERIVED]` |
| 11 | Cayley–Dickson "endomorphic routing" `[SOURCE cayley_dickson.rs:13-44]` | **Wrong (mislabelled)** | `from_u32` is a multiplicative hash and `lie_product` is cyclic 2×2 minors, not a CD commutator (the repo's own audit agrees). The separate `transformerless/cd_space.rs` octonion product is genuine: \|ab\|=\|a\|\|b\| to 4e-15, alternative, non-associative `[MEASURED e8]`. Both are irrelevant to the LM |
| 12 | Non-commutative group state gives state tracking beyond transformers/SSMs | **Correct as a theorem, not unique, not implied learnable** | §2.3 `[LITERATURE 2404.08819, 2411.12537, 2502.10297, 2505.16381]`, `[MEASURED e6]` |
| 13 | Fixed zeta phases are useful multiscale coordinates | **Correct numerics; irrelevant/decorative** | §2.5 `[MEASURED e3]` |
| 14 | Fixed zeta constants do not require RH | **Correct** | Finite verified list; table = mpmath `[MEASURED e3]` |
| 15 | Prime identities / ordered n-lets | **Correct as identity plumbing; decorative as semantics** (repo agrees) | §2.5: prime-phase codes are smooth in ln p, i.e. in rank |
| 16 | H⁴ shell law sinh³r ⇒ "measure-consistent routing capacity" `[SOURCE router-research/hyperbolic_router_math_review.md]` | **Formula correct; capacity claim overstated** | The area of a geodesic sphere in H⁴ is 2π² sinh³r. Load depends on the *embedding's* density; quantile bins balance any density `[DERIVED]` |
| 17 | Hopf (χ,θ1,θ2) sectors, equal-mass χ bins `[SOURCE router_math_contract.py:94-195]` | **Correct but a weak partition** | sin²χ is uniform (Archimedes), so the bins are equal-area on S². Cells degenerate at the two singular circles. The 600-cell beats the best Hopf grid by 0.55 dB `[MEASURED e4]` |
| 18 | Router "phase transport" α + (λ/2)cos2χ·δ | **Overstated** | This is the Hopf connection ω = dα + ½cos2χ dδ evaluated pointwise, i.e. a gauge/chart choice. Parallel transport needs a path, and its holonomy is ½ the enclosed S² area `[DERIVED §2.6]` |
| 19 | "H⁴ × H⁴ coupled field" `[SOURCE CORE_PROJECT_GOALS.md]` | **Undefined** | A field F: H⁴→H⁴ is a map, not a product manifold. "H4" also collides with Coxeter H4 in "E8 = H4×H4" (unrelated objects) |
| 20 | "Least-energy" routing score d_R4^ℓ `[SOURCE formal_vocabulary.md:129]` | **Undefined (normative score NOT_YET_IMPLEMENTED)**; the implemented version is too coarse | The 2I word metric has diameter 3/5/8 (for 20 order-6 / 12 order-10 / 2 generators), so at most 9 energy levels `[MEASURED e1]`. This explains the recorded 6/6 ties `[SOURCE adr/0003:422]` |
| 21 | "Minimal theorem for spectral emergence" | **Correct but trivial** | The spectral theorem for the graph wave equation. Says nothing about LMs |
| 22 | 600-cell universal optimality | **Correct theorem, tangential** (repo states it correctly `[SOURCE stuck-point-review-response-2026-09-24.md:107-110]`) | Cohn–Kumar prove energy minimization for completely monotone potentials, which is not MSE quantization `[LITERATURE https://arxiv.org/abs/math/0607446]`. Measured MSE: 600-cell 0.0705 < Lloyd best-of-4 0.0718 < Hopf grid 0.0800 < random 0.105 `[MEASURED e4]` |
| 23 | D4/E8 lattices are better quantizers | **Correct & useful, bounded** | G(Z⁴)=0.0834, G(A3*)=0.0785, G(D4)=0.0767, G(E8)=0.0717, i.e. gains of 0.26/0.36/0.66 dB (0.04/0.06/0.11 bit/dim) `[MEASURED e4]`. The ceiling is 1.53 dB |
| 24 | "Quaternion 4× only under equivariance" `[SOURCE direction-decision-2026-09-24.md:30]` | **Correct** | Hamilton weight-sharing gives 4× fewer parameters per map, and helps only if the task respects that symmetry |
| 25 | RH manuscripts / Lean | **No ML dependency** | `hRH` is a hypothesis in 5 theorems and there are 0 `sorry` `[SOURCE research/riemann-lean/formal/lean/*.lean]`. Manuscript gaps were already documented `[SOURCE mathematical-foundations.md:103-157]` |

### 2.2 Transport group theory

q = cosθ + sinθ·n̂ gives L_q, which rotates span{1,n̂} and its orthogonal complement by θ. Its eigenvalues are e^{±iθ},
each doubled. L: S³→SO(4) is an injective homomorphism onto SU(2)_L, and SO(4) ≅ (SU(2)_L×SU(2)_R)/{±1}.
Compositions stay in SU(2)_L.

H(u)H(e0) is the rotation by 2ψ in span{e0,u}, with cosψ = \|u₀\|; it is the identity on the orthogonal 2-plane. Its
one-step generators e0∧v span 3 dimensions, and [e0∧v, e0∧w] = ±v∧w, so the brackets generate so(4). Consequences:

- **Q:** every element of any finite subgroup of SU(2) is a single step. Those subgroups are the cyclic, binary dihedral,
  2T, 2O and 2I groups, so one lane can run the word problem of Z_n, D_n, A4, S4 or A5 over *arbitrary* alphabets. S5
  is not reachable.
- **H:** single steps must be simple rotations through e0. In A5's standard 4-D representation, 24 of the 59
  non-identity elements (the 5-cycles) are double rotations. Only 9 of the 35 simple rotations have a plane containing
  (e₀−e₁)/√2, but those 9 include the generators (01k)^{±1} `[MEASURED e7]`. So H can track A5 over that generator
  alphabet but not over all 60 letters.
- **Both** have non-real eigenvalues, so both pass Grazzi et al.'s necessary conditions for parity and mod-m counting.

### 2.3 Expressivity theorems and what they do not say

- **Barrington (1989):** the word problem of any finite non-solvable group is NC¹-complete (stated in Merrill et al.
  §3.1, `[LITERATURE https://arxiv.org/abs/2404.08819]`).
- **Merrill–Petty–Sabharwal:** log-precision SSMs that are non-gated, diagonal, or simultaneously diagonalizable
  (Thms 4.2/4.4/4.6) lie in L-uniform TC⁰. So, unless TC⁰=NC¹, they cannot solve the S5 or A5 word problem (Cor. 4.7).
  Complex-diagonal (commuting) lanes are included. With input-dependent *non-diagonal* transitions (IDS4) or a
  per-step nonlinearity, one layer recognizes every regular language (Thms 5.1/5.2).
- **Grazzi et al. (ICLR 2025):** a finite-precision LRNN needs an eigenvalue outside [0,∞) to solve parity (Thm 1), and a
  non-real eigenvalue in some product to count mod 3 (Thm 2). Products of generalized Householders with eigenvalues in
  [−1,1] recognize every regular language (Thm 4) `[LITERATURE https://arxiv.org/abs/2411.12537]`.
- **DeltaProduct:** if G ⊂ SO(n+1) with n even, n Householders per token suffice (Thm 4). Empirically, n_h=2 learns S4
  and A5 "from their isomorphism to subgroups of SO(3)" `[LITERATURE https://arxiv.org/abs/2502.10297]`. That is the
  project's control family with both vectors learned.
- **PaTH attention:** a one-layer PaTH transformer with log precision solves an NC¹-complete problem (Thm 2.1). At 760M
  parameters it improves LM perplexity over RoPE (Wiki 18.03 vs 19.01) `[LITERATURE https://arxiv.org/abs/2505.16381]`.

**What follows [DERIVED].** One 4-D lane h_t = L(ĝ(x_t))h_{t−1}, with ĝ a lift A5→2I, solves A5 exactly (h_T = ±ĝ_prod
h₀; quadratic features identify ±ĝ). With per-step error ε, decoding survives T ≲ 0.31/ε (worst case) or (0.31/ε)²
(random errors), because 2I has a 36° minimum angle. Measured with the project's grids: no failure in 200k steps
`[MEASURED e9]`.

**What does not follow:**
- learnability;
- that quaternions are needed (DeltaProduct n_h=2, PaTH, RWKV-7, and 5-point permutation gathers have the same power,
  and gathers cost zero arithmetic);
- any perplexity gain on natural text;
- any gain for the current model: its cell `candidate = tanh(W_in e + W_s·RMS(h) + b)`
  `[SOURCE joint_model.rs:949-957]` is already a nonlinear RNN (Merrill Thm 5.1), and it is trained by sequential
  unroll, so parallel-scan associativity is unused.

The remaining theoretical merit of the transport is compactness. A bilinear (second-order) transition represents A5 in 4
dimensions, whereas first-order threshold constructions need on the order of |Q|·|Σ| units.

**Measured learnability (E6).** Setup: `exp/math/e6_a5_lane.py`, `e6m_a5_lanes.py`; outputs `e6c_run1.txt`,
`e6m_run*.txt`. Autograd, Adam lr 0.02, batch 128, length curriculum 2→4→8→16 (2,000 steps for 1 lane, 2,400 for 4
lanes). Readout: linear on [h, hᵢhⱼ]. The table gives accuracy at the final position for test lengths 16/64/256;
chance is 0.017. Each cell is seed 0; the extra seeds are listed after the table.

| Lanes × alphabet | Q (quaternion) | H (repo control, H(u)H(e0)) | P (two learned reflections, DeltaProduct n_h=2) | D (complex diagonal) |
|---|---|---|---|---|
| 1 × 6 generators | **1.00 / 1.00 / 1.00** | 0.02 (representable, not found) | — | 0.01 (impossible) |
| 1 × all 60 | 0.02 (representable, not found) | 0.02 (not representable) | — | — |
| 4 × 6 generators | — | **1.00 / 1.00 / 1.00** | — | 0.02 (impossible) |
| 4 × all 60 | **1.00 / 1.00 / 1.00** (also at α=0.1, the repo's scale) | 0.02 (not representable) | **1.00 / 1.00 / 1.00** | 0.02 (impossible) |

Seed 1 repeats the 4 × all 60 row: Q 1.00/1.00/1.00, P 1.00/1.00/1.00, H 0.015–0.020. Without the curriculum,
1 × all 60 also stayed at chance for Q and H (2,500 steps). These are 1–2 seeds of a synthetic task, so the results
are evidence for the theory's predictions, not a benchmark.

What the table shows:
- Theory predicts exactly which cells can succeed.
- Mild over-parameterization (4 lanes) plus a curriculum makes every representable cell learnable.
- The learned solutions are *exact*: zero error at 16× the training length.
- The repo's control fails on the full alphabet *only* because one reflection is pinned at e0. A general
  two-reflection rotation (P) matches the quaternion.
- The capability belongs to non-diagonal rotation transport, not to quaternions.
- **Discretization is lossless here** (`e6_snap.py`, `e6_snap_output.txt`). In the trained 4-lane Q model (seed 0),
  lanes 2 and 3 are exact A5 homomorphisms: their 60 generators have SO(3) angles {0°:1, 72°:12, 120°:20, 144°:12,
  180°:15}, the A5 class sizes. Each lane's multiplicative closure (tolerance 0.05) has exactly 120 elements with a
  35.8° minimum angle, i.e. a conjugate of 2I. Lanes 0 and 1 learned non-group auxiliary rotations. Serving lane 2 or 3
  by exact index composition in the 120×120 table gives accuracy 1.000 at L=256 and L=1,024. Gradient descent on the
  continuous SU(2) relaxation, followed by snapping, gives exact multiplier-free serving with no loss on this task.

### 2.4 Exact Z[φ]: cost and a no-go theorem [MEASURED e1, DERIVED]

Store doubled coordinates X=2x ∈ Z[φ]⁴ and use φ(a+bφ) = b+(a+b)φ and φ⁻¹(a+bφ) = (b−a)+aφ. Then (2g)X is always
even, so Y = ((2g)X)>>1 is exact. Cost per 4-D rotation:

- the 8 elements ±1,±i,±j,±k: signed permutation, 0 adds;
- the other 112 elements: 24 add/sub + 8 shifts.

For comparison, a float Hamilton product is 16 mul + 12 add, and pure group-index state is one table read. A sliding
window needs O(1), not 64, reads because W_t = P_t·P_{t−w}⁻¹. `compose_context_roots` recomputes 64 reads per token
`[SOURCE group_table.rs:82-104]`.

**No-go.** For 0 ≠ λ ∈ Z[φ], \|λ·σ(λ)\| = \|N(λ)\| ≥ 1. So a contraction \|λ\|<1 is an expansion \|σ(λ)\|>1 in the
Galois-conjugate embedding, and the integer coefficients grow like \|σ(λ)\|^T. With λ=φ⁻¹, measured growth is 11.5,
44.5, 177.9 and 711 bits at T = 16, 64, 256 and 1024, i.e. 0.694 bits/step.

Conversely, σ(2I) is again a set of unit quaternions (checked), so isometric accumulation s ← gs+u stays in a
rank-8 lattice ball of radius O(T): 53 = 5.7 bits at T=4096.

A dyadic factor λ=2^{-k} fails the same way, because λ^T has denominator 2^{kT}. So any *exact* forgetting recurrence
needs Θ(T) bits, and every forget gate must round `[DERIVED]`.

### 2.5 Zeta phases [MEASURED e3, e3b]

- **Table.** It matches mpmath at n = 1, 2, 100, 256, 512 (\|Δ\| ≤ 3.7e-10).
- **Landau.** Landau: Σ_{γ≤T} x^ρ = −(T/2π)Λ(x) + O(log T) `[LITERATURE https://mathworld.wolfram.com/LandausFormula.html]`.

  | x | mean cos(γₙ ln x), N=512 | predicted |
  |---|---:|---:|
  | 2 | −0.123 | −0.126 |
  | 3 | −0.163 | −0.163 |
  | 5 | −0.184 | −0.185 |
  | 7 | −0.186 | −0.189 |
  | 6, 10, 12 | +0.001 … +0.006 | 0 |
  | random, 95th percentile | 0.061 | — |

- **Equidistribution.** It holds only asymptotically (Rademacher 1956, under RH `[LITERATURE
  https://arxiv.org/pdf/2510.07710]`; Hlawka's unconditional version was not retrieved).

  | Sequence | Star discrepancy, N=512 |
  |---|---:|
  | {γₙ ln p/2π}, p = 2…10007 | 0.036–0.080 |
  | uniform random | 0.039 (5–95%: 0.024–0.062) |
  | golden {nφ} | 0.0036 |

- **Rigidity.** It is real: the fraction of unfolded spacings below 0.25 is 0.006 (Poisson 0.221), but a golden
  ladder is *more* regular (3 gap values).
- **As RoPE frequencies (D=1..255, range [2π/2048, π]).**

  | N | Frequency set | Far-field max K | Percentile vs random |
  |---|---|---:|---|
  | 32 | zeta | 0.318 | 28th |
  | 32 | random | 0.347 | — |
  | 32 | stratified | 0.339 | — |
  | 32 | geometric RoPE ladder | 0.400 | — |
  | 128 | zeta | 0.242 | worse than 99% (quasi-linear spacing ⇒ alias peak) |
  | 128 | random | 0.173 | — |
  | 128 | stratified | 0.153 | — |

- **As token-identity codes (the repo's use).** Similarity between primes p and q is (1/n)Σcos(γ_j ln(p/q)), a smooth
  kernel in ln p. Fraction of tokens whose nearest neighbour has cosine similarity > 0.99:

  | Setting | zeta | random frequencies | iid random phases (hash) |
  |---|---:|---:|---:|
  | 8 channels, 258 byte primes | 36.8% | 39.5% | 0% |
  | 8 channels, 4,096 primes | 95.6% | 96.4% | 0% |
  | 512 channels (`get_word_vector`) | 44.9% | 47.5% | 0% |

  So a zeta code is a *worse identity code than a hash*, and its only structure is locality in prime rank.

### 2.6 Router-era geometry [DERIVED]

- **Least energy.** For a curve γ, E(γ) = ½∫\|γ′\|² ≥ ½L², with equality for constant-speed minimizing geodesics. So
  "least-energy" selection among candidates is argmin d(x,y). On S³, d = arccos⟨x,y⟩; on H⁴, d = arccosh(−⟨x,y⟩_L). In
  both cases this is argmax of an inner product, i.e. hard attention or k-NN. All semantics lives in the embedding,
  which the prime/zeta assignment does not supply.
- **Hopf connection.** With z₁ = cosχ e^{iθ1}, z₂ = sinχ e^{iθ2}, α = (θ1+θ2)/2 and δ = θ1−θ2, the canonical connection
  is Im(z̄·dz) = dα + ½cos2χ dδ. The code's `transported_alpha = α + (λ/2)cos2χ·δ` `[SOURCE router_math_contract.py:115-131]`
  is this 1-form's δ-coefficient multiplied pointwise by δ. For λ=1, it is the fiber coordinate measured against the
  section obtained by horizontally lifting each latitude from δ=0 (a gauge choice). For other λ it is a sheared chart.
  In no case is it path-dependent parallel transport: the holonomy around a latitude is ½ of the enclosed S² area, mod
  2π. Routing by it is just another partition.
- **Locality.** The χ-bins are equal-area, but the δ-cells shrink to slivers near χ ∈ {0, π/2}, so nearby points can
  land in different sectors. A 600-cell Voronoi partition has none of these singularities.
- **What routing offers an LM.** A fixed partition of a *learned* embedding is LSH or hash routing. Locality is
  preserved up to boundary effects if and only if the embedding is locality-preserving, and prime ids are not. Used to
  choose experts, this is sparse routing and is excluded by the owner's constraint. Used as a memory address it is
  allowed, but then an ordinary LSH or learned k-NN at the same access budget is the right control (D5 already says
  this).

### 2.7 Quantization and the owner's original idea [LITERATURE, MEASURED e4]

- **TurboQuant** normalizes vectors, stores the L2 norm in float, then applies a random rotation plus per-coordinate
  Lloyd-Max. Its MSE is within ≈2.7× of the Shannon bound `[LITERATURE https://arxiv.org/abs/2504.19874]`.
- **PolarQuant** recursively converts coordinate pairs to polar form and stores the final radius in full precision
  `[LITERATURE https://arxiv.org/abs/2502.02617]`. For one 4-block, its level-1 angles are (θ1, θ2) and its level-2
  angle is χ = atan(‖x₃₄‖/‖x₁₂‖). Those are exactly Hopf coordinates.
- Neither method "ablates" the radius. The owner's idea is therefore a vector quantizer on S³ plus an explicit radius
  code. The 600-cell is a good such code, 0.55 dB better than the Hopf grid at 6.9 bits.
- VQ gains over scalar quantization are capped at 1.53 dB of space-filling gain. The proven practical win is E8
  lattice codebooks for 2-bit weights: QuIP#'s E8P uses 2¹⁶ entries decoded from a 256-entry table plus sign flips, and
  reaches near 3-bit quality `[LITERATURE https://arxiv.org/abs/2402.04396]`.

## 3. The mathematically strongest version of this project

**Load-bearing (theorem-backed, fits "no float matmul at serving, no MoE, transformerless"):**

**(A) Finite non-solvable group transport as a data-dependent multiplicative state and position mechanism.**

- *Structure.* Keep K small 4-D lanes. Each lane holds a group element G_t = a_t·G_{t−1} ∈ 2I (7 bits). The action
  a_t is chosen per token from the *input side only*, so the recurrence stays linear and can be parallel-scanned in
  training.
- *Serving.* One byte-table read per lane-step (exact, multiplier-free; `group_table.rs` already exists).
  - Relative transport between positions i and j is G_iG_j⁻¹, one more read.
  - A PaTH-style logit ⟨G_j k_j, G_i q_i⟩ rotates the key once at write time, at ≤24 integer adds per 4-D block.
  - Sliding windows cost O(1) via inverses.
  - The readout is a lookup of learned rows addressed by (lane, G_t). That is per-token sparse parameter access (D5)
    selected by a *state*, not by a learned expert gate. Whether this counts as forbidden "sparse routing" is the
    owner's call.
- *Training.* Use a continuous SU(2) relaxation, several lanes, a length curriculum, then snap to 2I. The 36° minimum
  angle gives an 18° decoding margin.
- *Guarantees.*
  - Proof: one layer expresses NC¹-complete tracking, which is impossible for diagonal SSMs or RoPE transformers at
    constant depth if TC⁰ ≠ NC¹.
  - Proof: states are exact for unbounded length.
  - Measured: learned solutions extrapolate 16× with zero error; gradient descent rediscovered 2I; snapped table
    serving was lossless to L=1,024; precision holds for 200k steps (E6, E9).
- *Not unique.* E6 shows that two learned reflections (DeltaProduct n_h=2) do exactly as well. Any finite rotation or
  permutation representation (e.g. the icosahedral group in SO(3), or S5 gathers) can be table-served the same way.
  The honest claim is: *the project's exact finite-group machinery is a clean, cheap implementation of a
  capability the linear-RNN literature has identified*. It is not a mechanism only geometry provides.
  The quaternion's one-step set is a closed group (SU(2)), and that is what beat the repo's pinned-reflection
  control. A lane with two learned reflections matches it, so no quaternion-specific edge was measured.

**(B) E8/icosian (and D4) lattice codebooks for 2–4-bit weights and KV cache.**
- They give theorem-backed packing and second-moment gains (measured 0.36 dB for D4 and 0.66 dB for E8 at high rate;
  larger shaping gains at 2 bits, per QuIP#).
- Decoding is a table read plus sign flips, with no multiplier.
- The win is lower memory bandwidth on an M1, which is the actual bottleneck for local decoding.
- This is where "E8 = H4⊕φH4" and the owner's "4-D polar with radius" idea can pay off.

**(C) Exact identity and addressing (UOR).** This is engineering, not geometry, but it is real.

**Decorative for an LM (keep only as identity or provenance, or drop):**
- zeta phases (measured: no better than random, worse than a hash as identity);
- primes as semantics;
- Hopf sectors and "phase transport" (a gauge choice);
- sinh³ shells;
- H⁴×H⁴;
- "least energy" (geodesic nearest-neighbour);
- Cayley–Dickson modules;
- √2/2i landmarks;
- the Galois companion (zero extra capacity);
- exact Z[φ] as a general serving arithmetic (impossible with forgetting).

**What the geometry cannot supply:**
- General prose quality. Language modelling is density estimation. The binding constraints are parameters, data and
  learnability: the 1.7M-parameter learners are 0.51–0.54 nats behind a 7.2M transformer.
- No theorem here addresses those constraints.
- NC¹-type tracking matters for entity, state and code tracking. There is no evidence it moves TinyStories NLL, and at
  760M parameters PaTH's measured gain over RoPE is about 1 Wikitext perplexity point (18.03 vs 19.01).

**Tensions to flag (not resolved here):**
- **D5 versus "no sparse routing."** Without learned gating, per-token sparse access needs a fixed or state-derived
  address function (hash, geometric cell, group state). The router-era (shell, sector) cells were exactly such routing
  to experts.
- **Scalar update gate.** In the current cell a *scalar* update gate ρ mixes all 256 coordinates every step
  `[SOURCE joint_model.rs:1052-1059]`. Transport norm-preservation therefore cannot carry long memory unless ρ→0
  `[HYPOTHESIS]`.

## 4. Recommendations (ranked)

1. **Retire zeta, prime, Hopf-sector and H⁴ "routing" as LM mechanisms.**
   - Keep primes only as identity plumbing. Zeta phases add nothing measurable (§2.5), and the router objects reduce to
     nearest-neighbour search or hashing (§2.6).
   - Impact: frees effort, and nothing is lost. Cost: 0.
   - Falsifier: a zeta arm beating a stratified-random frequency arm on held-out NLL at matched cost.
2. **Add the missing controls to the D8 transport comparison before attributing anything to geometry.**
   - Arms: no transport (plain gated RNN), an unconstrained input-dependent 4×4 map, and a two-learned-reflection
     (DeltaProduct₂) arm.
   - Cost: about 6 M1-hours per arm at the recorded 1,362 visits/s for 30M visits `[DERIVED from the lead's figures]`,
     or ~1/10-scale pilots.
   - Falsifier for "transport matters at all": no-transport ties both rotation arms on NLL.
3. **If geometry is to be causal, test (A) on the tasks where theory says it matters.**
   - Tasks: A5/S5 word problems (full alphabets), flip-flop, and entity or variable tracking in code.
   - Arms: continuous SU(2), 2I-snapped, DeltaProduct₂, S5 gathers, diagonal. Then check whether the winner changes
     TinyStories NLL at matched parameters.
   - E6 is the tiny-scale proof of concept.
   - Falsifier: the snapped 2I arm loses to DeltaProduct₂ or gathers at equal serving cost. In that case keep the
     generic mechanism and drop the quaternion framing.
4. **Test E8/D4 lattice codebooks on the #1017 reference weights.**
   - Compare at 2 and 3 bits per weight against 4-bit scalar at equal bits.
   - Expected impact: roughly 2× bandwidth at similar NLL (QuIP# precedent). Cost: CPU-hours.
   - Falsifier: no NLL gain at equal bits.
5. **Correct the wording.**
   - `B_ico` is an isometry theorem.
   - `learner/embedding.rs` implements S³×S³, not H4⊕φH4.
   - Router "phase transport" is a gauge choice.
   - "Least energy" is geodesic nearest-neighbour.
   - Exact Z[φ] is only valid for isometries.
   - The integer path is "multiplier-instruction-free", not multiplication-free.
6. **Cheap hygiene.**
   - Use prefix/inverse updates instead of 64 reads per token in `compose_context_roots`.
   - If transport lanes are meant to hold state, give them their own retention path.

## 5. Open questions

- Is the 600-cell MSE-optimal among 120-point S³ codes? It is the best of those I tested (it beat Lloyd best-of-4), but
  I know of no proof.
- Does 2I snapping preserve the learned solutions inside a real LM? Does multi-lane group transport survive the D0-b
  4-bit interfaces? E6 is a tiny synthetic.
- Does data-dependent group transport help NLL at about 2M parameters? PaTH's evidence is at 760M.
- Hlawka's unconditional equidistribution theorem and Barrington's original paper were not retrieved. The statements
  above rely on the retrieved secondary statements (Murahara–Onozuka; Merrill et al.). Cohn–Kumar was retrieved
  (introduction and §7); it treats energy, not MSE.
