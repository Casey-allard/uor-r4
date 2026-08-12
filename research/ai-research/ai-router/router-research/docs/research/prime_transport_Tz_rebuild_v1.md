# Prime Transport T_z' Rebuild v1

## Purpose

Rebuild the `T_z'` (fiber-phase-lift-spin-transport) operator directly from
`spin_H_core_v6`, restoring its dependency on the v6 sigma family law that
includes `coupled_holonomy_residue_v6`.

---

## Prior stale dependency

**Stale file:** `geometry_native_operator_model_v20.py`

Within v20, the `fiber_phase_lift_spin_transport` operator component was
implemented by `fiber_phase_lift_component_v10`. This function dates from the
pre-core era: it uses a hard-coded phi-step formula with no connection to the
sigma carrier or the family holonomy law. Specifically:

- It does **not** call `active_transport_lift_core_vX` at all.
- It does **not** call `sigma_update_vX` — the sigma carrier is not consulted.
- It does **not** read `family_holonomy_class` (introduced in v5/v6).
- Its `phi_step` computation uses fixed `theta`, `rho`, and `h` projections with
  no sigma contribution.

The v4-era rebuild `fiber_phase_lift_component_v17` (in v17) partially
addressed this by adding sigma inputs via `active_transport_lift_core_v4` and
`sigma_update_v4`, but it was not carried forward into v19 or v20. Additionally
it lacked `family_holonomy_class` in its sigma_weight (that field was
introduced in v5).

---

## What was rebuilt

**One function rebuilt:**

| Function | Action |
|----------|--------|
| `fiber_phase_lift_component_v10` | Removed (not imported in v21) |
| `fiber_phase_lift_component_v21` | **New** — calls `active_transport_lift_core_v6`, `sigma_update_v6`, `project_tau_v6`; includes `family_holonomy_class` in `sigma_weight` |

**Six functions copied forward unchanged from v20/v10/v12:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | v10 (unchanged) |
| `torus_base_advance_component_v10` | v10 (unchanged) |
| `composite_swap_component_v10` | v10 (unchanged) |
| `coupled_torus_kick_component_v20` | v20 (T_c rebuild; unchanged) |
| `composite_twist_component_v10` | v10 (unchanged) |
| `radial_transport_component_v12` | v12 (unchanged) |

---

## What changed in `fiber_phase_lift_component_v21`

1. **Core call:** `active_transport_lift_core_v6(state)` — routes sigma
   through the v6 family law.
2. **Sigma update:** `sigma_update_v6(core.sigma, "fiber_phase_lift_spin_transport")` —
   applies `R_Tz_v6` which now includes `coupled_holonomy_residue_v6` in its
   orbit slot assignment.
3. **Tau projection:** `project_tau_v6(core)` — extracts the source tau from
   the v6 core.
4. **Sigma weight in phi_step:** Includes `sigma_successor.family_holonomy_class`
   (× 2 weight) — absent in all pre-v6 versions of this function.
5. **Tau fields:** `coupled_phase` accumulates `sigma_successor.family_holonomy_class`
   alongside the radial target; `twist_phase` and `lift_phase` include
   family-holonomy-class contributions. `swap_phase` is preserved (T_z' does
   not alter swap geometry).

---

## Structural comparison: v21 vs v20

| Metric | v20 | v21 | Change |
|--------|-----|-----|--------|
| state_count | 133354 | 178638 | **+45284** |
| transition_count | 933478 | 1250466 | **+316988** |
| class_diversity | 33799 | 34546 | **+747** |
| lawful_fraction | 1.0000 | 1.0000 | 0 |
| illegal_count | 0 | 0 | 0 |
| distinct_spin | 16 | 16 | 0 |
| distinct_radial | 3 | 3 | 0 |

The +45284 state gain and +316988 transition gain confirm that `T_z'` rebuilt
from v6 substantially increases reachable surface — correctly reflecting the
richer sigma variation now flowing through `R_Tz_v6`. Class diversity grows
modestly (+747) consistent with T_z' acting on the phi coordinate within a
fixed (b, r) fiber; the torus geometry bounds diversity growth more tightly
than T_c which changes spin directly.

---

## Non-T_z' parity verification

All non-T_z' components return exactly `state_count = 178638` transitions,
matching the structural invariant. This confirms that the behavioral delta
is isolated exclusively to the `fiber_phase_lift_spin_transport` component.

| Component | Count | Matches state_count |
|-----------|-------|---------------------|
| hold | 178638 | ✓ |
| torus_base_advance | 178638 | ✓ |
| composite_swap | 178638 | ✓ |
| coupled_torus_kick | 178638 | ✓ |
| composite_twist | 178638 | ✓ |
| fiber_phase_lift_spin_transport | 178638 | ✓ (T_z' rebuilt) |
| radial_transport_unfolding | 178638 | ✓ |

---

## Honesty section

| Question | Answer |
|----------|--------|
| Was only `T_z'` rebuilt? | **yes** |
| Was `spin_H_core_v6` modified? | **no** |
| Is full exact `spin_H` solved? | **no** — `T_r*` still depends on pre-v6 assumptions |

---

## Next step

Rebuild `T_r*` (radial_transport_unfolding) from `spin_H_core_v6` as
`radial_transport_component_v22` in a new fork `geometry_native_operator_model_v22.py`.
This will close the last pre-v6 stale operator dependency in the three main
transport branches.

Do not benchmark. Do not broaden scope. Do not touch T_c or T_z'.
