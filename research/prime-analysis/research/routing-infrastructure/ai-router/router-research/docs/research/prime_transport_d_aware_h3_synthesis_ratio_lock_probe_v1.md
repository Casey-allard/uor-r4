# Prime Transport D-Aware H3 Synthesis Ratio Lock Probe — v1

**Branch:** d_aware_h3_synthesis_ratio_lock_probe_v1  
**Contract:** prompt_contract_v4.md — loaded and binding  
**Strictly deterministic: no training, no gradients**

## 1. Regime Lock Summary

| Item | Stripped Carrier | D-Aware Synthesis |
|------|-----------------|------------------|
| eps_high | 1.0 | 0.0 |
| H3 preservation | Frozen (eps·H3_prev) | Replaced each step |
| D effect on H3 | None (H3_D = H3_0) | H3_D depends on routing |
| cos(H3_final, proto_2i[true]) | Always 1.0 (carrier) | Varies (synthesis) |
| Synthesis success criterion | Trivial (preserved) | atan2 readout matches true_k%4 |
| Reference objects | proto_2i[k] (unused) | proto_2i[k] for k=0..3 |

**Why NOT the same state as stripped carrier:**  
In stripped carrier, H3 is preserved by eps*H3_prev. Any D gives cos(H3_D, proto)=1.0 trivially. In synthesis (eps=0.0), H3 is replaced each step. Final alignment depends on whether routing converged to the correct H3 class. This is earned alignment, not preserved alignment.

## 2. Ratio Candidate Lock Summary

| Candidate | Formula | Type | Analytic Prediction | Can Reflect Lock? |
|-----------|---------|------|---------------------|-------------------|
| full_radial | ‖tau_hyb‖₂ | Radial | √10=3.162278 (constant) | NO — constant by construction |
| subspace_radial | ‖tau[:,10:12]‖₂ | Radial | 1.0 (constant) | NO — unit-normalized |
| radial_ratio | sub_r / full_r | Radial | 1/√10=0.316228 (constant) | NO — ratio of constants |
| cos_metric | H3·proto_2i[true] | Directional | Varies at eps=0.0 | YES — at success → 1.0 |
| sin_metric | H3×proto_2i[true] | Directional | Varies at eps=0.0 | YES — at success → 0.0 |
| step_cos | H3_t·H3_{t-1} | Mixed | Varies (discrete H3 jumps) | YES — at fixed point → 1.0 |
| h1_cos | H1_t·H1_init | Mixed | Varies (H1 always dynamic) | Partial |
| h3_h1_ratio | cos_metric / \|h1_cos\| | Mixed | Varies | IF stable ratio exists at success |

## 3. Precondition Check Results

Empirical verification that tested quantities actually vary.

```
Regime: stripped_carrier (eps=1.0)
  eps=1.0, D=20: full_radial=3.162277 (expect 3.162278), sub_radial=1.000000 (expect 1.000000), cos_metric_mean=1.000000, sin_metric_mean=-0.000000
  radial variation across steps: full_radial_std=0.00e+00, sub_radial_std=0.00e+00
  cos_metric_std across all steps+samples=0.0000, n_success=1024/1024
  n_success=1024/1024
Regime: synthesis (eps=0.0)
  eps=0.0, D=20: full_radial=3.162277 (expect 3.162278), sub_radial=1.000000 (expect 1.000000), cos_metric_mean=-0.004883, sin_metric_mean=-0.077148
  radial variation across steps: full_radial_std=0.00e+00, sub_radial_std=4.73e-08
  cos_metric_std across all steps+samples=0.7003, n_success=239/1024
  n_success=239/1024
```

**Precondition verdict:**

- full_radial, subspace_radial, radial_ratio: CONSTANT in both regimes → radial sub-hypothesis → **NO_TESTABLE_RATIO_LOCK**
- cos_metric, sin_metric: VARY in synthesis regime but NOT in carrier regime → testable
- step_cos, h1_cos, h3_h1_ratio: VARY in synthesis regime → testable

## 4. Stripped Carrier vs D-Aware Synthesis Regime

