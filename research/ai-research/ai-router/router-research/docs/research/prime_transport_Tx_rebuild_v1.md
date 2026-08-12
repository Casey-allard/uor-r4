# Prime Transport T_x Rebuild v1

## Purpose

Rebuild the `T_x` (composite_swap) operator directly from `spin_H_core_v6`,
connecting it to the v6 sigma family law (including `coupled_holonomy_residue_v6`
and `family_holonomy_class`) at the operator-model level.  `R_Tx_v6` already
existed in the sigma law; this rebuild closes the gap at the operator level.

---

## Prior stale dependency

**Stale file:** `geometry_native_operator_model_v23.py`

Within v23, the `composite_swap` operator component was implemented by
`composite_swap_component_v10`.  That function:

- Delegates to `composite_swap_component_v4` → v2/v1, which swaps
  `query_semiprime` ↔ `binding_semiprime` and recomputes `composite_compat_class`
  but leaves `spin_h` completely unchanged.
- Applies `update_tau_v3("composite_swap", ...)` which calls `swap_phase_v3` —
  a fixed `(swap_phase + 1) % SWAP_PHASE_MODULUS_V3` (modulus = 2) with no
  sigma input.
- Does **not** call `active_transport_lift_core_vX`.
- Does **not** call `sigma_update_vX`.
- Does **not** read `family_holonomy_class`.
- Does **not** update `spin_h.bits` from the sigma successor.

`R_Tx_v6` exists in `geometry_native_spinH_core_v6` and is dispatched correctly
by `sigma_update_v6`, but was never invoked by the operator model prior to v24.

---

## What was rebuilt

**One function rebuilt:**

| Function | Action |
|----------|--------|
| `composite_swap_component_v10` | Removed (not imported in v24) |
| `composite_swap_component_v24` | **New** — calls `active_transport_lift_core_v6`, `sigma_update_v6` (→ `R_Tx_v6`), `project_tau_v6`; `swap_phase` and `lift_phase` updated with `family_holonomy_class`; `coupled_phase` and `twist_phase` preserved |

**Six functions copied forward unchanged from v23/v22/v21/v20/v10:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | v10 (unchanged) |
| `torus_base_advance_component_v23` | v23 (T_b rebuild; unchanged) |
| `coupled_torus_kick_component_v20` | v20 (T_c rebuild; unchanged) |
| `composite_twist_component_v10` | v10 (unchanged) |
| `fiber_phase_lift_component_v21` | v21 (T_z' rebuild; unchanged) |
| `radial_transport_component_v22` | v22 (T_r* rebuild; unchanged) |

---

## What changed in `composite_swap_component_v24`

1. **Core call:** `active_transport_lift_core_v6(state)` — routes sigma through
   the v6 family law including `coupled_holonomy_residue_v6`.
2. **Sigma update:** `sigma_update_v6(core.sigma, "composite_swap")` — dispatches
   to `R_Tx_v6`, which now receives orbit slot assignments from the v6 coupled-family
   residue.
3. **Tau projection:** `project_tau_v6(core)` — extracts the source tau from the v6 core.
4. **Swap geometry:** `query_semiprime ↔ binding_semiprime`, `composite_compat_class`
   recomputed via `_composite_compat_class_v1(swapped_query, swapped_binding)` —
   identical to the pre-v6 swap law; only sigma and tau handling change.
5. **Spin update:** `spin_h.bits = sigma_successor.current_mode` — absent in all
   pre-v6 versions, which left `spin_h` unchanged.
6. **Tau fields:**
   - `swap_phase`: `(source_tau.swap_phase + sigma_successor.regressive_phase + sigma_successor.family_holonomy_class) % SWAP_PHASE_MODULUS_V3` — T_x's primary tau field, now sigma-mediated with `family_holonomy_class`.
   - `coupled_phase`: preserved (semantic ownership belongs to T_c/T_r*).
   - `twist_phase`: preserved (semantic ownership belongs to T_y).
   - `lift_phase`: `(source_tau.lift_phase + binary(sigma_successor.current_mode) % 5 + binary(sigma_successor.fiber_mode) % 3 + sigma_successor.family_holonomy_class) % 12` — secondary accumulator consistent with other v6 rebuilds.
7. **Lawfulness invariants (tightened from v23):** `target.b == source.b`,
   `target.r == source.r`, `target.phi == source.phi`,
   `target.spin_h.horizon == source.spin_h.horizon`,
   `target.query_semiprime == source.binding_semiprime`,
   `target.binding_semiprime == source.query_semiprime`.

---

## Structural comparison: v24 vs v23

| Metric | v23 | v24 | Change |
|--------|-----|-----|--------|
| state_count | 287342 | 522675 | **+235333** |
| transition_count | 2011394 | 3658725 | **+1647331** |
| class_diversity | 34560 | 34560 | 0 |
| lawful_fraction | 1.0000 | 1.0000 | 0 |
| illegal_count | 0 | 0 | 0 |
| distinct_spin | 16 | 16 | 0 |
| distinct_radial | 3 | 3 | 0 |

The +235333 state gain is the largest single-operator gain in the rebuild chain.
This is expected: T_x swaps `query_semiprime` ↔ `binding_semiprime` for every
state, which doubles the reachable (query, binding, compat_class) space; combined
with v6 sigma mediation that now also differentiates `spin_h.bits` per state,
the composition of these two expansions produces the observed jump.

Class diversity remains at 34560 (unchanged at this depth), confirming that the
tau-class structure is saturated at depth=8 and the new states are distributed
across existing tau-class identities.

---

## Non-T_x parity verification

The six non-T_x components were copied forward unchanged from v23.  The bounded
audit confirms that each component's entry count equals `state_count = 522675`,
satisfying the structural invariant that every state has exactly one transition
per component.  Any count differences between v23 and v24 are sourced solely
from `composite_swap`.

| Component | Count | Matches state_count |
|-----------|-------|---------------------|
| hold | 522675 | ✓ |
| torus_base_advance | 522675 | ✓ |
| composite_swap | 522675 | ✓ (T_x rebuilt) |
| coupled_torus_kick | 522675 | ✓ |
| composite_twist | 522675 | ✓ |
| fiber_phase_lift_spin_transport | 522675 | ✓ |
| radial_transport_unfolding | 522675 | ✓ |

---

## Cumulative operator rebuild summary (v6 baseline)

| Version | Rebuild | state_count | Δ states |
|---------|---------|-------------|----------|
| v19 | T_c from v5 (baseline) | 119785 | — |
| v20 | T_c from v6 | 133354 | +13569 |
| v21 | T_z' from v6 | 178638 | +45284 |
| v22 | T_r* from v6 | 223357 | +44719 |
| v23 | T_b from v6 | 287342 | +63985 |
| v24 | T_x from v6 | 522675 | **+235333** |

---

## Honesty section

| Question | Answer |
|----------|--------|
| Was only `T_x` rebuilt? | **yes** |
| Was `spin_H_core_v6` modified? | **no** |
| Is full exact `spin_H` solved? | **no** — `composite_twist_component_v10` (T_y) remains stale |

---

## Next step

Rebuild `composite_twist_component_v10` (T_y) from `spin_H_core_v6` as
`composite_twist_component_v25` in a new fork `geometry_native_operator_model_v25.py`.
T_y is the last operator in the committed rebuild order (`T_x → T_y`) and the
final stale pre-v6 component.

Do not benchmark. Do not touch already-rebuilt operators.
