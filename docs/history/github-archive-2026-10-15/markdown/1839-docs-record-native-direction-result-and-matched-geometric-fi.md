# #1839 — docs: record native direction result and matched geometric fit decision

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T10:42:15Z
- **Updated:** 2026-10-09T13:17:42Z
- **Closed:** 2026-10-07T10:44:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1839
- **Merged:** 2026-10-07T10:44:30Z
- **Merge commit:** b5f1750156b899a68d5676ac0ee0bd15d64a8f5d
- **Base / Head:** main / UOR-Foundation:codex/native-direction-result-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The canonical plan still lists the native direction packet as the next task. Record its completed CUDA result and replace that instruction with the reviewed matched joint-fit decision.

The result changes exported geometry and native loss, but keeps 0/4 correct entries and mixed Copy/Generate effects explicit. The follow-up preserves the original training conditions and requires fresh legacy/categorical controls under one current executable because the historical fits predate CUDA dispatch changes.

Validation: claim-wording check and git diff --check pass at the PR head; numerical claims match the independently authenticated report, complete-file seal and full token-alias arithmetic audit linked on #820. Documentation only; no Rust source or runtime behavior changes.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6036245460 — 2026-10-07T10:43:05Z — Casey-allard

_edited 2026-10-07T10:43:37Z_

Exact-head self-review: `8ad7f1686535bdb0a88c15d861be8ddb421ea146`.

PASS: `python3 scripts/check_claim_wording.py`; `git diff --check`; named-file scope review (two documentation files, 8 added / 2 removed lines, direct base main).

Numerical claims were checked against report `f27608e2…`, manifest `9841cb08…` and independent audit `d6ffd26b…`: 11 exports / 385 checkpoint-file entries / 462 positions, full 4,096-token alias arithmetic, matched phase deltas at amplitude 1 and 0/4 entry correctness. Here "displacement 1" means the declared positive amplitude; the negative-amplitude control is recorded separately. No monotonic descent, geometric superiority, own-feedback or chat claim is introduced. Current-executable controls remain required because the historical training graph differs after CUDA dispatch changes.

The Rust record executable had its own source review, release CUDA-feature check/build and four focused tests, then actual CUDA execution and independent artifact audit before this documentation change. These are bound experiment results, not tests of a new runtime modification. No Rust source changes in this PR; additional Cargo/GPU tests are not applicable. READY for protected merge at this exact head.



---
_Comment/review ids in this file: 6036245460_

