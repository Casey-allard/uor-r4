# INC-0007: LM Proxy Transfer Smoke

## Hypothesis
The current best synthetic route `R5B` should retain at least a small advantage over `R0` on language-like proxy data.

## Configuration
- Dataset: PTB proxy via `tools/prepare_wikitext2.py --dataset auto`
- Evaluator: `tasks/router_proxy_eval.py`
- Subset: `train=2000`, `test=1000`
- Seed: `0`
- Mode: `fast_dev=1`

## Evidence
- `results/parsed/proxy_cmp_r0.json`
- `results/parsed/proxy_cmp_r5b.json`

## Results
- `R0`
  - `test_mse_after=0.0039455`
  - `total_sec=128.599`
  - `buckets=8`
  - `pmax_after=0.223`
- `R5B`
  - `test_mse_after=0.0039146`
  - `total_sec=127.130`
  - `buckets=2`
  - `pmax_after=0.875`

## Decision
- `R5B` wins slightly on the first transfer smoke.
- The route health is not yet acceptable because bucket concentration is extreme.
- Treat this as promising but inconclusive transfer evidence.

## Next Increment
- `INC-0008`: multi-seed larger-subset PTB transfer with bucket health explicitly monitored.
