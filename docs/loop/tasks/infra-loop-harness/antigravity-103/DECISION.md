# D68 Decision Card: Remove Explicit `docs/roadmap/items/*.md` Entries from `m2-pin-train-mask-sheet` and `m1-close-saved-model` Scopes (`antigravity-103`, #3299)

- **Lab / Session:** `antigravity` (`antigravity-103`, PR `#3299`)
- **Umbrella item:** [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (`#2904`, Setup row 2)
- **Target items:** [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`, M2), [`docs/roadmap/items/m1-close-saved-model.md`](../../../../roadmap/items/m1-close-saved-model.md) (`#2837`, M1)
- **Authority:** [`CHARTER.md §5.1`](../../../../../CHARTER.md), D68 (`docs/integration/DECISIONS.md`)

## 1. Whole-project context (`CHARTER.md §5.1`)

1. **Why explicit `docs/roadmap/items/*.md` paths in `scope:` cause a false lock against `infra-loop-harness` (`PR #3169`):**
   - `crates/uor-r4-harness/src/loopprcheck.rs:430` (`let item_path = format!("docs/roadmap/items/{id}.md");`) always implicitly permits a claimed item to update its own `docs/roadmap/items/<id>.md` file.
   - However, when `docs/roadmap/items/<id>.md` is listed explicitly in an item's `scope:`, `uor-harness loop next` and `uor-harness loop claim` match that path against `docs/roadmap/items/*.md` in `infra-loop-harness.md:10` (claimed by idle draft `PR #3169` `codex-1`), causing `uor-harness loop next` (`task-10031`) to refuse `m2-pin-train-mask-sheet` (`#2851`) with:
     `m2-pin-train-mask-sheet: scope docs/roadmap/items/m2-pin-train-mask-sheet.md overlaps docs/roadmap/items/*.md of item infra-loop-harness (PR #3169)`.
   - The same false overlap would hit `m1-close-saved-model` (`#2837`) once `r-ranking-with-admission-intact` (`PR #3295`) releases `crates/uor-r4-discrete/**`.

## 2. Exact changes approved under D68

1. **`docs/roadmap/items/m2-pin-train-mask-sheet.md` (`#2851`):**
   - Remove `docs/roadmap/items/m2-pin-train-mask-sheet.md` from `scope:`, leaving `[docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md]`.
2. **`docs/roadmap/items/m1-close-saved-model.md` (`#2837`):**
   - Remove `docs/roadmap/items/m1-close-saved-model.md` and `docs/roadmap/items/m1-fresh-panel-transfer.md` from `scope:`, leaving `[crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md]`.
3. **No bar, status, or technical constraint changed.**
