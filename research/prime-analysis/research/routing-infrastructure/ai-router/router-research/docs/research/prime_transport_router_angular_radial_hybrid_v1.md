# Angular + Radial Hybrid — Experiment v1

## Purpose

Test whether the correct training representation is **angular direction +
preserved radial magnitude** (S¹ × R⁺ per phase pair), rather than pure
Euclidean (R²) or pure torus projection (S¹).

Motivated by the circular-mean experiment failure: discarding resultant
magnitude broke learning because the model uses it as a
confidence/concentration signal.

## Hybrid Representation

After the einsum mixture `base = Σ wᵢ · (cos θᵢ, sin θᵢ)`:

1. **Resultant magnitude** per phase pair: r = ‖(Σ wᵢ cos θᵢ, Σ wᵢ sin θᵢ)‖
2. **Unit direction** per phase pair: d̂ = (cos θ̄, sin θ̄) = base_pair / r
3. **Hybrid tau**: `[d̂₀, d̂₁, d̂₂, d̂₃, r₀, r₁, r₂, r₃]` = 12 dims

| Component | Dims | Interpretation |
|-----------|------|----------------|
| direction | 8 | circular mean direction on S¹ per phase pair |
| magnitude | 4 | resultant length (concentration/confidence) per phase pair |
| **total** | **12** | **S¹ × R⁺ per phase pair** |

Direction is normalized to the unit circle (same as circular-mean).
Magnitude is preserved as a separate explicit feature (what circular-mean discarded).
The MLP sees both and can learn to use each independently.

Initial states (on-torus): all magnitudes = 1.0.
During training: magnitudes vary as the softmax distributes weight across operators.

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

## Four Variants

| Variant | Tau Encoding | D_TAU | D_IN | Mixture | Params |
|---------|-------------|-------|------|---------|--------|
| baseline_onehot | one-hot | 21 | 25 | Euclidean | 1403 |
| angular_euclidean | cos/sin | 8 | 12 | Euclidean | 779 |
| angular_circular | cos/sin | 8 | 12 | Circular mean | 779 |
| angular_hybrid | dir+mag | 12 | 16 | Direction+Magnitude | 971 |

## Convergence Comparison

| Step | BL Acc | AE Acc | AC Acc | AH Acc | BL α₀ | AE α₀ | AC α₀ | AH α₀ |
|------|--------|--------|--------|--------|--------|--------|--------|--------|
| 0 | 0.2630 | 0.2710 | 0.2410 | 0.2310 | 0.2440 | 0.2432 | 0.2432 | 0.2431 |
| 250 | 0.2880 | 0.2840 | 0.2530 | 0.2850 | 0.2460 | 0.2446 | 0.2429 | 0.2440 |
| 500 | 0.4060 | 0.3080 | 0.2720 | 0.3210 | 0.2533 | 0.2481 | 0.2432 | 0.2474 |
| 1000 | 0.7750 | 0.4050 | 0.2860 | 0.3750 | 0.3058 | 0.2681 | 0.2459 | 0.2689 |
| 1500 | 1.0000 | 0.6450 | 0.3300 | 0.5990 | 0.5968 | 0.3500 | 0.2528 | 0.3663 |
| 2000 | 1.0000 | 0.9970 | 0.4130 | 0.9910 | 0.8210 | 0.6698 | 0.2720 | 0.7313 |
| 2500 | 1.0000 | 1.0000 | 0.5760 | 1.0000 | 0.8572 | 0.8241 | 0.3346 | 0.8365 |
| 3000 | 1.0000 | 1.0000 | 0.7650 | 1.0000 | 0.8717 | 0.8596 | 0.5481 | 0.8618 |

## Final Summary

| Metric | Baseline | Ang+Euc | Ang+Circ | **Ang+Hybrid** |
|--------|----------|---------|----------|----------------|
| Final accuracy | 1.0000 | 1.0000 | 0.7650 | **1.0000** |
| Final α₀ | 0.8717 | 0.8596 | 0.5481 | **0.8618** |
| Route entropy | 1.7311 | 1.7265 | 1.6150 | **1.4226** |
| Transport frac | 0.5825 | 0.6600 | 0.6490 | **0.1516** |
| First solve step | 1500 | 2000 | never | **2000** |
| Parameters | 1403 | 779 | 779 | **971** |
| Training time (s) | 26.6 | 20.6 | 24.6 | **25.4** |
| Steps/sec | 112.7 | 145.5 | 121.8 | **117.9** |

## Radial-Law Measurement

### Resultant magnitude by checkpoint (angular variants)

| Step | Ang+Euc mag | Ang+Circ mag | **Ang+Hybrid mag** | Ang+Euc std | Ang+Hybrid std |
|------|-------------|--------------|--------------------|-----------  |----------------|
| 0 | 0.52659 | 0.54695 | **0.58444** | 0.20500 | **0.19685** |
| 250 | 0.52559 | 0.55170 | **0.58492** | 0.20529 | **0.19586** |
| 500 | 0.52480 | 0.55015 | **0.58557** | 0.20556 | **0.19311** |
| 1000 | 0.52498 | 0.54887 | **0.59105** | 0.20595 | **0.19609** |
| 1500 | 0.52314 | 0.56365 | **0.59526** | 0.20670 | **0.20074** |
| 2000 | 0.52356 | 0.57672 | **0.59808** | 0.20621 | **0.19811** |
| 2500 | 0.52709 | 0.56834 | **0.63877** | 0.20484 | **0.18565** |
| 3000 | 0.52981 | 0.56539 | **0.63925** | 0.20751 | **0.18585** |

### Per-phase-pair magnitude at final checkpoint

| Phase Pair | Factor | Ang+Euc | Ang+Hybrid |
|------------|--------|---------|------------|
| 0 | swap (Z/2) | 0.85024 | 0.96411 |
| 1 | coupled (Z/5) | 0.44516 | 0.52472 |
| 2 | twist (Z/2) | 0.35906 | 0.47990 |
| 3 | lift (Z/12) | 0.46479 | 0.58828 |

### Magnitude–accuracy correlation across checkpoints

- **Ang+Euc**: Pearson r(magnitude, accuracy) = 0.2709
- **Ang+Hybrid**: Pearson r(magnitude, accuracy) = 0.8344

### Magnitude trajectory interpretation

- **Ang+Euc**: magnitude stable (0.5266 → 0.5298)
- **Ang+Hybrid**: magnitude growing (0.5844 → 0.6392)

## Interpretation

**Hybrid converges at the same rate as Euclidean angular.** The explicit
direction/magnitude separation does not help beyond what the MLP already extracts
from the raw Euclidean angular vectors.

Key comparison: circular-mean FAILS (discards magnitude) while hybrid SUCCEEDS
(preserves magnitude). This directly confirms that radial magnitude is the
critical missing signal.


## Honesty Section

### What Improved

- Hybrid SOLVES while circular-mean does NOT (magnitude preservation is critical)

### What Did Not Improve

- Convergence: solve step 2000 vs 2000 (not faster)
- Runtime: 117.9 vs 145.5 sps (overhead from decomposition)

### Whether M4 is Now the Next Binding Mismatch

If hybrid representation resolves M2 fully, the remaining mismatches are:
- **M3**: SGD flat metric ignores torus curvature
- **M4**: Euclidean backward adjoint ≠ geometric adjoint of transport
- **M5**: No gradient distinction between transport/non-transport operators

M4 (geometric adjoint) is the most likely next binding mismatch because it
affects the backward pass — the dominant runtime component — and the current
backward pass treats all operators' gradients identically regardless of their
geometric transport structure.

