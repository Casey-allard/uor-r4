# #2232 — The route should be a spanning tree over the cells, not a top-k over addresses (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:29:32Z
- **Updated:** 2026-10-12T02:38:43Z
- **Closed:** 2026-10-12T02:35:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2232
- **Merged:** 2026-10-12T02:35:09Z
- **Merge commit:** 2f7dbd7a83aaf78edde9daf016b84de7dd0e4d25
- **Base / Head:** main / UOR-Foundation:deepseek/spanning-tree
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Matthew's suggestion, folded into the mechanism brief: do it as a layer-2 spanning tree.** He is right that it is the better structure, and the brief now says why in four concrete ways:

| | spanning tree over the cells | my pre-registered two-level address |
|---|---|---|
| **address** | the **path from a fixed root to the cell** — canonical (one node, one path), **translatable by re-rooting** (Spiralcore's *context translation across demarcs*), compared by **prefix** so agreement means **shared ancestry** | a hashed cell plus a hashed representative |
| **route** | the path through the **nearest common ancestor** → candidate set is the adjacent subtree, cost **O(depth)** per token | top-k by agreement over the whole window |
| **ties** | **cannot exist** — two nodes have exactly one path between them | the tie-breaking rule I had to specify |
| **the nulls** | become **structural**: tangent = *within a node* (same cell, moved representative — the coordinate the geometry calls arbitrary); normal = *a branch change* | hashes with a chosen step and offset |

That last row matters most for what this line keeps getting wrong: with a tree, the **attribution clause tests the tree's structure rather than my choice of scramble** — the tangent movement is the geometry's own notion of "no movement", not an arbitrary perturbation.

**What changes in the experiment:** the address is built over a **spanning tree of the cells** (root fixed and recorded, depth and branching recorded); the route is the tree path rather than a top-k; tangent and normal are the within-node and branch-change movements; and the reported compute becomes **O(depth) per token against the softmax read's O(n)**. The quality and attribution clauses are unchanged. The primitive in `stack_address.rs` already carries the cell/representative split the tree is built on, and **the tree itself is the next slice** (tests: canonical paths, single-path uniqueness, both movements being structural).

Line: invariant-addressed read · count 0/3 · headline **unchanged** (design correction from the mechanism's own author, before any compute)

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

