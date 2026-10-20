# #2236 — The routing decision as a pure function, with the null arms it must carry (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T03:17:39Z
- **Updated:** 2026-10-12T03:32:06Z
- **Closed:** 2026-10-12T03:29:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2236
- **Merged:** 2026-10-12T03:29:19Z
- **Merge commit:** 841a4279cb2f23dba19e8301ed964c5a274194ce
- **Base / Head:** main / UOR-Foundation:deepseek/route-decision
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The decision the read will make, as a pure function** — so the forward-path change is a *call* rather than a rewrite, and the routing logic can be tested without a model.

| | |
|---|---|
| `RouteConfig` | the tree (cells, edges, root), the address geometry (buckets, inner bits), `k`, and the null arm — with **`Default` = OFF** (no cells, `k = 0`), so a run that does not ask for this mechanism executes the model it had. `active()` is false for `k = 0` or a single cell |
| `RouteNull::Tangent` | moves the **representative** inside each cell, leaves the cells alone ⇒ **the candidate set is unchanged** and what changes is the fine coordinate the read scores. **That is the attribution clause** |
| `RouteNull::Normal` | moves the **cells** ⇒ the candidate set changes too; the coarser null that bounds how much any addressing could buy |
| `route_sources(config, query, sources, out)` | indices **nearest first**, from the tree over the cells; representatives travel with the candidates |

**The test that matters for the design:** a **tangent** null routes to **exactly the same candidates** while moving the representatives the read scores, whereas a **normal** null moves the cells and so may move the route. That asymmetry is the whole control, and it is now pinned by a test rather than by intent.

**Checks, all local before the push:** fourteen tests in the module pass, including the three new ones (inactive routes nothing — both `k = 0` and the default; the route follows the cells with real, non-repeating indices and the query's own cell first; the tangent/normal asymmetry above). Claim-wording gate passes; no formatting diff in the touched file. **Nothing is called by the read yet**, so the "off state is today's model, bit for bit" requirement still holds trivially.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (final preparation slice; no compute spent)

References #2029
PR; sleep 280; gh pr checks 2>&1 | grep -v skipping | head -2; gh pr merge --squash --delete-branch=false 2>&1 | tail -1


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

