---
id: report-root-reused
title: A retry written into a sealed report root
date: 2026-09-07
severity: block
class: evidence
trigger: [report root, report_output, sealed attempt, retry, claim, derived inputs, verify]
paths: [crates/uor-r4-core/src/**, docs/integration/handoff-2026-09-07.md, docs/mechanisms/**]
milestones: [m1, m2]
mechanism:
guard: none: an exclusive claim cannot be checked from the record's text after the fact; the tool's refusal is the check
evidence: [docs/integration/handoff-2026-09-07.md, docs/mechanisms/README.md, AGENTS.md]
status: active
---

## What happened

The pitfall list the takeover record carried forward is explicit: create every report
directory exclusively (`uor_r4_core::report_output::claim`), immediately after argument
validation and **before loading any model**; seal completed attempts and verify the
complete file set (`report_output::verify`); write derived inputs into their own claimed
attempt, never beneath a sealed root; **never reuse a report root for a retry**. It is in
`AGENTS.md` under "Chain traversal" because the rule had been broken in practice, and a
reused root makes an old number and a new number indistinguishable.

## Symptom

A retry, a resume or a derived input is written into a directory that already holds a
finished attempt, or a sealed root is edited after the fact. The record then shows one
directory for two different runs, and neither number can be trusted.

## Lesson

Exclusivity is what makes a number an identity. A root reused for a retry silently
destroys the evidence for both attempts; a derived input written beneath a sealed root
corrupts an artifact that a comparison may already have consumed.

## How to apply

- Claim a fresh root before loading the model, not after argument parsing only.
- Seal each completed attempt and verify the complete file set before publishing any
  number from it.
- Give derived inputs their own claimed attempt; never write beneath a sealed root.
- A retry gets a new root, even when it repeats the same command; the record cites the
  root, so the root is the artifact identity.
