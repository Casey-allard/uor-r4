# Prime Transport T_y Rebuild v1

## Purpose

Rebuild the `T_y` (composite_twist) operator directly from `spin_H_core_v6`,
connecting it to the v6 sigma family law (including `coupled_holonomy_residue_v6`
and `family_holonomy_class`) at the operator-model level.  `R_Ty_v6` already
existed in the sigma law; this rebuild closes the final operator-model gap.

With T_y rebuilt, all six non-hold operators (`T_c`, `T_z'`, `T_r*`, `T_b`,
`T_x`, `T_y`) are now built against `spin_H_core_v6`.

---

## Prior stale dependency

**Stale file:** `geometry_native_operator_model_v24.py`

Within v24, the `composite_twist` operator component was implemented by
`composite_twist_component_v10`.  That function:

- Delegates to `composite_twist_component_v4`, which flips the `twist` bit
  (`twist = 1 - state.twist`) and leaves all other coordinates (b, phi, r,
  spin_h, query/binding/compat_class) unchanged.
- Applies `update_tau_v3("composite_twist", ...)` which calls `twist_phase_v3` —
  a fixed `(twist_phase + 1) % TWIST_PHASE_MODULUS_V3` (modulus = 2) with no
  sigma input.
- Does **not** call `active_transport_lift_core_vX`.
- Does **not** call `sigma_update_vX`.
- Does **not** read `family_holonomy_class`.
- Does **not** update `spin_h.bits` from the sigma successor.

`R_Ty_v6` exists in `geometry_native_spinH_core_v6` and is dispatched correctly
by `sigma_update_v6`, but was never invoked by the operator model prior to v25.

---

## What was rebuilt

**One function rebuilt:**

| Function | Action |
|----------|--------|
| `composite_twist_component_v10` | Removed (not imported in v25) |
| `composite_twist_component_v25` | **New** — calls `active_transport_lift_core_v6`, `sigma_update_v6` (→ `R_Ty_v6`), `project_tau_v6`; `twist_phase` and `lift_phase` updated with `family_holonomy_class`; `swap_phase` and `coupled_phase` preserved |

