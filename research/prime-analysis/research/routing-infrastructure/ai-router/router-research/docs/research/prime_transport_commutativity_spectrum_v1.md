# Prime Transport Commutativity Spectrum v1

## Purpose

Characterise the full non-commutativity structure of the six-operator rebuilt layer
on the bounded v25 surface by computing the exact agreement fraction for every
unordered pair `{O_i, O_j}` drawn from the six non-hold operators.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface and operators

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count | 525355 |
| `T_b` | `torus_base_advance_component_v23` (non-transport, from `spin_H_core_v6`) |
| `T_x` | `composite_swap_component_v24` (non-transport, from `spin_H_core_v6`) |
| `T_c` | `coupled_torus_kick_component_v20` (transport, from `spin_H_core_v6`) |
| `T_y` | `composite_twist_component_v25` (non-transport, from `spin_H_core_v6`) |
| `T_z'` | `fiber_phase_lift_component_v21` (transport, from `spin_H_core_v6`) |
| `T_r*` | `radial_transport_component_v22` (transport, from `spin_H_core_v6`) |

---

## Symmetric agreement matrix

Entry `[i][j]` is the exact fraction of states `s` where `O_i(O_j(s)) == O_j(O_i(s))`.
Diagonal is 1.000000 (each operator commutes with itself).

|            |        T_b |        T_x |        T_c |        T_y |       T_z' |       T_r* |
| ---------- | ---------- | ---------- | ---------- | ---------- | ---------- | ---------- |
| T_b        | 1.000000   | 0.016431   | 0.001784   | 0.016412   | 0.000659   | 0.000202   |
| T_x        | 0.016431   | 1.000000   | 0.000171   | 0.018212   | 0.001016   | 0.000000   |
| T_c        | 0.001784   | 0.000171   | 1.000000   | 0.002337   | 0.000824   | 0.000482   |
| T_y        | 0.016412   | 0.018212   | 0.002337   | 1.000000   | 0.000670   | 0.001504   |
| T_z'       | 0.000659   | 0.001016   | 0.000824   | 0.000670   | 1.000000   | 0.000227   |
| T_r*       | 0.000202   | 0.000000   | 0.000482   | 0.001504   | 0.000227   | 1.000000   |

---

## Ranked list: all 15 unordered pairs (most commuting → least commuting)

| Rank | Pair | Agreement count | Agreement fraction | Img(O_i∘O_j) | Img(O_j∘O_i) | Overlap | Verdict |
|------|------|-----------------|--------------------|--------------|--------------|---------|---------|
| 1 | `{T_x, T_y}` | 9,568 / 525,355 | 0.018212 | 241,334 | 217,550 | 93,664 | DISTINCT |
| 2 | `{T_b, T_x}` | 8,632 / 525,355 | 0.016431 | 211,920 | 251,655 | 85,089 | DISTINCT |
| 3 | `{T_b, T_y}` | 8,622 / 525,355 | 0.016412 | 301,480 | 301,381 | 141,081 | DISTINCT |
| 4 | `{T_c, T_y}` | 1,228 / 525,355 | 0.002337 | 216,187 | 230,482 | 81,774 | DISTINCT |
| 5 | `{T_b, T_c}` | 937 / 525,355 | 0.001784 | 240,882 | 248,262 | 91,828 | DISTINCT |
| 6 | `{T_y, T_r*}` | 790 / 525,355 | 0.001504 | 280,229 | 281,756 | 126,138 | DISTINCT |
| 7 | `{T_x, T_z'}` | 534 / 525,355 | 0.001016 | 233,658 | 207,099 | 89,542 | DISTINCT |
| 8 | `{T_c, T_z'}` | 433 / 525,355 | 0.000824 | 207,798 | 225,625 | 76,382 | DISTINCT |
| 9 | `{T_y, T_z'}` | 352 / 525,355 | 0.000670 | 277,666 | 277,895 | 120,855 | DISTINCT |
| 10 | `{T_b, T_z'}` | 346 / 525,355 | 0.000659 | 287,021 | 293,857 | 128,839 | DISTINCT |
| 11 | `{T_c, T_r*}` | 253 / 525,355 | 0.000482 | 233,583 | 230,052 | 87,052 | DISTINCT |
| 12 | `{T_z', T_r*}` | 119 / 525,355 | 0.000227 | 270,761 | 272,474 | 120,375 | DISTINCT |
| 13 | `{T_b, T_r*}` | 106 / 525,355 | 0.000202 | 294,004 | 296,329 | 136,521 | DISTINCT |
| 14 | `{T_x, T_c}` | 90 / 525,355 | 0.000171 | 201,315 | 160,787 | 76,780 | DISTINCT |
| 15 | `{T_x, T_r*}` | 0 / 525,355 | 0.000000 | 236,469 | 219,778 | 105,802 | DISTINCT |

---

## Summary statistics

| Statistic | Value |
|-----------|-------|
| Minimum agreement fraction | 0.000000 |
| Maximum agreement fraction | 0.018212 |
| Median agreement fraction  | 0.000824 |
| Range (max − min)          | 0.018212 |
| Pairs with frac < 0.001    | 8 of 15 |
| Pairs with frac ≥ 0.01     | 3 of 15 |

---

## Interpretation

The 15 agreement fractions span a range of 0.018212 (min 0.000000, max 0.018212, median 0.000824).
The distribution is **not uniformly flat**: two distinct bands are visible.

**Intra-layer pairs (non-transport × non-transport: T_b, T_x, T_y):**
Agreement fractions cluster in the range ~0.016–0.018 (≈1.6–1.8%).
These pairs are non-commutative but show measurable partial agreement,
reflecting shared sigma-mediated structure via `spin_H_core_v6`.

**Cross-layer and transport-only pairs:**
Agreement fractions fall in the range ~0.000 (≈0.01–0.10%).
Cross-layer pairs (non-transport × transport) and transport × transport pairs
are substantially more non-commutative than intra non-transport pairs.
This confirms that the transport and non-transport sub-algebras occupy
structurally distinct regions of the operator space.

The rebuilt algebra has a **varied commutativity structure**: the spectrum
is not flat, and the two sub-algebras are distinguishable by their
intra-layer agreement levels. No pair is collapsed.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — this is a read-only audit on the accepted v25 surface |
| Were any operators rebuilt in this step? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Compute the triple composition closure test: for a sampled subset of states on
the v25 surface, verify that `O_i ∘ (O_j ∘ O_k) == (O_i ∘ O_j) ∘ O_k` holds
exactly for all 20 ordered triples drawn from `{T_b, T_x, T_y}` (the
non-transport sub-algebra), confirming associativity of the rebuilt non-transport
layer as a concrete operator semigroup.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
