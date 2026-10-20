# #251 — fix(certify): build R4G1 evaluation context from consecutive story-bounded history (#237)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T04:01:47Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-07-28T04:32:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/251
- **Merged:** 2026-07-28T04:32:52Z
- **Merge commit:** 643107eff367ef025a865ce5defcc99fc2a52318
- **Base / Head:** main / UOR-Foundation:issue-237-eval-context-lags
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #237.

## What

The certify C-section built its evaluation context by using per-slot **vector rotation offsets** (`derive_rotations()` = j·17 mod D = `[0,17,34,...,136]` — dimension-space rotation amounts for `bundle_plain`) as **time lags** into the corpus. Three compounding defects: stride-17 token sampling instead of the last consecutive window; reversed order with the true most-recent token placed first (while `predict_distribution` reads the *last* element as most recent) plus a phantom trailing zero in the unwritten 9th slot; and no story-boundary check. The R4G1 row has never measured coherent context — a plausible standalone explanation for the historical all-zero row.

- New `pub fn eval_context(c, i) -> Vec<u32>`: last `WINDOW` tokens, oldest → newest ending at `input[i]`, story-bounded — built on `runtime::history_token` (now `pub`; visibility-only change to the canonical helper observation already uses), so evaluation and observation share one context definition.
- The #232 readiness probe and the full evaluation both switch to it. The rotation table keeps its legitimate uses (bundling, equality witnesses).
- 4 tests: chronological window, story-boundary truncation, position-0 edge, regression guard against all three old defects.

## Interpretation note

After this merges, the C row measures coherent context for the first time. Expect its numbers to change materially from the historical 0.0% row; the #232 SKIPPED marker should now fire only for genuinely untrained/unscored graphs rather than for scrambled input.

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline); no runtime-kernel changes (`history_token` visibility only — P-4 scan unaffected).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