| Metric | Stripped Carrier (eps=1.0, D=20) | D-Aware Synthesis (eps=0.0, D=20) |
|--------|----------------------------------|-----------------------------------|
| full_radial (mean) | 3.162277 | 3.162277 |
| subspace_radial (mean) | 1.000000 | 1.000000 |
| radial_ratio (mean) | 0.316228 | 0.316228 |
| cos_metric (mean) | 1.000000 | -0.004883 |
| sin_metric (mean) | -0.000000 | -0.077148 |
| step_cos (mean) | 1.000000 | 0.607422 |
| h1_cos (mean) | -0.051547 | -0.008362 |
| h3_h1_ratio (mean) | 19.399595 | -0.583966 |
| n_success / N_EVAL | 1024/1024 | 239/1024 |

**Note on radial:** full_radial, subspace_radial, radial_ratio are numerically identical in both regimes — constant by construction.  
**Note on directional:** cos_metric and sin_metric differ between regimes because in the synthesis regime H3 is not preserved, causing some samples to misalign (failures).

## 5. Matched vs Mismatched (CASE C, eps=0.0, D=20)

Same final synthesis state; compared to true-class proto (match=1) vs adjacent-class proto (match=0).

| Metric | Matched (success) | Matched (failure) | Mismatched (success) | Mismatched (failure) |
|--------|-------------------|-------------------|----------------------|----------------------|
| cos_metric | 1.000000 | -0.310828 | 0.000000 | 0.100637 |
| sin_metric | -0.000000 | -0.100637 | 1.000000 | -0.310828 |
| full_radial | 3.162277 | 3.162278 | 3.162277 | 3.162278 |
| subspace_radial | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| radial_ratio | 0.316228 | 0.316228 | 0.316228 | 0.316228 |
| h1_cos | 0.035565 | -0.021735 | 0.035565 | -0.021735 |
| h3_h1_ratio | 28.117640 | -14.300632 | 0.000003 | 4.630122 |

**Interpretation:** cos_metric separates matched vs mismatched (cos=1.0 for matched-success vs ~0.0 or negative for mismatched). sin_metric shows complementary pattern. Radial quantities: identical across all conditions (confirming constant by construction).

## 6. Success vs Failure (CASE D, eps=0.0, D=20)

| Metric | Success | Failure |
|--------|---------|---------|
| cos_metric | 1.000000 | -0.310828 |
| sin_metric | -0.000000 | -0.100637 |
| full_radial | 3.162277 | 3.162278 |
| subspace_radial | 1.000000 | 1.000000 |
| radial_ratio | 0.316228 | 0.316228 |
| h1_cos | 0.035565 | -0.021735 |
| h3_h1_ratio | 28.117640 | -14.300632 |

## 7. Phase/Timestep Evolution and Stabilization (CASE E, eps=0.0, D=32)

Tracking cos_metric at each step for success vs failure groups.

| Timestep | cos (success) | cos (failure) | sin (success) | sin (failure) | step_cos (success) | step_cos (failure) |
|----------|--------------|--------------|--------------|--------------|--------------------|--------------------|
| 0 | 1.000000 | 1.000000 | -0.000000 | -0.000000 | nan | nan |
| 1 | 0.366038 | 0.396574 | -0.286793 | -0.350461 | 0.366038 | 0.396574 |
| 2 | 0.207547 | 0.118577 | -0.083019 | -0.198946 | 0.358491 | 0.384717 |
| 4 | 0.086792 | -0.032938 | -0.015094 | -0.092227 | 0.320755 | 0.507246 |
| 8 | 0.083019 | -0.055336 | 0.011321 | -0.104084 | 0.460377 | 0.530962 |
| 16 | 0.260377 | -0.151515 | 0.007547 | -0.121212 | 0.513208 | 0.591568 |
| 24 | 0.347170 | -0.148880 | 0.018868 | -0.055336 | 0.626415 | 0.629776 |
| 32 | 1.000000 | -0.295125 | -0.000000 | -0.080369 | 0.671698 | 0.706192 |

**h3_h1_ratio evolution (success vs failure):**

