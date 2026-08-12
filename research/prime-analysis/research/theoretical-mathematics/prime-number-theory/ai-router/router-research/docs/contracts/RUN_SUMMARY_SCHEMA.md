# Run Summary Schema v1.0

Every completed run must emit exactly one final line:

`__JSON_SUMMARY__ { ... }`

## Top-level fields
- `schema_version` (string): currently `"1.0"`.
- `parsed` (bool): producer-side validity marker.
- `args` (object): normalized CLI arguments for the run.
- `metrics` (object): key quantitative outcomes.
- `timings_sec` (object): timed phase durations.
- `artifacts` (object): paths/keys for cache and outputs.
- `git` (object): best-effort branch/commit context.
- `notes` (array[string]): warnings, assumptions, or fallbacks.

## Required `metrics` keys
- `test_mse_before`
- `test_mse_after`
- `train_label_sse_per`
- `test_label_sse_per`
- `buckets`
- `slots_used`
- `test_unseen_rate`

## Required `timings_sec` keys
- `dataset`
- `chart_opt`
- `routing_eval`
- `training_ema`
- `growth`
- `total`

## Validation behavior
- Missing required keys => parser sets `parsed=false` and writes `parse_error`.
