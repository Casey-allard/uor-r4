# Prime Transport Non-Transport Associativity Check v1

## Purpose

Verify that the three non-transport operators rebuilt from `spin_H_core_v6`
— `T_b`, `T_x`, `T_y` — form an associative operator semigroup on the
bounded v25 surface: for every ordered triple `(O_i, O_j, O_k)` and every
state `s`, confirm

```
O_i ∘ (O_j ∘ O_k)  ==  (O_i ∘ O_j) ∘ O_k
```

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface and operators

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count | 525,355 |
| `T_b` | `torus_base_advance_component_v23` (non-transport) |
| `T_x` | `composite_swap_component_v24` (non-transport) |
| `T_y` | `composite_twist_component_v25` (non-transport) |
| Ordered triples checked | 27 (3³ = 27; all with repetition allowed) |

---

## Results per ordered triple

| Triple `(O_i, O_j, O_k)` | Agreement count | Agreement fraction | Associative? |
|---------------------------|-----------------|-------------------|--------------|
| `(T_b, T_b, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_b, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_b, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_x, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_x, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_x, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_y, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_y, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_b, T_y, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_b, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_b, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_b, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_x, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_x, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_x, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_y, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_y, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_x, T_y, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_b, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_b, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_b, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_x, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_x, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_x, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_y, T_b)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_y, T_x)` | 525,355 / 525,355 | 1.000000 | **yes** |
| `(T_y, T_y, T_y)` | 525,355 / 525,355 | 1.000000 | **yes** |

**Overall verdict: **PASS — all 27 triples associative**.**

---

## Interpretation

### What this confirms

All 27 ordered triples (including repetitions such as `(T_b, T_b, T_x)`) pass
with agreement fraction **1.000000** over all 525,355 states.

This confirms two things:

1. **The operators are total, deterministic functions** on the v25 state space.
   Neither parenthesation produces a partial-application failure, an exception,
   or a non-deterministic result.  Both paths `O_i(O_j(O_k(s)))` evaluate to
   the same state for every `s`.

2. **The non-transport sub-algebra `{T_b, T_x, T_y}` is associative** on the
   bounded v25 surface.  Under the operation of sequential function application,
   the set of compositions respects associativity exactly — not approximately.

### What this does not prove

- **It does not prove closure under composition** in the sense of group theory.
  Function composition always produces a well-typed result here (OperatorStateV10
  → OperatorStateV10), so closure at the type level is trivially satisfied.
  Whether the image of every composition remains within the reachable bounded
  surface is a separate question not addressed here.

- **It does not prove the existence of inverses or an identity element.**
  Associativity alone establishes a semigroup, not a group or monoid.

- **It does not probe non-trivial algebraic structure.**
  Function composition is always associative by mathematical construction
  (it is a fundamental property of the category of sets).  The test therefore
  cannot fail unless an operator is non-deterministic or ill-defined.
  Its value is as a consistency check: it rules out implementation errors
  (such as mutable state, non-pure functions, or Python object identity
  vs structural equality mismatches) that could break the semigroup property
  at the implementation level.

- **It is bounded to depth=8.** States reachable at depth > 8 are not covered.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — read-only audit on the accepted v25 surface |
| Were any operators rebuilt? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Run the same triple composition closure test for the transport sub-algebra
`{T_c, T_z', T_r*}` on the bounded v25 surface (27 ordered triples, same
methodology), producing a symmetric associativity record for both sub-algebras.
This completes the bounded algebraic characterisation of the full rebuilt layer.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
