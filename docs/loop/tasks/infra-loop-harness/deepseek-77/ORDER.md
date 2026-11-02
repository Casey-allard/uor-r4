# ORDER: repair harness registry reconciliation metadata, deepseek-77

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5 (registry-aware PR-check and status fixtures; no bar, result, hold or serving change). Base: `12f2c7ef7`.

## Goal

Repair the review findings on PR #3551 without changing the already reviewed behavior.

## Scope

- `docs/loop/tasks/infra-loop-harness/deepseek-77/**`

## Work completed

- The original reconciliation is commit `4c2b99a9e`.
- The stale command literal was corrected in `7b098e108`.
- Current `main` was merged in `849105dbd`; that merge introduced no semantic change.
- A fresh reviewer approved the exact merge head and noted the active ORDER still needed a valid setup row.

## Done when

- This repair ORDER names setup row 5 for the `infra-loop-harness` build plan.
- The complete harness test suite passes.
- A fresh exact-head review approves this metadata repair.
