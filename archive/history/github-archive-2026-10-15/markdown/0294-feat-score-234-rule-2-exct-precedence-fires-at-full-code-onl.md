# #294 — feat(score): #234 — Rule 2 EXCT precedence fires at full code only

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T03:05:59Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-30T04:25:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/294
- **Merged:** 2026-07-30T04:25:22Z
- **Merge commit:** 68daf4dae20dd404f9c162252c3976191d82c9c7
- **Base / Head:** main / UOR-Foundation:issue-234-rule2-depth
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the maintainer decision recorded on #234 (Casey, 2026-07-29): Rule 2 EXCT precedence fires only when the probe resolves at the FULL graded code (level == STAGES) with support ≥ EXCT_SUPPORT_MIN. Sub-full-depth resolutions are prefix backoff: still recorded in the witness (level/total, admitted 0), but they admit no evidence and no longer preempt the graph — the position falls through to Rule 1. Both live probe surfaces change together (score_candidates + streaming step scorer); witness replay rebuilds a fresh scorer and stays consistent by construction (64/64 replays ok on the evidence run); the legacy Σ-over-cloud comparison path is intentionally unchanged.

## The honest Gate C row (pinned fixture harness, 30,036 held-out)

| | before (tie) | after (honest) |
|---|---|---|
| status split | ExactContext 30,036 / 0 / 0 | ExactContext 14,919 / Graph 15,080 / Novel 37 |
| Rule 1+2 blended | 30.02% / 10.24 bits | **21.05% / 12.53 bits** |
| per-status exact | 30.02% / 10.24 | 41.11% / 8.55 |
| per-status graph | — (never exercised) | **1.23% / 16.47** |
| win/loss vs baseline | 0 / 0 (identical argmax) | 117 scorer-only / 2,811 baseline-only |

The regression vs the old tie is the accepted, recorded consequence: Gate C now measures the graph path's actual contribution on the ~50% strict-miss slice instead of restating exact-context recall. The ExactContext split at 14,919 equals the probe-level histogram's L4 count exactly (#293's instrumentation) — the two changes cross-verify.

Serving statuses shift accordingly (Graph/Novel → D4 policy where prefix backoff used to serve as EXCT); the C serving row re-measures under #243 Phase C adoption.

Advances #234 (with #293); the graph-path number (1.2%) is now the one to improve.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

