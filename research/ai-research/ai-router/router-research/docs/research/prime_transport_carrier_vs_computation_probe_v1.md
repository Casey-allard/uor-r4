# Prime Transport Carrier vs Computation Probe v1

**Branch:** carrier_vs_computation_probe_v1  
**Strictly train-free: no weights, no gradients, no label lookups**

## Mechanism Lock Summary

### Current mechanism (H3 carrier readout)

1. `prepare_tables()` encodes all states via `convert_onehot_to_angular_multi()`
   then `apply_anchor_two_i()` rotates every pair (cos,sin)→(−sin,cos).
2. For block-3 (dominant cyclic block, period=12, n_h=3):
   - h1 at indices 6,7: **replaced** each step (not carrier)
   - h2 at indices 8,9: **preserved** (eps_high=1.0), encodes k%6 (period-6)
   - h3 at indices 10,11: **preserved** (eps_high=1.0), encodes k%4 (period-4)
3. `apply_split_transport(eps_high=1.0)`: for h≥2, blending = 1.0×prev → frozen.
4. Readout: `atan2(tau_final[:,10], tau_final[:,11])` → k%4 → class.

### Why D=0 works

H3 is present in `tau_init` before any trajectory steps.
No dynamics contribute correctness. The class is embedded at state construction time.

### Carrier vs computation definition

- **CARRIER**: class exists in a preserved initial component; readout = extraction
  of a pre-existing signal; no trajectory dynamics needed for correctness.
- **COMPUTATION**: class emerges from dynamics; NOT accessible from any preserved
  component of the initial state.

## 4-Case Result Table

| Case | acc (original_s42) | acc (shift1_s42) | H3_err | Note |
|------|--------------------|------------------|--------|------|
| A — baseline H3 readout | 1.0 | 1.0 | 0.00e+00 | baseline; H3 preserved; direct carrier readout |
| B — H3 destroyed | 0.255859 | 0.255859 | 0.00e+00 | H3 zeroed at tau_init; eps_high=1.0 preserves zero; atan2(0,0)=0 → pre |
| C.h3 — H3 readout on H2 target | 0.329102 | 0.329102 | 0.00e+00 | H3 readout (k%4) applied to H2 target (k%6); wrong carrier → acc~=4/12 |
| C.h2 — H2 readout on H2 target | 1.0 | 1.0 | 0.00e+00 | H2 readout (k%6) on H2 target (k%6); H2 also preserved by eps_high=1.0 |
| D — CRT(H2,H3) on k%12 target | 1.0 | 1.0 | 0.00e+00 | Joint CRT from H2(k%6)+H3(k%4) → k%12; both carriers preserved; 12-cla |

## Per-Case Interpretation

- **CASE A (acc=1.0)**: H3 carrier present and readable. Baseline confirmed.
- **CASE B (acc=0.255859)**: H3 zeroed at tau_init; eps_high=1.0 preserves the zero;
  atan2(0,0)=0 → all predictions = constant(partition_offset). Accuracy collapses
  to fraction_of_dominant_class (~0.25). H3 is the load-bearing carrier.
- **CASE C.h3 (acc=0.329102)**: H3 readout gives k%4 (0..3); H2 target is k%6 (0..5).
  Match only for k=0,1,2,3 (first 4 of 12 positions) → acc≈4/12=0.333.
  H3 readout is carrier-specific and cannot adapt to a different period.
- **CASE C.h2 (acc=1.0)**: H2 (indices 8,9) also preserved by eps_high=1.0.
  H2 readout correctly extracts k%6. Still trivial carrier readout — different harmonic.
- **CASE D (acc=1.0)**: Both H2 and H3 preserved. CRT(k%4, k%6)→k%12 is
  geometrically valid (all 12 pairs unique). Uniquely recovers full cycle position.
  Still trivial carrier readout — from two preserved harmonics jointly.

## Conclusion

**CARRIER VS COMPUTATION STATUS: TRIVIAL CARRIER**

CASE A succeeds (carrier readout works). CASE B collapses (destroying H3 destroys accuracy). CASE C.h3 fails (H3 readout cannot solve H2 target). CASE C.h2 succeeds (H2 carrier also trivially readable). CASE D succeeds (joint CRT from two preserved carriers). No case demonstrates computation: all successes are carrier extractions.

## Honesty Section

### What was actually destroyed (CASE B)

- `tau_prev[:, 10:12]` was set to 0.0 before the trajectory loop.
- `eps_high=1.0` then froze this zero through all D steps.
- No other component of tau_init was modified.
- The trajectory dynamics (h1) continued to update normally.
- The collapse to ~0.25 is exact: all predictions = partition_offset,
  because atan2(0,0)=0 → k_mod4=0 → pred = (0+offset)%4.

### What target was changed (CASE C)

- H3 target (k%4, period-4, 4 classes) → H2 target (k%6, period-6, 6 classes).
- H2 target computed from raw cycle position k via `tau0_oh[:,9:21].argmax()%6`.
- No partition offset applied to H2 target (period-6 has no analogous shift).
- C.h3 attempt: same H3 readout mechanism, wrong carrier → fails at ~0.333.
- C.h2 attempt: different readout (H2 phase), correct carrier → acc=1.0.

### Why the result does not justify calling this nontrivial computation

- All successful cases (A, C.h2, D) are extractions of preserved harmonics
  from the initial geometric state.
- H3, H2, and their joint CRT combination are all present in `tau_init`
  BEFORE any trajectory step (D=0 suffices for all).
- The transport dynamics update only h1 (the fundamental harmonic).
  h1 has no role in any of these readouts.
- The mechanism is equivalent to: encode the answer into the initial state
  representation, preserve it exactly, then extract it by phase readout.
- This is encoding + extraction, not computation.
- No case demonstrates that the answer is derived from transport dynamics.
