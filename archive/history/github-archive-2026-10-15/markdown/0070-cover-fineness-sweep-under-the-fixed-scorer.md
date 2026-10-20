# #70 — Cover fineness sweep under the fixed scorer

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:22Z
- **Updated:** 2026-07-23T03:36:04Z
- **Closed:** 2026-07-23T03:36:04Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/70
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64. The current induced cover is 42 regions across 3 depths — far coarser than the 4×256 TLA class grid, which is one suspected cause of residual correlation and weak emissions. Sweep region budget (42 → 128 → 512) and split entropy threshold (0.25 → 0.1 bits) and measure reference top-1/top-M recall, frontier width, artifact bytes, and Gate C top-1 agreement under the fixed formula at each point. Acceptance: a rate-distortion table (regions × bytes × agreement) and a chosen operating point with the tradeoff documented; predictive_sufficiency module reused for the depth curves where applicable.


---

## Definition of Done
- A rate-distortion sweep of the induced cover (≈42 → 128 → 512 regions; split threshold 0.25 → 0.1 bits) is measured under the fixed (#64) scorer.
- An operating point (region budget + split threshold) is chosen and recorded with the tradeoff.

## Acceptance criteria
- [ ] Each point reports: reference top-1/top-M recall, frontier width, artifact bytes, Gate C top-1 agreement.
- [ ] Results are presented as a rate-distortion table (regions × bytes × agreement), deterministic double-run.
- [ ] The chosen point is justified against the 42-region baseline row.

## Comments (1)

### comment 5054018821 — 2026-07-23T03:36:03Z — auser

Complete (1f02520), verified by independent CLI reproduction (identical table + recommendation + byte-identical artifact rows).

Rate-distortion table (fixture corpus, held-out n=30,036 at every point; fixed scorer, add-one):

| point | regions | bytes | R1+2 top1 | bits/token | ref-top1 | ref-topM | frontier |
|---|---|---|---|---|---|---|---|
| k0=8/gain=0.25/B=128 (rec) | 42 | 10,867,179 | 31.7% | 9.8612 | 58.9% | 72.4% | 2.44/3 |
| k0=8/gain=0.25/B=256 (default) | 42 | 10,867,179 | 31.7% | 9.8612 | 58.9% | 72.4% | 2.44/3 |
| k0=8/gain=0.25/B=512 | 42 | 10,867,179 | 31.7% | 9.8612 | 58.9% | 72.4% | 2.44/3 |
| k0=8/gain=0.10/B=128 | 54 | 10,878,315 | 31.7% | 9.8612 | 73.0% | 87.0% | 2.64/3 |
| k0=8/gain=0.10/B=512 | 54 | 10,878,315 | 31.7% | 9.8612 | 73.0% | 87.0% | 2.64/3 |
| k0=16/gain=0.25/B=128 | 88 | 10,907,371 | 31.7% | 9.8612 | 63.2% | 75.6% | 2.47/3 |
| k0=16/gain=0.25/B=512 | 88 | 10,907,371 | 31.7% | 9.8612 | 63.2% | 75.6% | 2.47/3 |
| k0=16/gain=0.10/B=128 | 100 | 10,916,075 | 31.7% | 9.8612 | 71.1% | 81.2% | 2.47/3 |
| k0=16/gain=0.10/B=512 | 100 | 10,916,075 | 31.7% | 9.8612 | 71.1% | 81.2% | 2.47/3 |

Verdict: RECOMMEND NO CONFIG CHANGE — the 42-region default is adequate for the scoring metric. Gate C top-1 is bit-identical (0.317086) at every point: under EXCT precedence, routing refinement never changes a selected token on this corpus, so every finer point is dominated. The budget axis is inert (byte-identical at 128/512; the grid collapses to 4 distinct artifacts). Acceptance notes per DoD: (1) all per-point metrics present (regions/depth, ref top-1/top-M, frontier mean/max, bytes, Gate C top-1 + bits/token); (2) deterministic — three byte-identical full sweep runs (2 test driver + 1 CLI); (3) the 42-row is the chosen operating point, justified exactly (+0 bytes/+0.0000 agreement vs recommendation). Metric-conditional note for the record: routing recall DOES move — 54 regions buys +14.1pp d3 ref-top1 for +11 kB (slope 130x floor) — so if recall ever becomes the objective metric, k0=8/gain=0.10/B=128 is the point to adopt.


---
_Comment/review ids in this file: 5054018821_

