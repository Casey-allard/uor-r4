# Proof Step Update (2026-02-19): K1 Source Shape Probe (Finite-Range)

## Goal
Run a data-first probe aligned to the exact remaining source theorem shape:
\[
E(x)/x^\beta \approx a\cos(\tau\log x)+b\sin(\tau\log x)+\text{decaying tail}.
\]

## New tooling
Added script:
- `/Users/adminamn/Documents/New project/research/k1_source_shape_probe.py`

What it does:
- Uses cached zeta-zero data and cacheable signal construction.
- Builds truncated explicit-formula signal `E(x)/x^beta` on a log grid.
- Fits single-mode candidates over tau values from the zero list.
- Scores fit quality via tail-to-amplitude ratio and tail-envelope decay exponent.

## Runs
Primary run:
- `python3 research/k1_source_shape_probe.py --zeta-zeros-file research/data/zeta_zeros_odlyzko100k_2026-02-18.json --max-zeta-zeros 1024 --tau-candidate-count 64 --beta-grid 0.50,0.52,0.55,0.58,0.60 --x-min 1e4 --x-max 1e8 --grid-size 10000 --tail-frac 0.20 --output research/output/k1_source_shape_probe_2026-02-19.json`

Warm-cache repeat:
- `python3 research/k1_source_shape_probe.py ... --output research/output/k1_source_shape_probe_2026-02-19_warm.json`

Zero-count ablation (beta fixed 0.60):
- outputs:
  - `research/output/k1_source_shape_probe_2026-02-19_zero_count_scan.json`
  - `research/output/k1_source_shape_probe_2026-02-19_zero_count_scan.md`

## Key outcomes
- Best tau is consistently the first zeta-zero ordinate (`~14.134725`).
- But `tail_ratio_sup_to_amp` remains large (`~2.16` to `~3.06`, increasing with more zeros), so finite-range single-mode dominance is weak.
- Tail envelope still decays with x (positive decay exponent), but multi-mode contamination remains substantial.

## Interpretation for the proof frontier
- This empirically supports the current formal diagnosis: the missing source theorem is fundamentally about controlling the tail and mode interactions, not discovering a missing coding bridge.
- Open formal item count remains `1` (unchanged).

## Verification
- `python3 -m py_compile research/k1_source_shape_probe.py` ✅
