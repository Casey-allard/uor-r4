# #2221 — Starling FD tests robust on x86: conditioned steps, selection-stable address step; tolerances unchanged

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T00:32:27Z
- **Updated:** 2026-10-12T00:34:51Z
- **Closed:** 2026-10-12T00:33:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2221
- **Merged:** 2026-10-12T00:33:30Z
- **Merge commit:** b72817744914f9b4f6378726812479b8ee981628
- **Base / Head:** main / UOR-Foundation:claude/starling-test-robust
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** none. This test-robustness fix is not a result, so it does not count. Headline unchanged.

**Why.** Two starling finite-difference tests failed on an x86 GPU pod, at origin/main too, while passing on the ARM laptop:
- `starling_read_gradient_matches_finite_differences`: analytic 0.2447 against numeric 0.2527, tolerance 2e-2.
- `the_starling_content_gate_scores_kept_sources_by_query_and_key`: analytic 0.02275 against numeric 0.02251, tolerance 5e-3.

**Cause.** The central differences used steps of 1e-4 (address) and 1e-3 (content gate) on a loss evaluated in f32. The measured change (about 2e-5) was the same size as f32 rounding of a loss of about 10, so the numeric side was rounding-dominated, and its error depended on the platform's summation order.

**Fix. Tolerances are unchanged and nothing is skipped.**
- **`check_gradient_with(…, epsilon)`.** `check_gradient` keeps 1e-3 and delegates to it.
- **Content gate: step 1e-2.** The selection reads only the address maps, so q and k move the loss smoothly, and the tanh gate is smooth.
- **Address: the largest step in {1e-2, 3e-3, 1e-3, 3e-4} whose kept sets at −ε and +ε equal the unperturbed ones,** checked through the starling record. The difference is then provably over a fixed hard selection. If every step changes the selection, the test fails.

The previous step of 1e-4 assumed the selection was fixed without checking it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6115639790 — 2026-10-12T00:32:28Z — Casey-allard

**Exact-head checks at `717644cf08455d6ac20757465f6f93626bc9ab64`, run on the x86 GPU pod's CPU** (owner direction: no laptop builds):
- `cargo test --release -p uor-r4-training --lib -- starling content_order`: **10 passed, three times in a row**, in parallel test mode. Before the fix, 2 failed on this pod at origin/main.
- Tolerances are unchanged (2e-2 and 5e-3), so the tests' sensitivity to a wrong analytic gradient is unchanged.
- Not run on the ARM laptop, by owner direction. The change only enlarges finite-difference steps under stated smoothness conditions, and adds a selection-stability check.

### review 5485691607 — 2026-10-12T00:32:30Z — Casey-allard — COMMENTED

**Self-review at exact head `717644cf08455d6ac20757465f6f93626bc9ab64`.** Test-only change. Tolerances untouched. No assertion removed. The address step is now validated rather than assumed. APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6115639790, 5485691607_

