# Project Context: 4D Polar / Hyperbolic Routing

## The mechanism
We route points using:

1) Map points from Poincaré ball to tangent at origin:
   - `v = log_0(x)`
2) Apply chart (optional):
   - `z = chart(v)`
     - rotation `R ∈ SO(d)` (optional)
     - scaling:
       - global diagonal `diag(exp(s))`, or
       - radial-bin diagonal `diag(exp(S[bin(r)]))`
3) Route:
   - `shell = floor(||z|| / delta_r)`
   - `sector = argmax( normalize(z) · C_k )` (k-means on sphere), OR a phase/polar sectoring mode

Then local memory and growth operate within each `(shell, sector)` bucket.

## "What we learned so far" (from pasted runs)
- Radial scaling + growth budget helps.
- `sector_mode=kmeans` is generally stronger than `sector_mode=phase2` on post-growth test MSE (seed variance exists).
- `time_pressure_lambda` in [0.25, 1.2] tends to hurt pre-growth MSE and doesn't reliably improve post-growth.

## Why we’re moving to Codex
Runs are too slow for manual exploration. We need:
- machine-readable run summaries
- caching + fast dev mode
- automated staged sweeps
- a decisions log

