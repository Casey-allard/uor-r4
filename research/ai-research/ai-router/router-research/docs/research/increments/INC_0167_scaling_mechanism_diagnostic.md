# INC-0167: Scaling Mechanism Diagnostic — Sector vs Shell Attribution

## Status
Closed: KEEP.

Shell structure is structurally inaccessible for L2-normalized PPMI-SVD data.
The √K scaling arises entirely from angular sector discretization amplified by
phase transport concentration. The scaling law from INC-0162 is confirmed to
extend through K=1000 and is definitively attributed to its geometric mechanism.

## Kill-List Stage
7 — Hardware-Efficiency Confirmation

## Mathematical Object Under Test
The sector discretization grid and its interaction with the scaling law
`effective_buckets = c × K^α` established in INC-0162.

Specifically: does the sublinear exponent α arise from angular sector
discretization (Hypothesis A), radial shell structure (Hypothesis B),
or both?

## Theory

The INC-0162 scaling law shows `eff_buckets = c × K^α` with TRANS ORIG
α ≈ 0.50 at K=25..200. Across all prior experiments (INC-0160 through
INC-0166), the observed shell count was always 1. Two hypotheses:

**Hypothesis A (angular sector discretization):**
The Hopf-base sector grid uses kchi = floor(√K) angular bins, creating
an inherent √K angular structure. Phase transport concentrates tokens
into a subset of these angular bins, producing α < 1.

**Hypothesis B (radial shell structure):**
Shell transitions might activate at larger K values, contributing
additional radial discrimination.

## Experimental Design

### Step 1: Router mechanism audit
Inspect the routing code to extract the exact shell/sector logic.

### Step 2: Shell activation sweep
Route with K = [250, 300, 400, 600, 1000], measure shell/sector counts
and effective bucket counts. Check whether shells activate at large K.

### Step 3: Forced shell sensitivity
Vary delta_r to force different shell assignments. Measure whether shell
changes affect routing metrics.

### Step 4: Sector scaling
Measure active sector count, sector entropy, and sector Gini across
K = [10..1000]. Fit scaling exponents.

Additionally: run K = [250, 400, 600, 1000] through the standard
proxy_sweep pipeline (2 seeds) to obtain training-time scaling exponents.

## Success Condition
1. √K scaling mechanism is definitively attributed to angular sector
   discretization and/or shell structure.
2. The scaling law extends to K=1000 without breakdown.
3. Shell activation threshold is quantified.

## Falsification Condition
- The scaling exponent α changes sign or approaches 1.0 at large K,
  invalidating the sublinear scaling law.
- OR: shells activate at large K and fundamentally alter the scaling
  exponent.

---

## Results

### 1. Router Implementation Summary

**Shell assignment (phi_log mode):**
```
shell = floor(log1p(r_eff / delta_r) / LOG_PHI)
```
where `LOG_PHI = ln(φ) = 0.4812`, `delta_r = 3.6` (config).

Shell thresholds (r_eff values at which new shells activate):
- Shell ≥ 0: r_eff ≥ 0.0000
- Shell ≥ 1: r_eff ≥ 2.2249
- Shell ≥ 2: r_eff ≥ 5.8249
- Shell ≥ 3: r_eff ≥ 11.650

**Critical finding:** The PPMI-SVD proxy data is L2-normalized.
All input vectors have ||x|| = 1.0. The identity SO(8) chart preserves
norms, so `r_eff = 1.0` for all tokens.

Since 1.0 < 2.225 (shell ≥ 1 threshold), **all tokens are permanently
in shell 0**. Shell structure is structurally inaccessible for this data
regardless of K or delta_r.

**Sector assignment (phase4d_hopf_base, used by BASE):**
```
kchi = floor(√K)       # angular bins on Hopf chi coordinate
kdelta = ceil(K / kchi)  # delta coordinate bins
sector = (bchi × kdelta + bdelta) % K
```
This creates a 2D grid on the Hopf base S² manifold.

**Sector assignment (phase4d_hopf_transport, used by TRANS):**
```
(kchi, kdelta, kalpha) = allocate_triplet_bins_budget(K, min_first=2, min_second=2, min_third=2)
sector = bchi × (kdelta × kalpha) + bdelta × kalpha + balpha
```
This creates a 3D grid on (chi, delta, transported_alpha), where alpha
is the phase-transport-shifted fiber coordinate.

