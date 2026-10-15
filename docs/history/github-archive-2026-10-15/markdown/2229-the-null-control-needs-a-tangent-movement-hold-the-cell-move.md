# #2229 — The null control needs a tangent movement: hold the cell, move the representative (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:12:44Z
- **Updated:** 2026-10-12T02:22:09Z
- **Closed:** 2026-10-12T02:18:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2229
- **Merged:** 2026-10-12T02:18:21Z
- **Merge commit:** a1ef4b2b15c5fc8df95bbb2abcd7592e5a9ca86d
- **Base / Head:** main / UOR-Foundation:deepseek/tangent-null
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Owner correction, applied: "a null requires a tangent movement."** My pre-registered disposition used a **permutation** of the address space as the null — which destroys the structure, so it asks whether *any* address works, not whether the *geometry* does. That is the wrong null for this mechanism, and the fix is structural rather than cosmetic.

| | |
|---|---|
| `Address { cell, representative }` | the structure made explicit: the **cell** is the coarse invariant the route compares, the **representative** is the fine coordinate inside it. `packed()` is coarse-to-fine, so a cell match always outranks a fine one |
| `Address::tangent(step, inner_space)` | **the tangent null**: holds the cell, moves the representative. If the model does as well with the representative moved, it is reading **the cell — the geometry** — and not an arbitrary fine coordinate; if it degrades, that coordinate is load-bearing and the address is a **lookup, not an invariant** |
| `Address::normal(cell_space, offset)` | the coarser null (the previous slice's permutation is its extreme case): changes the cell. Separates *routing beats noise* from *routing uses structure* |

**The brief's fair test now requires all three controls**, with the bar split accordingly: the softmax comparison is the **quality clause**, the **tangent** comparison is the **attribution clause**, and the normal null is reported **for scale** (it bounds how much any address could buy). The owner correction is recorded in the brief itself so it cannot be lost.

**Checks:** seven tests in the module pass locally (`cargo test -p uor-r4-training --lib stack_address`) — including the three new ones: a tangent movement keeps the cell and moves the representative; a normal movement changes the cell and keeps the representative; packing orders coarse-to-fine. Claim-wording gate passes. **Nothing is called by the read yet**, so the "off state is today's model bit for bit" requirement still holds trivially.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (preparation and design correction; the nine-fit experiment stays pre-registered and unstarted)

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

