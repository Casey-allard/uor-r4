# Prime Transport Guarded Benchmark Hookup Plan

## Purpose

This note defines the smallest guarded benchmark-side hookup plan for the
current prime-transport routing policy, without touching the live MUDBench
router seam.

The policy carried forward is:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

This plan is strictly research-only.

## Guarded Benchmark-Side Boundary

The guarded benchmark-side boundary is:

- a research-only harness wrapper around the existing bounded trace-building
  path already packaged in
  `/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/base_gap_benchmark_wrapper.py`

That boundary already knows how to:

- construct bounded exact traces
- compute exact-layer state inputs
- expose route key, route mode, promotion, and fallback behavior
- summarize benchmark-style metrics

So the hookup plan does not require a new data path or a live seam change.

## Inputs

The guarded benchmark-side wrapper must receive:

- bounded trace rows from the existing `TRACE_SPECS` path
- per-position exact chart data:
  - base phase `b`
  - fiber coordinates `phi`
  - wheel depth `r`
- per-position residual field:
  - `next_return_gap`
- fallback-only predictive word:
  - `spin_H`
- policy threshold for promotion

## Outputs

For each evaluated policy/control, the wrapper must emit:

- route key
- emitted route mode
- promotion flag
- fallback usage
- aggregate benchmark-style summary rows

## Metrics That Must Be Recorded

The first guarded benchmark-side experiment should record, for each policy:

- unique route-key count
- unique emitted route-key count
- route reuse fraction
- promotion route fraction
- promotion step fraction
- effective resolved fraction
- route decision instability
- mean unresolved fraction among nonpromoted routes

These are the directly comparable bounded metrics already established in the
research notes and CSV summaries.

## Fallback Accounting Safeguards

Fallback accounting must remain explicit.

At minimum the wrapper should continue to record:

- promotion route fraction
- promotion step fraction
- effective resolved fraction after fallback
- unresolved fraction among nonpromoted routes

This is required because the current exact-layer conclusion is:

- `base_gap` is the right first guarded integration policy
- but fallback burden remains a real bounded cost
- and that burden is moderately structured rather than tightly concentrated

So fallback must be observable, not hidden inside a single score.

## Benchmark Comparison Setup

The first guarded benchmark-side comparison should remain exactly:

1. `static_only`
2. `gap_only`
3. `base_gap + fallback`

Reason:

- all three are already supported by the research-only benchmark harness
- they bracket the current design tradeoff cleanly
- and they produce directly comparable bounded metrics

## Smallest Future Hookup Point

The smallest future guarded benchmark-side experiment should hook:

- the existing research-only benchmark wrapper

to:

- a benchmark-facing runner that accepts the same bounded trace inputs and
  simply logs the emitted routing metrics for the three policies above

That is smaller and safer than any MUDBench seam change because it:

- stays outside production routing
- uses only the bounded exact trace family already validated
- preserves explicit fallback accounting

## Recommendation

The first future guarded benchmark-side experiment should:

- reuse the current bounded trace family unchanged
- keep `base_gap` unchanged
- compare only `static_only`, `gap_only`, and `base_gap + fallback`
- log fallback burden explicitly

No further offline policy refinement is required before that step.

The remaining unresolved risk is:

- whether the observed fallback burden remains acceptable once the policy is
  exercised in a slightly more benchmark-facing wrapper, even though it has
  already been stable in the current research-only harness
