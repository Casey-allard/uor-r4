# Angular Phase Encoding — Experiment v1

## Purpose

Direct test of Training-Rule Alignment Analysis v1, mismatch M1:
one-hot tau encoding destroys cyclic adjacency of the discrete torus
T⁴ = Z/2 × Z/5 × Z/2 × Z/12.

## Angular Encoding

Each cyclic phase k ∈ Z/m is encoded as (cos(2πk/m), sin(2πk/m)).

| Phase | Modulus | One-Hot Dims | Angular Dims |
|-------|---------|-------------|-------------|
| swap_phase | Z/2 | 2 | 2 (cos/sin) |
| coupled_phase | Z/5 | 5 | 2 (cos/sin) |
| twist_phase | Z/2 | 2 | 2 (cos/sin) |
| lift_phase | Z/12 | 12 | 2 (cos/sin) |
| **Total D_TAU** | | **21** | **8** |
| **D_IN (emb+tau)** | | **25** | **12** |

**Key property**: In angular encoding, adjacent phases on Z/m are
adjacent on S¹ — including the wrap-around (e.g., lift_phase=11 and
lift_phase=0 are nearby). In one-hot encoding, ALL distinct phases are
equidistant (distance √2).

**Soft mixture meaning**: Weighted average of angular TN entries produces a
**circular mean** — the geometrically correct averaging operation on S¹.
In one-hot encoding, the same operation produces a probability vector with
no geometric meaning on the torus.

## Locked Configuration

| Item | Value |
|------|-------|
| device | cpu |
| D_HIDDEN | 32 |
| batch_size | 256 |
| D (delay) | 24 |
| training budget | 3000 steps |
| optimizer | SGD, lr=0.02 |
| temperature | 2.0 → 0.1 (exponential) |
| grad clip | 1.0 |
| pos0 bias init | 2.0 |
| seed | 42 |

## Convergence Comparison

| Step | Baseline Acc | Angular Acc | Baseline α₀ | Angular α₀ | Baseline Ent | Angular Ent |
|------|-------------|-------------|-------------|-------------|-------------|-------------|
| 0 | 0.2630 | 0.2710 | 0.2440 | 0.2432 | 1.7507 | 1.7539 |
| 250 | 0.2880 | 0.2840 | 0.2460 | 0.2446 | 1.7507 | 1.7542 |
| 500 | 0.4060 | 0.3080 | 0.2533 | 0.2481 | 1.7505 | 1.7544 |
| 1000 | 0.7750 | 0.4050 | 0.3058 | 0.2681 | 1.7497 | 1.7548 |
| 1500 | 1.0000 | 0.6450 | 0.5968 | 0.3500 | 1.7443 | 1.7546 |
| 2000 | 1.0000 | 0.9970 | 0.8210 | 0.6698 | 1.7354 | 1.7478 |
| 2500 | 1.0000 | 1.0000 | 0.8572 | 0.8241 | 1.7321 | 1.7343 |
| 3000 | 1.0000 | 1.0000 | 0.8717 | 0.8596 | 1.7311 | 1.7265 |

## Final Summary

| Metric | Baseline (one-hot) | Angular (cos/sin) | Δ |
|--------|-------------------|-------------------|---|
| Final accuracy | 1.0000 | 1.0000 | +0.0000 |
| Final α₀ | 0.8717 | 0.8596 | -0.0121 |
| Final route entropy | 1.7311 | 1.7265 | -0.0046 |
| Parameters | 1403 | 779 | -624 |
| D_TAU | 21 | 8 | −13 |
| D_IN | 25 | 12 | −13 |
| Training time (s) | 29.8 | 23.3 | -6.5 |
| Steps/sec | 100.6 | 128.8 | +28.2 |

## Key Questions

**Did both solve D=24?** Baseline: YES, Angular: YES

**First solve checkpoint:** Baseline: 1500, Angular: 2000
  → Baseline converges **faster** (500 steps earlier)

**Does angular encoding improve accuracy?** 
MARGINAL — difference is +0.0000 (within noise)

**Does angular encoding change α₀?** 
YES — α₀ decreased by -0.0121

**Is angular faster at runtime?** 
YES — 1.28× faster (128.8 vs 100.6 sps)

## Interpretation

Angular encoding achieves **comparable accuracy** with fewer parameters and dimensions.
The MLP is powerful enough to learn the cyclic structure from one-hot data at this scale.
The representation mismatch (M1) exists but does not materially limit learning at D=24.

## Honesty Section

### What Improved

- Runtime: 1.28× faster (smaller matrices)
- Parameter count: 624 fewer parameters

### What Did Not Improve

- Final accuracy: +0.0000 (no material gain)
- Convergence speed: not clearly faster

### Next Mismatch If This Only Partially Helps

If angular encoding helps convergence but not final accuracy, the next
mismatch to investigate is M2: the soft mixture still computes a **Euclidean**
weighted average of angular vectors rather than a proper circular mean with
normalization to the unit circle. A follow-up experiment would replace the
einsum mixture with per-phase-pair normalization after averaging.

If angular encoding does not help at all, the binding constraint is likely
NOT the tau coordinate system but rather the sequential recurrence structure
itself (the D=24 loop), and the mismatches M3/M4 (SGD metric, Euclidean adjoint)
may be more relevant targets.

