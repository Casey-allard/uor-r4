# Prime Transport — Sigma-Stable Sub-Algebra Closure Test

**Audit type:** Read-only closure characterization
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total states on surface:** 525,355

---

## Setup

For each transport operator `T_src ∈ {T_c, T_z', T_r*}`:

1. Compute the **sigma-stable subset** `SS(T_src) = { s : spin_h(T_src(s)) == spin_h(s) }`
2. For each non-transport operator `N_act ∈ {T_b, T_x, T_y}`:
   - Apply `N_act` to each `s ∈ SS(T_src)` to get `s' = N_act(s)`
   - Check whether `s'` is **still sigma-stable under T_src**: `spin_h(T_src(s')) == spin_h(s')`
   - Report `preserved_count / |SS(T_src)|`

---

## Sigma-stable subset sizes (from prior audit)

| Transport operator | sigma-stable count | sigma-stable fraction |
|---|---|---|
| T_c | 209,227 | 0.398258 |
| T_z' | 207,484 | 0.394941 |
| T_r* | 197,789 | 0.376486 |

---

## Closure preservation fractions

Each cell shows the fraction of `SS(T_src)` states that remain sigma-stable
under `T_src` after `N_act` has been applied.

| Transport source | N_act = T_b | N_act = T_x | N_act = T_y |
|---|---|---|---|
| T_c | 0.449182 (mostly-disrupted) | 0.466360 (mostly-disrupted) | 0.463750 (mostly-disrupted) |
| T_z' | 0.440357 (mostly-disrupted) | 0.450825 (mostly-disrupted) | 0.404850 (mostly-disrupted) |
| T_r* | 0.393283 (mostly-disrupted) | 0.439266 (mostly-disrupted) | 0.431121 (mostly-disrupted) |

---

## Per-source breakdown

### T_src = T_c  (|SS| = 209,227)

| N_act | preserved | fraction | verdict |
|---|---|---|---|
| T_b | 93,981 | 0.449182 | mostly-disrupted |
| T_x | 97,575 | 0.466360 | mostly-disrupted |
| T_y | 97,029 | 0.463750 | mostly-disrupted |

### T_src = T_z'  (|SS| = 207,484)

| N_act | preserved | fraction | verdict |
|---|---|---|---|
| T_b | 91,367 | 0.440357 | mostly-disrupted |
| T_x | 93,539 | 0.450825 | mostly-disrupted |
| T_y | 84,000 | 0.404850 | mostly-disrupted |

### T_src = T_r*  (|SS| = 197,789)

| N_act | preserved | fraction | verdict |
|---|---|---|---|
| T_b | 77,787 | 0.393283 | mostly-disrupted |
| T_x | 86,882 | 0.439266 | mostly-disrupted |
| T_y | 85,271 | 0.431121 | mostly-disrupted |

---

## Interpretation

### Which non-transport operator best preserves transport sigma-stability?

Mean preservation fractions across all three transport sources:

| N_act | mean preservation fraction |
|---|---|
| T_b | 0.427607 |
| T_x | 0.452150 |
| T_y | 0.433240 |

**T_x** has the highest mean preservation (0.452150).
**T_b** has the lowest mean preservation (0.427607).

### Bridge interpretation of T_b

**T_b does not preserve transport sigma-stability better than both T_x and T_y**
(mean 0.427607). The closure test does not support the bridge interpretation
at this level of analysis.

### Coherence of transport sigma-stable family under non-transport action

Preservation varies by operator pair. See the table above for details. The transport sigma-stable family is neither globally closed nor globally disrupted under non-transport action.

Per-actor consistency (max–min range across three transport sources < 0.05):

| N_act | consistent across sources? | range |
|---|---|---|
| T_b | no | 0.055899 |
| T_x | yes | 0.027093 |
| T_y | no | 0.058899 |

### What this means for the rebuilt algebra

The closure fraction measures how 'porous' the boundary of the transport
sigma-stable subsets is to non-transport action.  A high preservation rate
indicates that non-transport operators tend to map sigma-stable transport
states to other sigma-stable transport states (approximate invariant subspace).
A low rate indicates that non-transport operators scatter sigma-stable transport
states out of the invariant subspace, breaking the sigma-carrier structure.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **tau-holonomy orbit depth audit**: for each operator, compute the
length of the orbit under repeated application until the tau state returns to its
initial configuration (or a cycle is detected).  This characterises the
tau-cycle structure of each operator and tests whether transport operators have
systematically longer tau orbits than non-transport operators — the natural
follow-on to the finding that tau motion persists even within sigma-stable states.
