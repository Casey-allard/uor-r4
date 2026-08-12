# Prime Transport Commutativity Structure v1

## Purpose

Derive the algebra-structure interpretation of the rebuilt six-operator layer
from the already-computed `prime_transport_commutativity_spectrum_v1.csv`.
No new operator evaluations were performed.

---

## Surface and data source

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count | 525,355 |
| Source data | `prime_transport_commutativity_spectrum_v1.csv` (15 unordered pairs) |
| Operators analysed | T_b, T_x, T_c, T_y, T_z', T_r* (all non-hold, all from `spin_H_core_v6`) |

---

## Commutativity bands

The 15 agreement fractions separate into four bands with no ambiguous cases:

| Band | Range | Pair count | Description |
|------|-------|-----------|-------------|
| A (high)  | ≥ 0.010    | 3  | Non-transport intra-cluster only |
| B (mid)   | 0.001–0.010 | 4  | Cross-cluster pairs with partial structural coupling |
| C (low)   | 0.0001–0.001 | 7 | Transport intra-cluster + weakest cross-cluster |
| D (zero)  | = 0.000000  | 1  | Maximally non-commutative pair |

### Band A — Non-transport intra cluster (≥ 0.010)

| Pair | Agreement fraction |
|------|--------------------|
| `{T_x, T_y}` | **0.018212** |
| `{T_b, T_x}` | 0.016431 |
| `{T_b, T_y}` | 0.016412 |

All three are intra non-transport pairs.
Mean Band A fraction: **0.017018**.
The three non-transport operators are internally the most commutative subgroup,
reflecting shared sigma-mediated structure through `spin_H_core_v6`.

### Band B — Cross-cluster (mid coupling, 0.001–0.010)

| Pair | Agreement fraction | Note |
|------|--------------------|------|
| `{T_c, T_y}` | 0.002337 | Strongest cross-cluster pair |
| `{T_b, T_c}` | 0.001784 | T_b and T_c share torus-coordinate semantics |
| `{T_y, T_r*}` | 0.001504 | T_y bridges toward radial transport |
| `{T_x, T_z'}` | 0.001016 | Marginal mid-band; weakest Band B entry |

All four Band B pairs are cross-cluster (one non-transport, one transport).
No intra-cluster pair appears in Band B.
Mean Band B fraction: **0.001660**.

### Band C — Transport intra and weak cross-cluster (0.0001–0.001)

| Pair | Agreement fraction | Same cluster? |
|------|--------------------|---------------|
| `{T_c, T_z'}`  | 0.000824 | yes (transport) |
| `{T_y, T_z'}`  | 0.000670 | no |
| `{T_b, T_z'}`  | 0.000659 | no |
| `{T_c, T_r*}`  | 0.000482 | yes (transport) |
| `{T_z', T_r*}` | 0.000227 | yes (transport) |
| `{T_b, T_r*}`  | 0.000202 | no |
| `{T_x, T_c}`   | 0.000171 | no |

All three transport intra pairs (`{T_c,T_z'}`, `{T_c,T_r*}`, `{T_z',T_r*}`)
fall in Band C.  Mean transport intra fraction: **0.000511**.
Four cross-cluster pairs also fall in Band C, all involving T_z' or T_r*.
Mean Band C fraction: **0.000462**.

### Band D — Zero agreement

| Pair | Agreement count | Agreement fraction |
|------|-----------------|--------------------|
| `{T_x, T_r*}` | 0 / 525,355 | **0.000000** |

`T_x` (composite swap) and `T_r*` (radial transport unfolding) are maximally
non-commutative: no state `s` in the 525,355-state bounded v25 surface satisfies
`T_x(T_r*(s)) == T_r*(T_x(s))`.  This is a structural hard separation, not a
sampling artifact — it holds over the full reachable bounded surface.

---

## Cluster structure

The six operators partition naturally into two clusters aligned exactly with the
transport / non-transport split:

| Cluster | Operators | Mean intra-cluster fraction |
|---------|-----------|-----------------------------|
| **Non-transport** | T_b, T_x, T_y | **0.017018** |
| **Transport**     | T_c, T_z', T_r* | **0.000511** |
| Cross-cluster (all 9 pairs) | — | 0.000927 |

The non-transport cluster is **33.3×** more internally commutative than the
transport cluster.  The cross-cluster mean (0.000927) sits between the two but
is 18.4× below the non-transport intra mean, confirming that the partition is
structurally meaningful and not an artefact of the surface size.

---

## Bridge operators

Not all operators are equally isolated within their cluster.
The per-operator cross-cluster mean quantifies how much each operator reaches
across the cluster boundary:

