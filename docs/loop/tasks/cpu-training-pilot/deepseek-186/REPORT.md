# REPORT: cpu-training-pilot, deepseek-186

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `5e7d5c681`.

## Status

Complete. This records and releases the merged deepseek-185 table-anchor
isolation.

## What did not work

Nothing in this bookkeeping unit. Deepseek-185 remains an instrument result,
not a trained-model or language result; hybrid tables remain in-memory only.

## What changed

- Recorded PR #3739, reviewed head `c5e26e828`, protected merge `5e7d5c681`
  and fresh-main checks in deepseek-185.
- Removed the completed `deepseek/table-anchor-isolation` claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `c5e26e8286701d9a48d9b2a6f3764d2239f57ba0`;
- protected merge:
  `5e7d5c681534be2ba41a050aadf642488d8f1543`;
- fresh-main CPU library: **351 passed**, 4 expected ignores;
- fresh-main CPU binaries: all passed;
- fresh-main scoped strict Clippy, formatting, diff, report, FIT/evidence,
  D23/D32 ratchets and claim wording: passed.

## Delivery

PR #3740 was reviewed at exact head
`4ee7e12314b1908c11acc1cbc506d54a7ab601bf`. All required checks passed and
the protected queue merged it as
`63ce79e85ed5f97d2860f629bb7369036f8a0935`.

Final detached `origin/main` at that merge passed the full CPU package
(**351 library tests passed**, 4 expected ignores, all binary tests), scoped
strict Clippy, formatting, diff, report/FIT evidence and absence of the
original work claim.

## Checks run at the head

- exact-head review before queue;
- required CI checks for PR #3739;
- protected `loop queue` merge with the reviewed head pinned;
- clean detached `origin/main` CPU and evidence verification;
- release report, FIT/prep, scope and claim checks.

## Size

This unit changes only the deepseek-185 delivery record, this release record
and the completed claim.

## Skills used

No system-provided Codex skill applied. Repository claim, review, evidence and
protected-delivery procedures were used.

## Still open

The next model representation must preserve learned token input identity while
representing writer address rows separately, or train root-addressed writer
rows under an explicit input-code preservation objective. No hybrid table is
exportable.

## Plain summary

The table-anchor attribution instrument is merged and verified on main. It
shows that the shared table's input-code overwrite caused the decoding loss,
while direct writer-row anchoring did not. This PR records delivery and
releases the completed claim.
