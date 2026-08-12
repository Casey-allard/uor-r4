# Prime Transport — Radial Ratio Revalidation Probe v1

**Branch:** `radial_ratio_revalidation_probe_v1`

**CANONICAL SOURCES:**
- `results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv`
- `results/prime_transport_recursive_system/carrier_vs_computation_probe_v1.csv`
- `results/prime_transport_recursive_system/d_invariance_probe_v1.json`

---

## 1. Geometry / Measurement Lock Summary

### State representation

The current system constructs a 16-dimensional hybrid state vector:

```
tau_hyb = [tau_ang (12 dims) | tau_mag (4 dims)]

tau_ang layout (block-3 dominant, s=9,e=21,m=12,n_h=3):
  Block 0: h1 at [0,1]
  Block 1: h1 at [2,3]
  Block 2: h1 at [4,5]
  Block 3: h1 at [6,7]  h2 at [8,9]  h3 at [10,11]

tau_mag: one magnitude scalar per block at [12,13,14,15]
```

### Radial length definitions

| Name | Formula | Prediction |
|------|---------|------------|
| `full_radius` | `‖tau_hyb‖` | `sqrt(6·1 + 4·1) = sqrt(10) ≈ 3.162278` |
| `angular_radius` | `‖tau_hyb[:12]‖` | `sqrt(6) ≈ 2.449490` |
| `h3_radius` | `‖tau_hyb[10:12]‖` | `1.0` exactly |
| `h2_radius` | `‖tau_hyb[8:10]‖` | `1.0` exactly |
| `h2h3_radius` | `‖tau_hyb[8:12]‖` | `sqrt(2) ≈ 1.414214` |
| `block3_ang_radius` | `‖tau_hyb[6:12]‖` | `sqrt(3) ≈ 1.732051` |
| `magnitude_radius` | `‖tau_hyb[12:16]‖` | `2.0` exactly |

**Structural argument:** All (cos,sin) pairs in `TN_ang` are unit-norm by construction.
Therefore all h=1 magnitudes = 1.0. With `eps_high=1.0`, h≥2 pairs are frozen at
their tau_init values (also unit-norm). All magnitude entries thus = 1.0.
Ratios are structural constants, not data-dependent.

### Direction definition

The unit-norm direction vector `tau_hyb[:,10:12]` (H3 pair) encodes the identity class
as angular phase. Directional agreement = `dot(tau_init[:,10:12], tau_final[:,10:12])`.
With `eps_high=1.0`, this should be 1.0 to machine precision.

### Measurement schedule

- **Variants:** `original_s42` (offset=0), `shift1_s42` (offset=1)
- **D values:** 0, 1, 20, 32
- **Phases:** at tau_init and tau_final per D
- **Subspaces:** full, angular, H3, H2, H2+H3, block3_ang, magnitude

---

## 2. Measured Radii Across Variants and D

### Summary (full_radius and key subspace radii, mean ± std across N=2048 samples)

| variant | D | full_r | h3_r | h2_r | mag_r | ratio_h3/full |
|---------|---|--------|------|------|-------|---------------|
| original_s42 | D=0 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=0 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=1 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=1 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=20 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=20 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=32 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| original_s42 | D=32 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=0 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=0 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=1 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=1 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=20 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=20 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=32 (init) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |
| shift1_s42 | D=32 (final) | 3.16227746 | 1.0 | 1.0 | 2.0 | 0.31622776 |

---

## 3. Phase Evolution: Does Ratio Change Across D?

For each variant and subspace, the ratio `subspace_r / full_r` is measured at
tau_init (D=0 steps applied) and at tau_final for D=1, 20, 32.

