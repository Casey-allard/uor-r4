# ORDER: native-space-state-contract, codex-8

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-7](../codex-7/ORDER.md). Reviewed head: `c5978c5d5`.

## Why this unit exists

The merged Q31 CUDA work establishes one trainable native vector state, but the integer serving crate
still has no API that loads that 48-byte state or executes its transition. The previous unit's
PICKUP names this as the immediate next model step.

`loop claim` refused because DeepSeek's open PR #3937 declares overlapping item scope and changes
`crates/uor-r4-integer/src/stack/tests.rs`. The owner's direction is to continue the main vector
track without waiting on other labs. This unit will avoid `stack/tests.rs` and all other files in
that PR; the refusal is recorded as an instrument result, not treated as a code failure.

## Goal

Add the serving-side boundary for the merged native Q31 model: construct a serving session from
the exact 48-byte serialized state, expose its gate/epoch without conversion, and execute the
shared Q31 Hamilton transition using the same Rust implementation that the CUDA trainer uses.

This unit establishes the serving transition and parity receipt. It does not claim token-level
language capability or a completed end-to-end generation model.

## Files in scope

- `Cargo.lock`
- `Cargo.toml`
- `crates/uor-r4-integer/Cargo.toml`
- `crates/uor-r4-discrete/Cargo.toml`
- `crates/uor-r4-discrete/src/lib.rs`
- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/src/native_vector_device.rs`
- `crates/uor-r4-native-vector/Cargo.toml`
- `crates/uor-r4-native-vector/src/lib.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/q31_serving.rs`
- `docs/roadmap/items/native-space-state-contract.md`
- `docs/start/FILEMAP.md`
- `docs/loop/tasks/native-space-state-contract/codex-8/**`

## Size

About 80 to 180 lines.

## Done when

1. The integer crate can load and save the exact 48-byte Q31 model state without conversion.
2. Its serving transition calls the shared `uor_r4_native_vector` Q31 Hamilton operator.
3. A trained gate loaded from bytes produces exactly the same raw words as the Rust oracle.
4. Invalid version, trailing bytes, gate words or step shifts are refused.
5. Focused serving tests, the integer crate build, formatting, Clippy, `git diff --check`, and the
   report check pass.
6. The next step is recorded as connecting this serving adapter to the token/session path; no
   language or speed claim is made here.

## Cancel

Stop before implementation only if the integer crate cannot depend on the shared discrete Q31
module without creating a dependency cycle, or if the 48-byte state cannot be loaded without a
numerically different conversion. Do not introduce a second Q31 arithmetic implementation.
