# Prime Transport Cross-Layer Composition Check v1

## Purpose

Verify that the non-transport sub-algebra `(T_b, T_x, T_y)` and the transport
sub-algebra `(T_c, T_z', T_r*)`, both rebuilt from `spin_H_core_v6`, are
mutually non-degenerate: cross-layer pairwise compositions must be genuinely
non-commutative and non-collapsed on the bounded v25 surface.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface and operators used

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count | 525355 |
| `T_x` (non-transport) | `composite_swap_component_v24` (from `spin_H_core_v6`) |
| `T_y` (non-transport) | `composite_twist_component_v25` (from `spin_H_core_v6`) |
| `T_b` (non-transport) | `torus_base_advance_component_v23` (from `spin_H_core_v6`) |
| `T_c` (transport) | `coupled_torus_kick_component_v20` (from `spin_H_core_v6`) |
| `T_z'` (transport) | `fiber_phase_lift_component_v21` (from `spin_H_core_v6`) |
| `T_r*` (transport) | `radial_transport_component_v22` (from `spin_H_core_v6`) |

---

## Cross-layer operator pairs checked

### Pair 1: T_x ∘ T_c versus T_c ∘ T_x

For each state `s` in the bounded v25 surface:
- `a(s) = T_x(T_c(s))` — apply `T_c` first, then `T_x`
- `b(s) = T_c(T_x(s))` — apply `T_x` first, then `T_c`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 90 of 525355 |
| Exact state agreement fraction | **0.000171** (0.0171%) |
| Distinct states in image of T_x ∘ T_c | 201315 (38.32% of N) |
| Distinct states in image of T_c ∘ T_x | 160787 (30.61% of N) |
| Image set overlap | 76780 (14.61% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 90 of 525355 states (0.0171%) produce identical outputs under both orderings.  The two image sets are large, non-equal, and only partially overlapping.  The compositions are genuinely non-commutative.

---

### Pair 2: T_y ∘ T_z' versus T_z' ∘ T_y

For each state `s`:
- `c(s) = T_y(T_z'(s))` — apply `T_z'` first, then `T_y`
- `d(s) = T_z'(T_y(s))` — apply `T_y` first, then `T_z'`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 352 of 525355 |
| Exact state agreement fraction | **0.000670** (0.0670%) |
| Distinct states in image of T_y ∘ T_z' | 277666 (52.85% of N) |
| Distinct states in image of T_z' ∘ T_y | 277895 (52.90% of N) |
| Image set overlap | 120855 (23.00% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 352 of 525355 states (0.0670%) produce identical outputs under both orderings.  The compositions are genuinely non-commutative.

---

### Pair 3: T_b ∘ T_r* versus T_r* ∘ T_b

For each state `s`:
- `e(s) = T_b(T_r*(s))` — apply `T_r*` first, then `T_b`
- `f(s) = T_r*(T_b(s))` — apply `T_b` first, then `T_r*`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 106 of 525355 |
| Exact state agreement fraction | **0.000202** (0.0202%) |
| Distinct states in image of T_b ∘ T_r* | 294004 (55.96% of N) |
| Distinct states in image of T_r* ∘ T_b | 296329 (56.41% of N) |
| Image set overlap | 136521 (25.99% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 106 of 525355 states (0.0202%) produce identical outputs under both orderings.  The compositions are genuinely non-commutative.

---

## Overall verdict

The transport and non-transport sub-algebras rebuilt from `spin_H_core_v6` are
**mutually compositionally non-degenerate** on the bounded v25 surface.

- `T_x ∘ T_c` and `T_c ∘ T_x` are **DISTINCT** (agreement fraction: 0.000171).
- `T_y ∘ T_z'` and `T_z' ∘ T_y` are **DISTINCT** (agreement fraction: 0.000670).
- `T_b ∘ T_r*` and `T_r* ∘ T_b` are **DISTINCT** (agreement fraction: 0.000202).
- No cross-layer pair is collapsed.
- All image sets are large and only partially overlapping.
- The cross-layer non-commutativity is genuine, not a boundary artifact.
- The full six-operator rebuilt layer is compositionally distinct across layers.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — this is a read-only audit on the accepted v25 surface |
| Were any operators rebuilt in this step? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Compute the commutativity spectrum of the full six-operator rebuilt layer on the
v25 surface: for every unordered pair `{O_i, O_j}` among the six non-hold operators,
record the exact agreement fraction `|{s : O_i(O_j(s)) == O_j(O_i(s))}| / N`.
This produces a 6×6 symmetric agreement matrix that characterises the full
non-commutativity structure of the rebuilt algebra in one compact table.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
