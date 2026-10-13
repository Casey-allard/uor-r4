# INC-0163: Matched-Progress Compute Efficiency

## Status
Closed: KEEP. At matched training progress, ORIG uses 1.7–2.2× fewer effective buckets (TRANS) and 1.4–1.5× fewer (BASE) than PERM. ORIG also converges 1.9–2.4× faster (TRANS). Hardware-efficiency bridge confirmed.

## Kill-List Stage
7 -- Hardware-Efficiency Confirmation

## Mathematical Object Under Test
Whether the structural routing advantage (K^0.50 scaling, INC-0162) translates
into lower cumulative routed computation for equal learning progress.

INC-0162 established:
- TRANS ORIG effective_buckets ~ K^0.50 (square-root)
- TRANS PERM effective_buckets ~ K^0.79 (near-linear)
- BASE ORIG ~ K^0.88, BASE PERM ~ K^0.98

This increment tests: at matched training progress, does geometry-native routing
require fewer cumulative routing accesses than permuted routing?

NO MSE is used for success criteria, routing decisions, or primary comparisons.
Training progress is measured by cosine similarity (directional alignment of
predicted vs actual output vectors), NOT by MSE/reconstruction loss.

## Theory

If geometry concentrates tokens into fewer effective buckets, the per-step
"routing memory footprint" is smaller. Over many training steps, the cumulative
routing cost — measured as total unique bucket activations over time — should be
lower for ORIG than PERM at matched learning progress.

The key insight: ORIG not only converges with fewer active buckets at steady
state, it should also require fewer total bucket activations to reach any given
progress milestone, because the geometry pre-organizes tokens before learning
begins.

## Experimental Design

### Conditions
Seeds: [0, 1, 2, 3, 4]
K values: [75, 100, 150, 200]
Routes per K: TRANS_{K}_ORIG, TRANS_{K}_PERM, BASE_{K}_ORIG, BASE_{K}_PERM
Total: 4 K × 4 routes × 5 seeds = 80 runs

TRANS first (strongest scaling advantage). BASE included for completeness.

### Per-Step Measurements
- active_bucket_count: cumulative unique buckets accessed up to step t
- routed_event_count: t (one routing event per step)
- routing_lookup_count: t (one lookup per step)
- effective_bucket_count: perplexity of bucket visit distribution up to step t
- cosine_similarity: mean cosine(yhat, y) over sliding window (training progress)

### Periodic Eval Checkpoints
At regular intervals (every 250 steps), evaluate on held-out eval data:
- eval_cosine: mean cosine(yhat, y) on eval set (validation progress)
- eval_effective_buckets: effective bucket count on eval routing

### Derived Cumulative Cost Metrics
- cumul_unique_buckets(t): unique buckets activated by step t
- cumul_effective_buckets(t): rolling effective bucket count at step t

### Primary Analysis
Choose target progress levels (e.g., cosine = 0.3, 0.5, 0.7, 0.8, 0.9 of max).
For each target, estimate the step at which ORIG and PERM reach it.
Compute cumulative routing cost at that step for both.

Report:
- cosine vs step curves (learning progress)
- cumul_unique_buckets vs step (routing memory cost)
- effective_bucket_count vs step (routing concentration cost)
- matched-progress compute ratios: ORIG cost / PERM cost at same progress level
- summary table across K and seeds

### Config
`configs/proxy_transfer_inc0163_matched_progress.json`

### Tool
`tools/matched_progress_probe.py` (extends training_efficiency_probe.py mechanism)

## Success Condition
- At matched training progress (same cosine similarity level), ORIG requires
  lower cumulative routed computation than PERM at >= 3 of 4 K values.
  Measured by: unique_buckets at matched progress, effective_buckets at matched progress.
- OR: ORIG reaches the same progress level in fewer steps (faster convergence).
- Advantage visible across 5-seed mean.

## Falsification Condition
- ORIG and PERM reach the same progress levels with equal cumulative routing cost.
- OR: PERM reaches progress targets with LOWER routing cost than ORIG.
- OR: advantage visible at only 1 K value (not structural).

## Results

80 runs completed (5 seeds × 4 K × 4 routes × 2 modes).

### Convergence Summary (5-seed mean ± std)

#### TRANS

| K   | eff_ORIG      | eff_PERM       | ratio  | cos_ORIG | cos_PERM | cos_diff  |
|-----|---------------|----------------|--------|----------|----------|-----------|
| 75  | 32.94 ± 0.35  | 55.75 ± 0.30   | 1.693× | 0.0672   | 0.0636   | +0.0036   |
| 100 | 37.99 ± 0.32  | 74.92 ± 0.41   | 1.972× | 0.0657   | 0.0630   | +0.0027   |
| 150 | 46.96 ± 0.56  | 91.65 ± 1.05   | 1.952× | 0.0665   | 0.0591   | +0.0074   |
| 200 | 57.41 ± 0.65  | 119.74 ± 1.16  | 2.086× | 0.0661   | 0.0579   | +0.0083   |

