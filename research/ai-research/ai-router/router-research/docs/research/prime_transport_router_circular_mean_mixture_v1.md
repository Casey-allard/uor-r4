# Circular-Mean Mixture — Experiment v1

## Purpose

Direct test of Training-Rule Alignment Analysis v1, mismatch M2:
the soft operator mixture performs a **Euclidean weighted average** of
angular (cos θ, sin θ) pairs. The result lies inside the unit circle,
not on it, because |Σ wᵢ uᵢ| ≤ 1 for unit vectors uᵢ.
This discards the circular geometry of each phase factor.

## Circular-Mean Mixture Definition

For each cyclic phase factor Z/m represented as (cos θ, sin θ):

**Euclidean mixture** (current baseline):
```
τ̃ = Σᵢ wᵢ · (cos θᵢ, sin θᵢ)
```
Result: |τ̃| < 1 when weights are spread across operators.
Direction encodes the angular mean, but magnitude mixes with
concentration information.  Not on the torus.

**Circular mean** (this experiment):
```
τ̃ = Σᵢ wᵢ · (cos θᵢ, sin θᵢ)
τ  = τ̃ / |τ̃|     (per-phase-pair, independently)
```
Result: |τ| = 1 always.  Each phase pair lives on S¹.
This is the standard circular mean (direction of the resultant vector).

## Per-Phase-Pair Renormalization

Normalization applied independently to each of the 4 phase pairs:

| Pair | Phase Factor | Modulus | After Normalization |
|------|-------------|---------|---------------------|
| 0 | swap_phase | Z/2 | ‖(cos,sin)‖ = 1 |
| 1 | coupled_phase | Z/5 | ‖(cos,sin)‖ = 1 |
| 2 | twist_phase | Z/2 | ‖(cos,sin)‖ = 1 |
| 3 | lift_phase | Z/12 | ‖(cos,sin)‖ = 1 |

Applied after:
1. The einsum mixture of TN entries (every recurrence step)
2. The step-0 token injection (keeps injected tau on torus)

**NOT** applied to final attention pooling (goes to prediction head,
not fed back into recurrence).

Implementation: `tau.view(B, 4, 2)` → normalize each pair → `view(B, 8)`
Differentiable: ∂(x/‖x‖)/∂x = (I - x̂x̂ᵀ)/‖x‖

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

## Three Variants

| Variant | Tau Encoding | D_TAU | D_IN | Mixture | Params |
|---------|-------------|-------|------|---------|--------|
| baseline_onehot | one-hot | 21 | 25 | Euclidean | 1403 |
| angular_euclidean | cos/sin | 8 | 12 | Euclidean | 779 |
| angular_circular | cos/sin | 8 | 12 | Circular mean | 779 |

## Convergence Comparison

| Step | BL Acc | AE Acc | AC Acc | BL α₀ | AE α₀ | AC α₀ | BL Ent | AE Ent | AC Ent |
|------|--------|--------|--------|--------|--------|--------|--------|--------|--------|
| 0 | 0.2630 | 0.2710 | 0.2410 | 0.2440 | 0.2432 | 0.2432 | 1.7507 | 1.7539 | 1.6847 |
| 250 | 0.2880 | 0.2840 | 0.2530 | 0.2460 | 0.2446 | 0.2429 | 1.7507 | 1.7542 | 1.6639 |
| 500 | 0.4060 | 0.3080 | 0.2720 | 0.2533 | 0.2481 | 0.2432 | 1.7505 | 1.7544 | 1.6636 |
| 1000 | 0.7750 | 0.4050 | 0.2860 | 0.3058 | 0.2681 | 0.2459 | 1.7497 | 1.7548 | 1.6658 |
| 1500 | 1.0000 | 0.6450 | 0.3300 | 0.5968 | 0.3500 | 0.2528 | 1.7443 | 1.7546 | 1.6233 |
| 2000 | 1.0000 | 0.9970 | 0.4130 | 0.8210 | 0.6698 | 0.2720 | 1.7354 | 1.7478 | 1.5936 |
| 2500 | 1.0000 | 1.0000 | 0.5760 | 0.8572 | 0.8241 | 0.3346 | 1.7321 | 1.7343 | 1.6043 |
| 3000 | 1.0000 | 1.0000 | 0.7650 | 0.8717 | 0.8596 | 0.5481 | 1.7311 | 1.7265 | 1.6150 |

## Final Summary

| Metric | Baseline | Ang+Euc | Ang+Circ | Δ(Circ−Euc) |
|--------|----------|---------|----------|-------------|
| Final accuracy | 1.0000 | 1.0000 | 0.7650 | -0.2350 |
| Final α₀ | 0.8717 | 0.8596 | 0.5481 | -0.3115 |
| Final route entropy | 1.7311 | 1.7265 | 1.6150 | -0.1115 |
| Final transport frac | 0.5825 | 0.6600 | 0.6490 | -0.0110 |
| First solve step | 1500 | 2000 | never | — |
| Parameters | 1403 | 779 | 779 | 0 |
| Training time (s) | 30.2 | 22.3 | 26.5 | +4.2 |
| Steps/sec | 99.2 | 134.5 | 113.3 | -21.2 |

## Key Questions

**Did all three solve D=24?** Baseline: YES, Ang+Euc: YES, Ang+Circ: NO

**First solve step:** Baseline: 1500, Ang+Euc: 2000, Ang+Circ: never

**Does circular-mean mixture improve accuracy over Euclidean angular?**
NO — worse by 0.2350

**Does circular-mean affect runtime?**
SLOWER — 1.19× (113.3 vs 134.5 sps)

## Interpretation

**Circular-mean normalization BREAKS learning.** The normalization discards
magnitude information that the MLP needs. The resultant length of the
mixture encodes operator weight concentration, which is a useful training signal.
M2 is real but the fix direction is wrong: the MLP USES the off-torus signal.

## Honesty Section

### What Improved

- No material improvement from circular-mean normalization

### What Did Not Improve

- Runtime: slower (113.3 vs 134.5 sps)

### What Remains the Next Mismatch

If M2 circular mean does not materially help:
- **M3**: SGD flat metric ignores torus curvature — optimizer steps in R^n, not on T⁴
- **M4**: Euclidean backward adjoint ≠ geometric adjoint of transport operators
- **M5**: No gradient distinction between transport and non-transport operators

If M2 helps but is insufficient:
- The resultant-length (magnitude before normalization) carries concentration
  information. A hybrid approach could preserve magnitude as an auxiliary feature
  while still normalizing the direction.

