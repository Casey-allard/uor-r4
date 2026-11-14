# REPORT: native-space-state-contract, codex-4

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first CUDA-feature build after adding the second entry failed to compile because the old
  `native_vector_entry` helper still referenced the removed single-entry constant. The helper now
  names the forward entry and a separate training-entry helper names the step.
- The full `cargo clippy -- -D warnings` path was red on 68 pre-existing warnings in the
  `uor-r4-integer` dependency. That is an unrelated dependency baseline, not a result from this
  unit. The package-scoped `--no-deps` run passed; the dependency warning baseline is not hidden.
- The L1 acceptance rule is a bounded geometric credit step, not a full dense backward pass. The
  finite test proves a lowering transition; it does not claim language-model training.

## What changed

The Q31 native vector path now has a second operation:

`gate, value, target -> Hamilton prediction -> bounded Q31 credit -> candidate gate -> loss check`.

`q31_hamilton_train_step` computes the forward with the existing Hamiltonian rotation, forms an
integer credit from the residual and conjugate value, shifts it by an explicit `step_shift`, and
keeps the candidate gate only when its L1 loss is strictly smaller. `Q31TrainingStep` carries the
prediction, candidate, accepted gate, both losses and the acceptance bit.

`native_vector.cu` now exports `native_q31_hamilton_train_step` beside the existing
`native_q31_hamilton_rotate`. The build PTX inventory admits exactly those two entries.
`NativeVectorEngine::hamilton_train_step` validates the closed domain and shift, launches the AOT
kernel, and returns raw-word records. The Rust and CUDA tests compare the full records, not a
rounded score.

Size: 417 changed lines across six changed files, above the ORDER's 400-line upper estimate. The
overrun is the second kernel, its host wrapper, the raw-word control and the retained qualification
example. Size is reported, not capped (D48).

## Measurements

Device: NVIDIA GeForce RTX 4090, compute capability 8.9, driver 595.99.02.

- Forward parity: 4,096 Q31 quaternion pairs; 16,384 raw `i64` words bit-identical between CUDA
  and the Rust oracle.
- Training-step parity: 4,096 independent `(gate, value, target)` records; prediction, candidate
  gate, accepted gate and both loss words bit-identical.
- Finite geometric training: 64 accepted CUDA steps from identity gate lowered L1 loss from
  `2,147,483,648` to `1,671,643,382`.
- The finite example's training batch including H2D/D2H took `472,457 ns`; this is an
  instrumentation wall time, not a throughput or speedup claim.
- ptxas: rotate entry 40 registers, step entry 44 registers; both have 0-byte stack frames and
  zero spill stores/loads.
- PTX admission found only the two registered integer entries and no float, tensor, matrix or
  transcendental instruction.

The source/PTX/cubin/resource identities emitted by the final example run are:
`e07dbf3079f5ab97471e803b6412d9df7c70016ee99c30029f48c21efe904c88`,
`b22a54ae30b75dc71bf3753defe220e32cd223c405351d3001e4ade1554c1243`,
`5e6d18a5309f7886a46679bb55eeecb7012259dde5c5212a54b27995eb23342f`,
`82bd1a346e44164986eed03cccbd53e27088f5a71668ab7f51046ba578e8a5c3`.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- `cargo test --locked -p uor-r4-discrete native_vector --lib` - 9 passed.
- `cargo test --locked -p uor-r4-discrete --lib` - 365 passed, 1 ignored.
- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo test
  --locked -p uor-r4-discrete --features cuda --test native_vector_cuda` - 3 passed on the RTX
  4090.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --features cuda --test
  native_vector_cuda -- -D warnings` - passed. The full dependency-inclusive Clippy run was red
  on the unrelated `uor-r4-integer` baseline described above.
- Release `native_vector_qualify` example - passed and emitted the forward/training identities
  and finite loss values above.
- `git diff --check` - passed.

## Skills used

- `uor-research-architect`: kept the unit finite and tied the transition to the declared item.
- `solution-integrity`: reused the Hamilton rotation as the single forward definition, added a
  non-improving control, and rejected invalid domains before launch.
- `native-space-cycle`: preserved one Q31 state and raw-word acceptance.
- `working-discipline`: retained the negative Clippy baseline and reported instrumentation limits.

## Still open

- The native trainer's shared model parameter is still per-item in this finite operator; the
  next unit must connect it to a shared model state and the integer serving session.
- No corpus, language-model, serving, or throughput result is claimed.
- The branch remains stacked on the unmerged native-space control-plane PR #3967.

## Plain summary

The custom integer CUDA Hamilton operator now has a real bounded training step. On the RTX 4090,
4,096 train-step records match the Rust oracle word for word, and 64 accepted updates reduce the
finite Q31 L1 loss by about 22.2%. The next work is wiring this state into the shared trainer and
serving path, not adding more harness controls.
