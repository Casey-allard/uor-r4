# #2197 — The competing-mass margin: the read separates twice as sharply and the answers do not move (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T08:56:41Z
- **Updated:** 2026-10-11T09:05:01Z
- **Closed:** 2026-10-11T09:00:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2197
- **Merged:** 2026-10-11T09:00:35Z
- **Merge commit:** 72c4c621b1a1af3554e4eb3373cc741baac559cf
- **Base / Head:** main / UOR-Foundation:deepseek/margin-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The brief's own open question — *"a competing-mass margin loss, rather than only maximizing bound mass?"* — implemented as a hinge and trained at three seeds against the keeper's matched arms: the read separates twice as sharply and the answers do not move.**

Line: read binding, iterated (D22 order 4) · count 0/3 · headline before memory **65/120**, wrong-value **30** (keeper) → after **64/120**, wrong-value **30**

## The mechanism does exactly what it claims

| arm | bound mass | **competing mass** | hinge term | keeper's competing mass (matched seed) |
|---|---:|---:|---:|---:|
| M1 | 0.9012 | **0.0462** | 0.0397 | 0.0897 |
| M2 | 0.9078 | **0.0422** | 0.0361 | 0.0858 |
| M3 | 0.9060 | **0.0433** | 0.0373 | 0.0812 |

**Competing mass halves on every seed** — its own causal check, met — at **zero parameters and zero inference MACs**.

## The answers do not move

| pair | data order | margin arm | keeper | Δ memory | margin WV | keeper WV | Δ WV |
|---|---|---:|---:|---:|---:|---:|---:|
| M1 vs S1 | 20261009 | 18/40 | 19/40 | −1 | 10 | 12 | **−2** |
| M2 vs S2 | 20261010 | 20/40 | **24/40** | **−4** | **14** | **5** | **+9** |
| M3 vs S3 | 20261011 | **26/40** | 22/40 | **+4** | **6** | 13 | **−7** |
| **aggregate** | — | **64/120** | **65/120** | **−1** | **30** | **30** | **0** |

**Bar NOT MET** (≥ 5 fewer wrong-value failures on ≥ 2 of 3 seeds; one seed reaches it) → **REJECT this configuration**, mechanism stays open (D22 §1). M1's float BPB is 1.17485 against its matched arm's 1.17142 (+0.0034, inside the bar's 0.01); M2/M3 named pending.

## Compute ledger

| | keeper | **+ margin** | change |
|---|---:|---:|---:|
| parameters | 28,958,761 | 28,958,761 | **0** |
| MACs/token, served (context 384) | 30,203,136 | 30,203,136 | **0** |
| wall per training step | 0.058 s | 0.060 s | **+~4 %** |
| competing mass | 0.086–0.090 | **0.042–0.046** | **−50 %** |
| wrong-value over 120 | 30 | **30** | **0** |

## What it establishes

1. **The remaining wrong-value failures are not "the read spread onto the competitors."** The margin halves exactly that quantity and changes nothing in aggregate. Measured, not assumed.
2. **The keeper's gain came from steering, not sharpening.** Read binding moved the panel (47 → 30 against controls) by putting mass on the asked value at all; pushing the same quantity further buys nothing.
3. **Two cycles in a row find the read's *scoring* exhausted** — geometric exactness costs 13.4× and buys nothing ([#2194](https://github.com/UOR-Foundation/uor-r4/pull/2194)), sharper separation costs ~4 % of training wall and buys nothing (this). What remains on this path is the read's **computation**: an address instead of a search, the one change that also has a compute argument (the read is 17.6 % of the stack's MACs at context 384 and **40.5 % at 4,096**).

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass** at this head. The term itself is the prep PR [#2196](https://github.com/UOR-Foundation/uor-r4/pull/2196) with its focused unit test.
- [x] Three seeds, matched controls (the keeper's own arms), row-paired; supervision and margin active from step 0.

**Declared:** three seeds on a **40-row** panel (±3 rows of noise — the per-seed spread here is *larger* than that, which is why the aggregate is the summary and why one seed alone would have read the result either way); one margin configuration (0.5 at 0.03); M2/M3 BPB pending; masses are train-side.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

