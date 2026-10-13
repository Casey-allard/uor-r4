# H³ Tangent Full Mechanism Migration Probe — v1

## 1. Full Mechanism Lock Summary

| Mechanism | Old Basis | Status | H³ Tangent Equivalent |
|-----------|-----------|--------|----------------------|
| Subspace projection | cos(h3_2i, raw_proto) | **NOT MIGRATED** → FALSE NULL | cos(h3_2i, 2i_proto) |
| Radial measurement | L2 ‖tau‖₂ = √10 (constant) | NOT MIGRATED — no effect (constant) | Same constant |
| Directional comparison | cos(Δθ_H3) | NOT MIGRATED (old cos language) | sin(Δθ_H3) = cross product |
| Phase-to-phase delta | cos(θ_t − θ_{t-1}) | NOT MIGRATED | sin(θ_t − θ_{t-1}) |
| Routing similarity | cos(tau, TN_ang_2i) | **ALREADY UPGRADED** (H³-correct) | Unchanged |
| State initialization | apply_anchor_two_i | **ALREADY UPGRADED** | Unchanged |

**Root cause of false null**: cos(2i-rotated state, raw prototype) = 0 for EVERY class k.
This is a 90° rotation mismatch: `apply_anchor_two_i` shifts every class 90°,
so dot product with the unrotated reference is exactly zero.

## 2. Explicit Migration Map: Old Basis → H³ Tangent Equivalent

| Object | Old Basis | H³ Tangent Migrated |
|--------|-----------|---------------------|
| h3 raw proto[k] | [cos(πk/2), sin(πk/2)] | [−sin(πk/2), cos(πk/2)] (2i-rotated) |
| Comparison metric (same frame) | cos/dot product | cos/dot product (valid when both 2i) |
| Comparison metric (displacement) | cos(Δθ) ← alignment | sin(Δθ) ← displacement |
| Matched detection | cos = 1.0 (aligned) | |sin| = 0.0 (no displacement) |
| Mismatched detection | cos = 0.0 (adj) | |sin| = 1.0 (adj) |
| Radial norm | L2 √10 | L2 √10 (same — unit-normalized pairs) |
| Routing TN | 2i-rotated (already H³) | Unchanged |

## 3. Hybrid System vs Fully Migrated System

### Radial Quantities (both cases — constant, no comparison signal)

| Metric | Value | Notes |
|--------|-------|-------|
| full_radial | 3.162277 | = √(n_pairs + n_blocks) = √10 ≈ 3.162 (structural constant) |
| subspace_radial (H3) | 1.000000 | = 1.0 (unit-normalized H3 pair) |
| radial_ratio | 0.316228 | = 1/√10 ≈ 0.316 (structural constant) |

**Note**: Radial migration produces NO change — all angular pairs are unit-normalized
in the canonical transport system. The H³ tangent radial equals the L2 radial here.

### Direction Metric at t=0 (initial state, no dynamics)

| Case | direction_metric | Expected | Result |
|------|-----------------|----------|--------|
| A (HYBRID): cos(h3_2i, raw_proto) | -0.000000 | 0.0 (FALSE NULL) | ✓ CONFIRMED |
| B (MIGRATED): cos(h3_2i, 2i_proto) | 1.000000 | 1.0 (TRUE SIGNAL) | ✓ CONFIRMED |
| B sin displacement: |sin(h3_2i, 2i_proto)| | 0.000000 | 0.0 (matched → no displacement) | ✓ CONFIRMED |

**FALSE NULL in CASE A is systematic**: every class k gives cos(2i_k, raw_k) = 0
because apply_anchor_two_i shifts every class exactly 90°, making it orthogonal to its
own raw prototype. The cos comparison machinery cannot recover the matched signal.

**FULL MIGRATION in CASE B**: migrating the reference to the 2i frame makes
cos(2i_state, 2i_proto) = 1.0 for all matched classes. Signal fully restored.

## 4. Matched vs Mismatched Table (CASE C, eps=1.0, D=20)

