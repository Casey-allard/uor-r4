# Prime Transport Base-Gap Adapter Integration

## Purpose

This note records the first guarded non-runtime integration step for the
current prime-transport routing policy.

The packaged policy is:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

This adapter does **not** touch the live MUDBench router seam and does **not**
replace any existing router path.

## Adapter Contract

### What The Adapter Receives

The guarded adapter receives exact-layer inputs only:

- current base phase `b`
- current fiber coordinates `phi`
- wheel depth `r`
- current `next_return_gap`
- optional `spin_H` only when fallback is needed
- optional per-step update information:
  - fiber moduli
  - next admissibility bit
  - next return gap

### What The Adapter Stores

The adapter stores:

- the current `R_min` state for the active item
- a route-policy cache keyed by `base_gap`
- a promotion threshold consistent with the existing offline prototype

### How It Computes The Routing Key

For a minimal state `R_min = (b, phi, r, next_return_gap)`, the adapter emits:

- `base_gap:r={r}:b={b}:gap={next_return_gap}`

This keeps the stored state richer than the routing partition while preserving
the current exact-layer design:

- rich local state
- coarse reusable route class
- explicit fallback when unresolved

### When It Promotes

The adapter promotes only when:

- the cached or newly computed unresolved fraction for the current `base_gap`
  class exceeds the configured threshold

Promotion is local and selective:

- promoted cases fall back to `R_full = (b, spin_H)`
- unpromoted cases stay in `R_min`

### What It Returns

The adapter returns:

- emitted route key
- route mode (`R_min` or `R_full`)
- promotion flag
- resulting state object

## Implemented Module Boundary

The guarded research-side adapter module is:

- `tools/prime_transport/base_gap_routing_adapter.py`

It exposes:

- `initialize_adapter(...)`
- `initialize_entry(...)`
- `step_entry(...)`
- `route_decision(...)`
- `promotion_fallback(...)`
- `adapter_output(...)`

This is the smallest callable boundary that packages the selected policy into a
form suitable for later benchmark-harness hookup.

## Tiny Benchmark-Like Demo

The tiny guarded demo is:

- `tools/prime_transport/run_base_gap_routing_adapter_demo.py`

It runs the adapter in a benchmark-like loop on one bounded trace from each
trace source and shows:

- repeated route-key reuse
- stable cached route decisions
- explicit promotion to `R_full`
- fallback resolution without any live router integration

The demo artifact is:

- `results/prime_transport_recursive_system/prime_transport_base_gap_routing_adapter_demo.csv`

Aggregate demo reading:

- average route-policy cache size: `177.5`
- average route reuse fraction: `0.8386613386613386`
- average promotion step fraction: `0.4615384615384615`
- average effective resolved fraction: `0.966533466533467`
- route decision instability: `0.0`

## First Future Hookup Point

The first future benchmark-harness hookup point should be:

- a research-side harness wrapper that already iterates per-position exact trace
  rows

In practice, the cleanest first hookup point is a non-runtime wrapper around
the same bounded trace-building path currently used by:

- `run_mock_router_trace_eval.py`
- `run_base_gap_routing_loop_eval.py`

That keeps the adapter:

- offline
- observable
- and separate from the live MUDBench router seam

## Recommendation

The guarded adapter is now mature enough to serve as the first non-runtime
integration boundary for the current routing policy.

The next step should be:

- connect this adapter to a research-only benchmark-style harness boundary
- keep promotion and fallback explicitly logged
- avoid any live router or MUDBench seam changes until that bounded hookup has
  been exercised cleanly
