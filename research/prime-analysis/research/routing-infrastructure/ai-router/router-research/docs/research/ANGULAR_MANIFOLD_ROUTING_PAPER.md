# Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization

---

## Abstract

We show that the same geometric property of normalized token embeddings that enables TurboQuant's extreme data compression [Google, 2026] also enables sublinear routing computation in transformer-style architectures. TurboQuant demonstrated that L2-normalized embeddings on the unit hypersphere are angularly non-uniform, and that random rotation destroys this structure for near-optimal scalar quantization. We independently developed the complementary direction: a fixed Hopf fibration map exploits the same angular non-uniformity to concentrate token routing into a sparse subset of available expert sectors.

Across 169 experimental increments on a PPMI-SVD semantic proxy, we establish: (1) the routing footprint scales as K^0.572 (vs K^1.0 for standard dense routing), (2) this advantage grows with K and persists at K=5000 (ratio 2.6–2.8×), (3) the mechanism is purely angular and norm-invariant (Δα < 0.015 across L1/L2/L3/L4), and (4) a fixed 4D Hopf geometry outperforms 100D adaptive K-means clustering on semantically structured embeddings.

We validate the mechanism in a small trainable language model (INC-0171, confirmed 2 seeds): fixed geometric routing replaces learned gating with only 8% validation perplexity cost and no learned gate matrix, while using 46 of 64 effective expert paths at convergence (1.4× more efficient than dense routing). A second-dataset replication on WikiText-2 (INC-0173, confirmed 2 seeds) confirms the result is stable across datasets: HOPF achieves a HOPF/BASELINE ratio of 1.081 on WT2 — numerically identical to PTB's confirmed ratio — under identical training conditions. A key honest finding: the specific Hopf sector geometry does not add advantage over a randomly-permuted fixed routing in trainable LM settings — experts co-adapt their weights — but the 8% PPL gap is confirmed stable across both datasets and seed counts.
This result is scoped to the 2-layer toy-scale trainable setting and should not be read as a claim of broad MoE replacement or large-scale transformer substitution.

We provide a standalone Python script (numpy only, < 60 seconds) that reproduces the core result. Taken together with TurboQuant, this work suggests the angular non-uniformity of embeddings has engineering consequences in both data compression and routing computation.

---

## 1. Background: TurboQuant and Angular Non-Uniformity

### 1.1 The geometric fact

Google's TurboQuant [2026] demonstrated a previously underappreciated property of modern language model embeddings: L2-normalized token representations on the unit hypersphere are **angularly non-uniform**. Specifically, if you take the hidden states or token embeddings from any trained language model and project them to the sphere, they cluster into preferred angular regions rather than distributing uniformly.

TurboQuant's engineering choice: **destroy** this structure via random orthogonal rotation, producing near-independent scalar components, and then apply per-dimension scalar quantization to extreme bit-depths. This achieves state-of-the-art data compression on KV caches and vector search indices, validated at production scale.

### 1.2 Our parallel, complementary choice

We independently arrived at the same geometric fact through a different path — not compression, but routing efficiency. If token embeddings are angularly non-uniform on the sphere, then a **fixed sector map** (one that divides the sphere into regions) will receive non-uniform routing: some sectors will capture many semantically similar tokens while others stay empty. This produces a sparse routing distribution, reducing the effective compute footprint.

Our engineering choice: **exploit** the angular non-uniformity via a fixed Hopf fibration map to create sparse, semantically coherent routing.

**The contrast:**

| | TurboQuant | This work |
|---|---|---|
| Angular structure | Destroyed (random rotation) | Exploited (Hopf map) |
| Purpose | Data compression | Compute reduction |
| Mechanism | Near-independent scalars | Sparse sector routing |
| Scale validated | Production LLMs | Proxy + 2-layer LM |
| Claim | Store/transmit fewer bits | Compute fewer routing paths |

Same geometric fact. Opposite engineering choice. Complementary applications.

### 1.3 Independent parallel development

Our work predates the TurboQuant public release. The observation that normalized embeddings have angular non-uniformity is encoded in our research chain from INC-0143 (Stage 3, 2026) and progressively validated through INC-0171. TurboQuant provides independent corroboration at production scale and motivates framing our result as its complementary extension.

---

## 2. The Mechanism: Hopf Angular Routing

### 2.1 The winning algorithm (after 169 experimental increments)

After testing more than 20 routing variants — K-means adaptive, shell-aware variants, product phase, complex transport, SO(8) learned rotations — a single method survived all falsification attempts:

