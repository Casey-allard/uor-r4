---
id: merged-prs-without-a-headline-move
title: 48 merged PRs and no milestone moved
date: 2026-10-09
severity: block
class: process
trigger: [48 merged PRs, no milestone moved, rabbit hole, count 3/3, pivot card, narrower next step, line counts]
paths: [docs/integration/DECISIONS.md, docs/labs/**, docs/integration/current-state.md, .github/pull_request_template.md, scripts/harness/*.py]
milestones: [m1, m2, m4]
mechanism: none
guard: prlint.pivot_at_three
evidence: [docs/labs/lab-pivot-2026-10-09/README.md, docs/labs/no-cpu-gpu-only-2026-10-12/README.md, docs/integration/current-state.md]
status: active
---

## What happened

On 9 October the owner asked whether the DeepSeek and Codex labs had gone down rabbit
holes. The review found 48 merged PRs that day with no milestone moved: DeepSeek's M1
line had 33 PRs and the headline unchanged, Codex's M2 line had 15. Each PR was honest
and each line kept producing a narrower next step whatever the result. DeepSeek found a
real mechanism defect (the pointer attends the sentence frame, never the value's slot)
and then refined the reading instead of training the fix; Codex moved from reply quality
to solver numerics. The review produced D21 and the session-goal contract.

## Symptom

A session is merging steadily - diagnoses, plans, probes, readings, instruments - and the
milestone's headline acceptance number is exactly what it was at session start. Each PR
has a next step; none of them changes the model. The line count is the only thing moving.

## Lesson

Delivery volume is not progress. Count merged PRs on a line that do not move the
headline; at three, the next action is a pivot card on the milestone issue: stop and
archive the line, or one decisive model-changing run scored on the acceptance panel with
a pre-registered bar. Diagnosis, readings, plans and attribution PRs count toward the
three and never reset it; only a change in the headline number does. The count had to be
maintained by hand in PR prose, which is why it drifted - it is now checked.

## How to apply

- Every PR declares `Line: <name> · count n/3 · headline <before> → <after>`
  ([session goal](../../labs/session-goal.md) §3).
- At 3/3, announce the pivot in the body before opening another same-line PR:
  `python3 scripts/harness/cli.py lint check --body-file <body>` fails
  `prlint.pivot_at_three` when a body is at or over 3/3 with no pivot announced.
- Read the current counts, not your memory of them:
  `python3 scripts/harness/cli.py status` and `docs/harness/milestones.json`.
- Before opening, ask the contract's question: if this came out the other way, would the
  next step differ? If not, the PR cannot change a decision.
