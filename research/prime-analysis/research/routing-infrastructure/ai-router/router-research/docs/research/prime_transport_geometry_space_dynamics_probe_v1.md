# Prime Transport Geometry Space Dynamics Probe v1

**Probe:** geometry_space_dynamics_probe_v1  
**Contract:** prompt_contract_v4.md  
**Branch:** geometry_space_dynamics_probe_v1  
**Layer tested:** Layer 1 — Ambient/operative space (dynamic regime)  
**N samples:** 1024  (256 per class)  
**eps_values:** [1.0, 0.75, 0.5]  

---

## 1. Recap: Prior Invariant Result (Frozen Regime)

Probe `geometry_space_type_probe_v1` was run under **eps_high = 1.0** (frozen).
Under this condition, H3 is algebraically preserved at every step by construction:

```
h3_final = 1.0 * h3_initial + 0.0 * h3_proposed = h3_initial  [exact]
```

Result: **SPACE_INVARIANT_SIGNAL** — all three ambient space candidates
(CURRENT_BASELINE/R^16, R4_FLAT/R^4, REDUCED_BLOCK/R^12) produced identical
separation_delta = 1.333333 with mean_matched_cos = 1.0 and
mean_mismatched_cos = −0.333333.

**Interpretation at the time:** This result is conditional on eps_high=1.0.
Under eps_high=1.0, H3 invariance is a tautology — algebraic, not empirical.
The frozen-regime invariance cannot be promoted to the dynamic regime.

**This probe** extends the test to eps_high ∈ {1.0, 0.75, 0.5}.

---

## 2. Space Representations Tested

| Space | Dims | H3 subspace dims | Notes |
|:------|-----:|:-----------------|:------|
| CURRENT_BASELINE | 16 | [10, 11] | 12 angular + 4 magnitude |
| R4_FLAT | 4 | [2, 3] | h2+h3 pairs of dominant block only |
| REDUCED_BLOCK | 12 | [10, 11] | 12 angular only (magnitude removed) |

---

## 3. Controlled Dynamics Definition

Transport formula applied to H3 subspace of each state:

```
h3_final = eps_high * h3_initial + (1 - eps_high) * h3_proposed
```

Where **h3_proposed** = H3 encoding of class (k+1) % 4 — the adjacent class.
This is deterministic, training-free, and space-independent.

| eps_high | Interpretation |
|:---------|:---------------|
| 1.0 | Frozen — algebraic control (identical to prior probe) |
| 0.75 | 25% contamination from adjacent class H3 |
| 0.5 | 50% contamination from adjacent class H3 |

**Note:** eps = 0.0 excluded (full replacement — too destructive).

---

## 4. Structural Equivalence Check

**Status:** PASS

- Identical 1024 samples across all space candidates
- Identical prototype construction: class tokens {0,1,2,3}, apply_anchor_two_i applied
- H3 initial values: baseline vs R4_FLAT max_diff = 0.0
- H3 initial values: baseline vs REDUCED_BLOCK max_diff = 0.0
- Proposed H3 values: space-independent (derived from full-state build, same for all spaces)
- Proposed H3 finite: True

---

## 5. Dynamic Test Results Table

| Space | eps | matched_cos | mismatched_cos | separation_delta |
|:------|----:|------------:|---------------:|-----------------:|
| CURRENT_BASELINE | 1.0 | 1.000000 | -0.333333 | 1.333333 |
| CURRENT_BASELINE | 0.75 | 0.948683 | -0.316228 | 1.264911 |
| CURRENT_BASELINE | 0.5 | 0.707107 | -0.235702 | 0.942809 |
| R4_FLAT | 1.0 | 1.000000 | -0.333333 | 1.333333 |
| R4_FLAT | 0.75 | 0.948683 | -0.316228 | 1.264911 |
| R4_FLAT | 0.5 | 0.707107 | -0.235702 | 0.942809 |
| REDUCED_BLOCK | 1.0 | 1.000000 | -0.333333 | 1.333333 |
| REDUCED_BLOCK | 0.75 | 0.948683 | -0.316228 | 1.264911 |
| REDUCED_BLOCK | 0.5 | 0.707107 | -0.235702 | 0.942809 |

---

## 6. Comparison Across eps Values

separation_delta by space and eps:

| Space | eps=1.0 | eps=0.75 | eps=0.5 |
|:------|--------:|---------:|--------:|
| CURRENT_BASELINE | 1.333333 | 1.264911 | 0.942809 |
| R4_FLAT | 1.333333 | 1.264911 | 0.942809 |
| REDUCED_BLOCK | 1.333333 | 1.264911 | 0.942809 |

Spread (max − min separation_delta) across spaces at each eps:

- eps=1.0: spread = 0.00000000
- eps=0.75: spread = 0.00000000
- eps=0.5: spread = 0.00000000

---

## 7. Does Ambient Space Matter Under Dynamic Conditions?

**NO — ambient space does NOT matter under dynamic conditions.**

The separation_delta values are consistent across all three ambient space
representations (CURRENT_BASELINE, R4_FLAT, REDUCED_BLOCK) at every eps value.
The spread across spaces remains below the threshold (0.05) for all eps.

**Why is this the case?**

The measurement (`cos_metric` on the H3 subspace) depends only on H3 dims.
The transport is applied only to H3 dims, using proposed H3 values that are
space-independent (derived identically for all spaces from the same encoding).

Changing the ambient embedding (R^4, R^12, R^16) does not alter:
  - the H3 initial values (numerically identical across spaces, verified)
  - the H3 proposed values (same construction for all spaces)
  - the transport result (same formula applied to same values)
  - the cos_metric measurement (same H3 subspace content)

**Implication for Layer 1:** The geometric separation signal remains
space-invariant even under partial H3 replacement (eps < 1.0).
Invariance extends from the frozen regime into the dynamic regime.

**Boundary note:** This result is specific to the tested dynamic mechanism
(adjacent-class H3 perturbation). It does not rule out space sensitivity
under alternative dynamic regimes where non-H3 ambient dimensions
enter the transport computation.

---

## 8. Critical Question Answers

**Q1: Does ambient space begin to matter once H3 is no longer perfectly preserved?**
NO. All three space representations produce identical separation_delta at eps=0.75 and eps=0.5.

**Q2: Is invariance only a property of the frozen regime?**
NO, under this test. Invariance holds across the tested dynamic eps values.
The caveat is that the proposed H3 perturbation is space-independent by construction,
so the non-H3 ambient content never enters the transport or measurement.

**Q3: Does any space degrade slower or preserve signal better?**
NO. Signal degradation with decreasing eps is identical across all three spaces.
The degradation trajectory is determined by the H3 content alone.

---

## 9. Locked Variables Confirmation

| Variable | Value | Changed? |
|:---------|:------|:---------|
| Basis | apply_anchor_two_i (2i frame) | NO |
| Operator | cos_metric | NO |
| Comparison subspace | H3 pair (per-space equivalent) | NO |
| H3 initial construction | Identical to prior probe | NO |
| Prototype construction | Identical to prior probe | NO |
| eps_high | swept over [1.0, 0.75, 0.5] | CONTROLLED (variable under test is SPACE) |
| Training | None | NO |
| New constants | None | NO |
| Radial structure | Unchanged | NO |

---

GEOMETRY SPACE DYNAMICS RESULT: SPACE_INVARIANT_UNDER_DYNAMICS