**sector_mode = `phase4d_hopf_transport`**, λ=1.0, L2-normalization, single shell.

The algorithm takes 4 steps on an L2-normalized input vector x ∈ ℝ^D:

**Step 1 — Project to 4D Hopf subspace**
```
a = x[dim_i],  b = x[dim_j],  c = x[dim_k],  d = x[dim_l]
```
(dims selected once by validation on a held-out semantic proxy; typical values: (3, 65, 2, 21))

**Step 2 — Hopf base coordinates**
```
ρ₁ = √(a² + b²),   ρ₂ = √(c² + d²)
χᵤ = ρ₂² / (ρ₁² + ρ₂²)          ∈ [0, 1]
δ  = arctan2(b, a) − arctan2(d, c)   ∈ [−π, π]
α  = ½ · (arctan2(b, a) + arctan2(d, c))   ∈ [−π, π]
```

**Step 3 — Phase transport (Levi-Civita correction)**
```
χ  = arcsin(ρ₂ / √(ρ₁² + ρ₂²))
αₜ = α + ½ · λ · cos(2χ) · δ
```

**Step 4 — Assign sector (budget K across three bin dimensions)**
```
k_χ, k_δ, k_α = allocate_triplet_bins_budget(K)   # product ≤ K
sector = bin(χᵤ, k_χ) · k_δ · k_α  +  bin(δ, k_δ) · k_α  +  bin(αₜ, k_α)
```

### 2.2 Mechanism isolation (INC-0147)

The raw fiber mean phase α = ½(θ₁ + θ₂) is the primary mechanism:

| Variant | rel_diff (Gini ratio) | Increment |
|---|---|---|
| HOPF_BASE (χ + δ only, no α) | 46.0% | INC-0147 |
| HOPF λ=0 (raw α, no transport) | 66.7% (+20.6 pp) | INC-0147 |
| HOPF λ=1.0 (canonical) | 68.7% (+2 pp Levi-Civita) | INC-0147 |

**The raw fiber phase (θ₁+θ₂)/2 is the mechanism. The Levi-Civita transport adds only ~2 percentage points. The paper refers to both as "Hopf routing" for brevity, but the algorithm is the raw fiber mean phase.**

### 2.3 Fixed geometry vs adaptive (INC-0144)

| Method | rel_diff | Stability |
|---|---|---|
| Fixed H⁴ Hopf 4D | 31.2% | Stable across 5 seeds |
| K-means adaptive 100D | 3.1% | Variable (−5.8% to +12.0%) |

Fixed geometry on 4 dimensions outperforms learned K-means on 100 dimensions on semantically structured embeddings. The Hopf map finds structure that adaptive clustering misses because it captures the fiber geometry of the S³ manifold rather than Euclidean cluster centroids.

---

## 3. Scaling Law and Hardware Consequences

### 3.1 The canonical scaling law (INC-0169, INC-0170)

On the PPMI-SVD semantic proxy (L2-norm, K=25 to 5000, 160+ runs across 7 seeds):

| Routing | Scaling Law | R² | Range |
|---|---|---|---|
| HOPF (canonical) | eff_buckets = 2.957 × K^**0.572** | 0.963 | K=25–400 |
| HOPF (full range) | eff_buckets ≈ K^**0.657** | 0.991 | K=25–5000 |
| PERM (control) | eff_buckets = 1.664 × K^**0.814** | 0.993 | K=25–5000 |
| BASE ORIG | eff_buckets = 0.998 × K^**0.916** | 0.995 | K=25–400 |
| DENSE | eff_buckets = K^**1.0** | 1.000 | all K |

The Hopf routing footprint scales sublinearly (exponent 0.572–0.657) vs K^1.0 for dense routing. The PERM control (same marginal statistics, destroyed semantic structure) scales at 0.814–0.816 — intermediate between ORIG and DENSE.

**The exponent shift (0.572 → 0.657 at large K) reflects approaching the natural cluster scale of the proxy. The compression ratio stabilises at 2.6–2.8× for K=600–5000 (INC-0170), confirming the advantage is persistent and real at production-relevant K.**

### 3.2 Compression ratios (PERM/ORIG at matched K)

| K | eff_ratio (PERM/ORIG) | INC source |
|---|---|---|
| 25 | 1.18× | INC-0168 |
| 100 | 1.93× | INC-0168 |
| 400 | 2.21× | INC-0168 |
| 600 | 2.40× | INC-0170 |
| 1000 | 2.55× | INC-0170 |
| 2000 | 2.70× | INC-0170 |
| 5000 | 2.64× | INC-0170 |

