# #263 — kappa-addressed region objects + resolver: make UOR addresses participate in retrieval

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:05:51Z
- **Updated:** 2026-08-01T04:12:10Z
- **Closed:** 2026-08-01T04:12:10Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/263
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The graded store is one monolithic artifact; its regions (prefix subtrees) have no identity of their own. The product thesis — inference as content-addressed lookup, servable by whoever holds the region — requires regions to be **first-class UOR-addressed objects**: a κ-label per region, and a resolver interface that fetches region objects by address (local `ModelStore` first; network resolvers later without interface change).

This is path (a) from the UOR-usage audit: today κ-labels of store entries are parallel metadata around a router that works by other means; this issue makes addresses participate in retrieval.

### Sketch
- Region object = a prefix subtree (depth-d key + its descendant distributions), serialized canonically; κ via the canonical pipeline (see the R4G1-realization issue for the container half).
- Resolver trait: `resolve(region_kappa) -> Option<RegionObject>`, first impl backed by `ModelStore`'s CAS; the runtime's deepest-populated-prefix walk asks the resolver instead of the in-memory map when a region is not loaded.
- Region granularity (which depth becomes an object) is a measured decision — interacts with the #244 store-size outcome and the #243 address-space design.

### DoD
A store can be exported as region objects + manifest (list of region κs); a fresh process with only the manifest and a resolver serves predictions identical to the monolithic store (byte-identical outputs on the D3 held-out set); runtime kernel contract untouched (resolution is I/O-side, prediction stays mul-free).

*Depends on #243 (address space) and the #244 decision. Companion: R4G1 uor-addr realization. From the "UOR in the loop" gap analysis.*


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

