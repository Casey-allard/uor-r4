# ORDER: cpu-training-pilot, deepseek-85

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `de9cfc84a`.

## Goal

Correct the final disclosure findings from exact-head review of deepseek-84.

## Finding

- The PR description still named the superseded seed-2 K4 count of 225.
- The deepseek-82 REPORT said no configuration change was made, although
  review required a seed-2 rerun to honor the registered trainer seed.

The corrected evidence itself, checksums and bounded negative are correct.

## Repair

- State the corrected 238-block seed-2 result and seed-1 K4's expected exit 2
  in the PR description.
- Distinguish the protocol-compliance rerun from outcome-driven configuration
  tuning in the REPORT.

## Scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-82/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-85/**`
- PR #3588 description

## Done when

1. The REPORT does not claim that no configuration change occurred.
2. The PR description matches the bound 437/238 evidence and expected exits.
3. A fresh exact-head review approves the PR.

## Stop if

The change would alter a result, bar or evidence receipt; this is a disclosure
repair only.
