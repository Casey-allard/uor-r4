# Pull Project Plan (PR-by-PR)

## PR-0: Import + reproducibility lock
- Add run scripts under `runs/`
- Ensure deterministic seeds
- Confirm quickstart

## PR-1: Instrumentation + parsing
- Add a final JSON summary line:
  - prefix: `__JSON_SUMMARY__ { ... }`
  - includes args + key metrics
- Add parsers in `tools/` to extract and tabulate results

## PR-2: Runtime collapse (10–30× faster)
- Add `--fast_dev 1` mode (smaller N, fewer iters/candidates)
- Early stopping for chart optimization
- Cache chart params and optionally routing outputs

Acceptance: typical `--fast_dev 1` run ≤ 2 minutes.

## PR-3: Automated sweeps
- `tools/sweep.py` runs staged sweeps and writes logs
- automatic parse + summarize

## PR-4: Hypothesis-targeted experiments only
- Add decision gates so we stop exploring dead branches.

