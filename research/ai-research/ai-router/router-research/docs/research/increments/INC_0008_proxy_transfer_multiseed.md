# INC-0008: Multi-Seed PTB Proxy Transfer for R0 vs R5B

## Hypothesis
`R5B` retains its quality/runtime lead over `R0` on language-like proxy data under a larger subset and more than one seed.

## Configuration
- Config: `configs/proxy_transfer_inc0008.json`
- Runner: `tools/proxy_sweep.py`
- Dataset: PTB proxy via `tools/prepare_wikitext2.py --dataset auto`
- Subset: `train=3000`, `test=1500`
- Seeds: `0,1`
- Route set:
  - `R0`: `sector_mode=kmeans`
  - `R5B`: `sector_mode=phase4d`, `phase4_dims=0,2,4,6`

## Evidence
- Batch analysis: `results/analysis/inc0008_proxy_transfer_multiseed.json`
- Gate note: `docs/governance/gates/gate_20260305_141648.md`
- Logs:
  - `results/raw/inc0008_proxy_transfer_multiseed_R0_seed*.log`
  - `results/raw/inc0008_proxy_transfer_multiseed_R5B_seed*.log`

## Results
- `R0` mean
  - `test_mse_after=0.0039450`
  - `total_sec=26.112`
  - `buckets=8.0`
  - `pmax_after=0.205`
- `R5B` mean
  - `test_mse_after=0.0038858`
  - `total_sec=23.237`
  - `buckets=2.0`
  - `pmax_after=0.877`

## Decision
- `R5B` remains ahead of `R0` on proxy quality and runtime under a larger subset and multiple seeds.
- The transfer concern hardened rather than softened: the win still comes with severe route concentration.
- Do not promote `R5B` as a transfer-ready route yet. Treat it as a quality/runtime lead that still fails the route-health bar.

## Next Increment
- `INC-0009`: transfer stabilization for `R5B` by widening bucket usage without giving back the current quality/runtime edge.
