# ORDER: native-space-state-contract, codex-1

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Base: `ef2a89bce`.

## Owner approval

The owner directed the CUDA training work:

> "great thank you, please continue developing and improving the cuda training tool"

and corrected the intended target:

> "YOU WERE LITERALLY JUST DEVELOPING NEW CUDA STUFF FOR TRAINING INTO VECTORS, RIGHT???"

This unit therefore implements one native CUDA vector transition. It does not run a
training campaign or make a capability claim.

## Native transition

The first shared transition is the Q31 Hamilton rotation used for vector transport:

`NativeVectorState -> q31_hamilton_rotate -> NativeVectorState`

The state is four signed 64-bit storage words at Q31, with each admitted coordinate in
the signed 32-bit range including `+2^31`. The product is formed with sign-extended
64-bit integer multiplies of those admitted coordinates, accumulated modulo `2^64`,
rounded by `2^30`, shifted right by 31 and saturated to the same closed domain.
Modulo, rounding, shift and saturation order are part of the operator definition.

The CUDA implementation, Rust CPU oracle and focused test use those exact rules. This
first unit does not claim that the existing integer serving session already routes
through this new module; it establishes the native operator that the following wiring
unit will use.

## Cancel probe

Cancel this order if any of these is true:

1. the generated PTX contains a float, tensor, transcendental or matrix-multiply
   instruction;
2. CUDA and Rust outputs differ in any raw output word;
3. the CUDA build needs NVRTC, a host fallback or a runtime compiler;
4. the module cannot target the local RTX 4090 with an explicit AOT architecture;
5. implementing the operator requires a second incompatible state representation.

## Files in scope

- `docs/loop/tasks/native-space-state-contract/codex-1/**`
- `docs/roadmap/items/native-space-state-contract.md`
- `crates/uor-r4-discrete/build.rs`
- `crates/uor-r4-discrete/src/lib.rs`
- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/src/native_vector.cu`
- `crates/uor-r4-discrete/src/native_vector_device.rs`
- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`

## Size

About 250 to 450 changed lines across one Rust state/oracle, one AOT CUDA kernel, one
device handle, one focused test and the delivery record.

## Done when

1. The ORDER records the state fields, widths, exponent, operator, carry and
   serialization boundary.
2. `native_q31_hamilton_rotate` is compiled ahead of time into its own admitted
   integer PTX/cubin and rejects any forbidden instruction.
3. Rust and CUDA produce bit-identical raw `i64` words on finite adversarial inputs,
   including signs, extremes and non-axis-aligned quaternions.
4. A planted scale or shift change turns the focused parity check red.
5. The module compiles without CUDA, and the CUDA test passes on the local RTX 4090.
6. Focused Cargo tests, Clippy, `uor-harness check`, `git diff --check` and the REPORT
   check pass.

## Stop

- Do not launch full training.
- Do not change D85, CHARTER, D11 or GPU caps.
- Do not wire this into the serving session yet; that is the next dependency.
- STOP and report when this finite operator is qualified or the cancel probe fires.