#### BASE

| K   | eff_ORIG      | eff_PERM       | ratio  | cos_ORIG | cos_PERM | cos_diff  |
|-----|---------------|----------------|--------|----------|----------|-----------|
| 75  | 49.18 ± 0.19  | 67.06 ± 0.44   | 1.364× | 0.0648   | 0.0637   | +0.0011   |
| 100 | 70.82 ± 0.28  | 93.01 ± 0.26   | 1.313× | 0.0644   | 0.0624   | +0.0020   |
| 150 | 94.90 ± 0.59  | 134.93 ± 0.59  | 1.422× | 0.0633   | 0.0600   | +0.0034   |
| 200 | 123.71 ± 0.47 | 176.90 ± 0.80  | 1.430× | 0.0599   | 0.0567   | +0.0031   |

### Matched-Progress Compute Ratios (at 70% of PERM max cosine)

| Mode  | K   | ub_ratio (PERM/ORIG) | eb_ratio (PERM/ORIG) |
|-------|-----|----------------------|----------------------|
| TRANS | 75  | 1.327×               | 1.712×               |
| TRANS | 100 | 1.409×               | 2.010×               |
| TRANS | 150 | 1.699×               | 2.030×               |
| TRANS | 200 | 1.839×               | 2.225×               |
| BASE  | 75  | 1.134×               | 1.444×               |
| BASE  | 100 | 1.155×               | 1.375×               |
| BASE  | 150 | 1.211×               | 1.506×               |
| BASE  | 200 | 1.231×               | 1.496×               |

### Convergence Speed (steps to reach 90% of PERM max cosine)

| Mode  | K   | steps_ORIG | steps_PERM | speed_ratio |
|-------|-----|------------|------------|-------------|
| TRANS | 75  | 787 ± 254  | 1341 ± 234 | 1.882×      |
| TRANS | 100 | 664 ± 124  | 1521 ± 164 | 2.369×      |
| TRANS | 150 | 828 ± 175  | 1764 ± 106 | 2.205×      |
| TRANS | 200 | 925 ± 150  | 1913 ± 104 | 2.119×      |
| BASE  | 75  | 1172 ± 312 | 1431 ± 258 | 1.269×      |
| BASE  | 100 | 1376 ± 274 | 1699 ± 144 | 1.286×      |
| BASE  | 150 | 1663 ± 329 | 2107 ± 216 | 1.301×      |
| BASE  | 200 | 1706 ± 303 | 2530 ± 620 | 1.508×      |

### Success Criteria Check

1. At matched progress, ORIG lower cumulative routing cost at >= 3/4 K values:
   - TRANS: 4/4 PASS (eb_ratio 1.71–2.23×)
   - BASE: 4/4 PASS (eb_ratio 1.38–1.51×)
2. Advantage visible across 5-seed mean at all K: **PASS** (all 8 conditions > 1.0×)
3. ORIG reaches progress targets faster: **PASS** (TRANS 1.9–2.4×, BASE 1.3–1.5×)

**OVERALL: PASS -- KEEP**

### Key Findings

1. **ORIG achieves equal or better learning progress with ~2× fewer effective buckets (TRANS).**
   At K=200, ORIG uses 57 effective buckets vs PERM's 120, while achieving +0.008 higher
   eval cosine similarity.

2. **ORIG converges ~2× faster (TRANS).** To reach 90% of PERM's final progress, TRANS ORIG
   needs 664–925 steps vs PERM's 1341–1913 steps. The geometry pre-organizes tokens
   so the EMA converges faster.

3. **The advantage widens with K.** Matched-progress eb_ratio grows from 1.71× (K=75) to
   2.23× (K=200) for TRANS. This is consistent with the K^0.50 scaling law from INC-0162.

4. **BASE shows consistent but smaller advantage.** eb_ratio 1.38–1.51× at matched progress.
   Phase transport amplifies geometric concentration.

5. **No MSE used.** Progress measured by cosine similarity between predicted and actual
   output vectors. Cumulative routing cost is the primary comparison metric.

## Decision

**KEEP.** The structural routing advantage (K^0.50 scaling, bucket concentration) translates
directly into lower cumulative routed computation at matched learning progress. ORIG routing
achieves equal or better learning with 1.7–2.2× fewer effective bucket activations (TRANS)
and converges 1.9–2.4× faster. Stage 7: PARTIAL-PASS (strong, hardware-efficiency bridge confirmed).
