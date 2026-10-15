# #261 — feat(certify): A-single and query-beam rows — the #244 store-decision matrix

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T13:26:11Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-07-28T14:07:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/261
- **Merged:** 2026-07-28T14:07:55Z
- **Merge commit:** 1f5088bb7b9691aeced5df4cfcf2e533bb6fb490
- **Base / Head:** main / UOR-Foundation:issue-244-store-decision
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #244 (does not close it — the decision needs the measurement).

## What

Two additive `certify` rows completing the store-decision matrix, per the plan comment on the issue:

- **A-single**: the shipped mul-free assignment codes evaluated over a **single-key** store (no `add_evidence_multi` beam materialization) — isolates what the 3.71× extra keys actually buy.
- **A-single + query-beam**: the identical multi-membership beam applied at **read time** over that single-key store — per-position candidate keys from `assign_memberships_plain`, level distributions merged via the new unit-tested `merged_beam_distribution` before canonical argmax and backoff scoring.

Measurement-only: no store construction behavior changes in this PR. After the next legacy `certify` run, the four-way comparison (A-binary / A-f32 / A-single / A-single+query-beam) gets posted on #244 and the decision (single-key ship, query-beam ship, or keep-and-document) lands as its own follow-up.

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline). Certifier-side only; runtime untouched.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

