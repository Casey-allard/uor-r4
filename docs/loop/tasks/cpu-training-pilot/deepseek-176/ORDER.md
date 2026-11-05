# ORDER: cpu-training-pilot, deepseek-176 (reference-prefix ranker release)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `1b78a0652`.

## Goal

Complete bookkeeping release for the merged deepseek-175 reference-prefix
copy-candidate ranker experiment. The experiment remains
**INSUFFICIENT_TRAINING_DATA**.

## What changed

1. Record reviewed head
   `b35b19c802430bfeeb9cba28908a4048f3174704`, PR #3723 and protected merge
   `1b78a0652fa419c0200788e0ba82b15f480c9229` in the deepseek-175 report.
2. Record fresh-main CPU library and binary verification.
3. Remove `deepseek/copy-ranker-reference` from the claims registry.
4. Change no model, serving, CUDA, float-inference, termination or
   Claude-owned source.

## Done when

- The deepseek-175 REPORT states the reviewed head, merge and fresh-main checks.
- The completed claim is absent from `docs/harness/claims.json`.
- Report, policy, link, catalog and gate checks pass at the reviewed head.
- The bookkeeping PR merges through the protected queue and fresh main is
  verified.

## Stop rule

Stop on source, merge, report, claim or scope drift. Do not rerun seed 2767,
lower the 640-group floor or change the insufficiency classification.
