# ORDER: cpu-training-pilot, deepseek-84

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `de9cfc84a`.

## Goal

Correct the remaining stale seed-2 margin reading found by the exact-head review
of deepseek-83.

## Finding

The deepseek-82 REPORT still named the superseded trainer-seed-2 K4 margin sum
`29,602,710`. The bound trainer-seed-1 receipt records `29,471,323`. The
deepseek-83 REPORT, comparison and receipt already contain the correct value.

## Repair

- Correct the stale margin reading in the deepseek-82 REPORT.
- Recheck all remaining evidence and prose against the bound receipts.

## Scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-82/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-84/**`

## Done when

1. No superseded seed-2 reading remains in the REPORT or evidence summary.
2. Evidence JSON and checksums still pass.
3. A fresh exact-head review approves the corrected PR.

## Stop if

Any other receipt, checksum or reported value disagrees with the bound
evidence; report the discrepancy rather than editing around it.
