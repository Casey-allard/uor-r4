# #504 — feat(#502): drop the lexical weight on the deployed content-query path

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T16:51:26Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T17:14:23Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/504
- **Merged:** 2026-08-08T17:14:23Z
- **Merge commit:** 6cb6bd7c4b120fc1014829ec1f3325424d5a0fb8
- **Base / Head:** main / UOR-Foundation:fix/502-content-path-lexical-weight
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Drops the `shared_count * 100` lexical term (**W = 0**) on the deployed content-query retrieval path, the adoption gate spun out of #500/#501.

The default weight is now path-scoped (`UorR4Router::default_lexical_weight`):

| path | default weight | why |
|---|---|---|
| content query (deployed since #490) | **0** | cosine carries signal here; the term was tuned against a dead cosine |
| routing (measurement only) | `DEFAULT_LEXICAL_WEIGHT` = 100 | its `W=0` arm is confounded by `slice_norm` (#484), and it is not deployed |

An explicit `set_lexical_weight` still overrides on either path.

## The measurement (content path, standard caps, re-confirmed on this branch)

| W | top-1 | MRR | recall@20 | deranged null |
|---:|---:|---:|---:|---:|
| 100 (was deployed) | 0.7840 | 0.8542 | 0.9900 | 0.0000 |
| **0 (now deployed)** | **0.8160** | **0.8763** | **0.9900** | 0.0000 |

+0.022 MRR, +0.032 top-1 at equal recall, null dead in both arms. It is also a simplification — it removes the `100×` term that masked the dead cosine (#486) for months.

## Why this clears the gate

#480/#484 recorded a gate: "an ordering change moves the pinned #421 anchor-accuracy rows, so clear `router_reconnect` first." That premise is **backwards for this knob**, the same way #490 found it backwards for the content-query flip. The lexical weight is read at exactly one site — `retrieve_geometric_resonance`, reachable only via `get_top_resonances_native` (the resonance serving surface: server + WASM). `router_reconnect` ranks by **bare cosine over the stored vectors** (it says so in its own module docs — `get_top_resonances_native`'s per-item allocation is infeasible at that scale), and the score pipeline never calls this path. So the #421 anchor rows are **invariant** under the weight; there is no ordering to regress. Verified by code read (single consumer) plus the memory-lift/score-pipeline non-dependence.

## Honesty note

The +0.022 MRR is **below** #484's pre-registered +0.05 retune bar. This adoption does not rest on the MRR margin — it rests on the simplification and the strict top-1/recall dominance (W=0 is ≥ W=100 on every metric here). Recorded as such in the code and RESEARCH.md so the sub-threshold point is not lost.

## Tests / validation

- New **non-ignored** unit test `lexical_weight_default_is_content_path_conditional` pins the default logic (content→0, routing→100, explicit override wins) — the anti-vacuity guard, fails if a future edit changes the serving ranking.
- `lexical_weight.rs`: re-pins the deployed row to `CONTENT_W0_MRR` 0.8763 / top-1 0.8160, keeps the W=100 row as a reproducibility anchor, asserts W=0 ≥ W=100 and recall ≥ 0.97. Content-mode sweep run on this branch reproduces 0.8763 / 0.8542 exactly, deranged null 0.0000.
- `query_projection.rs`: the `content (deployed)` arm now tracks the W=0 default (its lower-bound assert only strengthens).
- fmt / clippy (`-D warnings`, `--all-targets --all-features`) / 36 non-ignored router tests / `wasm32-unknown-unknown` build: all green in the cloud pre-flight.

Closes #502.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

