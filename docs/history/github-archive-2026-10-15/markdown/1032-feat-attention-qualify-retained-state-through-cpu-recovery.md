# #1032 — feat(attention): qualify retained state through CPU recovery

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T08:07:31Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-01T08:08:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1032
- **Merged:** 2026-09-01T08:08:41Z
- **Merge commit:** bf2d8d86ef07d610f34aee6b42605d911012fe84
- **Base / Head:** main / UOR-Foundation:issue-973-full-decoder-block
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add the independently frozen `R4GroupAddressedRetentionDecoderV1` construction path and preserve its terminal deterministic-MPS envelopes;
- add the resource-only `R4GroupAddressedRetentionDecoderV1CpuRecovery`, enforcing deterministic Apple Accelerate CPU execution with one process, sequential arms, and four configured PyTorch/Accelerate threads before the create-once marker;
- preserve the exact CPU started/result envelopes and bind their CIDs;
- update the living project documentation, roadmap, claim vocabulary, #954 blocker record, and trainer guide to the observed result; and
- make measured backend/thread/worker selection normative for substantial offline jobs, using a small predeclared set of plausible plans rather than a one-core or maximum-worker default.

## Observed result

- result CID: `blake3:68355ad2f61d02dc73dbf22de4c24834815a23069ed5735630dc365081cf91db`;
- all 512 optimizer steps completed in `438.117083 s` under the 900-second wall;
- causality, incremental/full parity, gradients, equal work, replay, memory, and zero-heldout-read audits passed;
- on the disjoint construction-validation partition, state-off lost `0.967227` nats and 182 top-1 decisions, providing bounded causal retained-attention component evidence;
- the exact complete-decoder recipe did not satisfy the frozen full-decoder generalization criterion: validation CE moved `8.371911 -> 8.976155`, so terminal verdict is `RETAINED_DECODER_FAIL`;
- formal H4 specificity is `NOT_EVALUATED`; diagnostically, scrambled transport CE was `0.033049` nats better while exact H4 led top-1 by four decisions, below threshold.

This does not authorize heldout reveal, coherent generation, reasoning, #954, exact lowering, or release. The next independently frozen mechanism should retain the bounded attention component as a hypothesis inside a data-supported language path with an ordinary matched non-geometric decoder.

## Verification

- 57 focused group-retention/decoder/recovery tests: PASS
- Python compile-all: PASS
- real CLI command/help exposure: PASS
- raw started/result files byte-identical to the official create-once root: PASS
- independent source/artifact/CID/result audit: PASS
- `cargo fmt --check`: PASS
- `python3 scripts/check_claim_wording.py`: PASS
- `git diff --check`: PASS

The automatic PR/merge-queue checks are ruleset-transport acknowledgements and ran no QA. The decision-bearing verification for this change is the focused local suite above. Broad workspace and release certification remain dormant and `NOT_RUN` under the current project policy.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

