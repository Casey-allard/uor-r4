# REPORT: cpu-training-pilot, deepseek-194

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `fa19e6219`.

## Status

Complete. This records and releases the merged deepseek-193 suffix-memory
delivery. The experiment remains **REJECT**; no mechanism or model result is
promoted.

## What did not work

Nothing failed in this bookkeeping unit. Fresh-main verification found that
deepseek-193's report called all 370 selected tests passed when the actual
result was 366 passed and four expected ignores. This unit corrects that count.

The experiment itself remains a decisive configuration rejection: on fresh
seed 2772 the global suffix table reached 57/86 correct body positions against
514/514 for local longest-32 and made 14/227 premature stops.

## What changed

- Recorded PR #3748, reviewed head `d7d36773a`, protected merge `fa19e6219`
  and fresh-main verification in deepseek-193.
- Corrected the fresh-main test count to 366 passed plus four expected ignores.
- Removed the completed `deepseek/copy-suffix-memory` source claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- exact-head review: counted `uor-review v1` APPROVE;
- review head:
  `d7d36773a428b7944ba8630936044b0028e7673f`;
- protected merge:
  `fa19e62192af61552333b99ba47d99e3143d11d5`;
- all required PR checks: pass;
- fresh-main content identity: all 14 changed files;
- fresh-main CPU package: **366 passed**, 4 expected ignores, all binaries;
- source hashes:
  `11a252a46e94cb15f5dc809720926f019ad9a006a380efe6916f8226a91924a1`
  and
  `dc78a4080a8a7e8ee7bac83e57e813225f6817fe9babcffdc3c085475e04997b`;
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

- a continuation mechanism that preserves local exact copy rather than
  displacing it;
- complete-reply/fresh-phrasing capability on the one-model path;
- Claude's independent exact export, parity and scale track.

## Plain summary

The suffix-memory result is merged, verified on fresh main, and accurately
recorded as a rejection. This unit releases its source claim and corrects the
test-count wording.
