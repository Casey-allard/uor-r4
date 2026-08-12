# Prime Transport Base-Gap Benchmark Wrapper

## Purpose

This note records the first research-only benchmark wrapper that hooks the
guarded `base_gap` routing adapter into the existing bounded trace-building
path, without touching any live router seam.

The wrapped policy remains:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

## Minimal Trace-Building Path

The wrapper reuses the same bounded exact trace construction already used by:

- `run_mock_router_trace_eval.py`
- `run_base_gap_routing_loop_eval.py`

Specifically, it reuses:

- bounded trace sources from `TRACE_SPECS`
- admissibility reconstruction
- phase tuple reconstruction
- `next_return_gap` reconstruction
- `spin_H` construction at the known visible horizon
- unresolved-fraction computation against `R_full`

So the wrapper introduces no new data path. It only packages the existing one
behind a research-side benchmark boundary.

## Implemented Wrapper

The research-only wrapper module is:

- `tools/prime_transport/base_gap_benchmark_wrapper.py`

It exposes:

- `build_bounded_traces(...)`
- `run_trace(...)`
- `summarize_rows(...)`

The tiny driver is:

- `tools/prime_transport/run_base_gap_benchmark_wrapper.py`

The summary artifact is:

- `results/prime_transport_recursive_system/prime_transport_base_gap_benchmark_wrapper.csv`

## Bounded Benchmark Reading

Aggregate wrapper results:

- average route-policy cache size: `169.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- effective resolved fraction: `0.9813186813186814`
- mean unresolved among nonpromoted routes: `0.014803465594151983`
- route decision instability: `0.0`
- average unique emitted route keys: `301.0`

These numbers match the established bounded-loop reading, which is the main
thing this wrapper needed to preserve.

## Recommendation

The guarded adapter is ready for a future guarded benchmark-harness
experiment.

The smallest future hookup point is:

- a research-only harness wrapper around the bounded trace-building path now
  packaged in `base_gap_benchmark_wrapper.py`

That is the right next step because it:

- keeps the policy fully outside the live MUDBench seam
- preserves the current bounded exact-layer evidence path
- and exposes route key, route mode, promotion, and fallback behavior in a
  benchmark-like interface rather than in ad hoc evaluation scripts

## Conservative Boundary

This is still not a production router and not a live benchmark integration.

It is only the first guarded non-runtime benchmark boundary for the selected
`base_gap` routing policy.
