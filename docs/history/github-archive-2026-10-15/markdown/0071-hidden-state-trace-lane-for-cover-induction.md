# #71 — Hidden-state trace lane for cover induction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:24Z
- **Updated:** 2026-07-23T07:50:31Z
- **Closed:** 2026-07-23T07:50:31Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/71
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: cover fineness sweep (decide whether geometry or evidence is the bottleneck first). Cover induction currently runs on the deterministic context-bundle geometry (i64 dyadic bundle). The TeacherOracle trace surface (observe.rs: hidden_state(), top_k()) exists but is unused. Wire it as a second observation lane: run cover induction over final hidden states for a fixed sample, and compare region quality (reference recall, Gate C) against the bundle lane at the same region budget. Acceptance: a per-lane comparison table and a recorded lane choice; the trace surface gains a quantized (i8) spill path per plan §4.1 memory rules if adopted.


---

## Definition of Done
- A hidden-state observation lane (TeacherOracle trace surface) exists alongside the bundle lane, and both lanes are evaluated at the same region budget.
- A lane choice (bundle / hidden-state / union) is recorded with measured evidence.

## Acceptance criteria
- [ ] Cover induction runs end-to-end over final hidden states for a fixed observation sample.
- [ ] Per-lane reference recall and Gate C numbers are compared in one table.
- [ ] If adopted: quantized (i8) spill for hidden-state vectors per plan §4.1 memory rules, with the memory budget honored.
- [ ] If rejected: the comparison table showing no gain is attached.

## Comments (2)

### comment 5055842890 — 2026-07-23T07:50:29Z — auser

## Lane comparison — recorded choice: REJECTED (no gain)

Full run in #95 (fixture corpus, 150k positions, 119,964 train / 30,036 held-out, same 42-region default cover both lanes).

Reference recall (held-out), per depth:

| lane | d1 top1/topM | d2 top1/topM | d3 top1/topM | frontier (d3) |
|---|---|---|---|---|
| bundle | 78.9 / 93.7% | 69.8 / 82.8% | 58.9 / 72.4% | 2.44/3 |
| hidden | 80.4 / 92.8% | 66.9 / 82.8% | 60.5 / 76.2% | 2.54/3 |

Gate C (Rule 1+2, held-out):

| lane | regions | bytes | top-1 | bits/token |
|---|---|---|---|---|
| bundle | 42 | 10,867,179 | 31.7% | 9.8612 |
| hidden | 42 | 10,868,395 | 2.8% | 16.3302 |
| tla3 (shared bundle-lane baseline) | - | - | 31.7% | 11.8781 |

Reference geometry is comparable (hidden slightly better at d1/d3), but Gate C collapses (−28.9 pts top-1, +6.47 bits/token): hidden-state sign sigs route worse at emission time even when clustering looks fine. **Bundle lane stays; hidden lane not adopted** — the i8-spill box is therefore intentionally unticked. The lane machinery (`trace_lane.rs`, `lane-compare`) is merged and reusable if a richer teacher changes the picture (#73).

### comment 5055843009 — 2026-07-23T07:50:30Z — auser

Closed with evidence: comparison table above, choice recorded in lane_compare.json and the table; landed in 3b5e015 (#95).


---
_Comment/review ids in this file: 5055842890, 5055843009_

