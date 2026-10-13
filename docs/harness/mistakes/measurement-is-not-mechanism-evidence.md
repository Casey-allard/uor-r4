---
id: measurement-is-not-mechanism-evidence
title: A failed measurement cloaked as a mechanism negative; a good reading is not the outcome
date: 2026-10-09
severity: block
class: measurement
trigger: [mechanism inert, the term is inert, VSA term, mechanism read-out, reading not a result, diagnosis PR, attribution PR]
paths: [docs/labs/**, docs/mechanisms/**, docs/integration/current-state.md]
milestones: [m1, m4]
mechanism: exact-addressed-memory
guard: none: a human must judge whether a reading tested the mechanism; the panel and the record are the check
evidence: [docs/labs/vsa-review-2026-10-09/README.md, docs/labs/read-binding-margin-2026-10-11/README.md, docs/labs/pcopy-mass-read-2026-10-09/README.md]
status: active
---

## What happened

DeepSeek ran four VSA measurements on a frozen artifact. Two of them swapped a codebook
into an already-trained model *without retraining it* and switched the term off to see
what it contributed. That shows the term is inert in that frozen artifact; the review of
the same work found the sentence "the VSA term is inert" is accurate only with that
qualifier. Separately, the competing-mass margin halved the competing mass (its own
causal check passed) while the answers did not move: 30 -> 30 wrong-value failures over
120 rows, with per-seed signs flipping.

## Symptom

A record claims a mechanism is dead (or alive) from a proxy, a read-out, a diagnostic or
an unretrained transplant. The words are "inert", "no effect", "the mechanism fails",
and the experimental object was a reading rather than a model-changing run scored on the
frozen panel.

## Lesson

Two different errors hide here, and both cost cycles. A measurement is not mechanism
evidence: a proxy on a frozen artifact bounds a configuration, and D22 says a negative
closes a configuration, never a mechanism. And the reverse: passing your own causal check
does not mean the outcome moved - the margin's own check passed while the headline
failures stayed at 30/120, so it was REJECT on the pre-registered bar.

## How to apply

- Ask which object changed: the artifact, the training, or only the reading. Write that
  in the record's first paragraph.
- A mechanism negative needs the mechanism trained in (>= 2 seeds or >= 200 fresh rows),
  scored on the frozen acceptance panel, at the pre-registered bar.
- Report the mechanism reading and the outcome decision as separate lines, with the
  decision taken from the bar (`uor-harness panel score
  --panel <id> --value <v>`).
- Diagnosis, readings, plans and attribution PRs count toward the three (D21 §1): they
  are not free.
