# INC-0169: Canonical Architecture Law Freeze and Design Implications

## Status
Closed: KEEP.

INC-0169 is a synthesis increment. No new experiments are run. All results
cited are drawn from measurements validated across INC-0162 through INC-0168.
The canonical routing law is frozen here as the primary reference for future
experiments and paper drafting. Design implications are derived directly from
the validated evidence chain.

## Kill-List Stage
7 — Hardware-Efficiency Confirmation

## Scope
This increment synthesizes the following closed increments into a canonical
statement of the routing law and its design consequences:

| INC | Content |
|---|---|
| INC-0162 | Routing compute scaling law established (α≈0.50 training, K=25–200) |
| INC-0163 | Matched-progress compute efficiency (1.7–2.4× ORIG advantage) |
| INC-0164 | Scaling-law consistency (predicted vs measured within 1–11%) |
| INC-0165 | Hardware proxy closure (eff_cost 3.0–4.9×, LRU misses 2.5–2.9×) |
| INC-0166 | Architecture law freeze and K=100 boundary audit |
| INC-0167 | Scaling mechanism attributed to angular sector discretization |
| INC-0168 | Norm-geometry diagnostic: α=0.572 norm-invariant across L1/L2/L3/L4 |

No measurements were taken in INC-0169. Every number cited below is a direct
measurement from the increment listed in the source column.

---

## Part A — Canonical Routing Law

### Section 1: Scaling Law

The effective bucket count under ORIG (semantically structured) routing follows
a power law in the number of routing heads K:

```
effective_buckets(K) = c · K^α
```

**TRANS mode (phase transport, λ=1.0) — canonical regime:**

| Quantity | Value | Source |
|---|---|---|
| α — ORIG, static routing | **0.572 ± 0.001** | INC-0168 (160 runs, norm-invariant) |
| c — ORIG, static (L2 baseline) | 2.957 | INC-0168 |
| R² of fit | 0.963 | INC-0168 |
| α — PERM, static | 0.814 ± 0.002 | INC-0168 |
| c — PERM, static | 1.664 | INC-0168 |
| α — ORIG, training K=25–200 | ≈ 0.50 | INC-0162 (5 seeds) |
| α — ORIG, training K=250–1000 | ≈ 0.64 | INC-0167 (2 seeds) |
| α — PERM, training | ≈ 0.79–0.82 | INC-0162, INC-0167 |

**BASE mode (no phase transport) — reference:**

| Quantity | Value | Source |
|---|---|---|
| α — ORIG, static | 0.916 | INC-0168 |
| α — PERM, static | 0.986 | INC-0168 |

**Notes on regime differences.**
The static routing exponent (0.572) differs from the training-time exponent
(0.50–0.64) because:

- Static routing uses an identity chart (no learned rotation). The concentration
  advantage visible in static routing reflects only the structural angular
  alignment between PPMI-SVD token distribution and the Hopf base sector grid.
- Training-time routing uses learned chart parameters. At K=25–200, learning
  achieves α≈0.50 (sharper concentration than static at low-K implied by lower
  absolute exponent); at K=250–1000, α≈0.64 (learned concentration effect
  relatively weaker at high-K).
- The PERM control has no semantic structure to align with; the PERM exponent
  is consistently higher (0.79–0.82 training, 0.814 static), reflecting more
  uniform bucket usage.

**Compression ratios from K=100 and K=1000 (TRANS ORIG):**

| K | eff_ratio (PERM/ORIG) | Gini ratio (ORIG/PERM) | Source |
|---|---|---|---|
| 100 (static) | **1.932** | **1.836** | INC-0168 |
| 100 (static, shells active) | 2.012 | 1.486 | INC-0168 |
| 400 (static) | 2.212 | 1.438 | INC-0168 |
| 250 (training) | 2.18 | — | INC-0167 |
| 1000 (training) | **2.75** | — | INC-0167 |

The effective bucket compression ratio grows with K and has not saturated at
K=1000 in training conditions.

---

### Section 2: Mechanism Summary

The routing sparsity advantage (α_ORIG < α_PERM, effective_buckets_ORIG <<
effective_buckets_PERM) arises from a single identified mechanism:

**Angular sector discretization on the Hopf base manifold.**

The Hopf fibration maps H^4 routing tokens to a coarse base address on
S² × S¹ (the Hopf base). The H^4 routing chart assigns each token to a sector
on that base. PPMI-SVD token embeddings are semantically structured: their
angular distribution on the (pre-routing) hypersphere is non-uniform, and this
non-uniformity aligns with the Hopf base sector grid. The result is that most
tokens fall into a small fraction of the available sectors.

