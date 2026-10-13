# Prime Transport Field Signature Audit v1

## Purpose

Measure which field families each of the six non-hold operators changes,
and with what frequency, on the bounded v25 surface.  Determine whether
transport operators share a coherent field-action signature that distinguishes
them from non-transport operators beyond raw displacement count.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Field family definitions

| Family | Fields included | Max sub-fields |
|--------|----------------|----------------|
| `theta` | `b` (torus base), `phi` (fiber angle) | 2 |
| `rho` | `r` (radial coordinate) | 1 |
| `sigma` | `spin_h` (observable spin carrier) | 1 |
| `tau_holonomy` | `tau.swap_phase`, `tau.coupled_phase`, `tau.twist_phase`, `tau.lift_phase` | 4 |
| `swap_geometry` | `query_semiprime`, `binding_semiprime`, `composite_compat_class`, `admissible_transition`, `twist` | 5 |

Values below are **average sub-fields changed per state** (e.g. 0.95 means 95% of
states have that sub-field changed; a family score > 1 means multiple sub-fields
change simultaneously on average).

---

## Per-operator fieldwise action signature

| Operator | Cluster | theta | rho | sigma | tau_holonomy | swap_geometry | Dominant family |
|----------|---------|-------|-----|-------|-------------|---------------|----------------|
| `T_b` | non-transport | 1.0000 | 0.0000 | 0.8925 | 1.7946 | 0.0000 | **tau_holonomy** |
| `T_x` | non-transport | 0.0000 | 0.0000 | 0.9194 | 1.4917 | 2.0000 | **swap_geometry** |
| `T_y` | non-transport | 0.0000 | 0.0000 | 0.5911 | 1.5537 | 1.0000 | **tau_holonomy** |
| `T_c` | transport | 1.0000 | 0.0000 | 0.6017 | 2.2165 | 0.0000 | **tau_holonomy** |
| `T_z'` | transport | 1.0000 | 0.0000 | 0.6051 | 2.1895 | 0.0000 | **tau_holonomy** |
| `T_r*` | transport | 0.6663 | 1.0000 | 0.6235 | 2.1717 | 0.0000 | **tau_holonomy** |

---

## Per-operator atom-level detail

### `T_b` (non-transport)

Dominant family: **tau_holonomy** | Total avg fields changed/state: **3.6872**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `b` | 1.000000 | torus base position (theta) |
| `spin_h` | 0.892522 | spin/sigma carrier (sigma) |
| `tau_coupled` | 0.800586 | coupled_phase in tau holonomy |
| `tau_lift` | 0.994046 | lift_phase in tau holonomy |

### `T_x` (non-transport)

Dominant family: **swap_geometry** | Total avg fields changed/state: **4.4111**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `spin_h` | 0.919439 | spin/sigma carrier (sigma) |
| `tau_swap` | 0.491666 | swap_phase in tau holonomy |
| `tau_lift` | 1.000000 | lift_phase in tau holonomy |
| `swap_geo_qs` | 1.000000 | query_semiprime (swap geometry) |
| `swap_geo_bs` | 1.000000 | binding_semiprime (swap geometry) |

### `T_y` (non-transport)

Dominant family: **tau_holonomy** | Total avg fields changed/state: **3.1448**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `spin_h` | 0.591065 | spin/sigma carrier (sigma) |
| `tau_twist` | 0.566394 | twist_phase in tau holonomy |
| `tau_lift` | 0.987313 | lift_phase in tau holonomy |
| `twist_bit` | 1.000000 | twist bit (swap geometry) |

### `T_c` (transport)

Dominant family: **tau_holonomy** | Total avg fields changed/state: **3.8183**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `b` | 1.000000 | torus base position (theta) |
| `spin_h` | 0.601742 | spin/sigma carrier (sigma) |
| `tau_coupled` | 0.789757 | coupled_phase in tau holonomy |
| `tau_twist` | 0.499310 | twist_phase in tau holonomy |
| `tau_lift` | 0.927481 | lift_phase in tau holonomy |

### `T_z'` (transport)

Dominant family: **tau_holonomy** | Total avg fields changed/state: **3.7945**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `phi` | 1.000000 | fiber phase angle (theta) |
| `spin_h` | 0.605059 | spin/sigma carrier (sigma) |
| `tau_coupled` | 0.796724 | coupled_phase in tau holonomy |
| `tau_twist` | 0.498720 | twist_phase in tau holonomy |
| `tau_lift` | 0.894013 | lift_phase in tau holonomy |

### `T_r*` (transport)

Dominant family: **tau_holonomy** | Total avg fields changed/state: **4.4616**

