# ORDER: infra-loop-harness, antigravity-103

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-90, antigravity-98, antigravity-100, antigravity-101, and antigravity-102). Base: `8ac2ad9e3`.

## Goal

Remove explicit `docs/roadmap/items/*.md` paths from the `scope:` lists of [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`) and [`docs/roadmap/items/m1-close-saved-model.md`](../../../../roadmap/items/m1-close-saved-model.md) (`#2837`) so neither item collides with `infra-loop-harness`'s `docs/roadmap/items/*.md` glob (`PR #3169` `codex-1`) when `uor-harness loop next` and `uor-harness loop claim` check scope overlap:

1. **`docs/roadmap/items/m2-pin-train-mask-sheet.md` (`#2851`):**
   - Set `scope: [docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md]` (an item's own `docs/roadmap/items/<id>.md` is always implicitly allowed by `crates/uor-r4-harness/src/loopprcheck.rs:430`).
2. **`docs/roadmap/items/m1-close-saved-model.md` (`#2837`):**
   - Set `scope: [crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md]`.

## Cancel probe (run first)

1. Verify why `uor-harness loop next` (`task-10031`) refused `m2-pin-train-mask-sheet`: `m2-pin-train-mask-sheet: scope docs/roadmap/items/m2-pin-train-mask-sheet.md overlaps docs/roadmap/items/*.md of item infra-loop-harness (PR #3169)`.
2. Verify `crates/uor-r4-harness/src/loopprcheck.rs:430` (`let item_path = format!("docs/roadmap/items/{id}.md");`), which always allows an item's own `docs/roadmap/items/<id>.md` without listing it in `scope:`.
3. **Cancel probe result (executed 2026-10-26, classed `instrument`):** Confirmed that explicit `docs/roadmap/items/*.md` entries in `scope:` cause `loopclaim` / `loopnext` to overlap `infra-loop-harness` (`PR #3169`), and that removing them leaves zero overlap for `m2-pin-train-mask-sheet` with all open PRs (`#3169`, `#3295`, `#3298`).

## Files in scope

- `docs/roadmap/items/m2-pin-train-mask-sheet.md`
- `docs/roadmap/items/m1-close-saved-model.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-103/**`

## Done when

1. `docs/roadmap/items/m2-pin-train-mask-sheet.md` and `docs/roadmap/items/m1-close-saved-model.md` have no explicit `docs/roadmap/items/*.md` entries in `scope:`.
2. Fresh-context D68 decision review (`DECISION.md`) passes and is logged as `[antigravity] owner log:` on `#2904`, `#2851`, and `#2837`.
3. `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never stop to wait on Casey; log the D68 decision review on `#2904`, `#2851`, and `#2837`.
