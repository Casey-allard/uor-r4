# Proof Step Update (2026-02-19): Source-Term Literature Mapping

## Goal
Map currently available published theorem shapes to the exact final insertion classes in Lean.

## Fresh source scan artifact
- `research/data/latest_math_arxiv_k1_source_2026-02-19.json`
- `research/data/latest_math_arxiv_k1_source_2026-02-19.md`

## Candidate primary references
1. Schlage-Puchta (2019): "Oscillations of the error term in the prime number theorem"  
   - URL: `https://arxiv.org/abs/1912.00853`
   - Reported shape: interval-localized large oscillations caused by a given zero.
   - Lean target: `SchlagePuchta2019IntervalOscillationFormalized.theorem_term`.

2. Révész (2022): "Oscillation ... caused by a given zeta-zero" (Beurling setting)  
   - URL: `https://arxiv.org/abs/2202.01837`
   - Reported shape: explicit interval oscillation magnitude with constant near `π/2`.
   - Lean target: can feed interval/weak oscillation classes after specialization/translation.

3. Pintz (2017) theorem chain (existing source lock in repo)  
   - URL: `https://doi.org/10.1134/S0081543817010163`
   - Lean target: `Pintz2017ZeroToOscillationFormalized.theorem_term`.

## Current insertion ladder (now formalized)
- Interval-localized theorem term
  -> `SchlagePuchta2019IntervalOscillationFormalized`
  -> weak omega term via `weak_zero_oscillation_of_interval_oscillation`
  -> final normalized term via `pintz2017OfWeak`
  -> RH via existing pipeline.

## Net status
- This does not change remaining blocker count.
- It clarifies one concrete import target for the final missing theorem term, with exact class-level wiring already in place.
