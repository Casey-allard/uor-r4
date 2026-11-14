# REPORT: native-space-state-contract, codex-8

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- A direct `uor-r4-integer` dependency on `uor-r4-discrete` created a package cycle through
  `uor-r4-core`. The portable Q31 state and arithmetic now live in a dependency-free shared
  crate; the discrete path re-exports them and retains its CUDA handle.
- `loop claim` refused because DeepSeek PR #3937 declares overlapping scope. The refusal is an
  instrument result, not a code failure. This unit leaves PR #3937's known files untouched.
- Moving the Q31 module changed its concrete error type. The first CUDA build then failed in four
  discrete wrappers that returned that error directly; they now use the crate's existing
  `From<uor_r4_native_vector::Error>` conversion.
- The installed `uor-harness` was stale and returned `0 rules` for the named native-space check.
  That was not accepted as a pass; the current harness binary was rebuilt and returned one rule.
- The first exact-head review found that this REPORT overstated two serving test functions as six
  tests and left the Clippy acceptance ambiguous. `codex-9` corrects the count and records a
  package-scoped Clippy run that includes the new file.
- Package-wide integer formatting and `-D warnings` Clippy already fail on extensive unrelated
  baseline drift. Only touched files are formatting-checked. Package-scoped integer Clippy remains
  red on existing code and is not reported as green.
- The research protocol's live `model-time.json` is absent from this checkout. No training run,
  resource claim or throughput result is made from the finite local CUDA parity checks.

## What changed

The new dependency-free `uor-r4-native-vector` crate owns the portable Q31 state schema, domain
checks, serialization, Hamilton transition and shared training-step oracle. `uor-r4-discrete`
keeps its AOT CUDA entry points and re-exports the shared portable types through its existing
module path.

`Q31ServingSession` in the integer crate now restores the exact 48-byte gate/epoch/step-shift
model without numerical conversion, saves it losslessly and executes `q31_hamilton_rotate` from
the same shared definition used by training. Wrong width, unknown version, nonzero trailing
bytes, out-of-domain gate words and step shifts above 31 are refused.

Two new serving test functions are part of the integer test binary: one covers a 64-step trained
state and its raw transition words, and one covers every serialization refusal. The shared crate's
14 tests and four CUDA integration tests continue to compare raw Q31 words.

Size: 589 lines moved from discrete into the shared crate, 607 lines in that new crate including
its tests, about 120 new lines in the serving adapter, and small dependency/re-export edits. The
new serving surface is within the ORDER's 80-to-180-line estimate; the moved implementation is
not new behavior.

## Measurements

- Shared Q31 crate: 14 tests passed.
- Integer Q31 serving filter: 2 tests passed; 644 tests filtered out.
- Discrete library: 356 passed, 1 ignored.
- RTX 4090 native-vector integration: 4 tests passed on CUDA compute capability 8.9.
- CUDA transport parity compared 4,096 gate/value pairs and 4,096 training records word for word
  against the Rust oracle; 64 shared model steps kept the same gate and epoch.
- The serving fixture trained 64 steps, round-tripped the 48-byte state, then compared gate,
  epoch, step shift and three transition results as raw `[i64; 4]` words. No rounded value is
  used as a parity unit.
- No token generation, language result, throughput or end-to-end serving claim is made.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` - 14 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` - 2 passed.
- `cargo check --locked -p uor-r4-integer --features serving-wasm` - passed.
- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo test --locked
  -p uor-r4-discrete --features cuda --test native_vector_cuda` - 4 passed on the RTX 4090.
- `cargo test --locked -p uor-r4-discrete --lib` - 356 passed, 1 ignored.
- CUDA test Clippy with `--no-deps -D warnings` - passed.
- Current checkout harness build and `check --only harness.native_space_contract` - 1 rule, 0
  failed.
- Full current-checkout harness check - `348 rules, 315 failed (0 error, 314 warn)`; this is the
  repository warning baseline, with no error.
- Harness package with `TMPDIR=/var/tmp/uor-harness.KQCj53` - 685 library tests passed with 1
  network-only test ignored, then every integration and doc test passed.
- Touched-file `rustfmt --check` and `git diff --check` - passed after the CUDA wrapper repair.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib` - exited 0
  with existing warnings outside this unit; filtered `q31_serving.rs` diagnostics: 0.
- Package-wide `uor-r4-integer` Clippy `-D warnings` - failed on existing warnings outside this
  change; no green result is claimed.

## Skills used

- `uor-research-architect`: kept the unit finite, source-bound and separate from a training run.
- `native-space-cycle`: required one operator definition and raw-word state serialization.
- `working-discipline`: checked repository state, used an isolated worktree and reported the
  negative and unverified results first.
- `solution-integrity`: traced the dependency cycle and the CUDA error-type break to their actual
  callers before repairing them.

## Still open

- `IntegerStackSession` does not yet own or consume this loaded Q31 state in its real token
  recurrence. The adapter is a serving boundary, not proof that token generation uses it.
- The next replay must load one 48-byte state into the real session, compare served and CUDA/Q31
  transition words, and compare emitted token ids plus route and carry state on one fixed prompt.
- Host-side four-word training aggregation remains outside this unit.
- PR delivery, exact-head review, CI, merge and fresh-main verification remain after this record.

## Plain summary

Training can now write a 48-byte integer model state, and the integer serving crate can read the
same bytes and run the same quaternion transition without converting scales. CUDA and Rust still
agree on raw 64-bit words in the finite tests. This is not yet a served model: the normal token
session must load this state and use it during generation, then match routes, carry, read choices,
memory addresses and token ids. That session integration is the next model step.
