# ORDER: native-space-state-contract, codex-5

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-4](../codex-4/ORDER.md). Reviewed head: `34b24e2fe`.

## Goal

Turn the per-item Q31 training operator into one shared model parameter. A model owns one Q31
gate, one step shift and an epoch; a batch of values and targets produces one deterministic
candidate gate, one aggregate loss comparison and one accepted state transition. The Rust
oracle and the CUDA engine must agree on the shared model transition, including the gate and
epoch after each step.

## Files in scope

- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/src/native_vector_device.rs`
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`
- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`
- `docs/loop/tasks/native-space-state-contract/codex-5/**`

## Size

About 150 to 300 changed lines.

## Done when

1. A `Q31HamiltonModel` owns one gate, one admitted step shift and an epoch.
2. A batch step aggregates per-item candidate gates deterministically in integer words and
   accepts only a strictly lower aggregate L1 loss.
3. The model's raw gate and epoch serialize and restore losslessly.
4. CPU model steps and CUDA model steps agree after every step on a finite batch fixture.
5. A finite 64-step run lowers the shared model loss, with no language or throughput claim.
6. Formatting, focused CPU tests, the full discrete library suite, CUDA-feature tests, Clippy,
   the PTX admission, the retained example, `git diff --check`, and the REPORT check pass.

## Stop

- Do not add a language corpus, tokenizer, serving session or speed claim.
- Do not change the per-item Q31 arithmetic or its closed domain.
- Do not add a host fallback for the CUDA model step.