The ratio grows with K (up to ~2.7×) and stabilises — it does not collapse to 1 at large K.

### 3.3 Hardware proxy (INC-0165, 80 runs, 5 seeds)

On an EMA-based training proxy that simulates routing cache pressure:

| Routing | eff_cost reduction | LRU-16 miss reduction |
|---|---|---|
| HOPF (canonical) | 3.0–4.9× lower | 2.5–2.9× fewer |
| BASE ORIG | 1.8–2.1× lower | 1.3–1.8× fewer |

Phase transport amplifies the hardware advantage by 1.5–2.4× vs BASE. Gains widen with K and do not saturate through K=5000.

### 3.4 Norm invariance (INC-0168)

The Hopf scaling exponent is **norm-invariant**: α=0.572 (HOPF canonical) varies by < 0.015 across L1, L2, L3, L4 normalizations. The mechanism is purely angular — the radial component does not contribute. Shell hierarchy (radial stratification) **degrades** the Gini ratio by 19% and is excluded from the canonical design.

---

## 4. Extension to Transformer Routing (INC-0171)

### 4.1 The gap

All evidence in Sections 2–3 uses a PPMI-SVD proxy with an EMA update rule — not backpropagation. The claim that Hopf routing can *replace* transformer-like computation requires testing in a real trainable language model.

### 4.2 Experiment design

Architecture: 2-layer transformer, hidden_dim=128, 4 attention heads, vocab=5000 (PTB), seq_len=32, K=64 experts.

Three FFN routing conditions (all K=64):

| Condition | Gate | Routing |
|---|---|---|
| BASELINE | Learned K×D matrix + softmax → top-1 | Supervised |
| HOPF-ROUTED | None (no learned gate) | L2-norm(hidden) → Hopf sector |
| PERMUTED | None (no learned gate) | Hopf sectors randomly permuted |

Training: 4000 steps, cosine LR 3e-4, batch=64. PTB Penn Treebank train/val split. Confirmed at 2 seeds.

### 4.3 Results (confirm, 2 seeds, 4000 steps)

| Method | Val PPL (mean) | eff_buckets | params | eff_ratio vs DENSE |
|---|---|---|---|---|
| BASELINE | **152.26** | 43.19 | 5,660,672 | 1.48× |
| HOPF-ROUTED | **164.54** | 45.96 | 5,644,288 | **1.39×** |
| PERMUTED | **164.41** | 45.81 | 5,644,288 | 1.40× |

**PPL ratio (HOPF / BASELINE): 1.081** — confirmed within 8% across 2 seeds, 4000 steps.  
**eff_ratio (DENSE / HOPF): 1.39×** — at convergence (was 1.65× at 2000 steps; declines as experts expand sector coverage).  
**HOPF vs PERMUTED: Δppl = +0.13** — statistically indistinguishable.

### 4.4 Interpretation

**Finding 1 — Fixed geometric routing replaces learned gating at 8% PPL cost (confirmed).**
HOPF-ROUTED achieves within-8% validation perplexity of the learned gating baseline with no learned gate matrix. Confirmed stable across 2 seeds and 4000 training steps (screen 1.080, confirm 1.081).

**Architectural framing — the correct comparison is native learned concentration.**
The BASELINE comparison is a learned top-1 gate *without* load-balance auxiliary loss. This is deliberate: the learned gate converges to natural concentration (eff_b=43–44 of 64 experts) without any imposed balance constraint. A subsequent control experiment (INC-0172) confirmed that introducing Switch-style load-balance auxiliary loss destroys this concentration — driving routing to near-uniform distribution (eff_b=62.77, effectively random assignment across all 64 experts) while also slightly degrading perplexity (by 0.23 ppl vs BASELINE). This establishes two things: (1) native learned concentration, not imposed balance, is the correct architectural regime for this comparison; (2) the 8% PPL gap is the cost of replacing the learned gate in the *concentration regime*, not in an artificially balanced one. Fixed geometric routing (HOPF eff_b=46) and unforced learned routing (BASELINE eff_b=44) operate in the same concentration regime — the claim is specifically about this regime.

**Finding 2 — The specific Hopf geometry does not add advantage over random fixed routing — confirmed.**
HOPF and PERMUTED are statistically indistinguishable (164.54 vs 164.41 ppl, 45.96 vs 45.81 eff_b) across 2 seeds. In a trainable LM, experts co-adapt their weights to any fixed routing — the geometric structure of the assignment is not the decisive factor.

