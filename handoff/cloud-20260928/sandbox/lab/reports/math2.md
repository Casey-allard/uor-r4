# math2 — Where geometry pays in a small LM, and why hyperbolic wins on trees but barely on next-token loss

2026-09-26 · Mathematician, lab phase 2 · Scripts and JSON outputs: `$S/lab/exp/math2/` (named per result).
Labels: **M** Measured (script named; one thread per run, seed 0 unless stated), **D** Derived (shown), **L** Literature
(retrieved this session, arXiv id), **H** Hypothesis. No repository edits.

## 1. Verdict

- **Hyperbolic geometry is an inductive bias for sparse, hierarchy-determined retrieval, not extra capacity.** At the D8
  read width (64 dims), dot keys learn nothing beyond the always-root rule on this repository's code-scope tree
  (non-root 0.4%, unchanged with 3× training) or on a depth-6 synthetic tree (0.7%); hyperbolic keys reach 85.5% and
  97.4% [M `e4c`, `e4`]. A learned per-key potential recovers only a third (27.7%) [M `e11`]. On the small regular tree
  dot catches up with 2,000 steps (92–98%), so part of cycle 1's margin was learning speed [M].
- **Why ~0.02 nats:** in the saved cycle-3 models the Lorentz read beats Dot on copies whose last occurrence lies in an
  *enclosing* scope by 0.060/0.062/0.067 nats/token in all three code seeds (CIs exclude 0), but those are 5.4% of
  targets. The large seed-to-seed swings sit on non-copyable tokens: training variance, not geometry [M `e3`].
- **Ceiling:** a copy oracle given *exact* scope relations gains 0.010/0.013/0.020 nats at 128/512/2,048 tokens, while
  widening the window gains 0.40 [M `e12`]. Hierarchy-aware *scoring* cannot move mean NLL much; the long-context lever
  is *admission*, where hyperbolic geometry supplies an exact index.
- **The trained Lorentz read barely uses curvature:** keys on a shell of radius 3.47 ± 0.22; the angle explains 81% of
  score variance [M].
- **Dot→Lorentz transfer is exact by norm completion** (KL 6.3e−6 on the real D8 read, RoPE-invariant to 1e−13, 8
  fraction bits of log₂z) [M `e9`], **but exactness keeps the teacher's blind spots:** a converted head stays at 0.4% on
  the code tree after fine-tuning. Annealing the completion radius into the curved regime lifts it to 31%, a direct
  curved conversion to 52.5% (fresh hyperbolic: 85.5%) [M `e11`, `e11b`].
- **Hamiltonian heatmap:** the energy read is attention with a potential, which is the key radius [D]. Conservative S³
  dynamics are exact and reversible but never settle on a key; retrieval needs dissipation [M `e6`].
- **Triangulation:** Busemann landmark bounds give exact admission (5.1% of keys admitted in H⁸ vs 47% in R⁶⁴ at 32
  landmarks) [M `e5`], for keys that use the radius.
- **Primes/CRT/Galois:** no advantage for exact content memory (all universal hashes within 0.2% of ideal on 1.5M real
  keys); tabulation hashing is the multiplier-free choice; CRT only for positional rings [M `e8`].
- **Shortlist:** (1) E8/sign product-key parameter memory; (2) long-context hyperbolic multi-scale memory with Busemann
  admission; (3) Lorentz heads with free radii and a radius objective, judged per event (§7).

## 2. What geometry language and code have

