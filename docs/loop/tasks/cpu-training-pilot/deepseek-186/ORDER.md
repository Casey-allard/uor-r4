# ORDER: cpu-training-pilot, deepseek-186 (release table-anchor isolation)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `5e7d5c681`.

## Goal

Record the protected merge and fresh-main verification of deepseek-185, then
release the completed `deepseek/table-anchor-isolation` claim.

## Scope

- PR #3739 reviewed head `c5e26e828`;
- protected merge `5e7d5c681`;
- fresh-main CPU tests, strict Clippy, formatting, report/FIT/evidence checks
  and evidence identities;
- deepseek-185 delivery record and the stale claim.

No model, serving, CUDA, float-training, exporter or Claude-owned source
changes.

## Done when

1. deepseek-185 records PR #3739, exact review head, protected merge and
   fresh-main checks.
2. The completed table-anchor claim is absent.
3. Reviewed bookkeeping PR, protected merge and fresh-main verification finish.

## Stop rule

Stop on merge, source, evidence or claim drift.
