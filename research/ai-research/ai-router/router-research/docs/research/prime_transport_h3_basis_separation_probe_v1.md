# H³ Basis Separation Probe — v1

## Hypothesis

The null results in prior experiments are caused by **basis mismatch**: the H³ state is initialized in the 2i-rotated frame (via `apply_anchor_two_i`), but projections use the √2/cos metric (standard Euclidean dot product). If 2i is required for projection, the cos metric will give **zero signal** while the sin metric (2i projection) will give **full signal** for the same inputs.

## Mechanism

Three regimes, identical inputs, one variable changed per regime:

| Regime | Normalization | Projection | Basis |
|--------|---------------|------------|-------|
| A | √2 × unit_vec | cos(Δθ) = dot product | Current default |
| B | 2i = 2·R₉₀°·unit_vec (norm=2) | sin(Δθ) = cross product | 2i projection |
| C | √2 × unit_vec | sin(Δθ) = cross product | Split: √2 norm, 2i projection |

Two scenarios tested:
- **correct**: state and reference both in 2i-rotated frame (no mismatch)
- **mismatch**: state in 2i frame, reference in raw (unrotated) frame

**Analytic prediction** (verified before running):
- mismatch + same class: A gives cos=0.0 (null), B gives sin=1.0 (signal)
- correct + same class: A gives cos=1.0 (signal), B gives sin=0.0 (null)
- Class identification (argmax over 4 prototypes):
  - mismatch: A selects class (k−1)%4 (wrong), B selects true class k (correct)

## Configuration

| Parameter | Value |
|-----------|-------|
| N_EVAL | 2048 |
| BLOCKS_A | (0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3) |
| H3 indices | [10,11] |
| D_SWEEP | [0, 1, 5, 20] |
| EPS_SWEEP | [0.0, 0.5, 1.0] |

## Results: Analytic Predictions vs Measured (D=0, eps=0.0)

| Scenario | Regime | Predicted dir_true | Measured dir_true | Predicted acc | Measured acc | radial_ratio |
|----------|--------|--------------------|-------------------|---------------|--------------|--------------|
| mismatch | A | +0.0 | +0.0000 | 0.0 | 0.000 | 0.3536 |
| mismatch | B | +1.0 | +1.0000 | 1.0 | 1.000 | 0.3780 |
| mismatch | C | +1.0 | +1.0000 | 1.0 | 1.000 | 0.3536 |
| correct | A | +1.0 | +1.0000 | 1.0 | 1.000 | 0.3536 |
| correct | B | +0.0 | +0.0000 | 0.0 | 0.000 | 0.3780 |
| correct | C | +0.0 | +0.0000 | 0.0 | 0.000 | 0.3536 |

## Results: Transport Sweep — MISMATCH scenario (original_s42)

| D | eps | ratio_A | ratio_B | dir_A | dir_B | dir_C | acc_A | acc_B |
|---|-----|---------|---------|-------|-------|-------|-------|-------|
| 0 | 0.0 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |
| 0 | 0.5 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |
| 0 | 1.0 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |
| 1 | 0.0 | 0.3536 | 0.3780 | +0.2466 | +0.7534 | +0.7534 | 0.247 | 0.753 |
| 1 | 0.5 | 0.3536 | 0.3780 | +0.1744 | +0.9278 | +0.9278 | 0.247 | 0.753 |
| 1 | 1.0 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |
| 5 | 0.0 | 0.3536 | 0.3780 | +0.2466 | +0.7534 | +0.7534 | 0.247 | 0.753 |
| 5 | 0.5 | 0.3536 | 0.3780 | +0.2465 | +0.7614 | +0.7614 | 0.247 | 0.753 |
| 5 | 1.0 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |
| 20 | 0.0 | 0.3536 | 0.3780 | +0.2466 | +0.7534 | +0.7534 | 0.247 | 0.753 |
| 20 | 0.5 | 0.3536 | 0.3780 | +0.2466 | +0.7534 | +0.7534 | 0.247 | 0.753 |
| 20 | 1.0 | 0.3536 | 0.3780 | +0.0000 | +1.0000 | +1.0000 | 0.000 | 1.000 |

## Results: Transport Sweep — CORRECT scenario (original_s42)

| D | eps | ratio_A | ratio_B | dir_A | dir_B | dir_C | acc_A | acc_B |
|---|-----|---------|---------|-------|-------|-------|-------|-------|
| 0 | 0.0 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |
| 0 | 0.5 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |
| 0 | 1.0 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |
| 1 | 0.0 | 0.3536 | 0.3780 | +0.7534 | -0.2466 | -0.2466 | 0.753 | 0.000 |
| 1 | 0.5 | 0.3536 | 0.3780 | +0.9278 | -0.1744 | -0.1744 | 0.753 | 0.000 |
| 1 | 1.0 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |
| 5 | 0.0 | 0.3536 | 0.3780 | +0.7534 | -0.2466 | -0.2466 | 0.753 | 0.000 |
| 5 | 0.5 | 0.3536 | 0.3780 | +0.7614 | -0.2465 | -0.2465 | 0.753 | 0.000 |
| 5 | 1.0 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |
| 20 | 0.0 | 0.3536 | 0.3780 | +0.7534 | -0.2466 | -0.2466 | 0.753 | 0.000 |
| 20 | 0.5 | 0.3536 | 0.3780 | +0.7534 | -0.2466 | -0.2466 | 0.753 | 0.000 |
| 20 | 1.0 | 0.3536 | 0.3780 | +1.0000 | +0.0000 | +0.0000 | 1.000 | 0.000 |

## Results: Radial Ratio Summary

| Regime | Analytic prediction | Measured mean |
|--------|---------------------|---------------|
| A | √2 / √(2·6+4) = 0.353553 | 0.353553 |
| B | 2 / √(4·6+4) = 0.377964 | 0.377964 |
| C | √2 / √(2·6+4) = 0.353553 (same as A) | 0.353553 |

## Conclusion

**SUPPORT**

In the MISMATCH scenario, Regime A (√2/cos) gives direction=0.0 (null result) while Regime B (2i/sin) gives direction=1.0 (strong signal). Class identification accuracy: A=0% (misidentifies every sample), B=100% (correctly identifies all samples). The 2i projection basis is required to recover geometric signal when the state is in the 2i-rotated frame and the reference is in raw frame.

### Evidence
- **Signal emergence confirmed**: In the mismatch scenario, Regime A (√2/cos) direction collapses to 0.0 (null), while Regime B (2i/sin) rises to 1.0 (maximum signal). The two regimes are antipodal — one's null is the other's signal.
- **Structural separation is 1.0**: The difference in direction metric between A and B equals 1.0, the maximum possible difference.
- **Class identification flip**: Regime A misidentifies 100% of samples (cos selects class (k-1)%4, consistent 1-hop error). Regime B correctly identifies 100% (sin selects true class k).
- **Radial ratio**: A=√2/4≈0.354, B=2/√28≈0.378 — small but consistent structural difference from the amplitude scaling.
- **Regime C confirms**: the direction signal is fully explained by the projection axis (sin), not by the amplitude scaling (√2 vs 2i). C matches B for direction but A for radial.

### Limitations
- This probe tests H3 subspace only; cross-subspace mismatch not measured.
- The mismatch scenario is synthetic: it requires the reference to be in the raw frame while the state is 2i-rotated.
- Whether this mismatch arises naturally in the prior experiments is not established here; that is a separate investigation.

*Elapsed: 0.2s | N_EVAL=2048 | CSV: h3_basis_separation_probe_v1.csv*
