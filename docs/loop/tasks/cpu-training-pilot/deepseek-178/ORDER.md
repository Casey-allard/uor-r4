# ORDER: cpu-training-pilot, deepseek-178 (geometric candidate-union release)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `4ea097661`.

## Goal

Complete bookkeeping release for merged PR #3725, the geometric candidate-union
insufficiency experiment. The scientific result remains
**INSUFFICIENT_TRAINING_DATA**.

## What changed

1. Record reviewed head
   `ad2fdfa0ee51ed7f5a5fdf1d54b73cfd69895cc0`, PR #3725 and protected merge
   `4ea0976615b4b59e4394c30476931f15291b0951` in the deepseek-177 report.
2. Record fresh-main CPU library, binary, Clippy, evidence and no-corpus
   verification.
3. Remove the completed `deepseek/geometric-candidate-union` claim and the
   temporary release claim.
4. Change no model, serving, CUDA, float-inference, termination or
   Claude-owned source.

## Done when

- Deepseek-177 REPORT names the approved head, protected merge, CI and
  fresh-main checks.
- The completed experiment and temporary release claims are absent from
  `docs/harness/claims.json`.
- Report, scope, evidence, policy, link, catalog and gate checks pass at the
  reviewed head.
- The bookkeeping PR merges through the protected queue and fresh main is
  verified.

## Stop rule

Stop on source, merge, evidence, report, claim or scope drift. Do not rerun
seed 2768, lower the 2,048-group floor or change the insufficiency
classification.
