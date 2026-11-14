# ORDER: native-space-state-contract, codex-2

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-1](../codex-1/ORDER.md). Reviewed head: `2c1a7b120`.

## Owner approval

The repair continues the owner's direct instruction:

> "please continue developing and improving the cuda training tool"

No owner-only rule, cap or status changes in this repair.

## Review findings to repair

1. The admitted item count allowed 32-bit `item * 4` offsets to wrap.
2. Both random operands used the same stream, so a reversed Hamilton product could pass.
3. Output could leave the input domain, and deserialization accepted out-of-domain words.
4. The PTX denylist omitted `ldmatrix`, `stmatrix` and `movmatrix`.
5. The ORDER said 32x32 products while the generated artifact used sign-extended 64-bit
   `mul.lo.s64`.

## Goal

Close the Q31 batch bound, make argument order observable on CUDA, keep every serialized and
computed word in one closed domain, complete matrix-instruction admission, and align the written
product-width ABI with the artifact.

## Files in scope

- `crates/uor-r4-discrete/build.rs`
- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/src/native_vector.cu`
- `crates/uor-r4-discrete/src/native_vector_device.rs`
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`
- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`
- `docs/loop/tasks/native-space-state-contract/codex-1/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-2/**`

## Size

About 100 to 250 changed lines.

## Done when

1. Batches above `2^30` are refused before launch and kernel offsets are computed in 64 bits.
2. CUDA receives independent deterministic gate/value streams, and both Hamilton argument orders
   are checked explicitly.
3. Q31 input, serialized state and output all use the closed `-2^31..=2^31` domain; invalid CPU
   and GPU input is refused.
4. `ldmatrix`, `stmatrix` and `movmatrix` are refused by admission and its self-tests.
5. The ORDER and REPORT describe sign-extended 64-bit multiplication of admitted coordinates.
6. Focused CPU tests, RTX 4090 parity, Clippy, the full discrete library suite, formatting, the
   REPORT check and `git diff --check` pass.

## Stop

- Do not wire serving or launch training in this repair.
- Do not weaken the bit-identical comparison.
- STOP and report when the five review findings are closed.
