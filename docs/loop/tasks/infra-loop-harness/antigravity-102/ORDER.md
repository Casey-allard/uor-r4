# ORDER: infra-loop-harness, antigravity-102

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-90, antigravity-98, antigravity-100, and antigravity-101). Base: `9dd8fc11a`.

## Goal

Narrow the overbroad `scope:` of [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`, M2, `needs_gpu: false`) by removing `crates/uor-r4-training/**` and `data/**` under a fresh-context D68 decision review (`DECISION.md` and `#2904` / `#2851` owner log, following the `claude-90` precedent for `m1-bvg-k-block-apply-c2`):

1. **`docs/roadmap/items/m2-pin-train-mask-sheet.md` (`#2851`, M2):**
   - `m2-pin-train-mask-sheet` is a docs-only pre-registration sheet item (`needs_gpu: false`; `## Done when` at `:54`: "The sheet itself runs nothing: no training or optimizer update is authorised by it"). It writes only the one-page M2 pre-registration sheet (`docs/harness/prep/m2-pin-train-mask-sheet*`), updates `docs/labs/m2-pin-train-mask-sheet/**` and `docs/harness/milestones.json`, adds a row to `docs/research/notebook/m2-grounded-reply.md`, and updates `docs/roadmap/items/m2-pin-train-mask-sheet.md`.
   - Keeping `crates/uor-r4-training/**` in `m2-pin-train-mask-sheet.md`'s `scope:` caused `uor-harness loop next` (`task-9980`) to refuse claiming `#2851` because `d11-exporter-float-trunk` (`#2820`, `claude-134` PR `#3289`) claims `crates/uor-r4-training/**`.
   - Narrowing `scope:` to `[docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-pin-train-mask-sheet.md]` eliminates the false overlap with `#3289` so `uor-harness loop next` immediately offers `GO item m2-pin-train-mask-sheet`.

## Cancel probe (run first)

1. Verify why `uor-harness loop next` (`task-9980`) refused `m2-pin-train-mask-sheet` after `#3296`: `m2-pin-train-mask-sheet: scope crates/uor-r4-training/** overlaps crates/uor-r4-training/** of item d11-exporter-float-trunk (PR #3289)`.
2. Verify zero file overlap between `docs/roadmap/items/m2-pin-train-mask-sheet.md` / `docs/loop/tasks/infra-loop-harness/antigravity-102/**` and open PRs `#3169`, `#3289`, and `#3295`.
3. **Cancel probe result (executed 2026-10-26, classed `instrument`):** Confirmed that `m2-pin-train-mask-sheet` modifies no files in `crates/uor-r4-training/**` or `data/**` and has zero overlap with `#3169`, `#3289`, and `#3295` once `scope:` is narrowed to its actual docs paths.

## Files in scope

- `docs/roadmap/items/m2-pin-train-mask-sheet.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-102/**`

## Done when

1. `docs/roadmap/items/m2-pin-train-mask-sheet.md` has `scope: [docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-pin-train-mask-sheet.md]`.
2. Fresh-context D68 decision review (`DECISION.md`) passes and is logged as `[antigravity] owner log:` on `#2904` and `#2851`.
3. `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never stop to wait on Casey; log the D68 decision review on `#2904` and `#2851`.
