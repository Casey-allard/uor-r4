# Termination-weighted objective plan (September 27, 2026)

**Status boundary.** Bounded objective-change experiment on the accepted step-15,672
continuous parents. Development only: no candidate is promoted, the frozen prose
verdict and evaluator stand, and the parents are preserved. Read-only outside its own
campaign/evaluation roots. This is the mainline "ranking/emission" work card the
selection-policy result left open; it is an **objective** change chosen from causal
evidence, with a dose-matched control, not another exposure-only dose.

## Question and causal evidence

The [selection-policy diagnostic](selection-policy-diagnostic-result-2026-09-27.md)
measured: 13 of 20 failing greedy source rows are "correct noun followed by an extra
phrase" (the model does not terminate after a complete answer); 6 of 10 greedy free
continuations stop at the 128-token cap mid-clause and none of those passed; the only
two acceptable greedy stories are among the four EOS-terminated outputs. The frozen
sampled packet also ends by cap in every arm.

**Question.** With parents, data, optimizer, batch/context and full256 access fixed,
does upweighting sentence-final targets in the training loss at a bounded dose
(1,024 updates = 4,194,304 targets per fit) — versus a dose-matched plain
continuation — increase actual termination and improve the five-prompt frozen-criteria
acceptability, in both matched arms?

## Fixed conditions

| Quantity | Value |
|---|---|
| Parents | `fit-{quaternion,householder_pair}-6/checkpoint-final` (15,672) |
| Campaigns | copies of `{arm}-resume-6.json` with only declared differences: `total_steps=16696`, `max_process_seconds=7200`, `checkpoint_steps=[]`, new `stop_file`, `trial_scope`, plus `end_weight=2.5` for the treatment arms |
| Training | B16/T256, full256 causal access, `cpu_gradient_shards=2`, seed 240924, unchanged optimizer/data/evaluator |
| Objective change | `end_weight` multiplies the training-objective loss at targets in the sentence-final set (token ids decoding to `.` `!` `?`, plus EOS id 1); weighted mean with the weight sum as denominator. **Only** the training gradient is weighted: every reported metric (development NLL, retained-fit NLL, checkpoints) stays standard and comparable |
| Conditions | `{arm}-endw-16696` (treatment) and `{arm}-plain-16696` (dose-matched control) |
| Evaluation | `joint-evaluate` read and no-read per final; frozen `reference-evaluator-v2.json`; five-prompt packet read by the principal against the frozen four criteria; story-probe counts and stop reasons |
| Guardrails (per arm, vs parent) | comparison-tail NLL not worse than +0.05; NoRead penalty ≥0.02 retained; no increase in short-cycle stops; source-panel complete ≥ parent − 2 rows |

## Predeclared decision

| Observation | Decision |
|---|---|
| **EXTEND**: either arm gains ≥1 acceptable story versus both parent and dose-matched control (guardrails hold), or ≥4 of the 20 non-termination source rows resolve versus control | Termination emphasis is a working lever at this dose; a declared larger dose becomes the next work card |
| **INERT**: no meaningful movement (≤1 story, probe rows within ±2, boundary-stop change <10% relative) | Termination weighting is not the lever at this dose; stop this branch and record; the next rung is the state/read path (fourth-lab radial reader or a conditional depth hypothesis) |
| **HARM**: guardrails fail or prose worsens | Negative result preserved at exact scope; stop the branch |

Regardless of branch: no promotion; the frozen criteria, evaluator and thresholds are
unchanged; the parents remain the accepted artifacts.

## Projection, coordination and stop

- Code + focused tests ≤1 h wall. Four fits at `max_process_seconds=7200`, expected
  ≈55–70 min each (~1,000–1,200 targets/s), run **sequentially, one model process at a
  time**, for a ≤5 h envelope; evaluation ≤45 min; analysis/audit/docs ≤1 h; total
  envelope ≤8 h of machine wall against ≈25 h ledger headroom, charged once on
  delivery. New storage <300 MiB internal inside the existing model store; SSD build
  cache reused; 128 MiB stop margin untouched. Before each fit, check for other labs'
  live jobs and wait if one is training.
- **Cross-lab.** The fourth lab's active card owns `joint_campaign.rs` edits for its
  radial initializer; this plan's diffs are additive and absent-by-default
  (`end_weight` field, weighting call-site, tests), so either branch rebases over the
  other. Google's open PRs touch only `crates/uor-r4-integer`. The result will carry
  cross-lab implications (what a reader/state gain should move, and what my diagnostics
  imply for the radial comparison's evaluation).
- **Stop condition.** Finish with one supported decision, delivered through a protected
  PR referencing #973/#820, with the run handoff updated.
