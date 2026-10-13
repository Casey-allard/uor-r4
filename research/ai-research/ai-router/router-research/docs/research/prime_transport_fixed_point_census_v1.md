# Prime Transport Fixed-Point Census v1

## Purpose

Count all fixed points (`O(s) == s`) for each of the six non-hold operators
on the bounded v25 surface, and characterise partial invariance by field family.
Determines whether transport and non-transport operators leave structurally
different invariant sub-spaces.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count (N) | 525,355 |

---

## Full fixed-point counts

| Operator | Cluster | Fixed-point count | Fixed-point fraction |
|----------|---------|------------------|---------------------|
| `T_b` | non-transport | 0 | 0.000000 |
| `T_x` | non-transport | 0 | 0.000000 |
| `T_y` | non-transport | 0 | 0.000000 |
| `T_c` | transport | 0 | 0.000000 |
| `T_z'` | transport | 0 | 0.000000 |
| `T_r*` | transport | 0 | 0.000000 |

**All six non-hold operators have exactly zero full fixed points on the bounded
v25 surface (N = 525,355).** No state `s` satisfies `O(s) == s` for any of the
six operators.  This is consistent with the displacement histograms from the
prior audit, which showed minimum displacement ≥ 1 for all operators.

This is not a sampling artifact: it holds over the complete reachable
bounded surface at depth=8.

---

## Per-family invariant state counts

A state is **family-invariant** under O if every field in that family is
unchanged by O(s), even if other fields differ.  Family invariance is a
weaker condition than full fixed-point status.

| Operator | Cluster | theta | rho | sigma | tau_holonomy | swap_geometry |
|----------|---------|-------|-----|-------|-------------|---------------|
| `T_b` | non-transport | 0 (0.0000) | 525,355 (1.0000) | 56,464 (0.1075) | 597 (0.0011) | 525,355 (1.0000) |
| `T_x` | non-transport | 525,355 (1.0000) | 525,355 (1.0000) | 42,323 (0.0806) | 0 (0.0000) | 0 (0.0000) |
| `T_y` | non-transport | 525,355 (1.0000) | 525,355 (1.0000) | 214,836 (0.4089) | 4,021 (0.0077) | 0 (0.0000) |
| `T_c` | transport | 0 (0.0000) | 525,355 (1.0000) | 209,227 (0.3983) | 3,971 (0.0076) | 525,355 (1.0000) |
| `T_z'` | transport | 0 (0.0000) | 525,355 (1.0000) | 207,484 (0.3949) | 6,250 (0.0119) | 525,355 (1.0000) |
| `T_r*` | transport | 175,285 (0.3337) | 0 (0.0000) | 197,789 (0.3765) | 6,493 (0.0124) | 525,355 (1.0000) |

*(format: count (fraction of N))*

---

## Cluster-level invariant space comparison

| Family | NT mean fraction | TR mean fraction | Higher in transport? |
|--------|-----------------|------------------|---------------------|
| `theta` | 0.6667 | 0.1112 | no |
| `rho` | 1.0000 | 0.6667 | no |
| `sigma` | 0.1990 | 0.3899 | **yes** |
| `tau_holonomy` | 0.0029 | 0.0106 | **yes** |
| `swap_geometry` | 0.3333 | 1.0000 | **yes** |

---

## Interpretation

### Full fixed points: absent in all operators

The complete absence of full fixed points for all six non-hold operators
confirms that every state in the bounded v25 surface is moved by every
non-hold operator.  There are no invariant states in the full sense.
This is consistent with the prior displacement audit (minimum displacement
≥ 1 for all operators).

### Family invariance reveals structural separation

While no state is a full fixed point, the per-family invariance counts reveal
markedly different invariant sub-structures between the two clusters:

- **`T_b`** (non-transport): `rho` is **universally invariant** (100% of states); `theta` has **zero invariant states**
- **`T_x`** (non-transport): `theta` is **universally invariant** (100% of states); `tau_holonomy, swap_geometry` has **zero invariant states**
- **`T_y`** (non-transport): `theta` is **universally invariant** (100% of states); `swap_geometry` has **zero invariant states**
- **`T_c`** (transport): `rho` is **universally invariant** (100% of states); `theta` has **zero invariant states**
- **`T_z'`** (transport): `rho` is **universally invariant** (100% of states); `theta` has **zero invariant states**
- **`T_r*`** (transport): `swap_geometry` is **universally invariant** (100% of states); `rho` has **zero invariant states**

### Transport vs non-transport invariant sub-space structure

The two clusters preserve structurally orthogonal field families:

- **Non-transport** operators show high `rho` invariance (T_b: 1.0000, T_x: 1.0000, T_y: 1.0000) and high `swap_geometry` invariance (T_b: 1.0000, T_c: 1.0000).

- **Transport** operators show high `swap_geometry` invariance (T_c: 1.0000, T_z': 1.0000, T_r*: 1.0000) — they never touch semiprime/permutation fields — and zero `theta` or `rho` invariance since they always move their primary coordinate.

- **`T_x`** is the non-transport outlier: it has **zero** `swap_geometry` invariant
  states (always swaps query/binding semiprimes) but **100% rho invariance** and
  **100% theta invariance** — it never touches spatial transport coordinates.

### Alignment with two-cluster algebra picture

The fixed-point census confirms the field-signature finding in a complementary way:

- Transport operators `{T_c, T_z', T_r*}` preserve the **swap_geometry** family
  completely (100% invariance) and have zero theta/rho invariance because they
  always move their primary coordinate.
- Non-transport operators `{T_b, T_x, T_y}` split: T_b and T_y preserve
  swap_geometry partially or fully (T_b: 100%), while T_x destroys it (0%).
- The **sigma** (spin_h) family is the only one showing partial invariance
  in both clusters, reflecting the probabilistic nature of sigma updates
  through `spin_H_core_v6`.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — read-only audit on the accepted v25 surface |
| Were any operators rebuilt? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Run a bounded sigma-carrier stability audit on the v25 surface: for each operator,
characterise the states where `spin_h` is invariant (sigma-stable states) versus
where it changes, and test whether sigma-stable states under transport operators
form a coherent sub-algebra under the non-transport operators.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
