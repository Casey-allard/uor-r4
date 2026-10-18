# One-page plan linked from the front door 2026-10-18

D39 Phase 1 says the one-page `docs/integration/NOW.md` plan must be linked from README/STATUS/ROADMAP.
Before this change it was linked nowhere outside itself, so a reader landing on the repository front door
could not find the authoritative current plan. This is a documentation-only correction from the
`docs/labs/course-correction-2026-10-18/` tracker-hygiene queue; no code, model or evidence change.

## Change

- `README.md`: the "Start with ..." line now begins with the one-page current plan.
- `STATUS.md`: the evidence header names NOW.md as the authoritative plan when the two disagree.
- `ROADMAP.md`: the title block names NOW.md as the current plan and keeps this page as long-form history.

## Verification

- `test -f docs/integration/NOW.md` and all three links point at `docs/integration/NOW.md`.
- No Rust/model path touched; no compute, $0.