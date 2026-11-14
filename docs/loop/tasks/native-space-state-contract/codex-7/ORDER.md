# ORDER: native-space-state-contract, codex-7

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-6](../codex-6/ORDER.md). Reviewed head: `7951c5326`.

## Why this repair exists

The fresh exact-head review found two concrete blockers:

1. An accepted model step assigns the candidate gate before checking whether the epoch can
   increment. A valid serialized model at `u64::MAX` can therefore return an epoch-overflow
   error after mutating its gate.
2. The merge-forward reintroduced `native-space-cycle` into the architect skill pointer after
   final `main` deliberately removed that edge to break the dependency cycle.

The aggregate loss also uses unchecked `u64` addition even though the admitted batch bound can
make the mathematical sum exceed `u64`.

## Goal

Make the shared model transition atomic, reject an unrepresentable aggregate loss before any
state change, consistently enforce the batch and Q31 domains in the Rust model step, and restore
the final `main` skill dependency.

## Files in scope

- `crates/uor-r4-discrete/src/native_vector.rs`
- `docs/harness/skills/uor-research-architect/SKILL.md`
- `docs/loop/tasks/native-space-state-contract/codex-5/PICKUP.md`
- `docs/loop/tasks/native-space-state-contract/codex-5/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-6/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-7/**`

## Size

About 40 to 120 changed lines.

## Done when

1. Every error path in `step_from_records` leaves the model byte-identical.
2. Aggregate losses use checked accumulation and conversion before acceptance or mutation.
3. The shared Rust model step rejects batches outside the admitted bound and values or targets
   outside the closed Q31 domain.
4. A serialized model at `u64::MAX` returns an epoch-overflow error without changing its gate.
5. The architect skill pointer is byte-identical to fresh `main`.
6. Focused CPU tests, CUDA parity, the discrete library suite, formatting, Clippy, the harness
   package, `loop pr-check --local`, the report check and `git diff --check` pass.

## Stop

- Do not change Q31 multiplication, rounding, shift or saturation.
- Do not add serving code in this repair.
- Do not weaken or remove the existing parity tests.
- STOP and report when both blockers and the checked-loss defect are closed.
