# REPORT: cpu-training-pilot, deepseek-203

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d2313599a`.

## Status

Complete. This records and releases the merged deepseek-202 result. The
fixed strict-majority whole-continuation mode remains a non-vacuous
**REJECT**.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment emitted 187
whole closures per artifact, but tied tokenwise content at 17/17 exact replies
and made 71/64 premature stops.

## What changed

- Recorded PR #3759, reviewed head `42536cb25`, protected merge `d2313599a`
  and fresh-main verification in deepseek-202.
- Removed the completed `deepseek/continuation-closure` and
  `deepseek/continuation-closure-release` claims.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- exact-head review: counted `uor-review v1` APPROVE;
- reviewed head:
  `42536cb25443336e1cd7b057486be262da62acf3`;
- protected merge:
  `d2313599a94f84e57e40d2fa9c72d1cf38d07fb7`;
- all required PR checks: pass;
- fresh-main content identity: all 14 changed files;
- fresh-main CPU package: **389 passed**, 4 expected ignores;
- source hashes:
  `8c1b499b5b1db7d5236f89b9278d33ec29be0bd179943e0eb8a7764dfe4f31c4`
  and
  `fe6113182a2aae309ed4d6174f3ddde2bd873a4125f90655cc1cbfc4b0399c0c`;
- compact evidence manifest: pass;
- pre-registration: FIT, GO and complete.

## Checks run at the head

- protected `loop queue` at the reviewed head;
- merge-queue CI;
- fresh-main CPU tests, source hashes and compact evidence verification;
- release report and claim checks before delivery.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

- termination conditioned on a new continuation or geometric state signal;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale, export and serving track.

## Plain summary

The whole-continuation result is merged and verified on fresh main. It is
preserved as a non-vacuous reject while the positive content selector and the
search for a new termination state remain active.
