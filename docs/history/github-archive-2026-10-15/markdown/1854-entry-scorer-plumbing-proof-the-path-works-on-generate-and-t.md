# #1854 — Entry-scorer plumbing proof: the path works on Generate, and the mixed pool cannot be beaten by a bounded bonus (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T19:22:01Z
- **Updated:** 2026-10-07T19:24:33Z
- **Closed:** 2026-10-07T19:24:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1854
- **Merged:** 2026-10-07T19:24:33Z
- **Merge commit:** e5e25ea897dddd05a54f973a164020407c979d9d
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

Two things on top of the merged feasibility tool.

**1. Pair features in the feasibility read.** A comparison feature is not a single key; it is the set of pairs between the query's tokens and the record's tokens or fields, with a row scored by summing a learned table over its pairs. Added `QueryRelation`, `QueryView`, `QuerySourceToken` and `QueryContextToken`, each evaluated by leave-one-query-out.

Result on the retained panel — fit on every query but one, predict the held-out query's rows:

| pair feature | test rows | rows with a known pair | correct | accuracy |
|---|---:|---:|---:|---:|
| query × relation | 512 | 512 | 172 | 33.6 % |
| query × view | 512 | 512 | 172 | 33.6 % |
| query × source token | 512 | 512 | 128 | 25.0 % |
| query × context token | 512 | 512 | 148 | 28.9 % |

Every one is covered and every one is at or below chance for a two-class panel. The pairs that recur are function words with no bearing on which template is asked for. Combined with the earlier read (metadata-only and context-only lookups: 0 % at full coverage), **no lookup shape survives an unseen wording**, which is what puts a genuine comparison mechanism in the position of the only remaining candidate.

**2. The entry-scorer plumbing proof on the native path.** `mode = "entry_ceiling"` with `ceiling_scorer = true` fits a query-keyed table on one label-independent half of the panel and applies it at the entry position — as a bounded integer bonus on the learned token, scaled by the learned share — *before* the existing pool reduction. It then reports the base and scored entry decision on the held-out half. Nothing else about the forward, the pool or the clip changes.

Sealed result, `deepseek/entry-ceiling/scorer-proof-attempt1/` on the canonical volume, 280.6 s:

| | held-out half (256 rows) |
|---|---:|
| base, Generate-only pool | 2 / 256 |
| base, mixed pool | 0 / 256 |
| **with scorer, Generate-only pool** | **256 / 256** |
| with scorer, mixed pool | **0 / 256** |

`fit_rows` 256, `test_rows` 256, `test_rows_covered_by_table` 256, `distinct_features` 12.

**Reading it honestly.**

- The learned table moves the entry decision through the *real* forward and pool: 2 → 256 of 256 on the held-out half. The path works, and the instrument detects it.
- **In the mixed pool the same scorer scores zero.** A bonus bounded by the score clip cannot outrank a physical Copy atom sitting at the clip: adding the answer does not beat a saturated competitor. The entry position therefore cannot be fixed by a term inside the Copy-containing sum — the entry decision has to be *owned* by its own scorer, with the mixed pool resuming from position 1. That is exactly the shape of the core response-entry fit that already reports 32/32 (`docs/evidence/native_geometric_response_entry_973.json`), and it is now also the measured consequence of the pool arithmetic.
- The 256/256 is **memorisation of twelve query signatures**, and the covered-row count says so. It proves plumbing, not generalisation. Held-out wordings still require the comparison feature, which no lookup supplies.
- Not yet done: exporting the scorer inside the checkpoint. The table is fitted and applied in-process; carrying it through export is mechanical and is the next commit.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **12 passed / 0 failed**, touched-file `rustfmt --check` clean. Defaults unchanged: the mode is opt-in, zero-update, and `ceiling_scorer` defaults to false.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

