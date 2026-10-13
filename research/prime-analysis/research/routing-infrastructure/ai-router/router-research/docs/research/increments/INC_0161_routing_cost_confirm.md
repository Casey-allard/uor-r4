# INC-0161: Multi-Seed Routing Compute Sparsity Replication -- Confirm

## Status
Closed: KEEP. Routing compute compression replicated across 5 seeds and 4 K values. All success criteria met.

## Kill-List Stage
7 -- Hardware-Efficiency Confirmation

## Mathematical Object Under Test
Stability of routing compute compression across random seeds and bucket counts.

INC-0160 (1 seed, K=75) found:
- Effective bucket ratio (PERM/ORIG): 1.67x TRANS, 1.35x BASE
- Training Gini ratio (ORIG/PERM): 1.59x TRANS, 1.89x BASE
- Training sparsity matches eval sparsity (Gini within 0.02 of INC-0159)

This increment replicates across 5 seeds and 4 K values to test stability.

NO MSE is used anywhere (INC-0155: routing-agnostic).

## Theory

If the routing compute compression from INC-0160 is a genuine property of
H^4 Hopf geometry (not a seed artifact), then:
1. The effective bucket ratio should remain > 1.2x across all seeds
2. The Gini advantage should be positive at every seed
3. The compression should persist across K values (not K-specific)

The geometric growth boundary at K=75 (INC-0146) may produce the strongest
effect, but compression should be visible at K >= 25 (where bucket coherence
was established in INC-0158).

## Experimental Design

### Conditions
Seeds: [0, 1, 2, 3, 4]
K values: [25, 50, 75, 100]
Routes per K: TRANS_{K}_ORIG, TRANS_{K}_PERM, BASE_{K}_ORIG, BASE_{K}_PERM
Total: 4 K values x 4 routes x 5 seeds = 80 runs

### Metrics (all routing-structural, NO MSE)
- effective_bucket_count: exp(H(p)) perplexity of bucket visit distribution
- training_gini: Gini coefficient of per-bucket visit counts
- top_half_concentration: fraction of samples in top-50% most visited buckets
- unique_buckets: distinct bucket keys visited (secondary)

### Derived ratios (per seed, per K)
- effective_bucket_ratio: PERM_effective / ORIG_effective
- gini_ratio: ORIG_gini / PERM_gini

### Summary statistics
- Mean +/- std across 5 seeds for each metric at each K

### Config
`configs/proxy_transfer_inc0161_routing_cost_confirm.json`

### Tool
`tools/training_efficiency_probe.py` (same as INC-0160)

## Success Condition
- Effective bucket ratio > 1.2x at TRANS K=75 across 5-seed mean
- Gini ratio > 1.3x at TRANS K=75 across 5-seed mean
- Compression visible at >= 3 of 4 K values
- Per-seed effective bucket ratio > 1.0x at all 5 seeds (no sign flip)

## Falsification Condition
- Mean effective bucket ratio < 1.1x at TRANS K=75
- OR any seed shows PERM more concentrated than ORIG (ratio < 1.0)
- OR compression only visible at 1 K value (K-specific artifact)

## Results

80 runs completed (5 seeds x 4 K x 4 routes).

### Per-Route Summary (mean +/- std across 5 seeds)

| Route              | eff_bucket     | gini           | top_half       | unique     |
|--------------------|----------------|----------------|----------------|------------|
| TRANS_K25_ORIG     | 18.56 +/- 0.17 | 0.3934 +/- 0.006 | 0.7687 +/- 0.005 | 24.0     |
| TRANS_K25_PERM     | 21.81 +/- 0.08 | 0.2418 +/- 0.005 | 0.6681 +/- 0.005 | 24.0     |
| BASE_K25_ORIG      | 20.38 +/- 0.08 | 0.3454 +/- 0.002 | 0.7401 +/- 0.003 | 25.0     |
| BASE_K25_PERM      | 23.68 +/- 0.04 | 0.1873 +/- 0.003 | 0.6207 +/- 0.003 | 25.0     |
| TRANS_K50_ORIG     | 32.58 +/- 0.25 | 0.5067 +/- 0.005 | 0.8734 +/- 0.003 | 50.0     |
| TRANS_K50_PERM     | 44.28 +/- 0.21 | 0.2723 +/- 0.005 | 0.6834 +/- 0.004 | 50.0     |
| BASE_K50_ORIG      | 34.41 +/- 0.17 | 0.4747 +/- 0.003 | 0.8491 +/- 0.004 | 50.0     |
| BASE_K50_PERM      | 44.19 +/- 0.21 | 0.2732 +/- 0.005 | 0.6885 +/- 0.004 | 50.0     |
| TRANS_K75_ORIG     | 32.94 +/- 0.35 | 0.6437 +/- 0.007 | 0.9488 +/- 0.004 | 68.8+/-1.0 |
| TRANS_K75_PERM     | 55.75 +/- 0.30 | 0.3967 +/- 0.005 | 0.7843 +/- 0.004 | 73.0     |
| BASE_K75_ORIG      | 49.18 +/- 0.19 | 0.5019 +/- 0.003 | 0.8610 +/- 0.002 | 75.0     |
| BASE_K75_PERM      | 67.06 +/- 0.44 | 0.2590 +/- 0.007 | 0.6752 +/- 0.004 | 75.0     |
| TRANS_K100_ORIG    | 37.99 +/- 0.32 | 0.6778 +/- 0.006 | 0.9515 +/- 0.002 | 87.6+/-1.6 |
| TRANS_K100_PERM    | 74.92 +/- 0.41 | 0.3590 +/- 0.011 | 0.7481 +/- 0.010 | 93.8+/-1.0 |
| BASE_K100_ORIG     | 70.82 +/- 0.28 | 0.4569 +/- 0.003 | 0.8478 +/- 0.002 | 100.0    |
| BASE_K100_PERM     | 93.01 +/- 0.26 | 0.2166 +/- 0.005 | 0.6590 +/- 0.003 | 100.0    |

