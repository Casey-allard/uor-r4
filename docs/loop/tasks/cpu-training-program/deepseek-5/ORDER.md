# ORDER: restore required preparation-sheet blocks, deepseek-5

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `374c7bdd3`.

## Goal

Restore the D22-required block shape on the already-merged Kimi CUDA performance sheet so the repository-wide harness has no error. Preserve every existing claim, pin, rejected attempt, FIT record and measurement.

## Scope

- `docs/harness/prep/uor-cuda-inverted-file-performance-2026-11-01.md`
- `docs/loop/tasks/cpu-training-program/deepseek-5/**`
- `docs/harness/claims.json`

No code, serving, GPU run or new performance claim.

## Done when

- `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost` and `Minimum test power` are present and summarize only facts already in the sheet.
- Existing FIT heads, rejected run roots and measurement conclusions remain unchanged.
- `uor-harness check` has zero errors; `roadmap check`, formatting and diff checks pass.
- A fresh exact-head review approves the documentation-only repair.
