# #1751 — Train geometric cue credit on complete native endpoint actions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T19:32:01Z
- **Updated:** 2026-10-09T13:17:21Z
- **Closed:** 2026-10-07T00:07:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1751
- **Merged:** not merged
- **Merge commit:** 5c93c923148b18f000c3d79e2f6c1e8362a722db
- **Base / Head:** codex/geometric-endpoint-calibration-20261005 / UOR-Foundation:codex/geometric-cue-calibration-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The selected endpoint calibration improves response completion but leaves payload retrieval unchanged. This change gives only the existing geometric cue table learning credit from the full native Copy/Period/Stop action distribution, so the fitter can optimize source ranking without removing calibrated terminal behavior.

Adds an exact signed-Q4 warm start and current-parent binding validation. The target enters after the authoritative native read; all bank candidates remain admitted. Query/assertion encoders, parent Copy, prefix, endpoint tables and hard routing are frozen. The surrogate uses actual consumed angular addresses and masks structurally absent cues.

Validation at 19bef97b3edeb6f38c007cc122f687d11a686bc4: independent source review found no actionable flaw. Three new fixtures cover warm-start/reload/foreign-parent rejection, complete endpoint likelihood and gradient isolation, and two-source differential credit with a legal native quarter-step score-margin change. Scoped release pod library tests passed 8/8, including all three new fixtures, with Rust/Cargo build-source hashes and formatting verified. The earlier attempt failed because a fixture incorrectly treated a legitimate leading-space token alias as unsupported; the corrected test asserts absence of the unknown token before rejection. Both attempts are preserved. No fit or language improvement is claimed.

Stacked on #1748; parent delivery and final-head validation remain outstanding. References #1552 and #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6001664835 — 2026-10-05T19:37:18Z — Casey-allard

Exact-head executed checks at19bef97b3edeb6f38c007cc122f687d11a686bc4: pod Linux x86_64, CPU16/CUDA hidden, 1295 tracked Rust/Cargo/.cargo source hashes verified, rustfmt --check PASS, cargo test --offline --release -p uor-r4-training --lib cue_ -- --nocapture:8 passed/0failed (includes all3newtests). Competing-record test gives opposite cue gradients and exact +2^22 Q24 target-minus-distractor margin from legal +.25 cue perturbation, with honest prefix/endpoint rebind and unchanged candidate inventory. First attempt atda8d failed one test because token7 is a legitimate leading-space alias; corrected fixture verifies token2 absent before rejection. Retained both attempts; this is an instrument correction, not a model negative. Independent source/adversarial review found no actionable flaw. No fit or learned attention/transfer/chat claim. Receipts durable at /workspace/uor-r4/codex/cue-library-checks-20261005/. Driver proceeds separately on codex/geometric-cue-fit-20261005; this draft keeps exact validated head.

### comment 6027851945 — 2026-10-07T00:07:22Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6001664835, 6027851945_

