# #2123 — The pointer split: READER 10, EMITTER 2, and the full two-digit run is NEVER selected (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:31:59Z
- **Updated:** 2026-10-10T03:23:27Z
- **Closed:** 2026-10-10T01:33:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2123
- **Merged:** 2026-10-10T01:33:22Z
- **Merge commit:** 4d10555f5973c3969b08cb05ed087cf17be50014
- **Base / Head:** main / UOR-Foundation:numeric-pointer-split
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## 1. The instrument is now trustworthy on both counts
The last defect: the values file is tab-separated `id, value, forbid, keys` and the probe took **everything after the first tab** as the value, giving 12-token `digit_ids` for two-digit values. Fixed to column 1 — **and asserted rather than checked by eye**, as the turn-grouping bug was: **every value must be two digits and must tokenize to exactly two ids, or the run errors and is VOID.** The assertion did not fire.

| check | result |
|---|---|
| byte-for-byte reproduction of the sealed replies | **13 of 13** |
| `digit_ids` exactly two tokens (hard assertion) | **13 of 13** |

**Structural qualifier stays visible: the artifact that answered the v5 "memory" panel carries NO MEMORY OPERATOR — only a copy pointer — so "10 of 40 on the memory category" is 10 of 40 for a copy-pointer dialogue stack on requests *named* memory rows.**

## 2. The split, under the four conditions
**(1) Both classes present — NOT inconclusive: `READER 10, EMITTER 2, PARTIAL 1`.**
**(2) `mem-040` positive control PASSES** — one of its two stored digits was selected (1 selected, 2 emitted).
**(3) Per-step records shown** for two dominant-class rows: `mem-005` (value `41`, digit ids `[22, 19]`) runs **14 steps with 0 selecting a stored digit**; `mem-021` the same.
**(4) Token ids throughout.**

| row | value | class | selected / emitted |
|---|---|---|---|
| `mem-005` | 41 | **READER** | 0 / 0 |
| `mem-006` | 75 | **READER** | 0 / 0 |
| **`mem-008`** | **68** | **EMITTER** | **1 / 0** |
| `mem-010` | 85 | **READER** | 0 / 0 |
| `mem-013` | 88 | **READER** | 0 / 0 |
| `mem-021` | 71 | **READER** | 0 / 0 |
| `mem-022` | 89 | **READER** | 0 / 0 |
| **`mem-024`** | **54** | **EMITTER** | **1 / 0** |
| `mem-026` | 99 | **READER** | 0 / 0 |
| `mem-029` | 83 | **READER** | 0 / 0 |
| `mem-037` | 98 | **READER** | 0 / 0 |
| `mem-038` | 71 | **READER** | 0 / 0 |
| `mem-040` | 84 | PARTIAL | 1 / 2 |

**The dominant mechanism is READER-side: on 10 of 13 rows neither stored digit token is ever the pointer's selected source.** The 2 **EMITTER** rows are `mem-008` (stored `68`) and `mem-024` (stored `54`): at **step 8** the pointer selected `source_id` **24** (`6`) and **23** (`5`), and the emitted id was **19** (`1`) with `matches_source: false` — **exactly the sealed reply *"Your sister's locker number is 1."*** On those two the pointer selected the right digit and the decoder dropped it.

**And the finding underneath both: 0 of 13 rows — including `mem-040`, the one that delivers the value — had BOTH stored digits selected. The pointer never selects the whole two-token run.**

## 3. What it does not settle
It classifies **these 13 rows on this artifact**. It does **not** say that a reader-side or emitter-side fix would **raise the score**; it says nothing about the reply half of criterion 1; and since this artifact has **no memory reader**, nothing here transfers to an addressed-memory path without its own measurement.

## 4. The ledger
Token count **REFUTED** · minimal pairs/digit order **REFUTED** · value addressability **REFUTED** · **the learned read/emit path SPLIT — first measurement: READER 10, EMITTER 2, PARTIAL 1** — on an instrument that reproduces the sealed replies 13 of 13 byte for byte.

## Delivery
Record [`docs/labs/numeric-pointer-split-2026-10-09/README.md`](docs/labs/numeric-pointer-split-2026-10-09/README.md); the extraction fix with its two-token assertion; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store before any local deletion: `numeric-pointer-split-2026-10-09.tar`, 53,248 bytes, md5 `a240262860c0b8d0f13287501988654c`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