The PERM control (column-permuted PPMI-SVD) destroys all pairwise semantic
structure while preserving marginal statistics. The permuted embeddings
distribute more uniformly over the sector grid, yielding effective_buckets_PERM
closer to K.

**Phase transport amplification.** The TRANS mode (λ=1.0 Levi-Civita
transport correction applied to the fiber coordinate) amplifies the angular
concentration effect. INC-0147 isolated the mechanism: the raw fiber alpha
angle (θ₁+θ₂)/2 is the source of the improvement; the transport correction
term (λ/2)cos(2χ)·δ provides an additional 2 percentage points of uplift on
this proxy. TRANS mode reduces effective_buckets_ORIG by 1.7–2.2× relative to
PERM at matched progress (INC-0163).

**Radial shell structure plays no role.** INC-0167 established that the shell
structure is structurally inaccessible for L2-normalized data (r_eff = 1.0 for
all tokens; shell-1 threshold: r_eff ≥ 2.225). INC-0168 confirmed that forcing
shell activation does not improve the concentration advantage. The mechanism
is entirely angular.

---

### Section 3: Norm Invariance

INC-0168 tested five normalization variants in 160 static routing runs:

| Variant | Normalization | r_eff range | α (TRANS ORIG) | Δα vs L2 |
|---|---|---|---|---|
| A — L2 (baseline) | x → x / ‖x‖₂ | 1.000 (const) | **0.572** | — |
| B1 — L1, shells off | x → x / ‖x‖₁ | 0.157–0.329 | **0.572** | < 0.001 |
| B2 — L1, shells on (~73%) | x → x / ‖x‖₁, delta_r=0.415 | 0.157–0.329 | 0.558 | −0.014 |
| C1 — L3 | x → x / ‖x‖₃ | 1.094–1.514 | **0.572** | < 0.001 |
| C2 — L4 | x → x / ‖x‖₄ | 1.114–1.730 | **0.572** | < 0.001 |

**Finding:** α_TRANS_ORIG = 0.572 is identical (within floating-point precision)
for all shell-inactive variants, regardless of which Lp unit surface the tokens
inhabit. Maximum deviation across A, B1, C1, C2: Δα < 0.001.

**Interpretation:** the Hopf base coordinate system extracts angular structure
from the input. The angular distribution of PPMI-SVD tokens on the Lp unit
hypersurface is stable under changes of norm surface. The routing mechanism is
insensitive to normalization choice in the tested regime.

L3 and L4 normalization place tokens on higher-Lp unit surfaces (stretched
toward coordinate axes relative to L2), yet the Hopf map sees the same effective
angular distribution. The routing advantage is a property of the semantic
clustering of the token directions, not of the specific metric geometry of the
norm surface.

The small decrease under B2 (shells active, −0.014) is not a norm effect — it
is a consequence of shell activation expanding the effective bucket space in a
way that dilutes angular concentration (see Section 4).

---

### Section 4: Shell Non-Necessity

**Structural inaccessibility (INC-0167).** For L2-normalized PPMI-SVD tokens:
- r_eff = 1.0 for all tokens (exactly, by construction of L2 normalization)
- Shell-1 activation threshold (phi_log mode, delta_r=3.6): r_eff ≥ 2.225
- All tokens permanently in shell 0; shell structure is structurally inaccessible

**Forced activation result (INC-0168 Experiment B2).** Adjusting delta_r to
0.415 forces 73% of L1-normalized tokens into shell 1. Measured effect at K=100
TRANS:

| Metric | Shells inactive (A, B1) | Shells active (B2) | Change |
|---|---|---|---|
| eff_ratio (PERM/ORIG) | 1.932 | 2.012 | +4.2% |
| Gini ratio (ORIG/PERM) | 1.836 | 1.486 | **−19.1%** |

The eff_ratio nominally increases (+4%), but the Gini concentration advantage
decreases significantly (−19%). The mechanistic explanation: shell activation
expands the total bucket space (more routing addresses = nominally higher
PERM/ORIG eff_ratio at fixed K), but the additional addresses are distributed
roughly uniformly between ORIG and PERM, diluting the relative Gini signal.
Net effect: the concentrating advantage of angular routing is weakened, not
strengthened, by shell activation.

**Conclusion:** for L2-normalized embeddings, a single-shell (shell=0-only)
angular router is the optimal architecture. Shell hierarchy should not appear
in the default design unless future evidence — from non-L2-normalized embeddings
with substantial radial variation — justifies its reintroduction.

---

### Section 5: Hardware Consequence Summary