### Comparison Ratios (mean +/- std across 5 seeds)

| Pair                     | eff_ratio (PERM/ORIG) | gini_ratio (ORIG/PERM) | top_half_diff |
|--------------------------|----------------------|------------------------|---------------|
| TRANS K=25               | 1.175 +/- 0.011      | 1.628 +/- 0.047        | 0.101 +/- 0.008 |
| BASE K=25                | 1.162 +/- 0.006      | 1.845 +/- 0.039        | 0.119 +/- 0.005 |
| TRANS K=50               | 1.359 +/- 0.012      | 1.862 +/- 0.034        | 0.190 +/- 0.005 |
| BASE K=50                | 1.284 +/- 0.010      | 1.738 +/- 0.038        | 0.161 +/- 0.007 |
| TRANS K=75               | 1.693 +/- 0.019      | 1.623 +/- 0.023        | 0.165 +/- 0.004 |
| BASE K=75                | 1.364 +/- 0.013      | 1.939 +/- 0.057        | 0.186 +/- 0.005 |
| TRANS K=100              | 1.972 +/- 0.016      | 1.889 +/- 0.045        | 0.203 +/- 0.009 |
| BASE K=100               | 1.313 +/- 0.008      | 2.110 +/- 0.052        | 0.189 +/- 0.004 |

All 8 comparisons: every seed > 1.0x effective bucket ratio.

### Success Criteria Check

1. TRANS K=75 eff_ratio mean = 1.693 (threshold > 1.2): **PASS**
2. TRANS K=75 gini_ratio mean = 1.623 (threshold > 1.3): **PASS**
3. Compression at >= 3 of 4 K values:
   - K=25: 1.175 (< 1.2): FAIL
   - K=50: 1.359 (> 1.2): PASS
   - K=75: 1.693 (> 1.2): PASS
   - K=100: 1.972 (> 1.2): PASS
   - Result: 3/4 **PASS**
4. Every seed > 1.0x at TRANS K=75: **PASS** (all 5 seeds: 1.666, 1.678, 1.721, 1.699, 1.699)

**OVERALL: PASS -- KEEP**

### Key Findings

1. **Compression scales with K.** The effective bucket ratio grows monotonically: 1.18x (K=25) -> 1.36x (K=50) -> 1.69x (K=75) -> 1.97x (K=100). This is geometric: more buckets = more room for concentration.

2. **Ultra-low variance.** Std of the effective bucket ratio is 0.008--0.019 across all conditions. The compression is not a seed artifact -- it is a stable structural property of the routing geometry.

3. **TRANS concentrates more than BASE.** At K=100, TRANS achieves 1.97x compression vs BASE's 1.31x. The phase-transport coupling amplifies geometric concentration.

4. **K=25 narrowly misses 1.2x threshold** (1.175x) because the bucket space is nearly saturated at K=25 (24/25 buckets used). This is expected: small K leaves little room for concentration.

5. **Training Gini consistently 1.6--2.1x higher for ORIG.** The Gini advantage holds across all K values and both sector modes, with remarkably low variance.

## Decision

**KEEP.** Routing compute compression is replicated across 5 seeds with ultra-low variance. Compression grows monotonically with K and is visible at 3 of 4 K values. Stage 7 upgrades from PARTIAL to PARTIAL-PASS (replicated 5-seed structural training sparsity).
