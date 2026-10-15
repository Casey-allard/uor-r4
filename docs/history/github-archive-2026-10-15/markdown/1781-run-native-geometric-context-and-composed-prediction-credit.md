# #1781 — Run native geometric context and composed prediction credit on CUDA

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T03:54:40Z
- **Updated:** 2026-10-09T13:17:40Z
- **Closed:** 2026-10-07T00:07:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1781
- **Merged:** not merged
- **Merge commit:** 8714b1fccc108c091e8c6e5393c073995f5c85c3
- **Base / Head:** codex/geometric-composed-state-credit-20261006 / UOR-Foundation:codex/native-geometric-cuda-learning-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native geometric learner's custom context and composed-credit operations previously executed on CPU, even on rented GPU pods. This change adds CUDA context forward/backward over all 120 H4 choices, device Q4 operations, GPU composed cue/potential/NoRead and transport credit, and an explicit no-fallback real-data parity probe. Native integer serving and the normalizer are unchanged.

Validation at numerical head `1d52e3f51abddcd68c79ffce950d9cb0ea9684da`, Linux x86_64 / one RTX4090: 91 release checks passed with CUDA required. Actual twelve prediction positions have matching native traces and finite nonzero CPU/CUDA gradients for all nine families; four program-executed independent reload replies match. Independent thin-artifact review recounts tests/gradients/direct outputs; it did not replay raw reload files. Matched small composed work measures 1.20x and backward 1.92x; forward including host staging is slower. No full-fit throughput or energy claim.

This is zero-update backend validation. Existing fit modes remain CPU until explicit device/optimizer integration; no Generate, fresh evaluation or learned capability improvement is claimed. Next: native full-vocabulary Generate with joint resident CUDA context/decoder fit, fresh native snapshot/rebind after every update, and an actual one-update/export check before a larger fit. Failed bring-up attempts are retained.

Final head `e17ebb91` adds evidence/docs and verified preservation only after the numerical head. Claim-wording and diff checks pass. See `docs/integration/geometric-native-cuda-learning-2026-10-06.md` and its bound evidence JSON. References #820.

All 66 durable files match the canonical-store SHA256 inventory. Thin receipts are MD5-verified in iCloud. The lease is released and the own pod deleted.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6027855267 — 2026-10-07T00:07:39Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6027855267_

