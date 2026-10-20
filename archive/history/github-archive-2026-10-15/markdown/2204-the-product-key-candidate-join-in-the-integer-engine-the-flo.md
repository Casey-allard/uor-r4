# #2204 — The product-key candidate join in the integer engine: the float path's order, without multiplies or allocation (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T17:21:04Z
- **Updated:** 2026-10-11T20:39:35Z
- **Closed:** 2026-10-11T17:25:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2204
- **Merged:** 2026-10-11T17:25:05Z
- **Merge commit:** bfe0243ab749cfd4acb0345d08c2a4b399eb15d8
- **Base / Head:** main / UOR-Foundation:deepseek/port-kernel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Second slice of the native port, and the one part of a product-key memory that has no analogue in the served read** — so an error here would silently change which slots a served model reads. `stack_memory_select` reproduces `stack_memory.rs`'s rule exactly: per side the `top_k` best sub-keys by score descending with ties to the lower index, then the `top_k` best pairs by summed score with ties to the lower slot (`first * sub_keys + second`). **No allocation, no multiplication, no division**; stack buffers bounded by `MAX_SELECT_SIDE` (64), which `StackShape::validate` now enforces for a carried memory's `sub_keys`.

**Tests:** `memory_select` 4 passed, `memory_shape` 3 passed, library suite unchanged. The bound test **found a real bug in the first draft** — the taken-set is indexed by sub-key, so a side longer than the buffer read past it; the kernel now refuses such a side and the shape refuses to carry it.

Weights and the value gather are the rest of the port; **the exporter keeps refusing until the engine serves it**, so no container claims a capability it lacks.

Line: native port for the product-key memory · count 0/3 · headline unchanged (engine-side slice; the metric run is pre-registered)

References #2029
PR; sleep 150; gh pr checks 2>&1 | grep -v skipping | head -2; gh pr merge --squash --delete-branch=false 2>&1 | tail -1


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

