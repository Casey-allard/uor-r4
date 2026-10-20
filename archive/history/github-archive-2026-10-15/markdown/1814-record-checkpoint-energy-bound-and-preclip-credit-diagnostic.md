# #1814 — Record checkpoint energy bound and preclip credit diagnostic

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T00:30:04Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-10-07T00:31:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1814
- **Merged:** 2026-10-07T00:31:10Z
- **Merge commit:** f046e221781740c9db422a9fce77fbd6fdc2ed15
- **Base / Head:** main / UOR-Foundation:codex/prediction-energy-admission-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained step-128 checkpoint cannot produce a correct first-token argmax through a frozen bridge-only intervention: its Generate score bound is below a wrong Copy atom in every one of the 512 saved entry cases. Update current state to replace the previously proposed utility run with a zero-update diagnostic of preclip training credit.

Preserve the existing joint learner and its bounded improvements. Record that 351 entry cases saturate the Copy score clamp, distinguish direct score credit from other gradient paths, and keep the prospective margin loss explicitly unmeasured. No model, runtime, or training behavior changes.

Validation: independently verified all 512 saved row hashes, exported coefficient bound, and actual integer exponential lookup; inspected the current alias-loss clamp and existing trainable parameter families. Claim-wording and git diff checks pass. Cargo and CUDA checks are not applicable to this documentation-only change. References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6028133601 — 2026-10-07T00:30:17Z — Casey-allard

Exact-head self-review: 2f1506f5d300c74e96fa0612143ba8e671546d88.

Only docs/integration/current-state.md changes. The exported coefficient sum bounds all retained Generate scores at 26 << 20 Q24; this is an upper bound, not an attainable maximum. All 512 saved first-position row hashes were independently verified, with Generate-only targets and a wrong Copy atom exceeding the bound. The actual monotone integer exp lookup establishes a strict mass gap. The claim is restricted to frozen Copy/Generate parameters on this checkpoint.

Source review confirms the current clamp blocks direct score credit on the 351 out-of-range highest Copy atoms, while other paths and shared parameters can still receive gradients. Existing joint training and its Copy improvements are retained; no new learner is claimed. The preclip margin diagnostic is prospective and NOT_RUN.

Checks at this head: claim-wording gate PASS; git diff --check PASS. No Rust changes; Cargo, GPU and Metal execution are NOT_APPLICABLE for this documentation PR. No language or geometry-family qualification is asserted.



---
_Comment/review ids in this file: 6028133601_

