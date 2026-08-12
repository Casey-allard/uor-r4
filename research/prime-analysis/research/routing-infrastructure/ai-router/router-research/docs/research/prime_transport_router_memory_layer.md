# Prime Transport Router Memory Layer

## Purpose

This note defines and records the smallest research-side prototype that treats
the prime-transport router as a structured read/write memory layer rather than
only as a benchmark-side context router.

The prototype stays fully research-only:

- no live MUDBench seam changes
- no model replacement
- no training code
- no downstream quotient objects

## Router-Memory Design

### Memory State

The active routing state is still:

- `R_min = (b, phi, r, next_return_gap)`

with selective promotion to:

- `R_full = (b, spin_H)`

The memory layer adds one persistent memory cell per reusable routing class:

- coarse memory key:
  `base_gap = (b, r, next_return_gap)`

Each cell stores:

- write count
- read count
- unresolved-fraction burden for that route class
- observed next-bit frequencies
- observed full branch-key frequencies
- promotion status

So the router is no longer only choosing a context partition. It is maintaining
stateful route-local memory that survives across steps and can be queried later.

### What A Write Means

A write means:

1. route the current state through the existing guarded `base_gap` policy
2. update the corresponding memory cell
3. record the observed local outcome:
   - immediate next-bit signal
   - full branch key
4. mark the cell promoted if unresolved burden requires `R_full`

This is an explicit memory update, not just a routing decision.

### What A Read Means

A read means:

1. form the current `base_gap` route key from the current `R_min` state
2. query the stored memory cell for that route class
3. return either:
   - a coarse summary read, or
   - a promoted exact branch read when that route class has been escalated

The read therefore returns remembered structure from earlier visits to the same
route class rather than just recomputing a one-shot context bucket.

### Promotion / Refinement

Promotion remains the same guarded mechanism already validated in the routing
work:

- start in `R_min`
- track unresolved predictive ambiguity per `base_gap` class
- promote only the classes whose ambiguity stays above threshold
- store exact branch memory only for those promoted classes

This keeps delayed refinement and fallback accounting explicit inside the
memory-layer architecture.

### How This Differs From Plain Context Routing

Plain context routing only emits a route key for the current step.

This memory-layer prototype instead supports:

- persistent route-class storage
- repeated writes into the same class
- later reads from the same class
- selective promotion into richer branch memory

So the architectural object is now:

- a small state machine with persistent read/write memory

not only:

- a benchmark-side context partition

## Smallest Prototype Task Family

The smallest bounded task family is:

- cyclic exact-trace memory tasks on the existing tightly matched prime
  transport traces

Why this family is sufficient:

- repeated state updates occur at every step
- route classes recur naturally on the bounded exact traces
- reads can be evaluated after a warmup write pass
- the traces are deterministic and already part of the validated research tree

The prototype therefore uses:

- one warmup pass to write memory
- one evaluation pass to read memory from the same bounded traces

## Implemented Prototype

### Module

- `tools/prime_transport/router_memory_layer.py`

Implemented operations:

- `initialize_memory_layer(...)`
- `initialize_memory_state(...)`
- `update_memory_state(...)`
- `query_memory(...)`
- `write_memory(...)`

### Demo

- `tools/prime_transport/run_router_memory_layer_demo.py`

### Result Artifact

- `results/prime_transport_recursive_system/prime_transport_router_memory_layer_demo.csv`

## Demo Result

Aggregate bounded result:

- memory slots: `169.0`
- promoted slot fraction: `0.2167907268170426`
- warm read-hit fraction: `0.8151848151848151`
- evaluation read-hit fraction: `1.0`
- evaluation promoted-read fraction: `0.45274725274725275`
- evaluation exact retrieval fraction: `0.719060939060939`
- evaluation effective resolution fraction: `0.9807594836295799`
- route decision instability: `0.0`

## Reading

What this shows:

- the router can be run coherently as a persistent read/write memory machine
- memory cells are reused after warmup
- promotion remains selective rather than global
- the resulting memory process stays stable

What it does **not** show:

- exact branch retrieval is still incomplete, especially on deeper lifts
- the current minimal memory layer is not yet a full replacement for the
  richer predictive structure in `R_full`

So the architectural result is:

- strong enough to justify continuing
- not strong enough to claim that the minimal router-memory state has solved
  full predictive retrieval

## Recommendation

Yes: this branch is strong enough to continue toward a larger router-native
architecture experiment.

The reason is architectural, not rhetorical:

- the router now behaves as a genuine stateful memory layer
- read/write/update/promotion operations are explicit
- the bounded prototype preserves the validated fallback discipline
- the memory machine achieves high effective resolution with zero instability

The next larger experiment, if opened later, should therefore scale this same
read/write memory discipline to a richer multi-step benchmark task rather than
going straight to model replacement or training.
