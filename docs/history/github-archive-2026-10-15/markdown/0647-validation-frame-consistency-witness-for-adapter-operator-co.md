# #647 — validation: frame-consistency witness for adapter/operator comparisons (guard against reference-frame false nulls)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T22:06:05Z
- **Updated:** 2026-08-13T03:52:28Z
- **Closed:** 2026-08-13T03:52:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/647
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Research-available hardening (from historical-program mining). Not a mechanism replacement.**

## Basis (measured, historical program)
A reference-frame rotation (`apply_anchor_two_i`, a 90° class shift) produced `cos(rotated_state, raw_prototype) = exactly 0` for every class, every timestep, all 1024 eval samples — a categorical false null that *looks* like "no signal." Migrating prototypes into the same comparison frame restored cos = 1.0 for matched classes (matched/mismatched separation 1.327). The null was frame mismatch, not absence of signal.

## Proposal
For every registered comparison in our validation surfaces (#599 adapter conformance, #602 attention-operator specs, #605 route-fit overlaps, #606 certificate rows): before accepting any null/FAIL, assert a **matched-pair sanity score is materially nonzero**. A FAIL whose matched-pair control is ~0 must report **UNAVAILABLE (frame mismatch suspected)**, not FAIL — extending the #599 three-state discipline.

## Why it matters here
Our whole measurement regime treats FAIL as informative. A frame-mismatch false-zero would silently manufacture FAILs (or, worse, pass an anti-vacuity check that a null is "below threshold" when the whole instrument is mis-framed). This is a cheap, general guard.

## Non-goals
No mechanism change; no new operator. Pure validation hardening. Evaluate/adopt when convenient; no urgency.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

