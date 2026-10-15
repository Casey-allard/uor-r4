# #223 — fix(server): enable seamless fallback execution when R4G1 abstains on novel prompts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T12:59:21Z
- **Updated:** 2026-07-26T13:20:51Z
- **Closed:** 2026-07-26T13:20:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/223
- **Merged:** 2026-07-26T13:20:51Z
- **Merge commit:** 290c55c6daeb4a688d930288eb4fc0f4d9f796b8
- **Base / Head:** main / UOR-Foundation:fix/r4g1-abstain-fallback-and-routing-speed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Fixes two issues identified during interactive client testing on out-of-distribution prompts:

1. **Seamless Fallback on R4G1 Abstention**:
   When R4G1 encounters an out-of-distribution (Novel) prompt, it safely abstains (`abstained = true`) per the D4 manifest policy. Previously, the API endpoint checked `if final_response_text.is_empty() && !r4g1_abstained`, which prevented downstream fallback engines (`transformerless-fallback` and `teacher-oracle-fallback`) from running, returning `"Manifold resonance too sparse for synthesis."`.
   - **Fix**: Removed `&& !r4g1_abstained` so that abstaining on a novel prompt seamlessly falls back to local generation.

2. **Phase-Field Routing Prompt Truncation**:
   - Truncated phase-field manifold routing evolution prompts to 512 bytes for sub-millisecond autotuning routing passes.

### Verification
- Tested prompt `This PR adds a minimalist terminal client...` against local release server: completed in ~30s with `teacher-oracle-fallback` response text.
- All workspace quality gates (`fmt`, `clippy`, `check`) passed clean.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5083583983 — 2026-07-26T13:01:57Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5083598866 — 2026-07-26T13:06:20Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved. Merged `origin/main` into this branch and kept all of our fix changes in `src/server.rs` (routing prompt truncation to 512 bytes and removal of the `r4g1_abstained` guard on the fallback conditions). `cargo check` and `cargo fmt --check` pass clean. Merge commit: `9ebca76`.


---
_Comment/review ids in this file: 5083583983, 5083598866_