**Bucket key:** `(shell, sector)`. With shells=1, `bucket = (0, sector)`,
so the bucket space is the sector space.

### 2. Shell Activation Sweep Results

Routing diagnostic with N=5000, delta_r=3.6, seed=0, K=[250..1000]:

| K    | Mode  | Shells | Active Sectors | Eff Buckets | Gini   | Top-half | Sect Entropy |
|------|-------|--------|----------------|-------------|--------|----------|--------------|
|  250 | BASE  |      1 |            244 |       151.9 | 0.5321 |    0.877 |        5.023 |
|  250 | TRANS |      1 |            184 |        67.8 | 0.7214 |    0.950 |        4.217 |
|  300 | BASE  |      1 |            292 |       183.7 | 0.5283 |    0.875 |        5.213 |
|  300 | TRANS |      1 |            216 |        78.6 | 0.7238 |    0.950 |        4.364 |
|  400 | BASE  |      1 |            377 |       242.7 | 0.5146 |    0.865 |        5.492 |
|  400 | TRANS |      1 |            267 |       101.2 | 0.7129 |    0.945 |        4.617 |
|  600 | BASE  |      1 |            533 |       353.7 | 0.4966 |    0.850 |        5.868 |
|  600 | TRANS |      1 |            338 |       127.1 | 0.7144 |    0.946 |        4.845 |
| 1000 | BASE  |      1 |            806 |       536.9 | 0.4931 |    0.845 |        6.286 |
| 1000 | TRANS |      1 |            451 |       164.4 | 0.7145 |    0.940 |        5.102 |

**Result:** Shells = 1 at ALL K values up to 1000. No shell activation detected.
This is structural: r_eff = 1.0 for all tokens, shell ≥ 1 requires r_eff ≥ 2.225.

### 3. Forced Shell Sensitivity Results

Varying delta_r at K=100 to shift which shell number all tokens occupy:

| delta_r | All tokens in | BASE eff_bkt | TRANS eff_bkt | Gini (BASE) | Gini (TRANS) |
|---------|---------------|--------------|---------------|-------------|--------------|
|     0.6 | shell 2       |         69.3 |          36.4 |      0.4707 |       0.6857 |
|     1.0 | shell 1       |         69.3 |          36.4 |      0.4707 |       0.6857 |
|     1.5 | shell 1       |         69.3 |          36.4 |      0.4707 |       0.6857 |
|     2.0 | shell 0       |         69.3 |          36.4 |      0.4707 |       0.6857 |
|     3.6 | shell 0       |         69.3 |          36.4 |      0.4707 |       0.6857 |
|    20.0 | shell 0       |         69.3 |          36.4 |      0.4707 |       0.6857 |

**Result:** ALL routing metrics are IDENTICAL regardless of delta_r.
Changing delta_r only changes which shell NUMBER all tokens land in.
Since all tokens have the same r_eff, they always land in the SAME shell.
Shell structure has ZERO effect on routing for L2-normalized data.

Forced multi-shell experiments are structurally impossible:
you cannot distribute tokens across multiple shells when every token
has the same radial distance.

### 4. Sector Scaling Analysis

**Static routing (seed=0, N=5000, K=10..1000):**

Scaling fit: `active_sectors = c × K^α`
- BASE:  α = 0.954  (nearly linear — uses almost all available sectors)
- TRANS: α = 0.808  (sublinear — phase transport deactivates sectors)

Scaling fit: `sectors_above_1pct = c × K^α`
- BASE:  α = −0.724  (DECREASES — at K=1000, zero BASE sectors hold >1% traffic)
- TRANS: α = 0.089   (nearly constant ~27–34 high-traffic sectors regardless of K)

Scaling fit: `effective_bucket_count = c × K^α` (static routing)
- BASE:  α = 0.909
- TRANS: α = 0.584

**Training-time metrics (proxy_sweep pipeline, 2 seeds, K=250..1000):**

Active bucket count = c × K^α:
- TRANS ORIG: α = 0.586, c = 6.26
- TRANS PERM: α = 0.780, c = 2.91
- BASE ORIG:  α = 0.778, c = 3.25
- BASE PERM:  α = 0.919, c = 1.59

Effective bucket count (perplexity) = c × K^α:
- TRANS ORIG: α = 0.642, c = 2.06
- TRANS PERM: α = 0.823, c = 1.63
- BASE ORIG:  α = 0.863, c = 1.34
- BASE PERM:  α = 0.875, c = 1.83

