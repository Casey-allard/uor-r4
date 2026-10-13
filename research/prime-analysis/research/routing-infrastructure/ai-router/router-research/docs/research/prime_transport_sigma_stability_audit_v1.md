# Prime Transport — Sigma-Carrier Stability Audit

**Audit type:** Read-only characterization
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total states on surface:** 525,355

---

## Operators audited

| Label | Function | Cluster |
|---|---|---|
| T_b  | `torus_base_advance_component_v23`  | non-transport |
| T_x  | `composite_swap_component_v24`       | non-transport |
| T_y  | `composite_twist_component_v25`      | non-transport |
| T_c  | `coupled_torus_kick_component_v20`   | transport     |
| T_z' | `fiber_phase_lift_component_v21`     | transport     |
| T_r* | `radial_transport_component_v22`     | transport     |

---

## Definition

A state `s` is **sigma-stable** under operator `O` if:

    spin_h(O(s)) == spin_h(s)

i.e. the observable spin-H carrier is left unchanged by the operator action.
This is a weaker condition than a full fixed point (`O(s) == s`).

Within the sigma-stable subset, we additionally report the fraction of those
states that are also invariant under each other field family
(`theta`, `rho`, `tau_holonomy`, `swap_geometry`).

---

## Per-operator sigma-stable count and fraction

| Operator | Cluster | sigma-stable count | sigma-stable fraction |
|---|---|---|---|
| T_b | non-transport | 56,464 | 0.107478 |
| T_x | non-transport | 42,323 | 0.080561 |
| T_y | non-transport | 214,836 | 0.408935 |
| T_c | transport | 209,227 | 0.398258 |
| T_z' | transport | 207,484 | 0.394941 |
| T_r* | transport | 197,789 | 0.376486 |

Non-transport mean sigma-stable fraction: **0.198991**
Transport mean sigma-stable fraction:     **0.389895**

---

## Conditional invariance within sigma-stable subset

Fractions below are computed *within* the sigma-stable states for each operator.
A value of `1.000000` means every sigma-stable state is also invariant in that family.
A value of `0.000000` means no sigma-stable state is invariant in that family.

| Operator | Cluster | ss_count | theta | rho | tau | swap_geo |
|---|---|---|---|---|---|---|
| T_b | non-transport | 56,464 | 0.0000 | 1.0000 | 0.0008 | 1.0000 |
| T_x | non-transport | 42,323 | 1.0000 | 1.0000 | 0.0000 | 0.0000 |
| T_y | non-transport | 214,836 | 1.0000 | 1.0000 | 0.0168 | 0.0000 |
| T_c | transport | 209,227 | 0.0000 | 1.0000 | 0.0089 | 1.0000 |
| T_z' | transport | 207,484 | 0.0000 | 1.0000 | 0.0107 | 1.0000 |
| T_r* | transport | 197,789 | 0.3336 | 0.0000 | 0.0108 | 1.0000 |

---

## Cluster-level comparison

### Non-transport cluster {T_b, T_x, T_y}

**T_b** — sigma-stable fraction: 0.107478
  - theta invariant (within ss):    0.000000
  - rho invariant (within ss):      1.000000
  - tau invariant (within ss):      0.000779
  - swap_geo invariant (within ss): 1.000000

**T_x** — sigma-stable fraction: 0.080561
  - theta invariant (within ss):    1.000000
  - rho invariant (within ss):      1.000000
  - tau invariant (within ss):      0.000000
  - swap_geo invariant (within ss): 0.000000

**T_y** — sigma-stable fraction: 0.408935
  - theta invariant (within ss):    1.000000
  - rho invariant (within ss):      1.000000
  - tau invariant (within ss):      0.016771
  - swap_geo invariant (within ss): 0.000000

### Transport cluster {T_c, T_z', T_r*}

**T_c** — sigma-stable fraction: 0.398258
  - theta invariant (within ss):    0.000000
  - rho invariant (within ss):      1.000000
  - tau invariant (within ss):      0.008928
  - swap_geo invariant (within ss): 1.000000

**T_z'** — sigma-stable fraction: 0.394941
  - theta invariant (within ss):    0.000000
  - rho invariant (within ss):      1.000000
  - tau invariant (within ss):      0.010704
  - swap_geo invariant (within ss): 1.000000

**T_r*** — sigma-stable fraction: 0.376486
  - theta invariant (within ss):    0.333613
  - rho invariant (within ss):      0.000000
  - tau invariant (within ss):      0.010840
  - swap_geo invariant (within ss): 1.000000

---

## Interpretation

### Sigma-stable fraction comparison

The non-transport mean sigma-stable fraction (0.198991) and transport mean (0.389895) reveal whether one cluster systematically preserves the spin-H carrier more or less than the other.

### Transport coherence within sigma-stable subset

All three transport operators share **100% swap_geometry invariance** within their sigma-stable subsets — confirming that transport sigma-stable states are also semiprime/permutation-preserving, consistent with the field-signature finding that transport operators never touch swap_geometry fields.

Transport sigma-stable states have near-zero tau_holonomy invariance — even sigma-stable states are being moved by the tau carrier, consistent with tau being the primary motion axis for transport operators.

### T_b bridge status

**T_b** again shows bridge behavior: within its sigma-stable subset, swap_geometry invariance is 100% — matching the transport cluster — even though T_b belongs to the non-transport label. This is consistent with the fixed-point census finding that T_b preserves swap_geometry globally.

### What sigma-stability means for the rebuilt algebra

Sigma stability characterises the subset of states on which an operator acts
as a 'spin-carrier isometry': the observable spin-H is unchanged even though
other coordinates (tau, theta, rho, swap fields) may change.  The size and
field-composition of this subset determine which part of the bounded surface
is 'transparent' to the sigma observable under each operator.

The conditional invariance profile inside this subset reveals whether sigma
stability is correlated with or independent of stability in other field families.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **sigma-stable sub-algebra closure test**:
apply each non-transport operator to every sigma-stable state of each transport
operator and measure how often the output remains sigma-stable.
This tests whether the transport sigma-stable subsets are closed (or approximately
closed) under the action of the non-transport layer — the natural follow-on to
this characterization step.
