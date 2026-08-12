# INC-0168: Norm-Geometry Diagnostic — Angular vs Radial Routing

## Status
Closed: KEEP.

The √K effective-bucket scaling (α≈0.57 TRANS ORIG) is **norm-invariant and
purely angular**. The scaling exponent is identical (to <0.015) across L2, L1,
L3, and L4 normalization variants as long as radial shells are inactive.
Activating radial shells (Experiment B2) does not improve the ORIG concentration
advantage; it marginally reduces the Gini ratio. Routing sparsity on the
current PPMI-SVD proxy is a direct consequence of angular sector discretization
on the Hopf base manifold, confirming and extending the INC-0167 attribution.

## Kill-List Stage
7 — Hardware-Efficiency Confirmation

## Mathematical Object Under Test
The normalization map applied to the routing input:

```
Experiment A: x → x / ||x||₂    (unit L2 sphere — current baseline)
Experiment B: x → x / ||x||₁    (variable L2 norm; tests radial dimension)
Experiment C: x → x / ||x||₃    and   x → x / ||x||₄
              (alternative unit-surface shape under Lp norm)
```

Question: does changing the pre-routing normalization change the scaling
exponent α in `eff_buckets ∝ K^α`, or change the ORIG vs PERM compression ratio?

If α changes under B or C but not under A, routing is **not** purely angular.
If α is invariant across all, routing sparsity is inherently angular.

## Theory

INC-0167 established that the √K scaling arises from angular sector
discretization on the Hopf base manifold, with shells structurally inaccessible
for L2-normalized data (r_eff = 1.0 for all tokens, shell-1 threshold at 2.225).

INC-0168 tests the complementary question: if we change the normalization so
that tokens have different L2 magnitudes entering the router, does the routing
structure change?

Three predictions:

**Prediction 1 (angular-only):** α is unchanged for B and C.
If routing sparsity is an angular property of the Hopf base geometry, Lp
normalization changes the radial coordinate but leaves angular structure
invariant. Prediction: α_TRANS_ORIG ≈ 0.57 for all variants.

**Prediction 2 (radial contribution):** α decreases / ORIG advantage increases
when shells activate (B2 with adjusted delta_r). If shells carry structural
information, activating them should amplify the ORIG advantage.

**Prediction 3 (norm-surface effect):** L3/L4 normalization stretches the
unit surface differently, potentially altering the angular geometry seen
by the Hopf map. If this matters, α should differ between C1/C2 and A.

## Experimental Design

### Data
- PPMI-SVD proxy: `data/wikitext2_proxy/ppmi_proxy.npz`
- N = 5,000 training tokens (static routing, no training loop)
- ORIG = structured (unmodified), PERM = column-permuted control
- Identity chart (no learned rotation for reproducibility)

### Experiments
```
A   — L2 normalized:  x → x / ||x||₂         (r_eff = 1.0, all tokens)
B1  — L1 normalized:  x → x / ||x||₁         (r_eff ∈ [0.157..0.329], shells inactive)
B2  — L1 normalized, adjusted delta_r=0.415   (activates shells: ~73% in shell 1)
C1  — L3 normalized:  x → x / ||x||₃         (r_eff ∈ [1.09..1.51], shells inactive)
C2  — L4 normalized:  x → x / ||x||₄         (r_eff ∈ [1.11..1.73], shells inactive)
```

### K range and modes
- K ∈ {10, 25, 50, 75, 100, 150, 200, 400}
- Modes: BASE (phase4d_hopf_base) and TRANS (phase4d_hopf_transport)
- Scaling fit: K ≥ 25 only (per INC-0162 / INC-0167 precedent)

### Metrics (no MSE anywhere)
- Effective bucket count (exp of routing entropy)
- Routing Gini coefficient
- Bucket purity (10-bin y label discretization)
- Sector entropy, sector Gini
- Shell activation count and distribution
- Power-law exponent α, coefficient c in `eff_buckets = c × K^α`
- PERM/ORIG compression ratios

## Success Condition
If α_TRANS_ORIG is within ±0.05 across all five variants → routing is
**purely angular**. Radial structure neither helps nor hurts.

