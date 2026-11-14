# REPORT: native-space-state-contract, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first review found that a batch near the `u32` limit could wrap `item * 4`.
- Both generated operand streams were identical, so a reversed Hamilton product could pass.
- Arbitrary admitted inputs could produce an `i64` output outside the next input domain, and
  deserialization accepted such words.
- The shared PTX denylist omitted three matrix-load/store instructions.
- The written 32x32 product rule disagreed with the emitted sign-extended `mul.lo.s64`.

## What changed

The batch is capped at `2^30` items and kernel offsets use 64-bit arithmetic. Q31 input,
serialization and output now share the closed `-2^31..=2^31` domain; output is saturated after
the defined round/shift, and out-of-domain CPU or GPU input is refused.

Gate and value streams now use independent deterministic seeds. The CUDA test explicitly checks
both noncommuting axis orders. PTX admission and its self-test now reject `ldmatrix`, `stmatrix`
and `movmatrix` in addition to the existing float/tensor families.

The ORDER and REPORT now describe the actual sign-extended 64-bit multiply of admitted
32-bit-range coordinates.

Size: the full `2c1a7b120..838087961` repair is 359 changed lines across ten files, about
1.44 times the ORDER's 250-line upper estimate. The requested range excluding only the codex-2
ORDER is 298 lines across nine files: 168 code lines across six code files and 130
documentation/record lines. Size is reported, not capped (D48).

## Measurements

- Focused CPU state tests: 6 passed.
- Full `uor-r4-discrete` library suite: 361 passed, one ignored.
- RTX 4090 finite parity: 4,096 independent quaternion pairs, 16,384 raw words, bit-identical.
- Noncommutative CUDA check: `e_x ⊗ e_y = e_z` and `e_y ⊗ e_x = -e_z`.
- Planted one-bit scale/shift difference: detected.
- Generated PTX: one integer entry and no float/tensor/transcendental instruction.
- ptxas: 40 registers, zero stack frame and zero spills.

No trainer consumes the operator yet and no training or throughput claim is made.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- `cargo test --locked -p uor-r4-discrete native_vector --lib` - 6 passed.
- `cargo test --locked -p uor-r4-discrete --lib` - 361 passed, one ignored.
- CUDA integration test on the RTX 4090 - 2 passed.
- CUDA-feature Clippy for the focused target - passed.
- Release qualification example - passed and emitted a stable normalized image identity.
- `git diff --check` - passed.
- `uor-harness check` - 346 rules, 0 errors, existing warnings only.

## Skills used

- `uor-research-architect`: kept the repair finite and separate from a training campaign.
- `solution-integrity`: exercised the exact boundary, noncommutative order and planted fault.
- `working-discipline`: reran the final full suite after the earlier transient failure.
- `native-space-cycle`: closed the state, serialization and operator domains.

## Still open

- The existing serving session and native trainer still need to dispatch this operator.
- No full-model training step or optimizer uses the kernel yet.
- The branch remains stacked on the unmerged native-space control-plane work.

## Plain summary

The Q31 CUDA operator now has a closed state domain, safe launch bounds, independent operands,
explicit noncommutative checks and a complete integer-only admission. It is ready to be wired
into the trainer and serving path; it is not yet the model’s training step by itself.
