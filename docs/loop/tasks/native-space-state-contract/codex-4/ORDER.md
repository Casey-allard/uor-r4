# ORDER: native-space-state-contract, codex-4

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-3](../codex-3/ORDER.md). Reviewed head: `01521bbb6`.

## Goal

Use the existing Q31 Hamilton vector operator as the first integer training
transition, not only as a forward parity fixture. Add a bounded training step
that computes the next Q31 gate and value transport, measures an integer L1
loss before and after the candidate update, and accepts the update only when
the loss falls.

The Rust oracle and the AOT CUDA implementation must return the same raw words
for the prediction, candidate gate, accepted gate, and both losses. The CUDA
path must remain AOT, integer-only, and free of float, tensor, matrix,
transcendental, or NVRTC instructions.

## Files in scope

- `crates/uor-r4-discrete/build.rs`
- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/src/native_vector.cu`
- `crates/uor-r4-discrete/src/native_vector_device.rs`
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`
- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`
- `docs/loop/tasks/native-space-state-contract/codex-4/**`

## Size

About 150 to 400 changed lines, including the CPU oracle, the CUDA step entry,
the PTX inventory, the finite training control, and the delivery record.

## Done when

1. A Rust training-step oracle uses the existing Q31 Hamilton product, a
   bounded integer credit, and a loss-decreasing acceptance rule.
2. The CUDA module exports a second integer entry for the same step and rejects
   out-of-domain gate, value, target, or step parameters before launch.
3. The CUDA test compares raw prediction/gate/loss words with the Rust oracle on
   deterministic inputs and includes a planted wrong-scale or wrong-order fault.
4. A finite sequence of training steps lowers L1 loss on a known target from the
   identity gate, without a language or throughput claim.
5. Formatting, focused Rust tests, CUDA-feature tests on the RTX 4090, Clippy,
   PTX admission, `git diff --check`, and the REPORT check pass.

## Stop

- Do not run a corpus, language-model, or serving qualification.
- Do not change the existing Hamilton arithmetic or Q31 domain.
- Do not add a host fallback, matmul, softmax, squashing, or runtime CUDA
  compilation.
- Do not wire the serving session in this unit; that is the next dependency
  after the training transition is qualified.
