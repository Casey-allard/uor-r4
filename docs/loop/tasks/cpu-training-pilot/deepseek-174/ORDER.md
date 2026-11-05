# ORDER: cpu-training-pilot, deepseek-174 (pairwise ranker release)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `cc1fb4d6b`.

## Goal

Complete bookkeeping release for the merged deepseek-173 pairwise
copy-candidate ranker experiment. The experiment remains
**INSUFFICIENT_TRAINING_DATA**.

## What changed

1. Record reviewed head
   `528b169cd948cf821417e940ef9d9aaa83f5cbae` and protected merge
   `cc1fb4d6b36dbfbe50a56a17311efc8906d90fd1` in the deepseek-173 report.
2. Record fresh-main CPU library and binary verification.
3. Remove `deepseek/copy-ranker` from the claims registry.
4. Change no model, serving, CUDA, float-inference, termination or
   Claude-owned source.

## Done when

- deepseek-173 REPORT states the reviewed head, merge and fresh-main checks.
- The completed claim is absent from `docs/harness/claims.json`.
- Report, policy, link, catalog and gate checks pass at the reviewed head.
- The bookkeeping PR merges through the protected queue and fresh main is
  verified.

## Stop rule

Stop on source, merge, report, claim or scope drift. Do not rerun seed 2766,
lower the 640-group floor or change the insufficiency classification.
