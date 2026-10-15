# #502 — Adoption gate: drop the lexical weight (W=0) on the content-query serving path (+0.022 MRR)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T14:43:46Z
- **Updated:** 2026-08-08T17:14:25Z
- **Closed:** 2026-08-08T17:14:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/502
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Adoption gate for dropping the `shared_count` lexical term (`W = 0`) on the deployed content-query retrieval path. Spun out of #500 / #501, which established that on the post-#490 content-query path the lexical weight is no longer inert.

## The measurement (from #500, on `main`, standard caps)

| path | W | top-1 | MRR | recall@20 |
|---|---:|---:|---:|---:|
| content query (deployed) | 100 | 0.7840 | 0.8542 | 0.9900 |
| content query, lexical term dropped | 0 | 0.8160 | 0.8763 | 0.9900 |

Dropping the lexical term is worth **+0.022 MRR and +0.032 top-1** at equal recall, and it also removes the `shared_count * 100` term that masked the dead cosine (#486) for months — a simplification, not just a tune.

## Why this is a gate and not a silent flip

`DEFAULT_LEXICAL_WEIGHT = 100` is global. Flipping it to 0 has blast radius on the **non-content path**, where the scaled `W = 0` arm is confounded by `slice_norm` (a per-bucket scalar) — `lexical_weight.rs` documents this and uses an unscaled bare-cosine arm as the honest comparator there. So "drop the weight" on the content path is not the same one-line change as "set the default to 0" everywhere.

The exit discipline #480 and #484 both recorded: an ordering change moves the pinned #421 anchor-accuracy rows and must clear `router_reconnect` against a scored R4G1 artifact (and the memory-lift gate) before the serving default flips.

## Work

1. Decide the shape of the change: a content-path-only weight default (so the non-content path keeps `W = 100`), or a global flip with the non-content path re-measured on the unscaled comparator.
2. Run `router_reconnect` + memory-lift against a scored artifact at `W = 0` on the content path; confirm no regression on the pinned #421 rows.
3. If clean, flip the deployed default and re-pin `CONTENT_*` in `lexical_weight.rs` to the `W = 0` row.

## Not in scope

The #500 reassessment PR (#501) — that re-baselines and reconciles the record but deliberately does not alter serving behaviour. This issue is where the behaviour change is gated.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

