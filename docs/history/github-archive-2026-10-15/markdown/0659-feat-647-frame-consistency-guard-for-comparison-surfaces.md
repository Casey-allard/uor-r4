# #659 — feat(#647): frame-consistency guard for comparison surfaces

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T03:32:45Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-13T03:52:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/659
- **Merged:** 2026-08-13T03:52:27Z
- **Merge commit:** 9300be281e15cb5c75e4e422c6ad1d77de04245c
- **Base / Head:** main / UOR-Foundation:i647-frame-witness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #647.

## Summary
A sanctioned, tested frame-consistency guard (`uor-r4-graph-certify::frame_consistency`): before a comparison surface accepts a null/FAIL as informative, it confirms a matched-pair control is materially nonzero; a ~0 (or non-finite) control means the instrument is likely mis-framed, so the null/FAIL is UNAVAILABLE (frame mismatch suspected), never a real FAIL — extending the #599 three-state discipline.

**Measured basis**: a reference-frame rotation drove `cos()` to exactly 0 across every class/timestep (a categorical false null); same-frame restored `cos=1.0`. #486 was a live instance.

## Respects pre-registration (the key design choice)
route-fit (#605) already embodies this via its **pre-registered** N2 shifted-support anti-vacuity gate. Other surfaces (#599/#602/#606) adopt this primitive by **declaring a frame-control in their contracts** — never by retroactively bolting a new verdict branch onto an already-pre-registered instrument, which would break its declared identity. So this lands the sanctioned mechanism + discipline rather than force-wiring a verdict change.

## Verification
`FRAME_CONTROL_EPSILON` is a false-zero detector, not a strength threshold (a weak-but-real signal stays informative). 4 unit tests (framed / categorical-zero / exclusive-epsilon-floor / non-finite+negative). fmt clean, clippy --all-features -D warnings clean, R4 clean.

## Breaking changes
None. Additive pure module.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

