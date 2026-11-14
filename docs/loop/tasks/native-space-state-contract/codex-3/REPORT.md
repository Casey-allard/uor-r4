# REPORT: native-space-state-contract, codex-3

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The prior state type exposed its word array, so an out-of-domain state could be serialized even
  though deserialization refused it.
- The CUDA host refusal existed but had no integration assertion.
- The codex-2 order omitted the amended codex-1 ORDER from its scope.
- The codex-2 size statement mixed the full repair range with a narrower requested range.

## What changed

`Q31VectorState` now has a private word array and a checked `new` constructor. Both
serialization and every other constructor pass through the same closed-domain admission.
`words()` exposes a copy only after construction.

The CUDA integration test now supplies `Q31_MAX + 1` and `Q31_MIN - 1` as both gate and value
and asserts refusal before launch. The codex-2 report now states the measured size bases
explicitly.

Size: 147 changed lines across five files, about 1.5 times the ORDER's 100-line upper estimate.
The repair record and two-sided tests account for the overrun; size is reported, not capped.

## Measurements

- Focused CPU state tests: 6 passed.
- RTX 4090 integration: 2 passed, including two-sided invalid gate/value refusal.
- The operator arithmetic and generated CUDA image are unchanged by this repair.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- Focused `native_vector` library tests - 6 passed.
- CUDA-feature integration test on the RTX 4090 - 2 passed.
- `git diff --check` - passed.
- `loop report --check` - passed.

## Skills used

- `solution-integrity`: made invalid states unrepresentable instead of checking only at read time.
- `working-discipline`: measured both sides of the domain and kept the check at the shipped API.
- `native-space-cycle`: preserved one closed state and serialization domain.

## Still open

- The operator still needs to be wired into the native trainer and serving session.
- No training run or throughput claim is part of this repair.

## Plain summary

An invalid Q31 vector can no longer be constructed or serialized, and the 4090 test now proves
the GPU API rejects invalid gate and value words on both sides of the allowed range. The CUDA
arithmetic itself did not change.
