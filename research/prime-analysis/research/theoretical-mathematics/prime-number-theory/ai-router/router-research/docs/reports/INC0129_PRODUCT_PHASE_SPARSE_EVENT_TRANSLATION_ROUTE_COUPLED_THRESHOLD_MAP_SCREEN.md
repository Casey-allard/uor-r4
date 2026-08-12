# INC0129 Product Phase Sparse Event Translation Route-Coupled Threshold Map Screen

## Summary
- `INC-0129` closed negative/explanatory at screen stage.
- The train-gate-prune coupling is genuinely downstream-live.
- No threshold window was viable for carry-forward.

## Key Read
- `0.010` is the best quality-preserving point:
  - `keep_frac=0.992`
  - `top1=0.0448`
  - `cand_frac=0.187042`
  - but runtime regresses versus uncoupled near-hard
- `0.015` through `0.022` show the expected monotone pattern:
  - more pruning
  - smaller candidate fraction
  - progressively worse top-1

## Decision
- Keep the current translated sparse-event references unchanged.
- Close train-gate-prune as a carry-forward translated sparse-event surface.
- Move next to a softer route-coupled mechanism that can change downstream
  behavior without deleting train items.
