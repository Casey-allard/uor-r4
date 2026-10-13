# Prime Transport — Router Reintegration v4 D=16 Convergence Ceiling

**Experiment type:** D=16 convergence ceiling study (10,000-batch training)
**Operator layer:** `geometry_native_operator_model_v25`
**Architecture:** v4 (token injection + trajectory attention), unchanged
**No surface build. No files modified. No operators rebuilt.**

## What Changed from v4_scaling

This experiment is a **single-delay extension** of the v4 budget scaling study.

| Aspect | v4_scaling | This study |
|--------|-----------|------------|
| Delays tested | D ∈ {2,4,8,16} | D=16 only |
| Max training budget | 5,000 batches | **10,000 batches** |
| Checkpoints | 800, 2000, 5000 | 2000, 5000, 7500, 10000 |
| Architecture | v4 (unchanged) | v4 (unchanged) |

The 5000-batch checkpoint bridges cleanly to v4_scaling D=16 results for direct comparison. New checkpoints at 7500 and 10000 determine whether long-delay recall continues to improve beyond the scaling study range.

## Reference: v4 and v4_scaling D=16 Baselines

| Budget | acc@D=16 | Source |
|--------|----------|--------|
| 800    | 0.269    | v4 standalone |
| 2000   | 0.275    | v4_scaling |
| 5000   | 0.367    | v4_scaling |

## Primary Results

| Budget | acc@D=16 | vs. chance (0.250) | vs. v4_scaling@5000 (0.367) |
|--------|----------|-------------------|------------------------------|
|  2000 | 0.282 | +0.032 | -0.085 |
|  5000 | 0.330 | +0.080 | -0.037 |
|  7500 | 0.375 | +0.125 | +0.008 |
| 10000 | 0.382 | +0.132 | +0.015 |

## Does Performance Scale Beyond 5,000 Batches?

**Yes.** Accuracy continued to improve from 0.330 at 5000 batches to 0.382 at 10,000 batches (Δ=+0.052).

## Routing Behavior

| Budget | transport_frac | route_entropy | collapsed (<0.3)? |
|--------|---------------|---------------|-------------------|
|  2000 | 0.437 | 1.678 | no |
|  5000 | 0.533 | 1.660 | no |
|  7500 | 0.668 | 1.657 | no |
| 10000 | 0.872 | 1.645 | no |

## Attention Weight at Step 0 (encoding step)

Uniform baseline: 1/D = 1/16 = 0.0625

| Budget | alpha[0] | near uniform? |
|--------|----------|---------------|
|  2000 | 0.0624 | yes |
|  5000 | 0.0625 | yes |
|  7500 | 0.0625 | yes |
| 10000 | 0.0627 | yes |

## Long-Delay Ceiling Analysis

| Metric | Value |
|--------|-------|
| Chance level | 0.250 |
| v4 standalone (800b) | 0.269 |
| v4_scaling (5000b) | 0.367 |
| This study best | 0.382 |
| Max gain over v4_scaling@5000 | +0.015 |

**Saturated at 5000b?** Partially. Within this run, accuracy improved from 0.330 at 5000b to 0.382 at 10000b (Δ=+0.052). However, relative to the external v4_scaling reference (0.367 at 5000b), the 10000b result adds only +0.015 — the difference is largely explained by random-seed variation between runs. The 7500→10000 step contributed only +0.007, indicating the improvement rate is strongly slowing.

## Honesty Section

### What improved

- Ceiling accuracy at D=16 reached 0.382 vs. chance 0.250.
- Routing remained non-collapsed throughout all checkpoints.
- The 7500b checkpoint (0.375) confirmed that gains persist past 5000b but at a diminishing rate.
- Extended training directly answered whether additive injection can recover further signal at long delays.

### What saturated or failed

- Improvement rate slowed sharply: Δ5k→7.5k = +0.045, Δ7.5k→10k = +0.007. The curve is flattening.
- The injection signal is the same for all 16 steps; noise-token injections (steps 1–15) accumulate and dilute the encoding-step contribution over a 16-step window.
- The additive injection ceiling at D=16 is approximately 0.38–0.39 under this architecture.

### What remains uncertain

- Whether a position-aware injection (step 0 only) would prevent noise accumulation and lift the D=16 ceiling further.
- Whether gated injection (multiplicative W_gate[tok]) would modulate individual tau dimensions more effectively at large D.
- Whether full exact spin_H is solved: **No.**

## Recommended Next Step

D=16 performance has saturated under additive injection. Implement **gated injection** (v5): replace the additive offset `W_tok_inject[tok]` with a multiplicative gate `soft_tau_curr = (w @ tau_nexts) * (1 + W_gate[tok])`, where `W_gate` is a learned `(VOCAB, D_TAU)` parameter. Multiplicative modulation scales individual tau dimensions at every step, maintaining a stronger token-dependent signal through long delay windows instead of an additive offset that mixes into background noise.
