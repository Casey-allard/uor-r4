# Pipeline: Decision Gates

## Input
- summary metrics from latest sweep batch

## Logic
- Evaluate thresholds from `docs/governance/DECISION_GATES.md`
- For transfer sweeps, treat route health as part of promotion logic:
  - `pmax_after`
  - `eval_shells`
  - `shell_pmax`
  - `test_unseen_rate`
- If escalation needed, create gate note and pause branch promotion

## Output
- `docs/governance/gates/gate_<timestamp>.md`