| Metric | Matched (same class) | Mismatched (diff class) | Separation |
|--------|---------------------|------------------------|------------|
| cos(h3_A, h3_B) | 1.0000 | -0.3270 | 1.3270 |
| |sin(h3_A, h3_B)| | 0.0000 | 0.6730 | 0.6730 |

**Interpretation**:
- cos: matched=1.0 (perfectly aligned), mismatched varies (0.0 for adjacent, −1.0 for opposite)
- |sin|: matched=0.0 (no displacement), mismatched varies (1.0 for adjacent, 0.0 for opposite)
- Both metrics discriminate matched vs mismatched; cos has broader mismatch coverage.
- Separation: cos=1.3270, sin=0.6730

## 5. Success/Alignment Discrimination Table (CASE D, eps=0.0)

| Outcome | n | direction_new (cos,2i) | h3_sin_disp | Notes |
|---------|---|----------------------|-------------|-------|
| Success (class preserved) | 2712 | 1.0000 | 0.0000 | H3 class matches init |
| Failure (class changed)   | 360 | -0.0000 | 1.0000 | H3 class drifted |
| Separation                | — | 1.0000 | 1.0000 | |

**Notes on CASE D**:
- eps_high=0.0: all harmonics including H3 are replaced at each transport step.
- cos routing (H³-correct) tends to keep state at its initial class → high success rate.
- When failures occur, direction_new drops below 1.0; sin_disp rises above 0.0.
- Both metrics discriminate success from failure when failures are present.

## 6. Did Full H³ Tangent Migration Change the Result?

**Yes — and the change is categorical, not incremental.**

In the HYBRID PARTIAL system (CASE A):
- The comparison support mechanism (cos with raw prototype) gives direction_metric = 0.0
  for EVERY matched class at EVERY timestep.
- This is a SYSTEMATIC FALSE NULL — not noise, not partial signal, but exactly zero
  for all 1024 evaluation samples across all D and eps_high configurations.
- The FALSE NULL makes it IMPOSSIBLE to distinguish matched from any mismatch
  using the hybrid comparison mechanism.

In the FULLY MIGRATED system (CASE B):
- Migrating prototypes to the 2i frame (matching the state frame) restores
  direction_metric = 1.0 for all matched classes at all timesteps.
- Matched vs mismatched discrimination is now clear (matched=1.0, others≤0.0).
- The sin displacement metric confirms: matched has |sin|=0 (no displacement from anchor),
  while mismatched inputs are displaced.

**The migration is not optional**: the HYBRID system cannot produce comparison signal
because the mismatch between state frame (2i) and reference frame (raw) systematically
zeros out the cos projection for ALL classes simultaneously.

**Routing was already H³-correct** (2i TN_ang, cos nearest-neighbor) and was not changed.
**Radial metrics are structural constants** (√10) — no change from migration.
**The decisive migration is in the reference frame of comparison prototypes.**

## 7. Final Conclusion

H3 TANGENT FULL MIGRATION STATUS: STRONG_SUPPORT

### Evidence Summary
- FALSE NULL confirmed: cos(h3_2i, raw_proto) = -0.000000 ≈ 0.0 for all matched pairs
- SIGNAL RESTORED: cos(h3_2i, 2i_proto) = 1.000000 ≈ 1.0 for all matched pairs
- MATCHED/MISMATCHED separation: cos=1.3270 under migrated framework
- SUCCESS discrimination: direction_new=1.0000 (success) vs -0.0000 (failure)
- N_EVAL=1024 | D_SWEEP=[1, 5, 20] | EPS_SWEEP=[1.0, 0.0]

### Limitations
- FALSE NULL is a single-mechanism result (prototype frame mismatch); routing was already H³-correct.
- Radial metrics are structural constants; no radial migration effect is measurable.
- Verdict applies to this specific representation (BLOCKS_A, VOCAB=4, H3 harmonic).
- One branch — do NOT promote to canon.
