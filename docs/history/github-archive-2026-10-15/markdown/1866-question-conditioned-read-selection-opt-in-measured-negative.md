# #1866 — Question-conditioned read selection (opt-in): measured negative, and the missing piece it localises (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:01:01Z
- **Updated:** 2026-10-09T13:19:58Z
- **Closed:** 2026-10-08T02:02:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1866
- **Merged:** 2026-10-08T02:02:58Z
- **Merge commit:** 5e5c436c340922811b04b8b519e03bb86256016e
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

An **opt-in** read-selection policy, default unchanged everywhere: select the occurrence whose cumulative source state is closest to the query state instead of the earliest raw Copy maximum. The distance is an L1 sum over H4 code indices — integer only, no product of runtime values — with ties falling back to the summed Copy score and then to the earliest occurrence, so the previous policy is the degenerate case.

Implemented in both the harness read (`geometric-frozen-map-fit.rs`) and the production bank read (`native_bank_generate.rs`, `with_query_conditioned_selection`). Enabled with `"query_conditioned_read": true`.

## Measured result: the gate it was built for fails

This is a **preserved negative**, recorded because it localises the real requirement. On the prospectively fixed transfer panel:

| | before | after |
|---|---:|---:|
| accuracy of the best relation→form mapping for the selected occurrence | 0.594 | **0.5625** |
| within vs across question-form state distance | 7.931 / 7.931 | 7.946 / 7.941 |
| entry correct (transfer / retained) | 0/32, 0/512 | 0/32, 0/512 |

The selection moved but did not align, and the boundary state still does not cluster by question.

## Why, exactly

The comparison has nothing factorised to compare. The query enters the read only as raw token ids folded into the same monolithic H4 hash as everything else (all 32 query-state vectors distinct across 16 rows per form; every code changes with both question and content). The candidate side carries a full frame (`commit`, `record`, `entity`, `relation`, `view`, `event`), and the panel enumerates which frames exist:

```
segment kinds: Context, Source
Source frame:  {commit, entity, event, kind, original_source_ids, record, relation, scope, view}
case keys:     {actual_prefix_ids, id, query_ids, segments}
```

There is **no query segment and no query frame** — the question is a bare token sequence while the records are typed. So no distance between two hashes can compute "which occurrence answers this question": the asked relation is never represented on the query side.

**Requirement this produces:** type the query onto the same frame vocabulary the records use (asked `relation`/`view`/`entity`), then select by integer frame match. The answer side already has this (`answer_oracle` authored intent, relation admission); the query side has none, and that — not the selection rule — is the missing piece.

## Verification

- `cargo test -p uor-r4-core --lib`: **1,118 passed / 0 failed / 64 ignored** with the change in place.
- `cargo test -p uor-r4-training --example geometric-frozen-map-fit`: **15 passed / 0 failed**.
- `cargo check` clean on both packages; touched files `rustfmt --check` clean.
- The flag defaults to false, so every existing parity path — including the saved `selected_ordinal` comparisons in the native bank parity/generalisation examples — keeps its policy unchanged; `BankSelection::EarliestCopyMax` is passed explicitly on the diagnostic path that must not change.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