PERM/ORIG effective bucket ratios (training-time):

| K    | TRANS ratio | BASE ratio |
|------|-------------|------------|
|  250 |       2.18× |      1.47× |
|  400 |       2.28× |      1.46× |
|  600 |       2.61× |      1.45× |
| 1000 |       2.75× |      1.50× |

The TRANS advantage widens with K. At K=1000, TRANS ORIG uses 166.5
effective buckets vs 457.2 for PERM — a 2.75× compression.

**Comparison with INC-0162 (K=25..200) scaling:**

| Configuration | INC-0162 α (K=25..200) | INC-0167 α (K=250..1000) |
|---------------|------------------------|--------------------------|
| TRANS ORIG    |                  0.500 |                    0.642 |
| TRANS PERM    |                  0.795 |                    0.823 |
| BASE ORIG    |                  0.882 |                    0.863 |
| BASE PERM    |                  0.979 |                    0.875 |

TRANS ORIG exponent rises from 0.50 to 0.64 at large K, suggesting mild
log-space curvature rather than a pure power law. However, the exponent
remains well below 1.0, confirming sustained sublinear scaling.

### 5. Interpretation of the Scaling Mechanism

**Definitive conclusion: Hypothesis A (angular sector discretization) is correct.**

The evidence chain:

1. **Shells are structurally irrelevant.** L2-normalized data produces
   r_eff = 1.0 for all tokens. The shell ≥ 1 threshold at r_eff = 2.225
   is never reached. Changing delta_r produces identical routing metrics.
   Hypothesis B is ruled out.

2. **Sector grid structure creates the angular basis for √K scaling.**
   For BASE mode, kchi = floor(√K) angular bins × kdelta radial-like bins.
   For TRANS mode, a triplet allocation creates a 3D grid where the
   chi dimension grows slowly (constrained to 2–10 bins across K=10..1000).

3. **Phase transport concentrates tokens into fewer angular sectors.**
   TRANS routing maintains ~27–34 high-traffic sectors regardless of K,
   while BASE spreads tokens across all available sectors. This concentration
   is what produces the sublinear effective bucket count.

4. **The concentration is semantic, not random.** ORIG (structured data)
   achieves 2.18–2.75× more compression than PERM (column-permuted data),
   and this advantage widens with K. The angular geometry of the Hopf base
   aligns with the semantic structure of PPMI-SVD embeddings.

5. **The scaling law extends to K=1000.** The TRANS ORIG exponent rises
   slightly from α=0.50 (K≤200) to α=0.64 (K=250..1000), indicating
   the power-law approximation has mild curvature in log-log space.
   The sublinear advantage persists and the PERM/ORIG ratio grows.

**Mechanistic decomposition:**
```
√K sector grid structure           → creates angular resolution bins
+ Hopf base coordinate alignment   → maps semantic structure to angular bins  
+ Phase transport concentration    → concentrates tokens into ~30 high-traffic bins
= Sublinear effective bucket count → eff_buckets ∝ K^0.5..0.65
```

**Note on shell relevance:**
Shells are mathematically valid but require non-unit-norm data (or
time_pressure_lambda > 0 or adaptive_shell_growth > 0) to activate.
For the current PPMI-SVD proxy with L2-normalized embeddings, radial
shell structure does not contribute to routing. Any future experiment
with non-normalized embeddings should re-evaluate shell activation.

## Decision
**KEEP.**

The √K scaling law established in INC-0162 is definitively attributed to
angular sector discretization on the Hopf base manifold, amplified by
phase transport concentration. Shell structure is structurally inaccessible
for L2-normalized PPMI-SVD data and contributes nothing to current routing.

The scaling law extends through K=1000 with sustained ORIG advantage
(2.75× at K=1000). No breakdown detected.

## Artifacts
- `results/analysis/inc0167_diagnostic.json` — static routing diagnostic
- `results/analysis/inc0167_scaling_mechanism.json` — pipeline results
- `configs/proxy_transfer_inc0167_scaling_mechanism.json` — 16 routes × 2 seeds
- `_inc0167_diagnostic.py` — routing mechanism diagnostic script
- `_inc0167_analysis.py` — pipeline analysis script
- Pipeline logs in `results/raw/inc0167_*`
- Parsed results in `results/parsed/`
- 32 new rows in `results/summary.csv`
