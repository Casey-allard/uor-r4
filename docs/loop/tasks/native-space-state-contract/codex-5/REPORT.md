# REPORT: native-space-state-contract, codex-5

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first shared-model CPU test ran 65 transitions because it took one probe step after the
  64-step sequence and then asserted epoch 64. The probe now runs on a copy, so the trained model
  remains at epoch 64.
- The shared model still aggregates per-item CUDA candidate gates on the host. The per-item
  forward and credit are CUDA; the deterministic four-word reduction is Rust. This is enough for
  the finite shared-state contract, but it is not a device-resident optimizer or a throughput
  result.
- The dependency-inclusive Clippy baseline remains red on unrelated `uor-r4-integer` warnings.
  The package-scoped `--no-deps` run passed and the baseline is not presented as green.
- Retargeting PR #3968 with `gh pr edit` failed on GitHub's deprecated classic Projects query;
  the REST pull-request endpoint changed the base without touching code.
- Merging the final control plane produced duplicated control-record conflicts. The CUDA source
  did not conflict; each duplicated control record was resolved to its newer `main` version.
- The finalized ORDER parser then found that this chain's roadmap link was wrapped onto the next
  line. `codex-6` records the metadata-only repair.

## What changed

`Q31HamiltonModel` now owns one Q31 gate, one admitted `step_shift` and a checked epoch. A batch
step takes already-computed per-item candidate gates, averages their integer gate deltas in a
fixed order, clips the candidate to the shared Q31 domain, recomputes the aggregate L1 loss and
accepts only a strictly lower loss. The model state serializes as 48 little-endian bytes: four
gate words, the epoch, the step shift, a version byte and zero trailing bytes.

`NativeVectorEngine::train_model_step` uses the existing CUDA per-item training entry for every
value/target pair, then commits the same records through the shared Rust aggregation. The CPU
reference runs the same per-item oracle and commit. A 64-step RTX 4090 test compares the complete
model transition, gate and epoch after every step.

Size: 291 changed code/example/test lines across four files. The REPORT is additional record
material. This is within the ORDER's 300-line code estimate but reports the exact basis.

After PR #3967 merged as `e2175455e`, the CUDA branch merged that squash forward at `d12c29a11`.
The merge keeps the final native-space contract and harness checks from `main` and preserves the
Q31 CUDA operator, shared-model transition and integration tests unchanged in capability.
Claude PRs #3969 and #3970 later advanced `main` to `6434f9ab3`; the CUDA branch merged that
baseline cleanly at `76f9aa3e2`.

## Measurements

- Shared model fixture: 8 identical value/target rows, step shift 8.
- Initial aggregate L1 loss: `17,179,869,184`.
- Final aggregate L1 loss after 64 accepted CUDA model steps: `13,373,147,056` (about 22.2%
  lower).
- Final shared gate: `[2008113292, 336469910, 0, 0]`; epoch `64`.
- CPU and CUDA shared-model transitions: equal after every one of the 64 steps.
- 48-byte shared state round-trips gate, shift and epoch exactly.
- No language, corpus, serving or throughput claim is made.

The forward/training image identities remain:
`e07dbf3079f5ab97471e803b6412d9df7c70016ee99c30029f48c21efe904c88`,
`b22a54ae30b75dc71bf3753defe220e32cd223c405351d3001e4ade1554c1243`,
`5e6d18a5309f7886a46679bb55eeecb7012259dde5c5212a54b27995eb23342f`,
`82bd1a346e44164986eed03cccbd53e27088f5a71668ab7f51046ba578e8a5c3`.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- `cargo test --locked -p uor-r4-discrete native_vector --lib` - 11 passed.
- `cargo test --locked -p uor-r4-discrete --lib` - 367 passed, 1 ignored.
- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo test
  --locked -p uor-r4-discrete --features cuda --test native_vector_cuda` - 4 passed on the RTX
  4090.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --features cuda --test
  native_vector_cuda -- -D warnings` - passed; dependency-inclusive Clippy remains red on the
  existing `uor-r4-integer` baseline.
- Release `native_vector_qualify` example - passed with the shared-model fields above.
- Merged-head CUDA integration rerun with `NVCC=/usr/local/cuda/bin/nvcc`,
  `UOR_NVCC_ARCH=compute_89` and `UOR_REQUIRE_CUDA=1` - 4 passed on the RTX 4090.
- Merged-head discrete library rerun - 367 passed, 1 ignored.
- Merged-head full harness package with `TMPDIR=/var/tmp/uor-harness.KQCj53` - 685 passed,
  0 failed, 1 ignored, plus every integration and doc test.
- Package-scoped formatting for `uor-r4-discrete` and `uor-r4-harness` - passed.
- Package-scoped CUDA Clippy with `--no-deps ... -D warnings` - passed.
- Release `native_vector_qualify` rerun - 16,384 raw words bit-identical, 4,096 training records
  bit-identical, and 64 shared-model steps lowering loss from `17,179,869,184` to
  `13,373,147,056`.
- `git diff --check` - passed.

## Skills used

- `uor-research-architect`: kept the shared model finite and source-bound.
- `solution-integrity`: separated candidate generation from aggregate acceptance and retained the
  failed 65-step probe as a control lesson.
- `native-space-cycle`: preserved one gate, one transition and raw-word state serialization.
- `working-discipline`: reported the host reduction and dependency Clippy limit explicitly.

## Still open

- The gate-delta aggregation is host-side; the next performance unit can move the reduction to an
  admitted CUDA kernel.
- The integer serving session still needs to consume this same model state.
- No language-model or serving result is claimed.
- PR #3967 is merged; PR #3968 is based directly on fresh `main`.

## Plain summary

The Q31 CUDA training step now trains one shared model gate instead of independent test records.
On the RTX 4090, CPU and CUDA agree on the gate and epoch after every step, and 64 accepted steps
reduce the finite aggregate loss by about 22.2%. The branch now contains the final native-space
control plane and passes the merged-head CUDA, CPU, harness, formatting and Clippy checks. The
next missing link is serving the serialized state through the same transition.