| Structure | Evidence | Geometry | Capacity result |
|---|---|---|---|
| Latent trees: syntax, scopes, discourse | Mutual information decays as a power law under PCFG-like generation because the tree distance between positions is ≈2 log_q τ; Markov/HMM sources decay exponentially [L 1606.06737] | Hyperbolic | Sarkar: trees embed in H² with worst-case distortion 1+ε at edge scale τ = (1+ε)/ε · 2 log(deg_max/(π/2)); precision Θ((ℓ/ε) log deg_max) bits (matching lower bound); Euclidean space cannot reach arbitrarily low distortion in any dimension; WordNet MAP 0.989 in 2 dims vs 0.87 for an optimized 200-dim Poincaré embedding [L 1804.03329] |
| Heavy-tailed co-occurrence (Zipf, hubs) | Heterogeneous degrees plus any metric imply an effective hyperbolic geometry; degree exponent γ = 2α/ζ + 1 [L 1006.5169] | Hyperbolic | Radius = generality: expected degree ∝ e^{−ζr/2} |
| Position, periodicity; entity/state tracking | wave-1 `math.md` | S¹ factors (RoPE); SU(2)/finite groups | Lorentz isometries (§4); exact table-served group state |
| Symmetries of meaning (variable renaming, paraphrase) | — | none needed | Invariance to renaming is what copy/pointer reads and exact memory provide; no metric geometry supplies it |

**Star bound [D].** For a star with n leaves in Rᵈ at distortion D < √2, centred leaf vectors satisfy
⟨vᵢ,vⱼ⟩ ≤ D² − 2 < 0, and at most d+1 vectors are pairwise obtuse, so d ≥ n−1; for constant D, packing gives
d ≥ ln n / ln(D+1). In H² leaves at radius r are 2r + 2 ln sin(θ/2) + o(1) apart, so distortion 1 + O(ln n / r).

Cautions: distortion bounds concern metrics, while retrieval needs only ranking. 64 Euclidean dims *represent* it (98%
on a densely supervised tree) but fail to *learn* it under sparse signal (§3a). Hyperbolic space trades dimension for
precision, about d bits per distance d [L 1804.03329]; the cycle-2 radius+direction quantizer pays that cost.

## 3. Why large on hierarchical retrieval, small on the LM

