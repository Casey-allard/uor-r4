# #67 — Emission smoothing calibration on held-out

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:16Z
- **Updated:** 2026-07-23T02:52:57Z
- **Closed:** 2026-07-23T02:52:57Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/67
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64. Current: add-one smoothing P(v|n)=(count+1)/(total+V) with the floor baked into the EMIT root header. Compare add-one vs Witten-Bell vs absolute-discounting (delta sweep 0.1-1.0) on held-out bits/token and top-1 agreement, deterministic double-run. Acceptance: the smoothing rule is chosen by measured bits/token on the D3 held-out set, recorded in score_report.json config, with the losing alternatives and their numbers in the issue for the record.


---

## Definition of Done
- Add-one, Witten-Bell, and absolute-discounting (delta sweep 0.1–1.0) smoothing rules are compared on held-out bits/token and top-1 agreement with deterministic double-runs.
- The chosen rule is recorded in `score_report.json` config and the losers' numbers are preserved here.

## Acceptance criteria
- [ ] Each rule has a measured bits/token and top-1 agreement on the D3 held-out partition.
- [ ] The winning rule shows a real (non-noise) margin or the choice is documented as a tie with rationale.
- [ ] Byte-identical outputs across two runs for the chosen rule (determinism evidence).

## Comments (1)

### comment 5053789833 — 2026-07-23T02:52:56Z — auser

Complete (139209c) — add-one wins and stays default. Full sweep table (fixture corpus, 30,036 held-out, Rule 1+2 deployed scorer; deterministic double-runs all byte-identical):

| variant | top-1 | bits/token |
|---|---|---|
| add-one (KEPT) | 31.71% | 9.8612 |
| witten-bell | 31.71% | 14.1782 |
| abs-disc:0.1 | 31.71% | 15.8307 |
| abs-disc:0.5 | 31.71% | 14.6800 |
| abs-disc:1.0 | 31.71% | 14.1963 |
| TLA3 baseline | 31.71% | 11.8781 |

DoD status: (1) all five variants measured on held-out bits/token + agreement (table above; losers preserved here for the record). (2) The win is decisive, not within-noise: 4.32 bits/token better than the best alternative at tied agreement. Mechanism: agreement is rule-invariant (all rules count-monotone, every position fires ExactContext status); bits diverge because WB/AD sharpen small-denominator EXCT distributions, charging ~15+ nats on the 68% out-of-candidate misses where add-one charges ~9.9 — heavy regularization matches the EXCT-precedence regime. (3) Default preservation proven: add-one artifact is byte-identical to HEAD (kappa de04eec8...), diff of score_report.json is exactly {schema 2->3, +config.smoothing}. New: Smoothing enum threaded through compile_emissions + emit_residual_exct via ScoreConfig/EmissionTables (one provable rule per artifact), --smoothing CLI flag, hand-computed formula tests for all three rules. Legacy probe-time quantize_exct deliberately untouched (the #68 parity reference).


---
_Comment/review ids in this file: 5053789833_

