# #1909 — Two tool limits: attribution refuses snapped artifacts; d11 needs model= (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T13:58:44Z
- **Updated:** 2026-10-09T13:22:50Z
- **Closed:** 2026-10-08T14:00:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1909
- **Merged:** 2026-10-08T14:00:23Z
- **Merge commit:** c8a1888880799222cc81384cde1bada43f1e6b7e
- **Base / Head:** main / UOR-Foundation:deepseek/tool-limits
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Two tool limits that constrain the next experiments

**1. `rounding-attribution` refuses snapped artifacts.**

```
"rounding attribution measures the D10 comparator, which computes the free transport"
```

It attributes quantisation loss against a comparator computing *continuous* transport, so it cannot speak about an artifact whose model was trained with `transport_snap=icosian`. **Round 30's follow-up — "run `rounding-attribution` on the fine-tuned artifact to see which groups absorbed the tripled loss" — is therefore not executable as written, and I'm withdrawing it rather than leaving it standing as a plan.**

**2. `d11-evaluate` needs `model=` for a snapped artifact.**

```
"a snapped artifact needs model= for its float side"
```

The float side is required because such an artifact is scored against **its own float model** rather than a second integer engine — which is also why `d10` is null in that record. Recorded so the next run doesn't rediscover it.

## Why this matters for the plan

Round 30 left exactly one concrete question — *which parameter groups absorbed the tripled quantisation loss* — and this round shows the available tool **cannot answer it**. The remaining executable path is empirical: **sweep the fine-tune length and measure the served result directly**, because neither the float proxy (round 29) nor the attribution tool predicts it.

That sweep is running: **250 steps** → `snap-evaluate` → `export` → `d11-evaluate`, on the same held-out stream, to be compared against:

| arm | served (D11) |
|---|---:|
| no tune (round 16) | **0.93277** |
| 500-step tune (round 29) | 0.95150 |

Scope: the limits are tool behaviours, quoted from their own error messages; no model-quality result is claimed this round. Evidence: `docs/evidence/attribution_tool_limits_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

