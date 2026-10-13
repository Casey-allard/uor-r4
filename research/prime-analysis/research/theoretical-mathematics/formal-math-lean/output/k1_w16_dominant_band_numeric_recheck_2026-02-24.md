# K1-W16 Dominant-Band Numeric Recheck (2026-02-24)

## Command
`python3 research/k1_multimode_phase_probe.py --zeta-zeros-file research/data/zeta_zeros_odlyzko100k_2026-02-18.json --max-zeta-zeros 20000 --tau-candidate-count 64 --beta 0.6 --x-min 1e7 --x-max 1e20 --grid-size 4096 --zero-chunk 512 --max-modes 6 --tail-frac 0.20 --include-decay-term --cache-dir research/cache/k1_source_shape_probe --output research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m6_recheck.json`

## Runtime
- cache status: warm
- completed in ~1.5s

## Result
- best mode count: `6`
- best score: `2.696621`
- best tail ratio `tail_sup/amp_total`: `1.451030`
- interpretation: `weak_multimode_dominance_finite_range`

Reference output files:
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m6_recheck.json`
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m6_recheck.md`

## Comparison against prior m8 run
- prior file: `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m8.json`
- prior best tail ratio: `1.425831`
- both m6 and m8 are in `weak_multimode_dominance_finite_range`

## Implication for final gate
Current numeric evidence does **not** indicate a clean single dominant-band collapse over this tested finite range; it still behaves as a multi-band mixture. This does not prove impossibility, but it means the remaining theorem target likely needs additional analytic assumptions (separation/regularity/cancellation structure) beyond current fitted evidence.