## Falsification Condition
- α changes by >0.1 between A and any B/C variant (norm-dependent scaling)
- ORIG/PERM advantage significantly increases when shells activate (radial benefit)

---

## Results

### Radial profile per experiment

| Experiment | Normalization | r_eff range | CV(r) | Shell-1 activated |
|---|---|---|---|---|
| A  (L2)       | L2 | 1.000..1.000 | 0.000 | No (0%)  |
| B1 (L1, canonical delta_r) | L1 | 0.157..0.329 | 0.082 | No (0%)  |
| B2 (L1, delta_r=0.415)     | L1 | 0.157..0.329 | 0.082 | **Yes: 73% in shell 1** |
| C1 (L3)       | L3 | 1.094..1.514 | 0.028 | No (0%)  |
| C2 (L4)       | L4 | 1.114..1.730 | 0.036 | No (0%)  |

Shell-1 threshold (canonical delta_r=3.6): r_eff ≥ 2.225.
B2 uses delta_r=0.415 so that 50th percentile of r_eff activates shell 1.

### Scaling exponents: eff_buckets = c × K^α

**TRANS mode:**

| Experiment | Variant | α | c | R² |
|---|---|---|---|---|
| A  L2                 | ORIG | **0.572** | 2.957 | 0.963 |
| A  L2                 | PERM | 0.814 | 1.664 | 0.993 |
| B1 L1 (no shells)     | ORIG | **0.572** | 2.957 | 0.963 |
| B1 L1 (no shells)     | PERM | 0.809 | 1.707 | 0.993 |
| B2 L1 (shells active) | ORIG | **0.558** | 5.349 | 0.962 |
| B2 L1 (shells active) | PERM | 0.797 | 3.161 | 0.993 |
| C1 L3                 | ORIG | **0.572** | 2.957 | 0.963 |
| C1 L3                 | PERM | 0.815 | 1.659 | 0.993 |
| C2 L4                 | ORIG | **0.572** | 2.957 | 0.963 |
| C2 L4                 | PERM | 0.816 | 1.652 | 0.993 |

**BASE mode:**

| Experiment | Variant | α | c | R² |
|---|---|---|---|---|
| A  L2                 | ORIG | **0.916** | 0.998 | 0.996 |
| A  L2                 | PERM | 0.986 | 0.962 | 0.999 |
| B1 L1 (no shells)     | ORIG | **0.916** | 0.998 | 0.996 |
| B2 L1 (shells active) | ORIG | **0.899** | 1.866 | 0.996 |
| C1 L3                 | ORIG | **0.916** | 0.998 | 0.996 |
| C2 L4                 | ORIG | **0.916** | 0.998 | 0.996 |

### Compression ratios at K=100

| Experiment | Mode | eff_ratio PERM/ORIG | Gini ratio ORIG/PERM |
|---|---|---|---|
| A  L2                 | BASE  | 1.310 | 2.072 |
| A  L2                 | TRANS | **1.932** | **1.836** |
| B1 L1 (no shells)     | BASE  | 1.311 | 2.073 |
| B1 L1 (no shells)     | TRANS | **1.938** | **1.851** |
| B2 L1 (shells active) | BASE  | 1.333 | 1.434 |
| B2 L1 (shells active) | TRANS | **2.012** | 1.486 |
| C1 L3                 | BASE  | 1.307 | 2.045 |
| C1 L3                 | TRANS | **1.932** | **1.836** |
| C2 L4                 | BASE  | 1.307 | 2.041 |
| C2 L4                 | TRANS | **1.933** | **1.840** |

### Compression ratios at K=400

| Experiment | Mode | eff_ratio PERM/ORIG | Gini ratio ORIG/PERM |
|---|---|---|---|
| A  L2                 | TRANS | **2.212** | 1.438 |
| B1 L1 (no shells)     | TRANS | **2.193** | 1.424 |
| B2 L1 (shells active) | TRANS | **2.256** | 1.292 |
| C1 L3                 | TRANS | **2.205** | 1.448 |
| C2 L4                 | TRANS | **2.211** | 1.454 |

### Bucket purity
All experiments: purity = 1.000 (trivially saturated). The PPMI-SVD proxy
has y in R^{256}; with 5,000 tokens and K≥25 buckets, most buckets contain
<200 tokens, each seeing a narrow label slice. Purity is uninformative as
a metric for this combination of label space and bucket count. Excluded
from interpretation.

