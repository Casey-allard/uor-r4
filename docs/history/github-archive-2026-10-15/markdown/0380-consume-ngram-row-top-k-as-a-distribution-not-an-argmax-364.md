# #380 — Consume NGRAM row top-k as a distribution, not an argmax (#364 follow-up b — rows beat baseline top-1, bits collapse)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T16:27:37Z
- **Updated:** 2026-08-03T17:29:27Z
- **Closed:** 2026-08-03T17:29:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/380
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up (b) from the #364 campaign synthesis (results: #374). With the Gate C window mode (#375/#378), NGRAM rows measured **41.00% top-1 — above the 39.09% store baseline, the first configuration on record to do so** — while bits collapsed to 10.79 (vs baseline 8.45) because a row hit yields a single-candidate distribution: `context_prediction` returns only the argmax entry, `candidate_count: 1`.

The NGR1 format already stores up to `context_entries` (64) scored candidates per row. Scope: consume the row's full entry list as the emission distribution on a row hit (certifier scorer + serving engine `context_backoff`), preserving the deterministic argmax tie-break so served tokens are unchanged — only the distribution (and therefore bits/coherence and downstream sampling) changes. Measure with the window-mode A/B (#374 tables as the before).

This is the missing piece for the #362/#364 "lexical rows replace emission conditioning, geometric path keeps routing" architecture: top-1 already clears the baseline; coherence is what's left.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

