# Prime Transport T_c Rebuild v1

## Purpose

Rebuild the coupled operator branch (`T_c`) directly from `spin_H_core_v6`.

This step replaces the stale v5-based T_c implementation with one that calls
`sigma_update_v6`, which flows through `sigma_family_holonomy_law_v6` with the
`coupled_holonomy_residue_v6` primitive.

No other operators are rebuilt in this step.

---

## Old Stale Dependency

File containing the pre-v6 T_c implementation:

- `geometry_native_operator_model_v19.py`

Stale imports in that file:

```python
from geometry_native_spinH_core_v5 import (
    active_transport_lift_core_v5,
    project_tau_v5,
    sigma_update_v5,
)
```

Stale call inside `coupled_torus_kick_component_v19`:

```python
sigma_successor = sigma_update_v5(core.sigma, "coupled_torus_kick")
```

This call routed through `sigma_family_holonomy_law_v5` with
`_component_role_order_v5` — the `(0, 1, 2)` stub that collapsed T_c's
coupled-family signature.

---

## What Was Rebuilt

**Rebuilt function (one):**

- `coupled_torus_kick_component_v19` → `coupled_torus_kick_component_v20`

**Implemented in:**

- `geometry_native_operator_model_v20.py`

The rebuild changes only the sigma update call:

```python
# v19 (stale):
sigma_successor = sigma_update_v5(core.sigma, "coupled_torus_kick")

# v20 (rebuilt):
sigma_successor = sigma_update_v6(core.sigma, "coupled_torus_kick")
```

`sigma_update_v6` routes through `sigma_family_holonomy_law_v6` which calls
`coupled_holonomy_residue_v6(source_sigma, proposed_sigma)`. This produces a
component-specific orbit role shift derived from the cyclic coupling signature
of the proposed transition, restoring T_c's genuine non-commutativity with the
radial branch.

**Functions copied forward unchanged from v19/v10/v12:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | `geometry_native_operator_model_v10` |
| `torus_base_advance_component_v10` | `geometry_native_operator_model_v10` |
| `composite_swap_component_v10` | `geometry_native_operator_model_v10` |
| `composite_twist_component_v10` | `geometry_native_operator_model_v10` |
| `fiber_phase_lift_component_v10` | `geometry_native_operator_model_v10` |
| `radial_transport_component_v12` | `geometry_native_operator_model_v12` |

---

## Bounded Structural Results

On the bounded lawful surface (depth=8):

- reachable state count: `133354`
- total nonzero transitions: `933478`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `33799`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Dependency checks:

- `T_c` calls `active_transport_lift_core_v6`: `yes`
- `sigma_update_v6` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

---

## Comparison vs v19 (T_c from spin_H_core_v5)

| Metric | v19 | v20 | Change |
|--------|-----|-----|--------|
| state count | 119785 | **133354** | **+13569** |
| transition count | 838495 | **933478** | **+94983** |
| class diversity | 31465 | **33799** | **+2334** |
| spin diversity | 16 | 16 | 0 |
| radial diversity | 3 | 3 | 0 |
| lawful fraction | 1.0 | 1.0 | 0 |
| illegal count | 0 | 0 | 0 |

The rebuild from v6 reverses the expressiveness loss that occurred when T_c
was rebuilt from v5:
- v4→v5 T_c rebuild: −5892 states, −41244 transitions, −2766 class diversity
- v5→v6 T_c rebuild: **+13569 states, +94983 transitions, +2334 class diversity**

T_c from v6 also surpasses v4 in raw counts (v4 had 125677 states, 879739
transitions). The coupled-family residue restored and extended T_c's structural
expressiveness.

---

## Non-T_c Path Parity

All non-T_c component paths (hold, torus_base_advance, composite_swap,
composite_twist, fiber_phase_lift_spin_transport, radial_transport_unfolding)
call the same v10/v12 implementations as v19. Their per-component transition
counts each equal the total state count (`133354`), confirming the structural
invariant holds. The only behavioral delta confirmed as sourced from
`coupled_torus_kick`.

---

## Honesty

**Was only `T_c` rebuilt?**

- **Yes.** Only `coupled_torus_kick_component_v19` was rebuilt.
  All other component functions were copied forward unchanged.

**Was `spin_H_core_v6` modified?**

- **No.** `geometry_native_spinH_core_v6.py` was not changed.

**Is full exact `spin_H` solved?**

- **No.** The sigma carrier is still bounded and mode-word-based.
  The coupled-family residue improved T_c expressiveness within the current
  bounded architecture. Full exact `spin_H` requires further carrier refinement
  beyond the scope of this step.

---

## Next Step

Rebuild `T_z'` from `spin_H_core_v6` before anything else.