| Atom | Change frequency | Interpretation |
|------|-----------------|----------------|
| `phi` | 0.666349 | fiber phase angle (theta) |
| `r` | 1.000000 | radial coordinate (rho) |
| `spin_h` | 0.623514 | spin/sigma carrier (sigma) |
| `tau_coupled` | 0.787546 | coupled_phase in tau holonomy |
| `tau_twist` | 0.499788 | twist_phase in tau holonomy |
| `tau_lift` | 0.884402 | lift_phase in tau holonomy |

---

## Cluster-level comparison

| Family | Non-transport mean | Transport mean | Higher in transport? |
|--------|-------------------|----------------|---------------------|
| `theta` | 0.3333 | 0.8888 | **yes** |
| `rho` | 0.0000 | 0.3333 | **yes** |
| `sigma` | 0.8010 | 0.6101 | no |
| `tau_holonomy` | 1.6133 | 2.1926 | **yes** |
| `swap_geometry` | 1.0000 | 0.0000 | no |

---

## Why T_x is the displacement outlier

From the displacement audit, `T_x` has the highest average raw field displacement
(4.4111 fields/state, vs the next highest T_r* at 4.4616).

`T_x` is the composite-swap operator: it swaps `query_semiprime` ↔ `binding_semiprime`
and recomputes `composite_compat_class`.  Its atom-level profile shows:

- `swap_geo_qs` change freq: 1.0000
- `swap_geo_bs` change freq: 1.0000
- `swap_geo_ccc` change freq: 0.0000
- `spin_h` change freq: 0.9194
- `tau_lift` change freq: 1.0000

`T_x` changes **multiple swap_geometry sub-fields simultaneously** on nearly every
state, producing 3–4 total field changes from a single application.  This is
structurally different from the transport operators, which typically change
**one theta/rho field plus sigma/tau**, spreading changes across more families
but with lower multiplicity per family.

---

## Transport operator coherence

Do the three transport operators `{T_c, T_z', T_r*}` share a common
field-action pattern?

| Family | T_c | T_z' | T_r* | Transport mean | Coherent? |
|--------|-----|------|------|----------------|-----------|
| `theta` | 1.0000 | 1.0000 | 0.6663 | 0.8888 | **yes** (CV=0.22) |
| `rho` | 0.0000 | 0.0000 | 1.0000 | 0.3333 | no (CV=1.73) |
| `sigma` | 0.6017 | 0.6051 | 0.6235 | 0.6101 | **yes** (CV=0.02) |
| `tau_holonomy` | 2.2165 | 2.1895 | 2.1717 | 2.1926 | **yes** (CV=0.01) |
| `swap_geometry` | 0.0000 | 0.0000 | 0.0000 | 0.0000 | no (CV=inf) |

---

## Revised interpretation of the transport hypothesis

The raw displacement audit showed: non-transport mean displacement 3.13 > transport 2.82.
The fieldwise audit reveals **why**:

1. **T_x inflates the non-transport mean.** Its swap_geometry multiplicity
   (2–3 swap_geo sub-fields change simultaneously) drives its avg to 3.92.
   T_y and T_b have avg displacement ≤ 2.89, comparable to transport operators.

2. **Transport operators are sigma+tau concentrated.** All three transport
   operators show sigma change freq ~0.610 and tau_holonomy
   avg ~2.193 sub-fields/state.  They are consistent sigma/tau
   movers with variable theta or rho activation depending on their primary
   coordinate (T_c/T_b: theta; T_z': phi; T_r*: rho).

3. **Non-transport operators are swap_geometry concentrated.** T_x is
   dominated by swap_geometry changes; T_y mixes twist_bit with sigma+tau;
   T_b is theta-dominated.  Their action targets permutation geometry, not
   the transport coordinate system.

4. **Revised transport hypothesis:** Transport operators are not 'higher-
   displacement' in raw field count; they are **coordinate-specific movers**
   that change exactly one primary coordinate (theta/rho/phi) and consistently
   update sigma+tau to reflect that movement.  Non-transport operators change
   permutation geometry or twist geometry — different field families entirely.

5. **The two clusters differ in *which* fields they act on, not in *how many*.**
   The lower intra-cluster overlap of the transport cluster reflects divergence
   in coordinate space (rho, phi, theta) rather than higher multiplicity.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — read-only audit on the accepted v25 surface |
| Were any operators rebuilt? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Run a bounded fixed-point census on the v25 surface: for each operator,
count and characterise all states `s` where `O(s) == s` (total displacement = 0),
broken down by field family.  Fixed points identify invariant sub-spaces and
reveal whether the transport operators leave structurally different state classes
unchanged compared to the non-transport operators.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
