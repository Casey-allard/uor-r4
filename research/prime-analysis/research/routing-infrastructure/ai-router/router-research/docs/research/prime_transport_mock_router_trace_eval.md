# Prime Transport Mock Router Trace Evaluation

## Purpose

This note records the first trace-real offline exercise of the mock router
module on actual bounded exact rows, rather than on summary-row placeholders.

The trace sources were:

- `visible_threshold_tight_density_matched_rows.csv`
- `visible_threshold_second_tight_density_matched_rows.csv`

For each row, the driver reconstructed real per-position transitions for:

- `R_static = (b, phi, r)`
- `R_min = (b, phi, r, next_return_gap)`
- `R_full = (b, spin_H)`

## Result

The update path is clean on the bounded traces:

- `R_static` transition match fraction: `1.0`
- `R_min` transition match fraction: `1.0`
- `R_full` transition match fraction: `1.0`

So the mock module now tracks the exact transport transitions correctly on the
selected per-position traces.

## Promotion Reading

The important trace-level outcome is:

- `R_min` promotion fraction: `0.0`
- `R_min` sufficient-without-promotion fraction: `1.0`

This is **not** evidence that `R_min` has solved the predictive problem.

It is a trace-design pathology:

- the current literal route keys are too fine-grained
- `R_min` route keys are effectively pointwise on these bounded traces
- so unresolved ambiguity disappears before the promotion test has any work to
  do

This is reflected in the trace summary:

- for `2310 -> 30030`, `R_min` unique route keys = `385`
- for `30030 -> 510510`, `R_min` unique route keys = `5005`

That is, the current trace-level route representation is still behaving like an
addressing scheme rather than a reusable routing partition.

## Interpretation

The trace-real exercise therefore confirms two things at once:

1. the mock router state updates are now correctly implemented
2. the current literal routing key is too exact to expose the delayed-refinement
   mechanism in trace operation

So the key mismatch is no longer a transport-update bug. It is a design-level
issue in how routing classes are grouped.

## Conservative Conclusion

The present trace-level result is:

- transitions behave cleanly on real exact traces
- but the current mock routing key is too fine-grained to yield meaningful
  promotion behavior

So the next offline design step is not a runtime integration step.

It is:

- define a bounded exact-layer grouping key for `R_min` that is coarser than
  literal state identity, while still grounded in the same exact objects

That is the minimal next step needed before the trace-level prototype can be
used as a meaningful delayed-refinement router.