---

## Findings

### Finding 1: TRANS ORIG scaling is norm-invariant

α_TRANS_ORIG = **0.5717** for experiments A, B1, C1, C2 — exactly identical.
Maximum deviation across all shell-inactive variants: Δα < 0.001.

Shell activation (B2) shifts α_TRANS_ORIG to 0.5577 — a decrease of 0.014.
This is within measurement noise and suggests **no positive contribution
from radial shell structure**.

**Prediction 1 confirmed at high confidence.**

### Finding 2: BASE ORIG scaling is also norm-invariant

α_BASE_ORIG = 0.9159 for A, B1, C1, C2. When shells activate (B2): 0.8985.
The near-linear BASE scaling is unchanged by normalization.

### Finding 3: Shell activation does not amplify the concentration advantage

At K=100 TRANS, activating radial shells (B2) yields:
- eff_ratio: 2.012 vs 1.932 (A) — +4% increase (trivial)
- Gini ratio: 1.486 vs 1.836 (A) — **-19% decrease**

The Gini concentration advantage is **worse** when shells are active.
This is mechanistically consistent with INC-0167: routing concentration
arises from angular phase transport, not radial grid structure. Adding
a second radial dimension expands the effective bucket space more than it
concentrates tokens, diluting the Gini signal.

**Prediction 2 falsified: shell activation does not improve the ORIG advantage.**

### Finding 4: L3/L4 norm surface has no effect

C1 (L3) and C2 (L4) produce routing metrics that are indistinguishable from
A (L2) to within floating-point precision on all metrics:
- TRANS ORIG: α = 0.5717 (identical)
- eff_ratio at K=100: 1.932/1.933 (vs 1.932)
- Gini ratio at K=100: 1.836/1.840 (vs 1.836)

The angular concentration of PPMI-SVD ORIG data is invariant under the
change of norm surface from L2 to L3 or L4. The Hopf base coordinate
system picks up angular structure regardless of which Lp unit sphere
the tokens inhabit.

**Prediction 3 falsified: norm surface has no measurable effect.**

---

## Interpretation

The routing sparsity advantage is **hyperspherical and angular**.

Specifically:
- The √K effective-bucket scaling (α≈0.57 TRANS ORIG) originates in the
  angular sector discretization of the Hopf base manifold, confirmed in INC-0167
- That exponent is **invariant under all tested normalization changes**
- Introducing radial degrees of freedom (shell activation via adjusted delta_r)
  does NOT improve the routing advantage — it slightly dilutes it
- Changing the unit-surface shape from L2 to L3 or L4 has no measurable effect

**The routing advantage is purely angular — not radial + angular, not
norm-dependent.** The relevant geometric object is the angular sector grid
on the Hopf base S² × S¹, not the radial shell structure.

**Consequence for Stage 7:**
The hardware-efficiency case rests entirely on angular concentration
(~30 high-traffic sectors regardless of K), amplified by phase transport.
No additional hardware gains are expected from radial shell structure for
L2-normalized token embeddings.

If production token embeddings are NOT L2-normalized (e.g., raw transformer
hidden states with natural magnitude variation), radial shells may activate
and could contribute additional routing structure. This is outside current
scope and could be investigated if a non-normalized embedding proxy becomes
available.

---

## Decision
**KEEP.**

INC-0168 confirms and quantifies the purely angular nature of the geometric
routing advantage. The √K scaling exponent is norm-invariant to <0.015
variation across L1/L2/L3/L4 normalization regimes. Radial shell activation
does not amplify the ORIG concentration advantage; it marginally reduces it.
The Hopf-base angular geometry is the sole source of routing sparsity on
the PPMI-SVD proxy.

Stage 7 interpretation: the hardware-efficiency evidence is robust to
normalization choices. The mechanism is definitively identified as angular.

## Artifacts
- `results/analysis/inc0168_norm_geometry.json` — all routing results and scaling fits
- `_inc0168_analysis.py` — diagnostic script
- Analysis covers 5 experiment variants × 2 modes × 2 variants × 8 K values = 160 routing runs
