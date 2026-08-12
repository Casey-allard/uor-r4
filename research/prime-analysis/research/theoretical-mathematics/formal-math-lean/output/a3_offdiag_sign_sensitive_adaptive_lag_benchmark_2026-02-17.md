# A3 Sign-Sensitive Adaptive-Lag Benchmark (2026-02-17)

## Setup

- Grid: bases `30,210,2310,30030`
- n-grid: train `3e5,1e6,2e6`; valid `5e6,1e7`
- `x_step=2500`, `A_eta=4.0`
- Warm cache (`cache_hits=4`, `cache_writes=0`)

## Results

### Dense baseline
- Artifact: `research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_dense32_warm.json`
- lag schedule: `1..32` (`lag_count=32`)
- total: `6.7654s`
- lagprep: `6.1842s`
- `C_ss_uplifted=8.36279288664`

### Adaptive candidate A
- Artifact: `research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_adapt32_warm.json`
- lag schedule: `1..8,12,17,24,32` (`lag_count=12`)
- total: `3.4422s` (1.97x faster)
- lagprep: `2.8485s`
- `C_ss_uplifted=8.49575826684` (+1.59% vs dense)

### Adaptive candidate B (selected default)
- Artifact: `research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_adapt32_d12_warm.json`
- lag schedule: `1..12,17,24,32` (`lag_count=15`)
- total: `3.9488s` (1.71x faster)
- lagprep: `3.2853s`
- `C_ss_uplifted=8.48963425154` (+1.52% vs dense)

## Decision

Set default probe lag mode to adaptive with:
- `lag_mode=adaptive`
- `lag_dense_cut=12`
- `lag_tail_mult=1.4`

Rationale: keeps near-dense quality while materially reducing warm runtime.

