# #1268 — Carry whole phrases through geometric query updates

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T05:25:12Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-14T05:26:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1268
- **Merged:** 2026-09-14T05:26:23Z
- **Merge commit:** 004f362f220fda1d825496f9df0880c5be10612c
- **Base / Head:** main / UOR-Foundation:codex/phrase-query-update
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The existing updater rejected spaces even when the span reader selected the correct phrase. Add an explicit bounded phrase payload window to the shared learned query-update and completion path, preserving the historical word entrypoint. Re-encode the complete updated query; retain exact selected source/span identity independently from updater-input controls.

**PASS_WHOLE_PHRASE_QUERY_UPDATE:** 864/864 answers, paths, intermediate queries/canonical encodings and span bounds; 72 complete families and 864 identical reloads. Phrases occur in first, middle and both update positions. Truncating to the first word loses all 576 multiword cases. Actual baseline stale payloads lose all 288 active-source cases while preserving baseline/inactive cases. The prior span artifact and every learned parameter remain unchanged; zero fits or retries.

The new Full path preserves all 5,440 earlier full traces and completion decisions, including 512 typed unresolved cases. Separately, all 19,008 previous span-control responses and older legacy controls replay identically. Preserve all 54 prior sealed roots / 1,334 files, three predecessor source/binary versions and original dirty checkouts; the new report adds 27 sealed files.

Validation: one optimized offline Rust build, 47 focused tests, one actual sealed report, format/claim/diff checks. Model/build/evaluation 298,872/1,800,000 ms. Necessary local parent time/storage extensions were recorded before use under standing authorization; no paid compute or cleanup. CI status names are compatibility acknowledgements, not local test evidence.

Scope remains authored sequential phrase references with familiar grammar and finite word roles. ExactIdentity equals Full; no new metric advantage, general prose or independent final holdout is claimed. Retain normal model15baec48, no promotion. Current state records the next unchanged-artifact overlapping-occurrence diagnostic.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

