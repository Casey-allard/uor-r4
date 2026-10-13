# Proof Step Update (2026-02-19): Phase-Oscillatory Candidate Refresh

## Purpose
Re-anchor work on the single remaining phase-oscillatory term by producing a direct finite-range witness candidate in the exact `cos/sin + majorant` shape.

## What was added
- Upgraded numerical probe:
  - `research/k1_source_shape_probe.py`
  - now exports explicit remainder-majorant fields:
    - `remainder_majorant_eta`
    - `remainder_majorant_C_all`
    - `remainder_majorant_C_tail`
    - bound-ratio diagnostics on full/tail ranges.
- New outputs:
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant.json`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant.md`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_zero_scan.json`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_zero_scan.md`
  - `research/output/k1_phase_majorant_candidate_2026-02-19.json`
  - `research/output/k1_phase_majorant_candidate_2026-02-19.md`

## Main numerical findings
- Best `tau` remains locked to the first zeta zero: `tau ≈ 14.134725142`.
- Best `beta` over tested grid remains `0.60`.
- Across zero-depth scan (`128..4096` zeros), `tau` and `beta` are stable.
- Fitted majorant exponent is consistently positive (`eta ~ 0.12..0.13`) in finite range.
- Single-mode dominance remains weak (`tail_sup/amp > 2`), so this is not yet a theorem witness.

## Status impact
- The project is still at one remaining open kernel (`K1-SOURCE`).
- This step strengthens the candidate shape for that kernel and narrows what the missing theorem term must certify.