**3a. Capacity vs learnability [M `e4_crossover.py`, lead's `geoattn.py` for the code tree].** Non-root accuracy at
48/96/192 stored nodes (depth 6: 96/384/768):

| Tree (always-root share at the training count) | Steps | dot 8 | dot 16 | dot 64 | hyp 8 | hyp 64 |
|---|---:|---|---|---|---|---|
| Synthetic 4-ary depth 5 (53%) | 2,000 | 99.0/97.4/92.1 | 99.8/98.8/96.6 | 99.9/99.5/98.0 | 99.7/–/99.4 | 100/100/100 |
| Same, cycle 1 (raw accuracy) | 1,500 | –/–/54.1 | –/–/68.2 | — | –/–/98.5 | — |
| Synthetic 4-ary depth 6 (69%) | 2,000 | — | — | 6.5/2.7/0.7 | — | 99.8/98.6/97.4 |
| Code-scope tree (92%) | 1,500 | — | 0.3/1.7/0.4 (cycle 2) | 0.0/1.7/0.4 | — | 89.6/88.3/85.5 |
| Code-scope tree | 4,500 | — | — | 0.0/1.5/0.4 | — | — |
| Code-scope tree, dot 64 + learned per-key potential | 1,500 | — | — | 30.4/31.5/27.7 | — | — |

Dot keys fail once the non-root training signal is sparse (always-root share ≥69%): they settle on "attend to the root". In hyperbolic space the nearest
stored point along a query's ray is the deepest ancestor by default. A learned per-key potential, which is what a free
radius contributes to first order (§4), recovers only a third of the gap (27.7% vs 85.5%) [M `e11`]; curvature supplies
the rest.

**3b. What the data offers [M `e12_scope.py`; code held-out, 12,000 targets].** Scope relation of in-window
candidates; gain of a D8-shaped copy+bigram oracle (recency bins, induction match, NoRead slot) from exact scope features:

| Window | same scope | enclosing | closed child | other | depth s.d. | oracle scope gain (nats) | target in window |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 128 | 44.6% | 23.6% | 17.5% | 14.3% | 0.54 | 0.0096 | 57% |
| 512 | 20.8% | 24.5% | 17.9% | 36.8% | 0.80 | 0.0133 | 73% |
| 2,048 | 8.2% | 19.1% | 15.0% | 57.7% | 1.01 | 0.0201 | 85% |
| 8,192 | 3.6% | 14.8% | 13.2% | 68.4% | 1.15 | — | — |

Widening 128 → 2,048 without scope features: 3.505 → 3.108 nats.

**3c. Where the cycle-3 gain sits [M Rust `probe` + `e3_analyze.py`].** Queries/keys rebuilt from the saved models
(read masses match the models' own to 1.7e−5); 101,600 code targets; Lorentz flat minus Dot, nats/token, 95% CI:

| Target | Share | Seed 2 | Seed 3 | Seed 4 |
|---|---:|---:|---:|---:|
| Not in window | 57.4% | +0.040 [0.032, 0.048] | −0.045 [−0.053, −0.037] | +0.006 (n.s.) |
| Copy, same scope | 31.3% | −0.050 | −0.029 | +0.013 |
| **Copy, enclosing scope** | 5.4% | **−0.060 [−0.096, −0.024]** | **−0.062 [−0.095, −0.030]** | **−0.067 [−0.102, −0.031]** |
| Copy, other scope | 2.6% | −0.071 | −0.063 | −0.013 (n.s.) |
| All | | −0.004 (n.s.) | −0.046 | +0.003 (n.s.) |

WikiText seed 1: −0.040 overall (−0.047 not in window, −0.026 copyable).

**3d. Read geometry.** Lorentz keys: radius 3.47 ± 0.22 (within-window s.d. 0.21), queries 3.69 ± 0.16; R²(score, cos θ)
= 0.81 (Dot: 0.97); equalizing radii changes the read by KL 0.043 and keeps 76% of top-1 choices [M].

**Synthesis [D].** Average gain = Σ (event share × per-event gain). Hierarchy-determined retrieval events are ~8% of
targets at 128 tokens and worth ≲0.01 nats even with perfect scope knowledge; the Lorentz read realizes ~0.005, and the
rest of cycle 3's differences is seed variance. The advantage becomes measurable with (1) windows ≥2k, where 58–68% of
candidates lie in unrelated scopes and the oracle value doubles; (2) sparse signal that dot keys never learn (3a), as
in long chat memories over many topics; (3) event-level evaluation (enclosing-scope copies, scope/entity resolution);
(4) an objective that pays for radius (§5e); (5) narrow keys, for energy.

Footnote [D + M `e6`]: keys filling a hyperbolic ball would localize the Gibbs read only for β ≳ n−1 (in H⁸, far-shell
mass 96/33/6% at β = 3.5/7/8.75); the trained β ≈ 3.5 at n = 64 works only because the keys form a thin shell.

## 4. Converting pretrained dot heads into Lorentz heads

**(1) Pattern match.** The first-order (Dot-matched) lift holds only near z₀. The D8 teacher is peaked (mean
max−median logit 8.3, p95 13.1; |q|,|k| ≈ 13) and the naive lift is far off: KL 1.42 nats, 0.78 with β, δ fitted
[M `e9`]. Two causes: the radius term penalizes large keys where dot rewards them (salience, attention sinks), and at
large norms arcosh z ≈ ln 2|q||k| + ln(1 − cos θ), a power law in angle.

**Exact construction [D].** Give keys a completion coordinate √(C²−1−|k|²) and queries a separate one
√(Q²−1−|q|²). Then every q₀ = Q and k₀ = C, and z = QC − q·k exactly. With β = QC/√r and s = q·k/√r,
β(δ − arcosh z) = const + s + (√r/2QC)s² + O((QC)⁻²). Measured KL: 6.7e−4 / 6.3e−6 / 6.4e−8 at QC = 10³/10⁴/10⁵
(radii 4.1/5.3/6.4). Every dot head is therefore a limit of Lorentz heads. Freeing a key's completion adds a per-key
potential and temperature: arcosh(QCe^ρ − x) ≈ ln 2QC + ρ − x/(QCe^ρ).

**(2) RoPE.** Rotating the spatial block is an SO(n) ⊂ SO⁺(1,n) isometry fixing the origin; completion coordinates are
not rotated. z(R_m q̃, R_n k̃) = z(q̃, R_{n−m} k̃) holds to 8.5e−14 [M]. Caveats: RoPE touches angle only, so the radius
stays position-free (good for a hierarchy channel); position cannot move into boosts, whose norms grow exponentially;
RoPE's long-range decay carries over unchanged.

**(3) Exceeding the teacher.** Not by compression: k-dim students fitted to the D8 Dot read reach held-out KL
0.61/0.29/0.06 (dot) vs 0.72/0.45/0.29 (Lorentz) at k = 4/8/16 [M]. On a real hierarchy
[M `e11`, `e11b`; code-scope tree, 64 dims, one seed, 1,500 fine-tuning steps from a dot teacher at 0.4% non-root]:

| Fine-tuning arm | Non-root at 48/96/192 |
|---|---|
| Continue as dot / dot + per-key potential | 0.0/1.5/0.5 · 0.0/1.5/0.4 |
| Exact conversion (QC = 10⁴), free radii and QC | 0.0/1.4/0.4 (QC moved 1.0e4 → 1.02e4) |
| Exact conversion, QC annealed 10⁴ → 1 over 750 steps | 35.2/34.6/31.3 |
| Direct curved conversion (natural lift; raw accuracy 91% → 1% at step 0) | 58.8/56.7/52.5 |
| Reference: hyperbolic from scratch | 89.6/88.3/85.5 |

Exact conversion keeps every curvature effect at O(1/QC), so the head inherits the teacher's basin. Exceeding the
teacher requires entering the curved regime and paying a pattern change. For open-transformer heads: convert exactly,
anneal QC per head under a KL-to-teacher penalty, and keep the heads whose event-level metrics improve.

**(4) Multiplier-free serving.** Per query, build by repeated addition a table of q·c for every 4-bit key code c (16
entries per coordinate, or 256 per coordinate pair); a candidate then costs d/2 table reads and adds. T-MAC does this
for low-bit GEMV on M-series CPUs, with up to 70% less energy than llama.cpp [L 2407.00088]. q₀k₀ comes from a
per-query table over a log-coded k₀. The weight is a per-head table on log₂z: 11 integer plus 8 fractional bits give
KL 0.002 (6 bits: 0.023) [M]. Near z ≈ 1 (small radii), use cycle 3's stable form with 24 guard bits.

## 5. Mechanism proposals

**(a) Multi-scale hyperbolic memory ("fractal association").**
- *Mechanism.* A level-ℓ entry summarizes c^ℓ tokens as the Lorentzian centroid of its children,
  μ = Σxᵢ/√(−⟨Σx,Σx⟩_L). One Gibbs read covers all levels, with level potentials.
- *Derived.* cosh r_μ = c·cosh r / √(c²cosh²r − sinh²r·|Σuᵢ|²). Coherent chunks keep their radius; incoherent ones
  sink toward the origin. So "abstraction = small radius" comes for free.
- *Level spacing.* Nested 45° cones need ≈ −ln tan(22.5°) = 0.88 of radius per level [L 1804.03329], so 4,096 tokens in
  16-token chunks span ~2.6 in radius.
- *Expected / serving.* Sublinear long-context recall; ranking gain bounded by the oracle (≈0.02 nats at 2k). Q8
  Lorentz scores (2 scalar products + 1 table read per admitted entry); centroids are running Lorentz sums plus one
  inverse-square-root table read per closed chunk.
- *Falsifier (here, ≤15 min).* MQAR/tree at 4k–16k candidates, two-level hyperbolic memory vs flat dot top-k with
  trained advertisements at equal comparisons; gate ≥99% admitted at ≤3% scored.
- *Measured negative.* A horocycle positional prior, d = 2 arcsinh(λΔ/2) (power-law recency, the KERPLE-log family
  [L 2205.09921]), fitted at 128 tokens extrapolates *worse* than exponential decay in the copy oracle: NLL at 2,048 was
  3.264 vs 3.226 (free bins 3.307) [M `e10`]. The positional prior is not the lever.

**(b) Learned curvature per head, snapped to the cheapest geometry.**
- *Derived.* Completion choices span the cheap geometries:
  - dot product: separate completions, QC → ∞;
  - Gaussian (squared-Euclidean) kernel: key-only completion k₀ = √(1+|k|²+C²), q₀ = C → ∞, giving
    z ≈ C² + ½ + ½|q−k|² − ½|q|²;
  - horospherical: a shared completion coordinate puts all points on a horosphere, so d = 2 arcsinh(|q−k|/2)
    (Euclidean at short range, logarithmic at long range);
  - log-angle: no completion, large norms;
  - genuine hyperbolic: moderate radii.
  So a head has a one-to-two-scalar "geometry dial". An H^a × S^b head adds a sign-bit factor served by XOR+popcount.
- *Snapping after training.* Large QC → int8 dot tables; the angular factor → sign bits; only radius-using heads pay
  for the log-z table.
- *Falsifier.* In a converted multi-head student ≥90% of dials drift to the dot end, or snapping costs >0.01 nats.

**(c) Hamiltonian heatmap.**
- *Read.* p(j) ∝ exp(−β(d(q,kⱼ) + V(kⱼ))) is attention with a potential, and V is the key radius of a completed Lorentz
  head (§4).
- *Thermodynamics.* The read localizes only above the key cloud's volume-growth rate, the analogue of the hot/cold
  transition at β = 1 in hyperbolic random graphs [L 1006.5169]; initialize β accordingly.
- *Dynamics [M `e6`].* Over 20,000 steps, a Lie-group leapfrog on S³ keeps |q| = 1 to 1.3e−14, energy error ≤1.7e−4 and
  reversal error 2.3e−12 (Euler: energy error 26). But the undamped query oscillates 20–62° from the key. With friction
  it reaches <1° in 105–273 steps, so a Hamiltonian read costs ~100× a softmax.
- *Capacity.* A superposed 64-wide state holds ~8 items at ~50% recall whatever the decay; conservative lanes only
  flatten recall over age (dissipative λ = 0.9 keeps the newest item at 60–82%) [M]. LinOSS's dissipative scheme beats
  its conservative one on average (67.8 vs 65.0), which wins only on energy-conserving dynamics [L 2410.03943].
- *Use and serving.* Exact conservative SU(2)/2I lanes for phase, position and counters: one 120×120 table read per
  lane-step, reversible, so O(1)-memory backprop. Dissipative lanes for content. The Gibbs read is today's exp-table
  softmax.
- *Falsifier.* Moving 16 of D8's 64 lanes to exact conservative lanes does not improve length generalization on
  bracket-depth or counting probes.

**(d) Triangulation = Busemann admission.**
- *Derived.* B_ξ(y) − B_ξ(x) = d(x,y) − 2(y|ξ)_x. The error is *additive*: the Gromov product, ≈ φ²/4 for an angular
  deviation φ at x. The Euclidean projection bound |y−x| cos φ errs *multiplicatively*.
- *Cost.* A coordinate B_u(x) = ln(x₀ − ⟨x_s,u⟩) is one inner product plus a log table per query, and m bytes per key.
- *Limit.* m landmarks resolve only ≈ log_b m tree levels. This is an exact coarse index, the owner's OSPF picture with
  a correctness guarantee.
- *Measured [`e5`].* 4-ary depth-6 tree, 30% stored, 300 leaf queries; the true nearest key was always admitted.
  Data-placed landmarks admit 19/5.1/1.5/0.35% of keys at m = 8/32/128/512 in H⁸, vs 74/47/26/11% for projections in
  R⁶⁴. Random landmarks admit ≥58% in both.
- *Falsifier.* On real LM keys the admitted fraction at m = 32 stays above 30% (expected today: thin shell; test after (e)).

**(e) Hyperbolic JEPA.**
- *Target and predictor.* An EMA copy of the key path (θ̄ ← 0.996θ̄ + 0.004θ) encodes tokens t+1..t+K; their
  Lorentzian centroid μ_t^K (stop-gradient, K ∈ {4, 16, 64}) is predicted from the state by P_K(h_t) → ŷ ∈ H^r.
- *Loss.* λ·Σ_K E[−⟨ŷ, sg μ⟩_L − 1] = E[cosh d − 1], with λ = 0.1. This is smooth, ≈ d²/2, avoiding arcosh's singular
  gradient at 0 [L 1804.03329] and the unsquared-distance failure in LLM-JEPA's ablation (2.2% accuracy vs 70.6% for
  MSE) [L 2509.14252].
- *Collapse prevention.* EMA with stop-gradient; the next-token loss as anchor; VICReg-style hinges on the
  per-coordinate s.d. of log₀ŷ and on the batch s.d. of target radii.
- *Expected (H).* Centroids of incoherent futures sit at small radius, so keys learn radius = scope generality:
  within-token radius–depth ρ from 0.08–0.24 to ≥0.4, better enclosing-scope copies, average NLL within ±0.01 (oracle
  bound). Multi-token prediction hurts small code models but helps induction formation below ~30M parameters
  [L 2404.19737]. Serving cost: zero.
- *Falsifier.* D8, 2 seeds × 2,000 steps: ρ fails to rise by 0.1, or enclosing-scope NLL fails to improve by 0.02
  while total NLL worsens by more than 0.01.

**(f) CRT/prime/Galois addressing [M `e8`].**
- *Measured.* 1,501,418 distinct trigram keys into 2²⁰ buckets (uniform expectation 1,074,912 colliding pairs): naive
  mod 2²⁰ 248,481,136 (max load 3,010); prime field 2⁶¹−1 1,075,638; multiply-shift 1,074,066; simple tabulation
  1,073,938 (max load 10); CRT pair 1021×1031 1,072,207 (expected 1,070,751).
- *Literature.* Prime-field polynomials are the textbook k-independent family. Simple tabulation (T₁[x₁]⊕…⊕T_c[x_c])
  gives Chernoff bounds, O(1/ε²) linear probing and working cuckoo hashing at the speed of one multiplication
  [L 1011.5200]; its table reads and XOR fit D0-b.
- *Where CRT genuinely wins.* Structured integer keys: coprime rings give collision-free joint addresses for every
  W ≤ p₁p₂ (verified; random addressing at the same load gives 526,637 colliding pairs). That makes exact multi-scale
  positional ring buffers.
- *Semiprime pair keys.* A sum of random 64-bit codes gives the same commutative identity without big integers.

## 6. Addressing a large parameter memory

**Product keys [L 1907.05242].** N = |C|² slots. Scores add over the two halves, so the exact top-k lies in the k×k
product of the per-half top-k lists: cost O((√N + k²)·d_q). A 12-layer model with one memory beat 24 layers; usage
needed query BatchNorm (25.8% → 80.3% of 1M slots). Memory+ with 1M values lands between dense models with 2× and 4×
the compute, with the largest gains on factual QA, and is bandwidth-bound [L 2412.09764].

**Geometric sub-codebooks [D].**
- **Sign halves.** The sub-keys are all 2^b sign patterns of a b-dim half-query, scored
  s(σ) = ‖q_h‖₁ − 2Σ_{i∈F}|q_{h,i}| over the flipped set F. The exact top-k patterns are the k smallest subset sums of
  |q_h|, enumerable in O(b log b + k log k) with no sub-key scoring. At N = 2²⁰ (b = 10), a 20-dim projection plus a heap
  replaces 2·1024·d_q/2 multiply-adds. The price: cells are orthants of 10 learned hyperplanes rather than 1,024 free
  centres.
- **E8 halves.** 240 roots per 8 dims (7.9 bits), with minimum angle 60° against 41.4° for 8-bit sign patterns: Voronoi
  margin 30° vs 20.7°, a 1.41× noise margin at equal bits. Scoring all 240 roots needs only additions and a halving (entries 0, ±1, ±½),
  and top-1 comes from the Construction-A decoder. Cycle 1 measured E8 ahead of sign bits at equal bits in trained
  routing (69.6% vs 62.3%).
- **Fixed symmetric codebooks** cannot collapse; the learned query map must spread its queries (BatchNorm/qk-norm).
- **Exact Hamming kNN at larger radii.** Multi-index hashing splits q-bit codes into m ≈ q/log₂n substrings. Any
  r-neighbour matches some substring within ⌊r/m⌋, so search is exact and sublinear, >100× faster than a linear scan
  on 1B codes [L 1307.2982].
- **Hyperbolic radius+direction codes (H).** Usage becomes power-law (in-degree ∝ e^{−ζr/2} [L 1006.5169]): general
  slots near the origin serve many queries, matching Zipfian knowledge access instead of forcing uniform usage.

**Verdict.** This is where the owner's Hamming/OSPF and E8 ideas give a genuine, derivable advantage: exact top-k with
additions only and no key parameters. The open, cheap question is whether the constrained cells cost quality against
learned product keys.

## 7. Ranked shortlist (most likely to give a measurable LM or chat advantage)

1. **Product-key parameter memory with E8/sign sub-codebooks.** Literature lever: 2–4× compute-equivalent, largest on
   factual QA; the geometric codebooks make exact top-k additions-only.
   - *Decisive test (M1, then here at 1/10 scale):* D8 plus one memory layer (2¹⁶–2²⁰ slots, value width 256). Arms:
     no memory, learned product keys, sign halves, E8 halves, at equal active parameters.
   - *Measure:* TinyStories NLL, slot usage and entity-recall probes. Kill if E8/sign trail learned keys by >0.02 nats.
2. **Long-context hyperbolic multi-scale memory with Busemann admission.** Context is the biggest measured lever (0.40
   nats in the oracle, 128 → 2,048 tokens); hyperbolic geometry adds an exact index and hierarchy-aware ranking.
   - *Decisive test:* first the synthetic gate of §5a (here); then D8 at context 1–2k on code.
   - *Measure:* NLL on enclosing- and other-scope copy targets and the admitted fraction.
3. **Lorentz heads with free radii plus a radius objective** (fresh, or converted from an approved open transformer by
   norm completion). This is the only mechanism with a consistent, event-localized LM gain so far (−0.06 nats/token on
   enclosing-scope copies).
   - *Decisive test:* per-event evaluation of §3c on the owner's TinyStories run, then §5e with and without the
     objective, 2 seeds.
   - *Kill criterion:* enclosing-scope copy NLL unchanged.

## 8. Recommended next experiments (ranked)

1. [here, 15 min] §5a synthetic gate: two-level hyperbolic memory with Busemann admission at 4k–16k candidates.
2. [here, per-event rerun] Apply `e3_analyze.py` to every future geometry comparison. Mean NLL hides the effect.
3. [M1] Memory layer with E8/sign vs learned product keys (§7.1).
4. [here, 4 × 1.1 h, needs a training-crate change] Hyperbolic JEPA vs none in D8 (§5e).
5. [here, 30 min] `e11`/`e11b` with 3 seeds and a KL-to-teacher penalty during QC annealing: the conversion recipe
   for open-transformer heads (keep raw accuracy, gain non-root accuracy).

## 9. Open questions for the owner

- Should a model be judged by mean NLL, or also by event-level metrics (scope/entity resolution, long-memory recall)?
  The hyperbolic mechanisms move the second and barely the first.
- A memory layer is sparse *parameter* access selected by geometric codes, not a learned expert gate. Is this allowed
  under the MoE restriction (D5 tension)?
- With weight transfer approved: how much teacher-pattern change (KL budget per head) may be spent to buy hierarchy?
  Exact conversion is lossless but gains nothing; annealed curvature gains on hierarchies at a measurable pattern cost
  (§4.3). Narrower hyperbolic students are cheaper but lossier on non-hierarchical heads.
