# #1342 — Correct observed-token session boundaries and plan contextual language roles

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T23:59:03Z
- **Updated:** 2026-10-09T13:17:45Z
- **Closed:** 2026-09-22T02:27:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1342
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/observed-text-session
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The observed-token session returned correct symbolic answers but accepted a gold goal at its wrapper, used placeholder artifact bindings, and could not safely restore completed EOS frames. Its reused final panel and BPE fragments were also described too broadly. This change makes `ObservedTextRuntime` the loaded, observed-question execution boundary, validates exact capture/phase/history/EOS state with typed errors, and preserves the useful chain-following result with complete evidence.

The corrected exposed replay retains 24/24 development and 24/24 primary regression answers, versus 20/24 membership continuation, 18/24 maximum-two-read and 0/24 NoRead. All 120 arm/request rows, 470 row transitions and 590 independently checked checkpoint continuations are retained. The source edit changes the actual selected segment path and answer; removing the terminal fact after two redirects returns Unresolved. Original reports and executable remain preserved.

The principal review corrects the historical claims and synchronizes active documentation, roadmap and evidence. The next brief targets learned contextual roles and exact spans from readable text with overlapping vocabulary, using ordered H4 and a competent contextual comparator through the same runtime. This remains a supplied symbolic-token component, not general language or a whole-path efficiency result.

Validation: eight focused library tests; 21 touched-runner tests (one explicitly ignored); offline Rust build and actual corrected generation; independent source/artifact/seal, event, span and every-phase restore audit; formatting, capability wording and document-link checks. CI's historical compatibility acknowledgements are not test results. Complete resource accounting and safe compiler-cache cleanup are recorded in the principal checks receipt.

References #973, #820, #1139, #962, #963, #964.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5770351179 — 2026-09-22T02:27:21Z — Casey-allard

Superseded by corrected PR #1343 at `6606d989cc5412671ec3b65e5f6fa3656f170e70`, now in the protected merge queue. The actual corrected #1342 head `f5919d8e8430f3c676f4c5b0509f729ca68d9e75` is an ancestor of that branch through merge commit `7bc8a3d4`; no parent paths are deleted. This restores the nine inherited review/evidence/handoff files dropped by the descendant rebase and preserves the corrected runtime adapter. Source and independently executed checks are documented in #1343.

Closing this conflicting duplicate delivery path as superseded, not claiming this PR itself merged. All programme/component issues remain open with their scoped outstanding work. The original reports, source and executables are preserved.



---
_Comment/review ids in this file: 5770351179_

