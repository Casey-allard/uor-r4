# ORDER: cpu-training-pilot, deepseek-172 (continuation release)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `fd6e6d4c8`.

## Goal

Complete bookkeeping release for the merged deepseek-171 continuation-sensitive
copy-lookahead experiment. The experiment remains a valid **REJECT**.

## What changed

1. Record reviewed head
   `0d1e8d7574d7623e83f1633419f6cab950068cb4` and protected merge
   `fd6e6d4c8c969b13410bca351ffbfa85a1821096` in the deepseek-171 report.
2. Record fresh-main CPU library and binary verification.
3. Remove `deepseek/copy-continuation` from the claims registry.
4. Leave model, serving, CUDA, float-training, termination and Claude-owned
   source unchanged.

## Done when

- deepseek-171 REPORT states the reviewed head, merge and fresh-main checks.
- The completed claim is absent from `docs/harness/claims.json`.
- Report, policy, link, catalog and gate checks pass at the reviewed head.
- The bookkeeping PR merges through the protected queue.
- Fresh main contains the bookkeeping and the claim remains absent.

## Stop rule

Stop on source, merge, report, claim or scope drift. Do not rerun or retune seed
2765 and do not change the experiment classification.
