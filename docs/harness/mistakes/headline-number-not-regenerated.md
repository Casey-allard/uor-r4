---
id: headline-number-not-regenerated
title: A PR declares the headline it copied, not the headline it just measured
date: 2026-10-12
severity: warn
class: delivery
trigger: [headline unchanged, declare the headline, headline before after, count n/3, session goal, status card]
paths: [.github/pull_request_template.md, docs/labs/**, docs/integration/current-state.md, docs/harness/milestones.json, scripts/harness/*.py]
milestones: [m1, m2, m3, m4]
mechanism:
guard: prlint.headline_vs_registry
evidence: [docs/labs/session-goal.md, docs/labs/lab-pivot-2026-10-09/README.md, docs/integration/current-state.md]
status: active
---

## What happened

After 9 October every PR has to declare `Line: <name> · count n/3 · headline before →
after` (session-goal §3). The declaration is prose in a PR body, so it was copied from
the previous PR or from memory. The registry and the bodies drifted: the same line was
declared 0/3 in a fresh PR while `docs/harness/milestones.json` already had it at 1/3, and
an `after` value could disagree with the registry's current headline without anyone
noticing. The [lab pivot](../labs/lab-pivot-2026-10-09/README.md) is what a session of
undetected drift looks like: 48 merged PRs, no number moved.

## Symptom

The `after` value equals the number from the last record rather than a measurement from
this PR, or the declared count is lower than the registry's. Nobody can tell whether the
line is at 0/3 or 3/3, so nobody knows whether the next action must be a pivot card.

## Lesson

The headline is a measurement, and the count is state. Read both from the live source at
the moment you write the body; do not carry them forward by hand. The declarations were
only unverifiable because nothing read the body - the numbers existed in two places and
were compared in neither.

## How to apply

- Generate the body instead of retyping it: `uor-harness lint format
  --body-file <last-body>` prints the compliant template with your fields.
- Check before opening: `uor-harness lint check --body-file <body>`.
  It warns `prlint.headline_vs_registry` when `after` is neither the registry's value nor
  a stated move, and `prlint.count_matches_registry` when the count is below the
  registry's for the named line.
- If the registry is wrong, fix `docs/harness/milestones.json` in the same PR and say so
  in the body; a moved headline is the only thing that resets a count (D21 §1).
