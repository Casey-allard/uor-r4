# REPORT: cpu-training-pilot, deepseek-184

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `60fb4b0bd`.

## Status

Complete. This releases the merged deepseek-183 query-port training claim.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment remains a
two-seed **REJECT** and the table-anchor cost remains unresolved between the
input-code and direct writer-row effects.

## What changed

- Removed the completed `deepseek/query-port-root` claim.
- Recorded PR #3736, reviewed head `56057171e`, protected merge
  `60fb4b0bd` and fresh-main verification in deepseek-183.
- Changed no model, serving, training, CUDA, float-inference, artifact or
  Claude-owned source.

## Measurements

- merged implementation head: `56057171e`;
- protected merge: `60fb4b0bd8a08c93896fb209775117c9dc16fb20`;
- fresh-main CPU library: **347 passed**, 4 expected ignores;
- fresh-main CPU binaries: **104 passed**;
- fresh-main strict Clippy, report/evidence, rustfmt and diff checks: passed;
- fresh-main loop gates: **11 / 11 green**.

## Checks run at the head

- Claims JSON, scope, ORDER, report, policy and loop gates.
- Exact-head review and required checks before the protected merge.
- Fresh detached main at `60fb4b0bd`, including the full CPU package and
  evidence inspection.

## Size

This bookkeeping unit changes only its task records, the deepseek-183 delivery
record and the completed claim.

## Skills used

No system-provided Codex skill applied. Repository claim and delivery
procedures were used.

## Still open

The query-port objective is closed for this eight-step gain configuration.
A later representation must preserve learned token identity while adding cell
addressing, and must measure the combined input/writer effect separately.

## Plain summary

The query-port trainer and its negative result are on main and verified there.
This PR removes the stale claim and records the reviewed delivery.
