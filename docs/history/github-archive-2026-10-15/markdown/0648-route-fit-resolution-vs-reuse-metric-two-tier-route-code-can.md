# #648 — route-fit: resolution-vs-reuse metric + two-tier route-code candidate (route-fit/2 research, evaluation-gated)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T22:06:08Z
- **Updated:** 2026-08-13T03:23:13Z
- **Closed:** 2026-08-13T03:23:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/648
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Research-available (from historical-program mining). Do NOT replace route-fit/1 — evaluation-gated per project principle: only pursue if a specific resolution need appears that we cannot otherwise solve.**

## Basis (measured, historical program)
In a guarded head-to-head on exact transport traces: a compact one-shot angular route partition (high key reuse 0.937, zero fallback) resolved **only 0.168** of transport mass (0.832 unresolved inside non-promoted classes). `base_gap` + **selective fallback** reached **0.990 resolved** at 0.331 fallback-step fraction, zero instability. Recorded lessons: delayed refinement > one-shot partitioning; account fallback explicitly; judge partitions on downstream unresolved burden, not key reuse.

## Relevance
R4RouteAttentionV1 top-M selection is a **one-shot partition** today. If a future workload exposes an unresolved-mass problem, route-fit/2 candidate:
1. Pre-register an **effective-resolved-fraction-under-fixed-fallback-budget** metric alongside overlap.
2. Prototype a **two-tier code**: coarse 288-bit code + selective promotion to a finer code for high-unresolved-mass classes.
3. Evaluate #643's MSA candidate on this axis, not overlap alone.

## Non-goals
route-fit/1 stays the shipped method. This is a pre-registered design direction to hold in reserve, not implement now.


## Comments (1)

### comment 5275565153 — 2026-08-13T03:23:13Z — Casey-allard

Closing as **evaluation-gated, methodology preserved** (per the issue's own project principle: do not pursue route-fit/2 unless a specific resolution need appears that route-fit/1 + selective fallback cannot solve).

Recorded activation condition for reopening: a measured workload where R4RouteAttentionV1 top-M one-shot selection leaves material unresolved mass (the historical basis: one-shot angular partition resolved only 0.168 of transport mass vs 0.990 with base_gap + selective fallback). Until that appears, route-fit/1 stays the mechanism. The head-to-head lessons (delayed refinement > one-shot; account fallback explicitly; judge on downstream unresolved burden, not key reuse) are captured here for whoever hits that need.


---
_Comment/review ids in this file: 5275565153_

