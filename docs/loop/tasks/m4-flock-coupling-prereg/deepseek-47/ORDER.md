# ORDER: m4-flock-coupling-prereg, deepseek-47 (non-author FIT review + consolidated decision card)

Lab: deepseek. Item: [m4-flock-coupling-prereg](../../../../roadmap/items/m4-flock-coupling-prereg.md) (#2860). Base: `7080fd6cf`. `References #2860`; the item stays `ready` until the five PM/owner answers land and a later FIT clears the sheet.

## Goal

antigravity's design note, re-derived finite controls and machine-complete DRAFT sheet are already on main (`ea257ef40`), with five open registration questions on #2860. This unit does not decide those questions: it (a) records the **non-author `TEST FITNESS: NOT_FIT`** review that pins exactly what each answer must resolve, and (b) posts one consolidated decision card with a recommendation per question so the PM/owner can answer in one pass.

## Cancel probe

Class **instrument** (docs verdict; no GPU). At the base: the draft says `TEST FITNESS: PENDING` / not requested, the five questions on #2860 have no PM/owner answers (issue and #2028 searched), and the finite-control test passes (`cargo test -p uor-r4-discrete --lib flock` = 1 passed). Task stands.

## Files in scope

- `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` (status + verdict; canonical per PM ruling `#2028` `6362816839`)
- `docs/labs/m4-flock-coupling-prereg/**` (lab record)
- this task folder

## Done when

- The canonical sheet (`docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`, per the PM ruling on #2028 `6362816839`) carries the non-author `TEST FITNESS: NOT_FIT` and the rendered `prep fit` block; the lab note and REPORT record the five decisions in priority order and the verified finite controls. The superseded 10-18 draft is untouched.
- The design-note conclusions are verified against current main (finite control test passes; re-derived literals still match; no bar is invented and no PM/owner question is answered on their behalf).
- A consolidated decision card on #2860 (mirrored on #2028) gives the PM/owner one recommendation per question, with evidence and the note that it releases nothing.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only).

## Stop and ask if

- A numeric bar or hold would have to be set to make the sheet reviewable (it would not: NOT_FIT is the outcome).