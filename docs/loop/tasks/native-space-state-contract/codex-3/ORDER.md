# ORDER: native-space-state-contract, codex-3

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-2](../codex-2/ORDER.md). Reviewed head: `838087961`.

## Review findings to repair

1. `Q31VectorState` exposed its word array, so an out-of-domain value could still be serialized.
2. The CUDA host-side invalid-input refusal had no integration control.
3. The codex-2 repair changed `codex-1/ORDER.md` without listing it in scope.
4. The codex-2 REPORT understated the repair size by one line and mixed two size bases.

## Goal

Make invalid Q31 state impossible to construct, test the GPU refusal on both sides of the
domain, and correct the active repair record without changing the operator arithmetic.

## Files in scope

- `crates/uor-r4-discrete/src/native_vector.rs`
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`
- `docs/loop/tasks/native-space-state-contract/codex-2/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-3/**`

## Size

About 20 to 100 changed lines.

## Done when

1. `Q31VectorState::words` is private and every constructor validates the closed domain.
2. Both `Q31_MIN - 1` and `Q31_MAX + 1` are refused on construction/deserialization.
3. The RTX 4090 integration test proves invalid gate and value words are refused before launch.
4. The codex-2 REPORT distinguishes the 359-line full repair from the 298-line requested range.
5. Focused CPU tests, CUDA parity, formatting, REPORT check and `git diff --check` pass.

## Stop

- Do not change kernel arithmetic.
- Do not wire serving or launch training.
- STOP and report when the four review findings are closed.