The hardware-efficiency path is a direct consequence of the scaling law:

```
angular concentration (sector discretization)
  → sublinear effective bucket scaling: eff_buckets ∝ K^0.572 (ORIG)
  → PERM/ORIG compression at K=100: 1.932× (TRANS static)
  → matched-progress compute advantage: ORIG uses 1.7–2.2× fewer buckets (TRANS)
  → hardware proxy cost reduction: eff_cost 3.0–4.9× lower (TRANS)
```

Validated hardware proxy numbers (INC-0165, 80 runs, 5 seeds, matched progress
at p=0.70):

| Mode | Metric | ORIG advantage vs PERM |
|---|---|---|
| TRANS | eff_cost | **3.0–4.9×** lower |
| TRANS | LRU-16 cache misses | **2.5–2.9×** fewer |
| BASE | eff_cost | 1.8–2.1× lower |
| BASE | LRU-16 cache misses | 1.3–1.8× fewer |

Phase transport (TRANS vs BASE) amplifies all hardware proxy advantages by
approximately 1.5–2.4× relative at matched progress. This is consistent with
the scaling law: TRANS reduces α_ORIG from 0.916 (BASE) to 0.572 (TRANS),
producing sharper effective-bucket compression.

**Advantages widen with K.** From INC-0167, the PERM/ORIG effective bucket
ratio grows monotonically: 2.18× at K=250, 2.75× at K=1000 (training). The
hardware gains are not saturation-bounded in the tested range.

**Norm-independence of hardware gains.** Because the scaling exponent is
norm-invariant (Section 3), the hardware advantage is equally robust to
normalization choices. The 3.0–4.9× eff_cost reduction is a property of the
angular geometry, not of any preprocessing choice.

---

## Part B — Design Implications

### Section 6: Design Implications

**Q1. Should normalized angular routing remain the default architecture?**

**YES.** The routing advantage is purely angular. Angular sector discretization
on the Hopf base is the sole source of the observed sublinear scaling and
hardware proxy gains. All tested normalization variants produce equivalent
routing structure. The architecture is:

- Robust: norm-invariant across L1/L2/L3/L4 (INC-0168)
- Seed-stable: ultra-low variance (std < 0.02 across 5 seeds, INC-0161)
- Mechanistically grounded: traced to Hopf base sector discretization (INC-0167)

L2 normalization is the validated baseline. No normalization change is
warranted by the current evidence.

**Q2. Should shell hierarchy be excluded from the default design?**

**YES, for L2-normalized embeddings.** Two independent lines of evidence
support this:

1. Shell structure is structurally inaccessible: L2-normalized tokens have
   r_eff=1.0, never reaching the shell-1 threshold (INC-0167).
2. Forced shell activation degrades the concentration quality metric (Gini
   ratio −19% at K=100 TRANS, INC-0168).

Exception condition: if production token embeddings are not L2-normalized (e.g.,
raw transformer hidden states with natural magnitude variation), radial shells
may activate and could provide additional routing structure. This case is
currently outside tested scope and would require a non-normalized embedding
proxy to evaluate.

**Q3. Priority of routing knobs:**

| Knob | Priority | Basis |
|---|---|---|
| Sector resolution K | **HIGH** | Primary routing parameter; compression grows with K through at least K=1000. Diminishing returns not yet observed. | INC-0162, INC-0167 |
| Phase transport (λ) | **HIGH** | TRANS mode reduces α from 0.916 to 0.572 and amplifies all hardware gains 1.5–2.4×. Phase transport is a dominant amplifier. | INC-0147, INC-0162–0165 |
| Angular concentration geometry | **HIGH** | Routing Gini and eff_ratio are the primary quality signals. Design choices that improve sector alignment will directly improve hardware efficiency. | INC-0168 |
| Normalization choice | **LOW** | Norm-invariant to <0.015 in α. Any Lp normalization is acceptable. | INC-0168 |
| Shell count / radial grid | **LOW** | Structurally inaccessible (L2-norm). Forced activation harmful. Exclude from default design. | INC-0167, INC-0168 |

**Q4. Most justified future experiments:**

1. **Large-K capacity test (recommended as INC-0170; see Section 7).**
   The PERM/ORIG compression ratio has not saturated at K=1000. Extrapolation
   of the static scaling law predicts eff_ratio ≈ 4.4× at K=5000. Testing this
   range would validate the scaling law ceiling and supply hardware-cost
   projections at production scales.

