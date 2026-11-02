# ORDER: repair BVG-K decision parity and batch atomicity, deepseek-12

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4b1d71121`.

## Goal

Repair the exact BVG-K decision and batch-apply slice so malformed records follow CUDA member and apply semantics, every committed member is validated before mutation, and a refused batch leaves the model, chooser and epoch unchanged.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-12/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, corpus/GPU run or language claim. Cursor persistence and replay remain disabled.

## Done when

- Member selection follows `discrete_bvgk_choose`: status, eligibility, inventory and an improving active lane determine membership; a malformed record kind is not silently removed from the decision.
- Batch apply preserves CUDA status order: member kind `14`, stale member epoch `3`, mode `9`, lane/inventory `4`, non-improving or epoch exhaustion `5`, duplicate index `15`, and empty commit `16`.
- Every committed lane is checked against the active inventory (`0..=2` for Gate and `0..=15` for Route/Table) before any model write.
- All committed virtual moves are staged on a clone; model and chooser state change together only after every move succeeds, and the epoch advances exactly once.
- Controls cover malformed kind, stale decision/member epoch, wrong mode, bad lane/index, non-improving member, duplicate index, empty commit, epoch `u64::MAX`, a valid batch still applying once, and byte/state-identical refusal with a bad later member.
- Full release tests, clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass.
- A fresh exact-head adversarial review approves the repaired change before merge-queue delivery.