| variant | subspace | ratio@D=0 | ratio@D=1 | ratio@D=20 | ratio@D=32 | max_change |
|---------|----------|-----------|-----------|------------|------------|------------|
| original_s42 | full | 1.0 | 1.0 | 1.0 | 1.0 | 0.00e+00 |
| original_s42 | angular | 0.77459675 | 0.77459675 | 0.77459675 | 0.77459675 | 0.00e+00 |
| original_s42 | h3 | 0.31622776 | 0.31622776 | 0.31622776 | 0.31622776 | 0.00e+00 |
| original_s42 | h2 | 0.31622776 | 0.31622776 | 0.31622776 | 0.31622776 | 0.00e+00 |
| original_s42 | h2h3 | 0.44721362 | 0.44721362 | 0.44721362 | 0.44721362 | 0.00e+00 |
| original_s42 | block3_ang | 0.5477224 | 0.54772246 | 0.54772246 | 0.54772246 | 6.00e-08 |
| original_s42 | magnitude | 0.63245553 | 0.63245553 | 0.63245553 | 0.63245553 | 0.00e+00 |
| shift1_s42 | full | 1.0 | 1.0 | 1.0 | 1.0 | 0.00e+00 |
| shift1_s42 | angular | 0.77459675 | 0.77459675 | 0.77459675 | 0.77459675 | 0.00e+00 |
| shift1_s42 | h3 | 0.31622776 | 0.31622776 | 0.31622776 | 0.31622776 | 0.00e+00 |
| shift1_s42 | h2 | 0.31622776 | 0.31622776 | 0.31622776 | 0.31622776 | 0.00e+00 |
| shift1_s42 | h2h3 | 0.44721362 | 0.44721362 | 0.44721362 | 0.44721362 | 0.00e+00 |
| shift1_s42 | block3_ang | 0.5477224 | 0.54772246 | 0.54772246 | 0.54772246 | 6.00e-08 |
| shift1_s42 | magnitude | 0.63245553 | 0.63245553 | 0.63245553 | 0.63245553 | 0.00e+00 |

---

## 4. Does Radial Ratio Add Information Beyond H2/H3/CRT?

**Test:** Group samples by their CRT identity code (k%12). Measure ratio
standard deviation within each group. If samples with identical CRT code
have identical ratio (std ≈ 0), the ratio is fully determined by the
identity encoding and adds no new information.

**Variant:** original_s42  **D:** 20

| k%12 | n | ratio_mean | ratio_std | ratio_range |
|------|---|------------|-----------|-------------|
| 0 | 180 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 1 | 180 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 2 | 175 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 3 | 180 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 4 | 163 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 5 | 191 | 0.31622776 | 0.00e+00 | 0.00e+00 |
| 6 | 172 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 7 | 157 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 8 | 159 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 9 | 175 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 10 | 164 | 0.31622773 | 2.99e-08 | 0.00e+00 |
| 11 | 152 | 0.31622776 | 0.00e+00 | 0.00e+00 |

**Overall ratio stats (D=20, h3/full):**
- Mean: `0.31622776`
- Std:  `0.00e+00`
- Range: `0.00e+00`
- Predicted 1/sqrt(10): `0.31622777`

**Max within-CRT-code ratio std:** `0.00e+00`

### Finding

The radial ratio is a structural constant determined entirely by the number of harmonic pairs and blocks in the system geometry (n_pairs=6, n_blocks=4). It does not vary with sample identity, phase, D, or variant. All subspace radii are unit-norm by construction. The ratio carries zero information beyond what is already encoded by the system architecture constants.

---

## 5. Final Conclusion

**RADIAL RATIO STATUS: REDUNDANT_DERIVED_SIGNAL**

### Supporting evidence

1. All angular subspace radii are unit-norm by construction (cos²+sin²=1 per pair).
2. All magnitude entries = 1.0 (TN_ang built from unit-norm (cos,sin) pairs).
3. Full radius = sqrt(n_pairs + n_blocks) = sqrt(10) ≈ 3.162278 for ALL samples,
   ALL D values, and BOTH variants.
4. Ratio = subspace_r / full_r is a fixed constant per subspace definition,
   with std < 1e-6 across all samples (pure floating-point rounding noise).
5. Samples with identical CRT code (k%12) have indistinguishable ratios.
6. The ratio therefore encodes NO class-dependent information.

### Why the prior observation was likely an artifact

Earlier branches involved learned scaffolding (W_attn, W_pred, readout networks).
These could introduce state-dependent radial structure by weighting or projecting
tau_hyb through learned weight matrices, creating apparent radial variation.
In the current fully train-free system, no such projection exists.
The raw tau_hyb vector has constant norm by geometric construction.

**RADIAL RATIO STATUS: REDUNDANT_DERIVED_SIGNAL**