| Operator | Cluster | Cross-cluster mean fraction | Interpretation |
|----------|---------|-----------------------------|----------------|
| **T_y**  | Non-transport | **0.001504** | Primary non-transport bridge; highest cross-cluster coupling of any operator |
| **T_c**  | Transport     | **0.001431** | Primary transport bridge; highest cross-cluster coupling on the transport side |
| T_b      | Non-transport | 0.000882 | Secondary non-transport bridge via torus-coordinate proximity to T_c |
| T_z'     | Transport     | 0.000782 | Moderate transport bridge |
| T_r*     | Transport     | 0.000569 | Weakly bridging; zero coupling to T_x |
| **T_x**  | Non-transport | **0.000396** | Most cluster-isolated non-transport operator; zero coupling to T_r* |

**T_y is the primary bridge operator.** Its cross-cluster mean (0.001504) is 3.8× higher
than T_x's (0.000396) and exceeds the transport intra mean (0.000511).
T_y reaches into all three transport operators, with its strongest link to T_c.

**T_x is the most cluster-isolated non-transport operator.** Despite being the
most internally commutative (tied for rank 1 via `{T_x,T_y}` and rank 2 via
`{T_b,T_x}`), T_x has the lowest cross-cluster mean and achieves exactly zero
agreement with T_r*.

---

## Most and least commuting pairs

| | Pair | Agreement fraction |
|-|------|--------------------|
| **Most commuting** | `{T_x, T_y}` | 0.018212 |
| **Least commuting** | `{T_x, T_r*}` | 0.000000 |

Both extremes involve T_x, which acts as a structural anchor: maximally close to
T_y within the non-transport cluster and maximally separated from T_r* across the
cluster boundary.

---

## Algebra structure interpretation

The rebuilt six-operator layer has a **non-uniform, two-cluster commutativity
structure** with the following properties:

1. **Hard cluster partition.** The transport / non-transport split is not a
   labelling convention — it is confirmed by a factor-of-33 difference in mean
   intra-cluster agreement fractions.  The partition was imposed by the `spin_H_core_v6`
   rebuild chain but is measured here purely from output distributions.

2. **Non-transport cluster is tightly coupled.** All three intra non-transport
   fractions are within 0.002 of each other (0.016412–0.018212), indicating the
   three operators share similar sigma-mediation depth through `spin_H_core_v6`.

3. **Transport cluster is loosely coupled.** The three intra-transport fractions
   span 0.000227–0.000824, a factor-of-3.6 range, indicating that T_c, T_z',
   and T_r* are structurally more independent of each other than the non-transport
   operators are of each other.

4. **No pair is collapsed.** Every ordered composition produces a non-singleton
   image set.  The algebra is everywhere non-trivial.

5. **One maximally non-commutative pair.** `{T_x, T_r*}` achieves exactly zero
   agreement, meaning the composite-swap and radial-transport-unfolding operators
   operate on entirely disjoint coordinate projections — their order of application
   never produces the same state.

6. **Bridge structure is asymmetric.** T_y bridges toward transport more than T_b
   or T_x; T_c bridges toward non-transport more than T_z' or T_r*.  The two
   clusters are connected primarily through the T_y–T_c link (fraction 0.002337).

---

## Summary statistics (from existing spectrum data)

| Statistic | Value |
|-----------|-------|
| Minimum agreement fraction | 0.000000 (`{T_x, T_r*}`) |
| Maximum agreement fraction | 0.018212 (`{T_x, T_y}`) |
| Median agreement fraction | 0.000824 |
| Non-transport intra mean | 0.017018 |
| Transport intra mean | 0.000511 |
| Cross-cluster mean | 0.000927 |
| NT-intra / TR-intra ratio | 33.3× |
| NT-intra / cross-cluster ratio | 18.4× |
| Pairs in Band A (≥ 0.010) | 3 of 15 |
| Pairs in Band B (0.001–0.010) | 4 of 15 |
| Pairs in Band C (0.0001–0.001) | 7 of 15 |
| Pairs in Band D (= 0.000000) | 1 of 15 |

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — this is a pure analysis of the existing `prime_transport_commutativity_spectrum_v1.csv`; no code was executed against the operator surface |
| Were any operators rebuilt? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Compute the triple composition closure test: for every ordered triple `(O_i, O_j, O_k)`
drawn from the non-transport sub-algebra `{T_b, T_x, T_y}` (20 ordered triples from
3 operators with repetition excluded: 3×2×1 = 6 ordered triples with all distinct,
plus 3×3×2 = 18 with one repeat — or simply all distinct ordered triples of the 3
operators), verify that `O_i ∘ (O_j ∘ O_k) == (O_i ∘ O_j) ∘ O_k` holds exactly
on the bounded v25 surface, confirming the non-transport layer is associative as
a concrete operator semigroup.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
