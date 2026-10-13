# Pipeline: Real Task (WikiText-2 Proxy)

## Input
- prepared LM corpus cache (`tools/prepare_wikitext2.py` output)
  - source resolution order in `--dataset auto`: `wikitext2 -> ptb -> tinyshakespeare -> synthetic`

## Steps
1. Build routing-compatible tensors via `tasks/wikitext2_proxy.py`
2. Run dense baseline via `tasks/dense_baseline.py`
3. Run single route candidates via `tasks/router_proxy_eval.py`
4. Run multi-route transfer batches via `tools/proxy_sweep.py` when comparing route variants
5. Compare quality/runtime/memory/route-health across candidates

## Output
- proxy tensor dataset
- baseline metrics JSON
- route batch analysis JSON in `results/analysis/`
- report update in `docs/reports/REAL_TASK_COMPARISON.md`
