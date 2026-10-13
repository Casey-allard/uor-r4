# Prime Transport Mock Router Module Implementation

## What Was Implemented

The first non-runtime implementation step is now present as a minimal offline
module:

- `tools/prime_transport/mock_router_module.py`

It implements the five functions from the mock router spec:

- `initialize_state(...)`
- `update_state(...)`
- `score_or_route(...)`
- `should_promote(...)`
- `promote_state(...)`

The module supports:

- baseline `R_static = (b, phi, r)`
- default `R_min = (b, phi, r, next_return_gap)`
- fallback `R_full = (b, spin_H)` only when promotion is triggered

## Demo Path

A tiny bounded demo runner is also present:

- `tools/prime_transport/run_mock_router_module_demo.py`

It exercises the module over the already-established offline summary rows and
shows:

- initial state construction
- one deterministic state update
- route/scoring decision
- promotion check
- fallback to `R_full` when the unresolved fraction crosses the offline
  promotion threshold

The demo output artifact is:

- `results/prime_transport_recursive_system/prime_transport_mock_router_demo.csv`

## Assumptions

This first implementation step stays intentionally small.

Assumptions used:

- the demo is driven by bounded offline summary rows rather than live per-step
  benchmark traces
- `next_return_gap` is represented directly as a cached residual field
- promotion threshold defaults to `0.30`, matching the offline reading that
  `R_min` leaves about `31%` of split-partition cases unresolved
- promotion is demonstrated locally and selectively, not globally

## Status

This is not a runtime router and does not touch the live MUDBench seam.

It is only the smallest executable mock module needed to make the first routing
prototype step concrete.
