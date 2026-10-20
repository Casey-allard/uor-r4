# #2234 — The spanning tree over the cells, so routing is a path rather than a search (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:42:55Z
- **Updated:** 2026-10-12T02:57:05Z
- **Closed:** 2026-10-12T02:53:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2234
- **Merged:** 2026-10-12T02:53:20Z
- **Merge commit:** d3aad9448ede45a6894f322386e63edb15bf5bd2
- **Base / Head:** main / UOR-Foundation:deepseek/cell-tree
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Matthew's layer-2 spanning tree, implemented as the pure module — landed with its tests before any forward-path change.**

| | |
|---|---|
| `CellTree::build(cells, edges, root)` | deterministic breadth-first from a **recorded** root, first edge wins, and reports how many cells it **reached** — so an edge list that does not span shows up as a forest rather than silently |
| `path(cell)` | the **canonical address**: the root-to-cell path. Re-rooting is a **re-encoding**, not a renumbering — the property Spiralcore calls context translation across demarcs |
| `common_ancestor` / `shared_ancestors` | the tree's distance: the node every path between two cells passes through, and the shared-ancestor count that replaces hash agreement |
| `route(query, candidates, out)` | source indices **nearest first** — the query's cell, its ancestors to the root, then its children. The candidate set is the **adjacent subtree**, bounded by branching × depth, not by the context |
| `tangent_cell` / `normal_cell` | the movements are **structural**: tangent is the identity on the tree (only the representative moves), normal leaves the node. The attribution clause tests the tree, not a chosen scramble |

**Two cells have exactly one path between them, so the tie-breaking the scored read needed does not arise** — which is the quiet reason this structure is worth more than the address hash it replaces.

**Checks, all local before the push:** eleven tests in the module pass (`cargo test -p uor-r4-training --lib stack_address`), including four new ones — the tree spans and paths are canonical (a forest is reported, not hidden); two cells have exactly one path and prefix agreement counts ancestry; the route walks the adjacent subtree nearest first and honours a bounded output; and the movements stay at, or leave, the node as the tangent rule requires. Claim-wording gate passes; no formatting diff in the touched file. **Nothing is called by the read yet**, so the "off state is today's model, bit for bit" requirement still holds trivially.

**Next slice, unchanged from the execution card:** the router inside the read's forward (walking this tree), its configuration gate, and the bit-exact off-state test — then the nine pre-registered fits with the tangent arm as the attribution clause.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (preparation; no compute spent)

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

