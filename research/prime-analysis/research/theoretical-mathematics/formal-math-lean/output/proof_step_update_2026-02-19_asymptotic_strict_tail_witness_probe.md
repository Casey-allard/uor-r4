# Proof Step Update (2026-02-19): Asymptotic Strict-Tail Epsilon-Witness Probe

## Objective
Evaluate the new asymptotic strict-tail theorem shape numerically in the exact form
`forall epsilon>0, eventually |R(x)/x^beta| <= epsilon * amplitude` (finite-range surrogate).

## Added tool
New script:
- `research/asymptotic_strict_tail_witness_probe.py`

It reuses cached `E(x)/x^beta` signals and outputs epsilon-witness tables (`x_epsilon`) from tail-sup envelopes.

## Runs executed
- `x_max=1e20, grid=6144`
- `x_max=1e20, grid=8192`
- `x_max=1e22, grid=8192`
- `x_max=1e24, grid=4096`

Artifacts:
- `research/output/asymptotic_strict_tail_witness_probe_2026-02-19_beta074_tau14_x1e7_1e20_g6144.json`
- `research/output/asymptotic_strict_tail_witness_probe_2026-02-19_beta074_tau14_x1e7_1e20_g8192.json`
- `research/output/asymptotic_strict_tail_witness_probe_2026-02-19_beta074_tau14_x1e7_1e22_g8192.json`
- `research/output/asymptotic_strict_tail_witness_probe_2026-02-19_beta074_tau14_x1e7_1e24_g4096.json`
- `research/output/asymptotic_strict_tail_witness_trend_2026-02-19.json`
- `research/output/asymptotic_strict_tail_witness_trend_2026-02-19.md`

## Key observations (finite-range)
- For all tested windows, witness points exist for `epsilon = 1.0` and `epsilon = 0.99` at sufficiently large `x` in the sampled tail.
- Witness thresholds move deeper into the tail as `x_max` increases (expected near-threshold behavior).
- This aligns with the asymptotic strict-tail direction but remains numerical evidence only.

## Status impact
- Remaining formal kernel unchanged: `K1-SOURCE`.
- Practical target sharpened: derive a non-circular theorem term proving epsilon-eventual tail control, not just finite-grid witness tables.