**Six functions copied forward unchanged from v24/v23/v22/v21/v20/v10:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | v10 (unchanged) |
| `torus_base_advance_component_v23` | v23 (T_b rebuild; unchanged) |
| `composite_swap_component_v24` | v24 (T_x rebuild; unchanged) |
| `coupled_torus_kick_component_v20` | v20 (T_c rebuild; unchanged) |
| `fiber_phase_lift_component_v21` | v21 (T_z' rebuild; unchanged) |
| `radial_transport_component_v22` | v22 (T_r* rebuild; unchanged) |

---

## What changed in `composite_twist_component_v25`

1. **Core call:** `active_transport_lift_core_v6(state)` — routes sigma through
   the v6 family law including `coupled_holonomy_residue_v6`.
2. **Sigma update:** `sigma_update_v6(core.sigma, "composite_twist")` — dispatches
   to `R_Ty_v6`, which now receives orbit slot assignments from the v6
   coupled-family residue.
3. **Tau projection:** `project_tau_v6(core)` — extracts the source tau from the v6 core.
4. **Twist geometry:** `target_twist = 1 - state.twist` — identical to the pre-v6
   flip law; only sigma and tau handling change.
5. **Spin update:** `spin_h.bits = sigma_successor.current_mode` — absent in all
   pre-v6 versions, which left `spin_h` unchanged.
6. **Tau fields:**
   - `swap_phase`: preserved (semantic ownership belongs to T_x).
   - `coupled_phase`: preserved (semantic ownership belongs to T_c/T_r*).
   - `twist_phase`: `(source_tau.twist_phase + sigma_successor.regressive_phase + sigma_successor.family_holonomy_class) % TWIST_PHASE_MODULUS_V3` — T_y's primary field, now sigma-mediated with `family_holonomy_class`.
   - `lift_phase`: `(source_tau.lift_phase + binary(sigma_successor.current_mode) % 5 + binary(sigma_successor.radial_mode) % 3 + sigma_successor.family_holonomy_class) % 12`.
7. **Lawfulness invariants (tightened from v24):** `target.b == source.b`,
   `target.r == source.r`, `target.phi == source.phi`,
   `target.spin_h.horizon == source.spin_h.horizon`,
   `target.twist == 1 - source.twist`.

---

## Structural comparison: v25 vs v24

| Metric | v24 | v25 | Change |
|--------|-----|-----|--------|
| state_count | 522675 | 525355 | **+2680** |
| transition_count | 3658725 | 3677485 | **+18760** |
| class_diversity | 34560 | 34560 | 0 |
| lawful_fraction | 1.0000 | 1.0000 | 0 |
| illegal_count | 0 | 0 | 0 |
| distinct_spin | 16 | 16 | 0 |
| distinct_radial | 3 | 3 | 0 |

The +2680 state gain is the smallest in the rebuild chain, which is expected:
T_y operates entirely within (b, phi, r) coordinates unchanged, only flipping
`twist` and updating spin and tau.  Because T_x (which also leaves b/phi/r
unchanged) has already been rebuilt and substantially expanded the reachable
surface, T_y's incremental contribution is bounded by the twist-bit doubling
of states not previously reachable through the T_x-expanded surface.

Class diversity remains at 34560 (unchanged), confirming tau-class saturation
at depth=8.

---

## Non-T_y parity verification

The six non-T_y components were copied forward unchanged from v24.  The bounded
audit confirms that each component's entry count equals `state_count = 525355`,
satisfying the structural invariant that every state has exactly one transition
per component.  Any count differences between v24 and v25 are sourced solely
from `composite_twist`.

| Component | Count | Matches state_count |
|-----------|-------|---------------------|
| hold | 525355 | ✓ |
| torus_base_advance | 525355 | ✓ |
| composite_swap | 525355 | ✓ |
| coupled_torus_kick | 525355 | ✓ |
| composite_twist | 525355 | ✓ (T_y rebuilt) |
| fiber_phase_lift_spin_transport | 525355 | ✓ |
| radial_transport_unfolding | 525355 | ✓ |

---

## Cumulative operator rebuild summary (v6 baseline)

| Version | Rebuild | state_count | Δ states |
|---------|---------|-------------|----------|
| v19 | T_c from v5 (baseline) | 119785 | — |
| v20 | T_c from v6 | 133354 | +13569 |
| v21 | T_z' from v6 | 178638 | +45284 |
| v22 | T_r* from v6 | 223357 | +44719 |
| v23 | T_b from v6 | 287342 | +63985 |
| v24 | T_x from v6 | 522675 | +235333 |
| v25 | T_y from v6 | 525355 | +2680 |

**v25 completes the rebuild of all six non-hold operators against `spin_H_core_v6`.**
Total state gain from v19 baseline: +405570 (338% increase).

---

## Honesty section

| Question | Answer |
|----------|--------|
| Was only `T_y` rebuilt? | **yes** |
| Was `spin_H_core_v6` modified? | **no** |
| Is full exact `spin_H` solved? | **no** — `hold_component_v10` is the only remaining pre-v6 operator; it is by construction a no-op (identity) and carries no sigma dependency by design |

---

## Next step

Run a full cross-operator composition audit on the v25 surface — the complete
analogue of the bounded transport composition check (`prime_transport_transport_composition_check_v1`)
but now including `T_b`, `T_x`, and `T_y` in the pair comparisons.  Specifically,
check `T_x ∘ T_y` versus `T_y ∘ T_x` and `T_b ∘ T_x` versus `T_x ∘ T_b` to
confirm the full operator algebra is non-degenerate after the complete rebuild chain.

Do not benchmark. Do not modify any operator.
