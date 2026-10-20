# #383 — feat(score): #380 — consume NGRAM row top-k as the emission distribution (first config to beat the baseline on both axes)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T17:00:28Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-03T17:29:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/383
- **Merged:** 2026-08-03T17:29:25Z
- **Merge commit:** 7e14df29ed2ad0e0c7fd7e2346e971d70a9e07f8
- **Base / Head:** main / UOR-Foundation:i380-row-distribution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements #380 from the #374 campaign. A row hit now carries the **whole bounded NGR1 row** (up to `context_entries`, ascending token order) as the emission distribution; selection stays the canonical argmax, so **served tokens are byte-identical** — only the distribution changes.

Measured on the campaign's own arm (fixture, TLA7, window-on rows-on):

| | before | after | store baseline |
|---|---|---|---|
| rule12 top-1 | 41.00% | **41.00%** (unchanged by construction) | 39.09% |
| rule12 bits | 10.7883 | **6.7112** | 8.4461 |

**First configuration on record above the baseline on both axes.** The #362/#364 replacement architecture — lexical rows emit, geometric path routes — is now empirically superior on the windowed Gate C surface.

Mechanics: `context_prediction` split into `context_row` + `context_row_argmax` (byte-identical selection rule); `ScoreOutcome` carries full candidates/components; witness `candidate_count` honest; `StepOutcome` reports row width; replay self-consistent. Unit test pins full-row candidates + unchanged argmax. Gates: fmt, clippy `--all-targets --all-features`, full workspace tests from cleaned targets.

Closes #380. Refs #374, #364, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