2. **Non-L2-normalized embedding proxy.**
   The current proxy uses PPMI-SVD embeddings with L2-normalized rows. All
   production transformer architectures produce unnormalized hidden states with
   substantial magnitude variation. A proxy built from unnormalized embeddings
   would test whether shells activate naturally and whether they contribute
   positive routing structure in that regime.

3. **Training-system implementation.**
   Implement the canonical router (TRANS mode, single shell, K tuned to hardware
   target) inside a trainable small language model. Measure end-to-end effective
   compute and cache locality. This is the Stage 7 close condition.

4. **Hardware-aware K selection.**
   Use the canonical law (α=0.572, c=2.957) to compute a hardware-optimal K
   given a cache budget (SRAM width, LRU associativity). Formalize the
   relationship between α, K, and hardware cost as a design tool.

---

### Section 7: Recommended INC-0170 Proposal

**Title:** Large-K Angular Capacity Test (K=1000–5000)

**Kill-list stage:** Stage 7 — Hardware-Efficiency Confirmation

**Objective:**
Determine whether the PERM/ORIG effective-bucket compression ratio continues to
grow at K > 1000 (as the static scaling law predicts) or saturates. Establish
the capacity ceiling of angular sector routing on the PPMI-SVD proxy, and
validate the inverse-K extrapolation of the canonical law.

**Scaling law prediction (extrapolation from INC-0168 static fit):**

```
ORIG eff_buckets ≈ 2.957 × K^0.572
PERM eff_buckets ≈ 1.664 × K^0.814

Predicted eff_ratio (PERM/ORIG) at K=5000: ≈ 4.4×
```

This is a testable prediction. INC-0170 either confirms or falsifies it.

**Design:**
- Static routing diagnostic (no training loop; matches INC-0167/INC-0168 protocol)
- K values: {400, 1000, 1500, 2000, 3000, 5000}
  (K=400 from INC-0168 included for scaling law continuity)
- Modes: TRANS and BASE
- Data: PPMI-SVD ORIG and PERM (N=5,000 tokens, identity chart)
- Normalization: L2 (canonical baseline)
- Metrics: eff_buckets, Gini coefficient, sector entropy, PERM/ORIG ratios
- Scaling fit: OLS in log-log space (K ≥ 25 per prior protocol)
- Single run per {K, mode, variant} — no seed variation needed for diagnostic

**Success condition (KEEP):**
The PERM/ORIG eff_ratio continues to grow monotonically through K=5000, OR
saturates at a clearly identifiable K*, consistent with the power-law model.
The α fit remains stable and the R² remains ≥ 0.95 across the full K range.

**Falsification condition (KILL / REFINE):**
One of:
a) ORIG and PERM become indistinguishable at high K (eff_ratio → 1.0), indicating
   the Hopf angular grid exhausts its capacity on this proxy.
b) α_TRANS_ORIG shifts by > 0.10 from the INC-0168 value (0.572), indicating
   the static law does not extend to this K range.
c) Anomalous non-monotonic behavior in eff_ratio (structural artifact in the
   routing at large K).

**Kill-list advancement:**
A positive result (eff_ratio continues to grow, law extends to K=5000) would
improve Stage 7 from PARTIAL-PASS to near-PASS and directly support hardware
cost projections at production K values. A saturation result would set the
practical ceiling for K-based hardware design.

**Dependencies:**
- Requires only `_inc0170_analysis.py` (extend `_inc0168_analysis.py` with
  K range {400, 1000, 1500, 2000, 3000, 5000})
- No new data, no new routing code changes

---

## Decision
**KEEP.**

INC-0169 freezes the canonical routing law as the permanent reference for
Stage 7 and future work:

- Scaling law: `eff_buckets = 2.957 × K^0.572` (TRANS ORIG, static, L2-normalized)
- Mechanism: purely angular, Hopf-base sector discretization
- Norm-invariance: confirmed across L1/L2/L3/L4 (Δα < 0.015)
- Shell non-necessity: established (inaccessible by construction; forced activation
  degrades Gini ratio by 19%)
- Hardware consequence: 3.0–4.9× eff_cost reduction, 2.5–2.9× fewer LRU misses
  at matched progress (TRANS mode)

The design implications derived here are fully consistent with the validated
evidence chain and provide actionable guidance for future router development.

Stage 7: PARTIAL-PASS (strong). The routing law and its hardware consequences
are definitively characterized. Remaining Stage 7 work focuses on large-K
capacity validation (INC-0170) and eventual training-system implementation.

## Artifacts
- This document is the artifact. No new experiments were run.
- All cited data: `results/analysis/inc0162_*.json` through
  `results/analysis/inc0168_norm_geometry.json`
- Source increments: INC-0162 through INC-0168
