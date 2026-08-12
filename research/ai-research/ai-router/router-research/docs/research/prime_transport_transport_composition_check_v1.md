# Prime Transport Cross-Operator Composition Check v1

## Purpose

Verify that the three transport operators rebuilt from `spin_H_core_v6` —
`T_c` (v20), `T_z'` (v21), `T_r*` (v22) — interact correctly at the operator
level: their pairwise compositions must be genuinely non-commutative and
non-collapsed on the bounded v22 surface.

This is the operator-level analogue of the `R_Tc ∘ R_Tr` commutativity audit
that validated `sigma_family_holonomy_law_v6`. The check is a read-only audit
on the accepted v22 surface. No files were modified.

---

## Surface and operators used

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v22.py`, depth=8 |
| State count | 223357 |
| `T_c` | `coupled_torus_kick_component_v20` (from `spin_H_core_v6`) |
| `T_z'` | `fiber_phase_lift_component_v21` (from `spin_H_core_v6`) |
| `T_r*` | `radial_transport_component_v22` (from `spin_H_core_v6`) |

---

## Operator pairs checked

### Pair 1: T_c ∘ T_r* versus T_r* ∘ T_c

For each state `s` in the bounded v22 surface:
- `a(s) = T_c(T_r*(s))` — apply `T_r*` first, then `T_c`
- `b(s) = T_r*(T_c(s))` — apply `T_c` first, then `T_r*`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 100 of 223357 |
| Exact state agreement fraction | **0.000448** (0.045%) |
| Distinct states in image of T_c ∘ T_r* | 111482 (49.91% of N) |
| Distinct states in image of T_r* ∘ T_c | 109139 (48.86% of N) |
| Image set overlap | 38369 (17.18% of N) |

**Verdict: DISTINCT and NON-COLLAPSED.**
Only 100 of 223357 states (0.045%) produce identical outputs under both
orderings. The two image sets are large, non-equal, and only partially
overlapping. The compositions are genuinely non-commutative.

---

### Pair 2: T_z' ∘ T_r* versus T_r* ∘ T_z'

For each state `s`:
- `c(s) = T_z'(T_r*(s))` — apply `T_r*` first, then `T_z'`
- `d(s) = T_r*(T_z'(s))` — apply `T_z'` first, then `T_r*`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 47 of 223357 |
| Exact state agreement fraction | **0.000210** (0.021%) |
| Distinct states in image of T_z' ∘ T_r* | 127400 (57.04% of N) |
| Distinct states in image of T_r* ∘ T_z' | 127543 (57.10% of N) |
| Image set overlap | 51767 (23.18% of N) |

**Verdict: DISTINCT and NON-COLLAPSED.**
Only 47 of 223357 states (0.021%) produce identical outputs under both
orderings. The image sizes are similar (within 0.11%) but the image sets
differ substantially: overlap is only 23.18%, confirming two distinct
transport paths.

---

## Comparison with sigma-level audit

The `sigma_family_holonomy_law_v6` composition audit (run at the sigma
carrier level, before operator construction) reported:

| Pair | Pre-v6 (collapsed) | v6 (fixed) |
|------|--------------------|------------|
| R_Tc ∘ R_Tr | 38315/38315 = **100.0%** agree | 13952/38315 = **36.4%** agree |
| R_Tz ∘ R_Tr | 27605/38315 = **72.1%** | 10837/38315 = **28.3%** |

At the operator level (v22 surface, 223357 states), the agreement fractions
are:

| Pair | Agreement count | Agreement fraction |
|------|-----------------|--------------------|
| T_c ∘ T_r* vs T_r* ∘ T_c | 100 | **0.045%** |
| T_z' ∘ T_r* vs T_r* ∘ T_z' | 47 | **0.021%** |

The operator-level non-commutativity is substantially stronger than the
sigma-level non-commutativity. This is expected: operator composition
compounds sigma divergence across tau, spin, and phi coordinates, amplifying
the structural separation introduced by `coupled_holonomy_residue_v6`.

---

## Overall verdict

The transport chain `(T_c, T_z', T_r*)` rebuilt from `spin_H_core_v6` is
**compositionally non-degenerate** on the bounded v22 surface.

- `T_c ∘ T_r*` and `T_r* ∘ T_c` are **distinct** (agreement fraction: 0.000448).
- `T_z' ∘ T_r*` and `T_r* ∘ T_z'` are **distinct** (agreement fraction: 0.000210).
- Neither pair is collapsed.
- Both image sets are large and partially but not fully overlapping.
- The non-commutativity is genuine, not a boundary artifact.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — this is a read-only audit on the accepted v22 surface |
| Were any operators rebuilt in this step? | **no** |
| Is full exact `spin_H` solved? | **no** — the v10-era non-transport operators (hold, torus_base_advance, composite_swap, composite_twist) do not yet consult the sigma carrier |

---

## Next step

Rebuild the v10-era non-transport operators against `spin_H_core_v6`, starting
with `composite_twist_component_v10`, which is the non-transport component most
likely to benefit from sigma mediation (it operates on the twist coordinate,
which is coupled to the torus structure that T_c and T_z' now influence via v6).
Implement as a localized fork `geometry_native_operator_model_v23.py`, replacing
only `composite_twist_component_v10` with a `composite_twist_component_v23` that
reads from `spin_H_core_v6`.

Do not benchmark. Do not touch T_c, T_z', or T_r*.
