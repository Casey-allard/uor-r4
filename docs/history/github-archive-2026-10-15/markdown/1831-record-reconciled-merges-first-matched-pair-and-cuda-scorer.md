# #1831 — Record reconciled merges, first matched pair and CUDA scorer evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T05:34:07Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-10-07T05:35:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1831
- **Merged:** 2026-10-07T05:35:48Z
- **Merge commit:** 639961028b4f37998bfe16e1a8fbe141dc4d023c
- **Base / Head:** main / UOR-Foundation:codex/reconciled-state
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The research state still described the reconciled integration and restart fitter as unmerged and the first raw-credit arm as running. Record the actual merged PRs, the completed first matched pair, the delivered conditional finite-map adapter and the measured CUDA Generate scorer with their exact limits.

All eighteen old stacked PRs are closed; retained code landed through #1810/#1812/#1813. #1822 and #1829 now also land directly on main. The six matched fits retain their original executable and conditions. This update makes no new mechanism, attention/chat or whole-training performance claim.

Validation: claim-wording checker and git diff --check pass. Merge receipts and changed-source equality verified for #1822/#1829; CUDA/CPU results and first-pair evidence are linked on #820 and the implementation PRs. Documentation only; no new model run. References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6031686915 — 2026-10-07T05:34:57Z — Casey-allard

Exact-head self-review at `2a333dfdc2f7498b80cc610ed23c1025b3f82733`: documentation only, one current-state file. `python3 scripts/check_claim_wording.py` PASS; `git diff --check` PASS; clean committed tree. Independent read-only evidence review PASS: refreshed all18 old stacked PR statuses and the actual merges, read final5-test CUDA log, reproduced rounded timing ratios and checked first-pair counts/times/CE against previously authenticated row receipts.

The update distinguishes the earlier failed joint fit from the active matched pairs, implementation/component parity from learned language, conditional-map implementation from its unmeasured learning usefulness, and scorer timing from whole-training cost. #1822 example and #1829 four source files were independently diffed against actual main after merging: byte equality confirmed. Healthy fit conditions remain unchanged. No new runtime code or model execution; Cargo/GPU reruns are unnecessary for this documentation-only change. Ready for protected merge at the exact head.



---
_Comment/review ids in this file: 6031686915_

