# Prime Transport T_r* Rebuild v1

## Purpose

Rebuild the `T_r*` (radial_transport_unfolding) operator directly from
`spin_H_core_v6`, restoring its dependency on the v6 sigma family law that
includes `coupled_holonomy_residue_v6` and the `family_holonomy_class` field.

---

## Prior stale dependency

**Stale file:** `geometry_native_operator_model_v21.py`

Within v21, the `radial_transport_unfolding` operator component was implemented
by `radial_transport_component_v12`. This function dates from the pre-core era:

- It does **not** call `active_transport_lift_core_vX` at all.
- It does **not** call `sigma_update_vX` — the sigma carrier is never consulted.
- It does **not** read `family_holonomy_class` (introduced in v5/v6).
- Its `target_r`, `target_phi`, and `target_tau` computations are driven by
  raw `state.r`, `state.phi`, and `state.tau` without any sigma mediation.

The v4-era rebuild `radial_transport_component_v18` (in v18) addressed sigma
inputs via `active_transport_lift_core_v4` and `sigma_update_v4`, but it was
not carried forward into v19/v20/v21. Additionally it lacked
`family_holonomy_class` in its sigma_weight (that field was introduced in v5).

---

## What was rebuilt

**One function rebuilt:**

| Function | Action |
|----------|--------|
| `radial_transport_component_v12` | Removed (not imported in v22) |
| `radial_transport_component_v22` | **New** — calls `active_transport_lift_core_v6`, `sigma_update_v6`, `project_tau_v6`; includes `family_holonomy_class` (× 2 weight in sigma_weight, and in coupled/twist/lift tau fields) |

**Six functions copied forward unchanged from v21/v20/v10:**

| Function | Source |
|----------|--------|
| `hold_component_v10` | v10 (unchanged) |
| `torus_base_advance_component_v10` | v10 (unchanged) |
| `composite_swap_component_v10` | v10 (unchanged) |
| `coupled_torus_kick_component_v20` | v20 (T_c rebuild; unchanged) |
| `composite_twist_component_v10` | v10 (unchanged) |
| `fiber_phase_lift_component_v21` | v21 (T_z' rebuild; unchanged) |

---

## What changed in `radial_transport_component_v22`

1. **Core call:** `active_transport_lift_core_v6(state)` — routes sigma
   through the v6 family law including `coupled_holonomy_residue_v6`.
2. **Sigma update:** `sigma_update_v6(core.sigma, "radial_transport_unfolding")` —
   applies `R_Tr_v6` which now receives orbit slot assignments from the
   v6 coupled-family residue.
3. **Tau projection:** `project_tau_v6(core)` — extracts the source tau from
   the v6 core.
4. **Sigma weight in phi_offset:** Includes `2 * sigma_successor.family_holonomy_class`
   — absent in all pre-v6 versions of this function, including `radial_transport_component_v18`.
5. **Tau fields:** `coupled_phase` accumulates `sigma_successor.family_holonomy_class`
   alongside the radial target; `twist_phase` and `lift_phase` include
   family-holonomy-class contributions consistent with v20 and v21 patterns.
6. **Radial target:** `target_r = core.rho.radial_target` (same derivation as
   v18; now routed through the v6 sigma carrier).
7. **Lawfulness invariants:** `target.b == source.b`, `target.r != source.r`,
   `target.composite_compat_class == source.composite_compat_class`,
   `target.spin_h.horizon == source.spin_h.horizon` — following the v18 pattern,
   tightened over v21's bare equality check.

---

## Structural comparison: v22 vs v21

| Metric | v21 | v22 | Change |
|--------|-----|-----|--------|
| state_count | 178638 | 223357 | **+44719** |
| transition_count | 1250466 | 1563499 | **+313033** |
| class_diversity | 34546 | 34560 | **+14** |
| lawful_fraction | 1.0000 | 1.0000 | 0 |
| illegal_count | 0 | 0 | 0 |
| distinct_spin | 16 | 16 | 0 |
| distinct_radial | 3 | 3 | 0 |

The +44719 state gain and +313033 transition gain confirm that `T_r*` rebuilt
from v6 substantially widens the reachable surface. The class diversity growth
is modest (+14): radial transport changes `r` and `phi` but not `b`, so its
direct contribution to tau-class diversity is bounded by the phi mod-3 range
already covered by `T_z'`. The state gain reflects new (b, phi, r, spin, tau)
combinations reachable through the v6-mediated radial step.

---

## Non-T_r* parity verification

The six non-T_r* components were copied forward unchanged from v21. The bounded
audit confirms that each component's entry count equals `state_count = 223357`,
satisfying the structural invariant that every state has exactly one transition
per component. Any count differences between v21 and v22 are sourced solely from
`radial_transport_unfolding`.

| Component | Count | Matches state_count |
|-----------|-------|---------------------|
| hold | 223357 | ✓ |
| torus_base_advance | 223357 | ✓ |
| composite_swap | 223357 | ✓ |
| coupled_torus_kick | 223357 | ✓ |
| composite_twist | 223357 | ✓ |
| fiber_phase_lift_spin_transport | 223357 | ✓ |
| radial_transport_unfolding | 223357 | ✓ (T_r* rebuilt) |

---

## Cumulative operator rebuild summary (v6 baseline)

| Version | Rebuild | state_count | Δ states |
|---------|---------|-------------|----------|
| v19 | T_c from v5 (baseline) | 119785 | — |
| v20 | T_c from v6 | 133354 | +13569 |
| v21 | T_z' from v6 | 178638 | +45284 |
| v22 | T_r* from v6 | 223357 | +44719 |

All three main transport operators now read from `spin_H_core_v6`.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Was only `T_r*` rebuilt? | **yes** |
| Was `spin_H_core_v6` modified? | **no** |
| Is full exact `spin_H` solved? | **no** — the remaining non-core operators (hold, torus_base_advance, composite_swap, composite_twist) still use v10 formulations that do not consult the sigma carrier; this is the next structural gap |

---

## Next step

Assess whether the v10-era non-transport operators (`hold`, `torus_base_advance`,
`composite_swap`, `composite_twist`) should be rebuilt against `spin_H_core_v6`,
or whether the immediate next step is a cross-operator consistency audit of the
now-complete three-operator v6 rebuild (T_c, T_z', T_r*) — specifically whether
their sigma successors interact correctly across combined paths.

**Single recommendation:** Run a bounded cross-operator composition check on the
v22 surface to confirm that the `T_c ∘ T_r*` and `T_z' ∘ T_r*` compositions
produce distinct, non-collapsed state distributions (analogous to the
`R_Tc ∘ R_Tr` commutativity audit done for `spin_H_core_v6`), before expanding
to the non-transport operators.

Do not benchmark. Do not touch non-transport operators yet.
