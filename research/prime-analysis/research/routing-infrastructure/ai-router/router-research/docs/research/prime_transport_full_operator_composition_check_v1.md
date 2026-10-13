# Prime Transport Full Operator Composition Check v1

## Purpose

Verify that the three non-transport operators rebuilt from `spin_H_core_v6` —
`T_b` (v23), `T_x` (v24), `T_y` (v25) — interact correctly at the operator level:
their pairwise compositions must be genuinely non-commutative and non-collapsed
on the bounded v25 surface.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface and operators used

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count | 525355 |
| `T_x` | `composite_swap_component_v24` (from `spin_H_core_v6`) |
| `T_y` | `composite_twist_component_v25` (from `spin_H_core_v6`) |
| `T_b` | `torus_base_advance_component_v23` (from `spin_H_core_v6`) |

---

## Operator pairs checked

### Pair 1: T_x ∘ T_y versus T_y ∘ T_x

For each state `s` in the bounded v25 surface:
- `a(s) = T_x(T_y(s))` — apply `T_y` first, then `T_x`
- `b(s) = T_y(T_x(s))` — apply `T_x` first, then `T_y`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 9568 of 525355 |
| Exact state agreement fraction | **0.018212** (1.8212%) |
| Distinct states in image of T_x ∘ T_y | 241334 (45.94% of N) |
| Distinct states in image of T_y ∘ T_x | 217550 (41.41% of N) |
| Image set overlap | 93664 (17.83% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 9568 of 525355 states (1.8212%) produce identical outputs under both orderings.  The two image sets are large, non-equal, and only partially overlapping.  The compositions are genuinely non-commutative.

---

### Pair 2: T_b ∘ T_x versus T_x ∘ T_b

For each state `s`:
- `c(s) = T_b(T_x(s))` — apply `T_x` first, then `T_b`
- `d(s) = T_x(T_b(s))` — apply `T_b` first, then `T_x`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 8632 of 525355 |
| Exact state agreement fraction | **0.016431** (1.6431%) |
| Distinct states in image of T_b ∘ T_x | 211920 (40.34% of N) |
| Distinct states in image of T_x ∘ T_b | 251655 (47.90% of N) |
| Image set overlap | 85089 (16.20% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 8632 of 525355 states (1.6431%) produce identical outputs under both orderings.  The compositions are genuinely non-commutative.

---

### Pair 3: T_b ∘ T_y versus T_y ∘ T_b

For each state `s`:
- `e(s) = T_b(T_y(s))` — apply `T_y` first, then `T_b`
- `f(s) = T_y(T_b(s))` — apply `T_b` first, then `T_y`

| Metric | Value |
|--------|-------|
| Exact state agreement count | 8622 of 525355 |
| Exact state agreement fraction | **0.016412** (1.6412%) |
| Distinct states in image of T_b ∘ T_y | 301480 (57.39% of N) |
| Distinct states in image of T_y ∘ T_b | 301381 (57.37% of N) |
| Image set overlap | 141081 (26.85% of N) |

**Verdict: **DISTINCT and NON-COLLAPSED**.**
Only 8622 of 525355 states (1.6412%) produce identical outputs under both orderings.  The compositions are genuinely non-commutative.

---

## Overall verdict

The non-transport chain `(T_b, T_x, T_y)` rebuilt from `spin_H_core_v6` is
**compositionally non-degenerate** on the bounded v25 surface.

- `T_x ∘ T_y` and `T_y ∘ T_x` are **DISTINCT** (agreement fraction: 0.018212).
- `T_b ∘ T_x` and `T_x ∘ T_b` are **DISTINCT** (agreement fraction: 0.016431).
- `T_b ∘ T_y` and `T_y ∘ T_b` are **DISTINCT** (agreement fraction: 0.016412).
- No pair is collapsed.
- All image sets are large and only partially overlapping.
- The non-commutativity is genuine, not a boundary artifact.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — this is a read-only audit on the accepted v25 surface |
| Were any operators rebuilt in this step? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Run a joint cross-layer composition audit pairing one non-transport operator
(`T_x` or `T_y`) with one transport operator (`T_c`, `T_z'`, or `T_r*`) on the
v25 surface, to verify that the non-transport and transport sub-algebras are
mutually non-degenerate — confirming that the full six-operator rebuilt layer is
compositionally independent as a whole.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
