# Prime Transport T_b Rebuild v1

## Purpose

Rebuild the `T_b` (torus_base_advance) operator directly from `spin_H_core_v6`,
connecting it to the v6 sigma family law (including `coupled_holonomy_residue_v6`
and `family_holonomy_class`) at the operator-model level.  `R_Tb_v6` already
existed in the sigma law; this rebuild closes the gap at the operator level.

---

## Prior stale dependency

**Stale file:** `geometry_native_operator_model_v22.py`

Within v22, the `torus_base_advance` operator component was implemented by
`torus_base_advance_component_v10`.  That function:

- Delegates entirely to `torus_base_advance_component_v4`, which in turn
  delegates to `torus_base_advance_component_v1` — the original pre-sigma
  implementation that simply computes `b → (b + 1) % 5`.
- Applies `update_tau_v3` for tau, which **returns tau unchanged** for the
  `"torus_base_advance"` component — no tau update at all.
- Does **not** call `active_transport_lift_core_vX`.
- Does **not** call `sigma_update_vX`.
- Does **not** read `family_holonomy_class`.

`R_Tb_v6` exists in `geometry_native_spinH_core_v6` and is dispatched correctly
by `sigma_update_v6`, but was never invoked by the operator model prior to v23.

---

## What was rebuilt

**One function rebuilt:**

| Function | Action |
|----------|--------|
| `torus_base_advance_component_v10` | Removed (not imported in v23) |
| `torus_base_advance_component_v23` | **New** — calls `active_transport_lift_core_v6`, `sigma_update_v6` (→ `R_Tb_v6`), `project_tau_v6`; includes `family_holonomy_class` in `coupled_phase` and `lift_phase`; `swap_phase` and `twist_phase` preserved |

**Six functions copied forward unchanged from v22/v21/v20/v10:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | v10 (unchanged) |
| `composite_swap_component_v10` | v10 (unchanged) |
| `coupled_torus_kick_component_v20` | v20 (T_c rebuild; unchanged) |
| `composite_twist_component_v10` | v10 (unchanged) |
| `fiber_phase_lift_component_v21` | v21 (T_z' rebuild; unchanged) |
| `radial_transport_component_v22` | v22 (T_r* rebuild; unchanged) |

---

## What changed in `torus_base_advance_component_v23`

1. **Core call:** `active_transport_lift_core_v6(state)` — routes sigma through
   the v6 family law including `coupled_holonomy_residue_v6`.
2. **Sigma update:** `sigma_update_v6(core.sigma, "torus_base_advance")` —
   dispatches to `R_Tb_v6`, which now receives orbit slot assignments from the
   v6 coupled-family residue.
3. **Tau projection:** `project_tau_v6(core)` — extracts the source tau from
   the v6 core.
4. **Base advance law:** `target_b = (state.b + 1) % BASE_PERIOD_V1` — unchanged
   from the original; only the sigma mediation and tau update are new.
5. **Spin update:** `spin_h.bits = sigma_successor.current_mode` — consistent
   with all v6 operator rebuilds.
6. **Tau fields:**
   - `swap_phase`: preserved (semantic ownership belongs to `composite_swap`).
   - `coupled_phase`: `(source_tau.coupled_phase + core.theta.base_angle + sigma_successor.regressive_phase + sigma_successor.family_holonomy_class) % 5` — base angle is `core.theta.base_angle = int(state.b)`, contributing base-local information.
   - `twist_phase`: preserved (semantic ownership belongs to `composite_twist`).
   - `lift_phase`: `(source_tau.lift_phase + core.theta.fiber_phase + binary(sigma_successor.current_mode) % 5 + sigma_successor.family_holonomy_class) % 12`.
7. **Lawfulness invariants (tightened from v22):** `target.b != source.b`, `target.r == source.r`, `target.phi == source.phi`, `target.spin_h.horizon == source.spin_h.horizon`.

---

## Structural comparison: v23 vs v22

| Metric | v22 | v23 | Change |
|--------|-----|-----|--------|
| state_count | 223357 | 287342 | **+63985** |
| transition_count | 1563499 | 2011394 | **+447895** |
| class_diversity | 34560 | 34560 | 0 |
| lawful_fraction | 1.0000 | 1.0000 | 0 |
| illegal_count | 0 | 0 | 0 |
| distinct_spin | 16 | 16 | 0 |
| distinct_radial | 3 | 3 | 0 |

The +63985 state gain and +447895 transition gain are the largest single-operator
gains in the v6 rebuild chain.  This is expected: `T_b` cycles through all five
`b` positions and is applied to every state, so introducing sigma mediation here
opens new (b, spin, tau) combinations that were previously unreachable because
the stale v10 implementation returned identical spin and tau regardless of the
current sigma state.

Class diversity remains at 34560 (unchanged), confirming that the tau-class
structure is already saturated at this depth and the new states are spread
across existing tau-class identities rather than creating genuinely new ones.

---

## Non-T_b parity verification

The six non-T_b components were copied forward unchanged from v22.  The bounded
audit confirms that each component's entry count equals `state_count = 287342`,
satisfying the structural invariant that every state has exactly one transition
per component.  Any count differences between v22 and v23 are sourced solely
from `torus_base_advance`.

| Component | Count | Matches state_count |
|-----------|-------|---------------------|
| hold | 287342 | ✓ |
| torus_base_advance | 287342 | ✓ (T_b rebuilt) |
| composite_swap | 287342 | ✓ |
| coupled_torus_kick | 287342 | ✓ |
| composite_twist | 287342 | ✓ |
| fiber_phase_lift_spin_transport | 287342 | ✓ |
| radial_transport_unfolding | 287342 | ✓ |

---

## Cumulative operator rebuild summary (v6 baseline)

| Version | Rebuild | state_count | Δ states |
|---------|---------|-------------|----------|
| v19 | T_c from v5 (baseline) | 119785 | — |
| v20 | T_c from v6 | 133354 | +13569 |
| v21 | T_z' from v6 | 178638 | +45284 |
| v22 | T_r* from v6 | 223357 | +44719 |
| v23 | T_b from v6 | 287342 | **+63985** |

---

## Honesty section

| Question | Answer |
|----------|--------|
| Was only `T_b` rebuilt? | **yes** |
| Was `spin_H_core_v6` modified? | **no** |
| Is full exact `spin_H` solved? | **no** — `composite_twist_component_v10` (T_y) remains stale |

---

## Next step

Rebuild `composite_twist_component_v10` (T_y) from `spin_H_core_v6` as
`composite_twist_component_v24` in a new fork `geometry_native_operator_model_v24.py`.
This is the last of the original five operators (`T_c`, `T_z'`, `T_r*`, `T_b`, `T_y`)
that has not yet been rebuilt against the v6 sigma family law.

Do not benchmark. Do not touch already-rebuilt operators.