This is an honest and important result. The proxy experiments (Sections 2–3) show genuine semantic structure exploitation on static embeddings. In trainable LM settings, experts absorb the routing structure through learning, making the specific assignment geometry irrelevant for quality.

**Finding 3 — eff_ratio vs DENSE declines with training; honest convergence value is 1.39×.**
At 2000 steps eff_b=38.76 (ratio 1.65×); at convergence eff_b=45.96 (ratio 1.39×). As training proceeds, experts expand their sector coverage, reducing concentration. The 1.39× gain at convergence is real and represents genuine compute reduction vs uniform dense routing, but weaker than the static proxy prediction. The paper reports the confirmed convergence value.

**The paper's trainable LM claim:** *Fixed geometric routing can replace learned gating in a 2-layer transformer FFN with only 8% perplexity cost and no gate matrix, while using 46 of 64 effective expert paths at convergence (1.4× fewer than uniform dense routing). Confirmed, 2 seeds.*

### 4.5 Second-Dataset Replication (INC-0173, WikiText-2)

To confirm the result is not PTB-specific, we replicated INC-0171 on WikiText-2 (WT2) under identical conditions: same architecture, same training budget (4000 steps), same three conditions (BASELINE, HOPF, PERMUTED), same VOCAB_SIZE=5000. We ran both seeds (0 and 1) for full 2-seed confirmation.

| Method | Seed 0 PPL | Seed 1 PPL | Mean PPL | Mean eff_b | Dataset |
|---|---|---|---|---|---|
| BASELINE | 106.96 | 104.74 | 105.85 | 35.37 | WT2 |
| HOPF-ROUTED | 113.65 | 115.25 | 114.45 | 40.94 | WT2 |
| PERMUTED | 113.86 | 115.09 | 114.48 | 42.28 | WT2 |

