# #1098 — Qualify isolated Python runtime readiness with zero model forwards (#1096)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T12:02:19Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-09-03T12:04:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1098
- **Merged:** 2026-09-03T12:04:55Z
- **Merge commit:** 6f21fc5f4c40b9620c9fec5e95a39097f812ae73
- **Base / Head:** main / UOR-Foundation:issue-1096-isolated-runtime-readiness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#1094 stopped before Python startup because the pinned interpreter could not execute inside its declared macOS file-access policy. This change retains the existing source/runtime/model read scope and adds literal interpreter-alias reads plus metadata for exact allowed-path ancestors. A separate coordinator binds the actual worker, runtime, source, artifacts, harmless probes and independent approval before one readiness-only attempt.

The sole attempt returned **ISOLATED_RUNTIME_READY**. It completed in 2.058100333 seconds, verified Python 3.12.14/Torch 2.7.1 on the fixed Apple M1 CPU plan, denied all four harmless corpus/reference/history/results probes, and emitted ready/done with null model states and zero model loads, row/batch forwards or optimizer updates. Conservative combined peak RSS was 704,806,912 bytes. The first identity-only freeze was corrected during static review and preserved unattempted; only the second freeze was executed.

Exact raw receipts, both freezes, independent pre-execution/result/delivery reviews, source audit, claim ledger and current roadmap are retained. Original #1094 stop/binding/profile bytes remain unchanged, and the withheld directory remains mode 000. #1079's weak-control result and #1082's descriptive limits are preserved. The two original receipt directories plus public evidence/index total 339,014 bytes; no unique evidence, source cache, model or user material was deleted.

Validation was limited to the declared readiness decision, independent source/manifest/result review, three changed-Python syntax checks, claim wording, content-pinned evidence and local-link integrity, and diff whitespace. Broad QA, fitting, corpus preparation, withheld access, model comparison and replay were NOT_RUN. Protected queue statuses are transport acknowledgements, not test evidence.

This qualifies one pinned startup and the tested harmless-read restrictions. It neither identifies the precise original denial cause nor qualifies raw-text model-output preservation or general isolation. #1094 remains open for separately frozen preparation with prior-budget accounting and independent release review; #973 remains open and #954 blocked. Accepted public records and live issue state will be ingested into the local knowledge index after merge.

References #1096. Its readiness scope will be closed after protected merge and exact delivered-content verification. No #1094 comparison is part of this PR.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