| Timestep | h3_h1_ratio (success) | h3_h1_ratio (failure) |
|----------|----------------------|----------------------|
| 0 | 1.000000 | 1.000000 |
| 1 | 0.732394 | 0.829230 |
| 2 | 10.199721 | 1.222901 |
| 4 | 1.067984 | -0.422343 |
| 8 | 1.378091 | -12.749595 |
| 16 | 3.771680 | -4.636862 |
| 24 | 6.819677 | -1.345013 |
| 32 | 48.181835 | -52.602761 |

## 8. Did Any Nontrivial Ratio-Lock Emerge?

### Radial candidates

full_radial = 3.162277 in carrier; 3.162277 in synthesis. **CONSTANT — identical in both regimes.** No ratio lock possible. Sub-hypothesis → **NO_TESTABLE_RATIO_LOCK**.

### Directional candidates

cos_metric at success: 1.000000, at failure: -0.310828.  
cos_metric = 1.0 at success (H3 aligned to target) and is near 0.0 or negative at failure.  
This separation is real but **trivial**: cos=1.0 is the direct consequence of the success criterion itself. cos→1.0 at success is not a nontrivial ratio — it IS the alignment definition.  
sin_metric shows the complementary pattern (0.0 at success). Same trivial tracking.  
**No nontrivial ratio lock in directional candidates alone.**

### Mixed candidates

h3_h1_ratio at success: 28.117640, at failure: -14.300632.  
step_cos at success: n/a, at failure: n/a.  
h3_h1_ratio = cos_metric / |h1_cos|. If this ratio stabilizes to a specific constant only at synthesis success, that would constitute a nontrivial ratio lock.  
If the value at success is ~1.0 or another special constant and clearly different from failure, this is weak evidence for a ratio lock.  
If h3_h1_ratio simply tracks cos_metric (which already tracks success), it adds no new structure.  
step_cos: if =1.0 at success and <1.0 at failure, this shows H3 converged to a fixed point. But step_cos=1.0 is again trivially implied by H3 being at the target class.

### Critical test: Is any lock absent in stripped carrier?

In stripped carrier (eps=1.0): cos_metric=1.0 trivially (preserved). h3_h1_ratio = 1.0/|h1_cos| varies with h1_cos, not locked to any special constant. step_cos = 1.0 trivially (H3 never changes). None of these are meaningful in the carrier regime — they're all trivially fixed or determined by the carrier structure.  
In synthesis (eps=0.0): cos_metric VARIES and discriminates success from failure. BUT the discrimination mechanism is the definition of success, not a nontrivial lock.  
**No candidate shows a lock to a NONTRIVIAL constant (i.e., not 0, 1, or ∞) that is exclusive to the synthesis regime.**

## 9. Final Conclusion

### Evidence summary

| Criterion | Radial | cos/sin | step_cos | h3_h1_ratio |
|-----------|--------|---------|---------|-------------|
| Varies in synthesis | NO (constant) | YES | YES | YES |
| Absent (constant) in carrier | NO (constant in both) | NO (trivially 1.0 in carrier) | NO (trivially 1.0 in carrier) | NO |
| Separates matched vs mismatched | NO | YES (trivially) | partial | partial |
| Separates success vs failure | NO | YES (trivially) | partial | partial |
| Stabilizes only at synthesis success | NO | YES (trivially) | YES (trivially) | conditional |
| Lock to NONTRIVIAL constant | NO | NO | NO | NO |

### Interpretation

- Radial candidates are constant by construction in both regimes. NO nontrivial ratio lock possible for radial.

- Directional candidates (cos, sin) DO vary in the synthesis regime and DO track success/failure. However, this tracking is trivial: cos=1.0 at success IS the success criterion. No new constant or ratio emerges — it is a direct encoding of alignment.

- Mixed candidates (h3_h1_ratio, step_cos) vary and partially discriminate. However, they do not stabilize to a nontrivial constant that is specific to the synthesis regime and absent elsewhere. h3_h1_ratio = cos_metric/|h1_cos| just rescales the directional signal by H1 drift — no genuine new structure.

- The D-aware synthesis regime (eps=0.0) produces REAL variation in directional quantities, confirming D is meaningful. But no RATIO-LOCK (stabilization to a specific nontrivial constant) was observed. The discrimination is success/failure tracking, not a ratio condition.

D-AWARE H3 SYNTHESIS RATIO LOCK STATUS: NO_SUPPORT