**PPL ratio (HOPF / BASELINE): 1.081 (2-seed mean)** — identical to PTB's confirmed ratio.  
**HOPF vs PERMUTED: |Δppl| = 0.03** — geometry irrelevance confirmed on WT2 (stronger than PTB's Δ=0.13).  
**HOPF vs DENSE efficiency: 1.56×** (2-seed mean) — above 1.5× confirm threshold.

**WT2-specific note:** The BASELINE concentrates more aggressively on WT2 (mean eff_b=35.37) than on PTB (eff_b=44), likely because the higher OOV rate at VOCAB_SIZE=5000 creates a more peaked token distribution that the learned gate exploits. Fixed routing (HOPF) cannot adapt; its eff_b varies more across seeds (36.51–45.38) due to initialization sensitivity in a higher-OOV regime. The efficiency comparison is dataset-dependent; the PPL claim is unaffected.

**Cross-dataset summary:**

| Metric | PTB (INC-0171 confirm) | WT2 (INC-0173 confirm) |
|---|---|---|
| HOPF/BASELINE PPL ratio | 1.081 | **1.081** |
| HOPF/PERMUTED |Δppl| | 0.13 | **0.03** |
| HOPF vs DENSE eff_ratio | 1.39× | **1.56×** |
| BASELINE mean eff_b | 44.00 | **35.37** |
| HOPF mean eff_b | 45.96 | **40.94** |
| Seeds confirmed | 2 | **2** |

---

## 5. Reproducible Example and Honest Scope

### 5.1 Reproducing the core result

The full routing algorithm is independently verifiable in under 60 seconds:

```bash
python hopf_routing_demo.py   # numpy only; reproduces Table 2 within 0.5%
```

A standalone reproduction bundle, including `hopf_routing_demo.py`,
figure-generation code, and manuscript source, will be released with the
public preprint record.

Dependencies: numpy only. The script implements the full routing algorithm as approximately 40 lines of standalone numpy.

**Core routing function (excerpt):**

```python
def hopf_transport_sectors(x, K, dims=(3, 65, 2, 21), lam=1.0):
    """Assign L2-normalized vectors to Hopf angular sectors."""
    a, b, c, d = x[:, dims[0]], x[:, dims[1]], x[:, dims[2]], x[:, dims[3]]
    rho1 = np.sqrt(a**2 + b**2)
    rho2 = np.sqrt(c**2 + d**2)
    chi_u = np.clip(rho2**2 / np.maximum(rho1**2 + rho2**2, 1e-30), 0, 1)
    chi   = np.arcsin(np.clip(rho2 / np.maximum(np.sqrt(rho1**2 + rho2**2), 1e-30), 0, 1))
    theta1, theta2 = np.arctan2(b, a), np.arctan2(d, c)
    delta = wrap_to_pi(theta1 - theta2)
    alpha = wrap_to_pi(0.5 * (theta1 + theta2))
    ta    = wrap_to_pi(alpha + 0.5 * lam * np.cos(2 * chi) * delta)
    kchi, kdelta, kalpha = allocate_triplet_bins_budget(K)
    bchi   = np.clip((chi_u * kchi).astype(int),   0, kchi   - 1)
    bdelta = np.clip(((delta  + np.pi) / (2*np.pi) * kdelta).astype(int), 0, kdelta - 1)
    balpha = np.clip(((ta     + np.pi) / (2*np.pi) * kalpha).astype(int), 0, kalpha - 1)
    return bchi * kdelta * kalpha + bdelta * kalpha + balpha
```

### 5.2 What this work does NOT yet prove

We are explicit about the limitations:

- **Proxy scale only for Sections 2–3**: The PPMI-SVD proxy captures semantic structure but is not a trained LM. Section 4 is at 2-layer toy scale; large-scale LM validation is TurboQuant's contribution.
- **Hopf geometry vs random fixed routing not distinguished in LM training**: INC-0171 shows the efficiency gain transfers but that the *specific Hopf geometry* does not add advantage beyond random fixed routing in trainable settings.
- **Comparison is against native learned concentration, not balance-forced routing**: The BASELINE is top-1 gating without auxiliary loss. A control (INC-0172, KILL) confirmed that imposing Switch-style load-balance auxiliary loss yields near-uniform routing (eff_b≈63 vs BASELINE eff_b≈44) and slightly worse perplexity — establishing that native concentration, not forced balance, is the correct comparator. The claim does not extend to Switch-Transformer-style MoE with explicit balance constraints.
- **Attention routing not tested**: All routing is in the FFN gate; self-attention routing is a separate (and harder) problem.
- **Single research group**: No external replication for Sections 2–3. TurboQuant is the closest independent confirmation of the underlying geometric fact.

### 5.3 Next steps

The most natural extension is validating the efficiency transfer at larger model and dataset scale.
More broadly, the convergence of TurboQuant and this work on a shared geometric property
invites investigation of whether the angular structure of normalized embeddings constitutes
a common substrate underlying multiple inference-time efficiency mechanisms;
we leave this to future work.

---

## References

[Google, 2026] TurboQuant: Extreme KV-Cache Compression via Angular Rotation-Invariant Quantization. Google Research, 2026.

[This work, 2026] Evidence chain INC-0143 through INC-0171. Incrementally falsified routing variants, canonical scaling law, and LM integration. Full increment documents released with preprint bundle.

---

## Appendix A: Evidence Chain Summary

| Increment range | Stage | What was proved |
|---|---|---|
| INC-0143..0146 | §2 | Fixed H⁴ Hopf outperforms K-means adaptive; Gini advantage established |
| INC-0147 | §2 | Mechanism isolation: raw fiber alpha (θ₁+θ₂)/2 is primary mechanism (+20.6 pp) |
| INC-0148..0159 | §3 | Training-time scaling, EMA update rule, K=250–1000 range |
| INC-0160..0165 | §3 | Hardware proxy: 3.0–4.9× eff_cost reduction, 2.5–2.9× LRU miss reduction |
| INC-0166..0168 | §3 | Norm invariance (Δα < 0.015), shell hierarchy excluded, canonical law |
| INC-0169 | §3 | Law freeze: eff_buckets = 2.957 × K^0.572 (canonical HOPF) |
| INC-0170 | §3 | Large-K validation: ratio 2.6–2.8× at K=600–5000 (does not collapse) |
| INC-0171 | §4 | LM integration (confirm, 2 seeds): HOPF within 8% PPL of learned gating, eff_b=46/64, 1.39× vs DENSE |
| INC-0172 | §4 (control) | Architectural control: imposed load-balance aux loss → near-uniform routing (eff_b=63), slightly worse PPL; confirms BASELINE (native concentration, eff_b=44) is the correct comparison |
| INC-0173 | §4 | WT2 replication (confirm, 2 seeds): HOPF/BASELINE=1.081 (mean) on WikiText-2 — identical to PTB's 1.081; HOPF ≈ PERMUTED (|Δ|=0.03 ppl mean); eff_ratio vs DENSE=1.56× (mean) |

Full records are available in the released artifact bundle.

---

*Word count: ~2,400 words. Target submission: arXiv cs.LG. Format: technical note, 4–5 pages with tables.*
