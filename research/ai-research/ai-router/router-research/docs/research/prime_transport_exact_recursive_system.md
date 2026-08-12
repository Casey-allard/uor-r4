# Prime Transport Exact Recursive System

## Scope

This note formalizes the prime admissibility system at the correct layer for
current work.

The project should be organized around the **exact recursive dynamical
system**:

1. exact transport orbit `n(j) = 5 + 6j`
2. odometer update `j -> j + 1`
3. exact phase-fiber-scale state `j -> (b, phi, r)`
4. recursive affine lift law across wheel depth
5. finite-horizon spin `spin_H`
6. compressed predictive state `(b, spin_H)`
7. delayed visibility of new refinement layers

For this task, downstream quotient objects such as `C^2`, `A_*`, packet
recoveries, or any quotient-geometry target are **not** primary. They should
not be used as the explanatory center here.

## Exact Definitions

### 1. Transport Orbit

Fix admissibility offsets `A = [0, 2, 6, 8]`.

For wheel depth `r`, let

- `W_r = 2 * 3 * 5 * 7 * Π q_m`
- `L_r = W_r / 6`

The transport orbit is

- `j in Z / L_r Z`
- `n_r(j) = 5 + 6j (mod W_r)`

The dynamics is the odometer update

- `j -> j + 1`

All structure is induced by filtering this single orbit.

### 2. Exact Admissibility Word

Define the exact admissibility bit

- `w_r(j) = 1`
  iff `gcd(n_r(j) + a, W_r) = 1` for all `a in A`

This gives a cyclic binary word on the orbit.

### 3. Phase-Fiber-Scale State

At finite depth, the exact state is organized as

- `j -> (b(j), phi(j), r)`

where

- `b(j) = j mod 35` is the base phase
- `phi(j)` is the refinement fiber phase, obtained from the higher-layer cyclic
  coordinates above the `2 * 3 * 5 * 7` base
- `r` is wheel depth

This is the exact finite-depth torus-valued state description.

### 4. Recursive Affine Lift Law

For a wheel extension `W -> pW`, admissibility is not rebuilt from scratch.
It is generated recursively from:

- a local stencil modulo the new prime `p`
- an affine shift inherited from lower layers

So each deeper wheel is an affine refinement of the lower wheel’s transport
structure.

### 5. Finite-Horizon Spin

For horizon `H >= 1`, define the finite-horizon spin by exact future-word
equivalence:

- `spin_H^W(j) = (w_W(j), w_W(j+1), ..., w_W(j+H-1))`

with indices taken cyclically modulo `L_W`.

This is an exact finite-horizon predictive summary of the orbit.

### 6. Compressed Predictive State

The working compressed predictive state is

- `sigma_H^W(j) = (b(j), spin_H^W(j))`

This is the correct compression object to track before any downstream quotient
geometry is attempted.

### 7. Delayed Visibility

For a wheel lift `W_parent -> W_child = p W_parent`, let

- `pi(j_child) = j_child mod L_parent`

be the exact projection from child orbit positions to parent orbit positions.

At horizon `H`, define:

- **internal split**:
  a parent position has internally refined if two child lifts over the same
  parent position have different `spin_H` words
- **visible split**:
  the child wheel has more unique predictive states `(b, spin_H)` than the
  parent wheel at the same horizon

Delayed visibility means internal split can occur strictly earlier than visible
split.

## Bounded Exact-State Experiment

The bounded experiment for this note is:

- script:
  [tools/prime_transport/run_recursive_state_visibility.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_recursive_state_visibility.py)
- detail results:
  [results/prime_transport_recursive_system/recursive_state_visibility_detail.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/recursive_state_visibility_detail.csv)
- summary results:
  [results/prime_transport_recursive_system/recursive_state_visibility_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/recursive_state_visibility_summary.csv)

What it measures:

1. exact admissibility words on the transport orbit
2. exact finite-horizon `spin_H`
3. predictive-state counts for `(b, spin_H)`
4. lift consistency under projection from deeper wheel to shallower wheel
5. first internal split horizon
6. first visible split horizon

The script checks two exact wheel lifts:

- `30030 -> 510510` (new prime `17`)
- `510510 -> 9699690` (new prime `19`)

with horizons `H = 1 .. 60`.

## What Was Confirmed

### 1. Internal refinement appears before visible refinement

For both tested wheel lifts, internal split is active immediately:

- `30030 -> 510510`: first internal split at `H = 1`
- `510510 -> 9699690`: first internal split at `H = 1`

So newly added layers affect the exact lift structure before any visible
predictive-state count increase is required.

### 2. Visible predictive-state refinement can be strongly delayed

For the `17`-layer lift:

- first visible split at `H = 51`
- delay from first internal split to first visible split: `50`

For the `19`-layer lift:

- no visible predictive-state count increase was observed for `H <= 60`
- internal split was still active from `H = 1`

So the checked exact system shows a strong delayed-visibility effect.

### 3. Projection consistency degrades as horizon grows

For each wheel lift, the fraction of parent positions whose child lifts remain
spin-consistent decreases as `H` increases.

Examples from the saved CSV:

- `30030 -> 510510` at `H = 56`:
  consistency rate `0.014785214785214773`
- `510510 -> 9699690` at `H = 56`:
  consistency rate `0.07648821766468827`

This confirms that deeper layers inject hidden future refinement long before it
necessarily appears as a larger visible predictive-state count.

## Conservative Interpretation

The exact recursive system itself is already rich enough to support:

- recursive affine refinement across wheel depth
- exact phase-fiber-scale organization
- finite-horizon predictive compression
- delayed visibility of newly added layers

That is a meaningful research object on its own.

The present result does **not** establish:

- a prime oracle
- a closed-form theorem for all layers
- any quotient geometry as primary

It only confirms that the exact recursive-system framing is operational and
empirically measurable without appealing to downstream quotient targets.

## Current Working Conclusion

Before any low-dimensional quotient geometry is attempted, the prime
admissibility system should be treated as:

- an exact recursive dynamical system on the odometer orbit
- with finite-depth state `(b, phi, r)`
- and compressed predictive state `(b, spin_H)`
- whose deeper layers become internally active before they become visibly
  distinct in predictive-state counts

That is the stable internal reference point for further implementation work.
