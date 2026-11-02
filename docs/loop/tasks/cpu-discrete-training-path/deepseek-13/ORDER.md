# ORDER: deterministic CPU discrete checkpoint and restart, deepseek-13

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `245f47089`.

## Goal

Persist the complete in-memory discrete CPU model with a CUDA-compatible corpus/coordinate cursor, then prove that a fresh restored applier makes the same next decision, applies the same update and reproduces the same checkpoint identity as an uninterrupted run.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-13/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`
- `docs/harness/claims.json`

No serving edit, corpus/GPU run or language claim.

## Done when

- A versioned CPU checkpoint carries the exact source-artifact digest, CPU source geometry, schedule identity, complete Route/Flock/Relation model state, current epoch, model hash, derived chooser state/hash, substrate and qualification labels.
- The cursor uses the reference `Cursor` and `BlockCursor` fields and validates corpus pass, next target, next coordinate, block presence, family cursors, accepted updates and committed moves by derivation rather than trust.
- Canonical and interleaved schedules are supported; carrier schedules are explicitly refused until their CUDA target selection is represented.
- A committed BVG-K batch advances both model epoch and accepted updates once. A no-commit batch advances completed batches without changing accepted updates, model epoch or model bytes.
- Checkpoint/restore revalidates all fields, hashes and inventory, rederives the chooser and refuses stale or corrupt state without partial construction.
- Tests prove uninterrupted-versus-restored equality for next proposals, decisions, receipts, readings, cursor and canonical checkpoint bytes, including a K=3 truncated-family boundary.
- Full release tests, clippy, formatting, ratchets, local PR check and all loop gates pass.
- A fresh exact-head adversarial review approves the change before merge-queue delivery.
