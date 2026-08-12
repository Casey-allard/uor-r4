# Prime Transport Threshold Law

## Purpose

This note records the current exact-layer threshold table for delayed
visibility in the prime admissibility system.

It stays entirely inside the canonical recursive-system framework:

- transport orbit `n(j) = 5 + 6j`
- symbolic admissibility word
- compressed predictive state `(b, spin_H)`
- wheel lifts `W -> pW`

No quotient geometry is used here.

## Definitions Used

Fix a tuplet pattern `A` and a wheel lift `W_parent -> W_child = p W_parent`.

For horizon `H`:

- `spin_H(j)` is the exact length-`H` admissibility future word at position `j`
- the compressed predictive state is `(b(j), spin_H(j))`

The threshold definitions used in the table are:

- **internal split at horizon `H`**:
  there exists a parent orbit position whose child lifts contain at least two
  positions with the same current `spin_H` word but different next admissibility
  bit
- **visible split at horizon `H`**:
  the number of distinct compressed predictive states `(b, spin_H)` on the
  child wheel is strictly larger than on the parent wheel
- **lag**:
  `first_visible_split_H - first_internal_split_H`

This internal-split definition captures hidden divergence in one-step future
evolution before that divergence necessarily appears as a larger visible
compressed-state count.

## Artifact Locations

- note:
  [docs/research/prime_transport_threshold_law.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_threshold_law.md)
- runner:
  [tools/prime_transport/run_threshold_law_table.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_threshold_law_table.py)
- summary table:
  [results/prime_transport_recursive_system/threshold_law_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv)
- detail table:
  [results/prime_transport_recursive_system/threshold_law_detail.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_detail.csv)

## Threshold Summary

Tested wheel lifts:

- `210 -> 2310` with new prime `11`
- `2310 -> 30030` with new prime `13`
- `30030 -> 510510` with new prime `17`
- `510510 -> 9699690` with new prime `19`

Tested tuplets:

- twins `[0,2]`
- triplet `[0,2,6]`
- quadruplet `[0,2,6,8]`

Measured summary:

| Tuplet | Lift | New Prime | First Internal | First Visible | Lag | Visible Status |
|---|---|---:|---:|---:|---:|---|
| twins | `210 -> 2310` | 11 | 1 | 1 | 0 | exact |
| triplet | `210 -> 2310` | 11 | 1 | 1 | 0 | exact |
| quadruplet | `210 -> 2310` | 11 | 1 | 1 | 0 | exact |
| twins | `2310 -> 30030` | 13 | 1 | 2 | 1 | exact |
| triplet | `2310 -> 30030` | 13 | 1 | 6 | 5 | exact |
| quadruplet | `2310 -> 30030` | 13 | 1 | 21 | 20 | exact |
| twins | `30030 -> 510510` | 17 | 1 | 4 | 3 | exact |
| triplet | `30030 -> 510510` | 17 | 1 | 16 | 15 | exact |
| quadruplet | `30030 -> 510510` | 17 | 1 | 51 | 50 | exact |
| twins | `510510 -> 9699690` | 19 | 1 | 6 | 5 | exact |
| triplet | `510510 -> 9699690` | 19 | 1 | 26 | 25 | exact |
| quadruplet | `510510 -> 9699690` | 19 | 1 | not found for `H <= 60` | n/a | range-censored |

## What Is Supported

### 1. Internal split is immediate in the current table

For every tested lift and every tested tuplet, the first internal split horizon
is

- `first_internal_split_H = 1`

So in the current exact table, newly added layers become internally active
immediately.

### 2. Visible split is often much later

Visible split is not usually immediate once the lift prime grows or the tuplet
becomes denser.

Observed lags:

- twins: `0, 1, 3, 5`
- triplet: `0, 5, 15, 25`
- quadruplet: `0, 20, 50`, and not yet visible by `H = 60` for the `19` lift

So internal split often occurs much earlier than visible split.

### 3. Larger tuplets show larger visible delays in the tested lifts

For each fixed lift in the tested table, visible split occurs no earlier when
moving from twins to triplet to quadruplet.

Examples:

- at prime `13`: visible `2, 6, 21`
- at prime `17`: visible `4, 16, 51`
- at prime `19`: visible `6, 26, >60`

So the currently tested data supports a strong dependence of visible threshold
on tuplet pattern complexity.

### 4. A naive “lag is usually small” law is unsupported

The table directly contradicts any simple claim that visible split typically
follows internal split by only a few steps.

That is false already for:

- quadruplet at `13`: lag `20`
- quadruplet at `17`: lag `50`
- triplet at `17`: lag `15`
- triplet at `19`: lag `25`

### 5. A naive law in the new prime alone is unsupported

The visible threshold does tend to increase along the tested lift sequence, but
the tuplet pattern matters strongly.

For example:

- at `p = 19`, twins split visibly at `6`
- at `p = 19`, triplet splits visibly at `26`
- at `p = 19`, quadruplet is still not visibly split for `H <= 60`

So no law in the new prime `p` alone is supported by the current exact table.

## Conservative Interpretation

The exact recursive system supports a clear two-stage threshold phenomenon:

1. immediate internal activation of the new layer
2. potentially long delay before visible compressed-state splitting

The magnitude of that delay depends strongly on the tuplet pattern and grows
substantially across the tested lifts for denser tuplets.

What is **not** supported yet:

- a closed-form threshold law in `p`
- a universal small-lag rule
- any Fibonacci-style law

The current result is a compact exact table, not a theorem.

## Next Preferred Exact-Layer Step

The next preferred step should remain at the exact recursive-system layer:

formalize candidate bounds or recurrence rules for

- `H_internal_first(A, W -> pW)`
- `H_visible_first(A, W -> pW)`
- `lag(A, W -> pW)`

and test those candidate laws on a broader but still exact family of tuplets and
successive wheel lifts.

The natural first narrowing is:

- keep the same threshold definitions
- add a few more admissible tuplets of different stencil density
- test whether visible threshold is better predicted by:
  - new prime `p`
  - number of offsets in `A`
  - local stencil density
  - or a combination of these

That remains fully inside the exact recursive-system framework.
