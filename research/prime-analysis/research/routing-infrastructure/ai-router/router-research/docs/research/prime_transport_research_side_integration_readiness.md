# Prime Transport Research-Side Integration Readiness

## Policy Under Consideration

The current prime-transport routing policy is:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

This remains an exact-layer routing abstraction only. It does not use any
downstream quotient geometry.

## What Is Already Established

### Exact-Layer Basis

The policy is grounded in the exact recursive-system picture:

- transport orbit `n(j) = 5 + 6j`
- exact chart `j -> (b, phi, r)`
- finite-horizon spin as the stronger predictive state
- delayed visibility with immediate internal split and delayed visible split
- canonical threshold-law class `density + first-splitting event`

### Offline Prototype Behavior

The current policy has already been exercised through:

- mock router state/update logic
- real bounded per-position traces
- coarser grouping-key selection
- bounded routing-loop evaluation
- guarded adapter packaging
- research-only benchmark wrapper execution

### Grouping-Key Choice

The first usable routing partition is now fixed as:

- `base_gap = (b, r, next_return_gap)`

This was chosen because it is:

- much more reusable than the literal `R_min` identity
- much less promotion-heavy than `gap_only`
- and still exact-layer and interpretable

### Wrapper Boundary

The current research-only integration boundary is:

- `/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/base_gap_benchmark_wrapper.py`

with the guarded adapter packaged in:

- `/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/base_gap_routing_adapter.py`

This boundary already constructs the bounded traces, runs the policy, and logs
route key, route mode, promotion, and fallback behavior.

### Guarded Benchmark Comparison

On the fixed bounded trace family:

- `static_only` is exact but address-like and therefore not a reusable routing
  policy
- `gap_only` is highly reusable but resolves well only by promoting almost
  everything
- `base_gap` sits in the intended middle position

Aggregate `base_gap` metrics:

- unique route keys: `169.0`
- unique emitted route keys: `301.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- effective resolved fraction: `0.9813186813186814`
- mean unresolved among nonpromoted routes: `0.014803465594151983`
- route decision instability: `0.0`

## Current Benchmark-Facing Result

`base_gap` is the balanced middle policy because:

- it avoids the address-like failure mode of `static_only`
- it avoids the overfallback-heavy behavior of `gap_only`
- it remains stable under repeated bounded harness use

So the current bounded evidence supports the following reading:

- `static_only` is too exact to be a practical reusable routing class
- `gap_only` is too coarse to be the preferred first policy
- `base_gap` is the best current tradeoff between reuse, selectivity, and
  fallback cost

The remaining risk is fallback burden:

- promotion still occurs on `45.27%` of bounded trace steps

That burden is moderately structured overall, but not concentrated enough to
justify one obvious cheap refinement before the first research-side integration
trial.

## Readiness Conclusion

The policy is now ready for a **research-side integration trial**.

The boundary that should be used is:

- the existing research-only benchmark wrapper around the bounded trace-building
  path

What must remain guarded:

- no MUDBench live seam changes
- no runtime router replacement
- no widening of the trace family in the first trial
- explicit fallback accounting on every run

## Next Action

The smallest next research-side integration trial should:

1. reuse the existing bounded trace family unchanged
2. run exactly the existing three-policy comparison:
   - `static_only`
   - `gap_only`
   - `base_gap + fallback`
3. log the established comparable metrics:
   - unique route keys
   - unique emitted route keys
   - route reuse fraction
   - promotion route fraction
   - promotion step fraction
   - effective resolved fraction
   - mean unresolved among nonpromoted routes
   - route decision instability
4. keep fallback burden explicit and observable

That is the smallest safe research-side integration trial justified by the
current exact-layer evidence.
