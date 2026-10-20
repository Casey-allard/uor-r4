# #1413 — Preserve learned parameters in explicit radial adaptation campaigns

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T03:12:52Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-09-27T03:26:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1413
- **Merged:** 2026-09-27T03:26:13Z
- **Merge commit:** 889266086881efe6b49a2c04b01cd37a679c69ae
- **Base / Head:** main / UOR-Foundation:codex/radial-transfer-delivery-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Preserve the retained learned Dot model when starting a radial-reader adaptation study. An explicit campaign transfer verifies the sealed parent, evaluator/tokenizer and complete shared tensor inventory, copies all21 shared arrays, and starts fresh Adam and sampling clocks. Child checkpoints carry an immutable receipt; resume keeps evolved weights, moments and sampling position without reopening or recopying the original parent. Legacy defaults and numerical contracts remain unchanged.

The startup example uses the same construction path. One actual zero-update observation from the retained step15,672 negative candidate completed on four fixed full256 development windows. Both readers retain identical shared arrays and finite nonzero measured gradients. Initial NLLs2.389537/2.400457 and the identical eight-token continuation are startup observations, not a fitted advantage or prose qualification. The roadmap now calls for a matched Dot-reset control before a three-arm adaptation study; no adaptation dose or fit was launched. Mainline termination work stays independently owned.

Validation at source `50c4e3a4a19c743d80f1e087c8d4272ab0c4791d`:

- Ten focused checks passed:4transfer,4retained initializer,2calibrated-checkpoint checks. The new persisted-resume fixture checks identical next updates after removing its synthetic original parent.
- Source-bound example build, rustfmt, whitespace and claim-wording checks passed. No full suite was required.
- Independent artifact audits verified all21 parent tensor hashes, exact canonical input windows, before/after array equality, prompt accounting, and all10 BLAKE3 manifest entries in the complete11-file report.
- Build/check commands822.46seconds, including60.22seconds of functional checks. Actual startup183.70seconds, supervised184.73seconds, maximum child RSS846,594,048bytes. Candidate updates0. Test profile and shared machine load supply no optimized throughput/energy evidence.

[Implementation and result](docs/integration/radial-parameter-transfer-2026-09-27.md) · [Source/output/resource receipt](docs/evidence/radial-parameter-transfer-validation-2026-09-27.json).

The all-zero test source fallback in duplicate #1412 is deliberately excluded; tests use the actual committed source identity. Queue acknowledgement statuses do not execute these checks.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

