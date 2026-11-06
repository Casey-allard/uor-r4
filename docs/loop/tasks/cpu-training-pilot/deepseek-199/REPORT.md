# REPORT: cpu-training-pilot, deepseek-199

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `67f7d35c9`.

## Status

Complete. This records and releases the merged deepseek-198 result. The
combined configuration remains **REJECT** while its content signal is retained
as positive and termination-limited.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment's only
combined-bar failure was premature stopping: 85/104 replies against a maximum
of four. Its content mechanism passed every body and exact comparison.

## What changed

- Recorded PR #3755, reviewed head `fb670e5f7`, protected merge `67f7d35c9`
  and fresh-main verification in deepseek-198.
- Removed the completed `deepseek/full-sequence-suffix` source claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- exact-head review: counted `uor-review v1` APPROVE;
- reviewed head:
  `fb670e5f78134ae66e57087ea0265378a3b23f99`;
- protected merge:
  `67f7d35c902c1caa788c140243922f85ae9d043b`;
- all required PR checks: pass;
- fresh-main content identity: all 15 changed files;
- fresh-main CPU package: **375 passed**, 4 expected ignores;
- source hashes:
  `2cd67076025a66801b7fc141b7bd68094b83fad796a38f403928ce307a31e9c5`,
  `7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37`,
  and
  `c03976fd9d8079a8323d2e7fd2ba88bf593f846a4fe0de084bf5cc0b1972422f`;
- compact evidence manifest: pass;
- pre-registration: FIT and complete.

## Checks run at the head

- protected `loop queue` at the reviewed head;
- merge-queue CI;
- `loop close --verify`: state, squash, content, all 11 gates, worktree and
  branch cleanup;
- fresh-main CPU tests and source/evidence identity;
- release report and claim checks before delivery.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

- termination aligned to the full-sequence content trajectory;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale and serving track.

## Plain summary

The full-sequence content result is merged and verified on fresh main. This
unit releases its source claim while preserving the combined rejection and
the positive content finding.
