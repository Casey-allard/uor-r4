# #282 — fix(certify): C row as deterministic stride subsample within the #278 budget

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T15:32:17Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-07-29T15:42:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/282
- **Merged:** 2026-07-29T15:42:55Z
- **Merge commit:** 81ca8e56a3fe12a93d1eb934b964ef007dad09b3
- **Base / Head:** main / UOR-Foundation:issue-244-certify-rows
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Replaces the C-row full-eval-or-discard behavior with a deterministic ~1,000-position stride subsample evaluated inside the #278 wall-clock budget. A budget-truncated prefix samples only the earliest held-out positions (biased) and throws away its partial counts; the stride sample is position-uniform, completes at the measured ~1.2–1.6 s/position rate, and records a **real row** labeled with its sample size. Budget default raised 600s→1800s to fit the sample; env overrides unchanged; #279's progress lines and probe budget kept as-is.

This is the exact construction that produced the first honest C measurement (0.0% top1/agreement, n=1002 — #244 amendment), which exposed the readout defect now tracked in #280.

Note: #279 already upstreamed the A-single/query-beam rows — this PR deliberately adds nothing there; it only converts the C-stage skip into a bounded measurement.

## Checks

cargo test --workspace --offline (71 suites ok), clippy all-targets all-features -D warnings clean, fmt clean. Certify-side only; no runtime-kernel change.

Refs #244 (decision + amendment), #278 (magnitude correction in comment 5119492653), #280.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5120148943 — 2026-07-29T15:43:25Z — Casey-allard

**Attribution correction + an unplanned reproduction result.**

The PR body's claim that "#279 already upstreamed the A-single/query-beam rows" is wrong. The rows landed in `1f5088b` (PR #261, merged 2026-07-28 10:01 EDT) — before #279's branch was cut. #279's diff is exactly its three stated files. The misattribution came from this session grepping a stale working tree and concluding the rows had never merged; they had — in a PR authored by this same session the previous morning, lost across a context boundary. The parallel session flagged the error and the git history confirms their account.

The accident produced something useful, though: last night this session **independently reimplemented** the rows (different code: memberships precomputed vs computed in-eval) and ran them; today's #279 verification run exercised main's #261 implementation on the same artifacts. The outputs agree **to every printed digit**:

```
A-single (mul-free, single-key store):        top1 27.0% | agreement 30.5% | WB 10.2304 | 134733 keys   (both)
A-single + query-beam (read-time expansion):  top1 28.2% | agreement 31.9% | WB  9.8742 | 134733 keys   (both)
```

That is an implementation-independent reproduction of the #244 decision rows — stronger evidence than the same binary run twice. Recorded here so the #244 decision and #281 implementation can cite it.

Also confirmed against `1f5088b`: this PR's final diff (23+/9−, C-stage subsample + budget default only) removed nothing from the #261 rows.


---
_Comment/review ids in this file: 5120148943_

