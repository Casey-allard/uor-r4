# ORDER: cpu-training-pilot, deepseek-180 (expanded candidate release)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `02d0a814b`.

## Goal

Complete bookkeeping release for merged PR #3728. The K12/seed-2769 selection
fault and K16/seed-2770 **REJECT** remain the scientific result.

## What changed

1. Record reviewed head
   `627c057279fa876faaf39153d76e11166298970a`, PR #3728 and protected merge
   `02d0a814bc18b0dc5fdbd8531db1002d71c0992f` in the deepseek-179 report.
2. Record fresh-main CPU, evidence, calibration, report and no-corpus checks.
3. Remove the completed `deepseek/expanded-candidate-anchor` and temporary
   release claims.
4. Change no model, serving, CUDA, float-inference, termination or
   Claude-owned source.

## Done when

- The deepseek-179 report names the reviewed head, protected merge, CI and
  fresh-main checks.
- Both completed claims are absent from `docs/harness/claims.json`.
- Report, policy, evidence, link, catalog and gate checks pass.
- The bookkeeping PR merges through the protected queue and fresh main is
  verified.

## Stop rule

Stop on source, merge, evidence, report, claim or scope drift. Do not rerun
seeds 2769 or 2770, change K12/K16 classifications or make a capability claim.
