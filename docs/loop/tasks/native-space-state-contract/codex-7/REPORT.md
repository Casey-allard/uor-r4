# REPORT: native-space-state-contract, codex-7

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The fresh review found that an accepted step assigned the candidate gate before checking the
  epoch increment. A serialized model at `u64::MAX` could return an error after changing its gate.
- The merge-forward retained a skill dependency that final `main` had explicitly removed to break
  the dependency cycle.
- The aggregate L1 sum used unchecked `u64` addition even though the admitted 2^30-item batch can
  exceed `u64` before a result is accepted.
- The first formatter check found layout drift in the new checked-loss helper; the package was
  formatted and the check then passed.

## What changed

`Q31HamiltonModel::step_from_records` now validates the admitted batch bound and every value and
target before calculating a candidate. Both aggregate losses are accumulated in `u128` through a
checked helper and converted to `u64` only when representable. The epoch increment is checked
before either field is assigned, so every error leaves the model byte-identical.

The architect skill pointer is restored to final `main`, removing the reintroduced
`native-space-cycle` edge. Q31 multiplication, rounding, shift and saturation are unchanged.

New focused tests cover epoch overflow without mutation, invalid targets without mutation and
aggregate-loss overflow refusal. The PICKUP records the current main and repair state.

Size: about 100 changed code and test lines plus the repair record.

## Measurements

- Focused native-vector tests: 14 passed.
- RTX 4090 CUDA integration: 4 passed.
- Discrete library: 370 passed, 1 ignored.
- Full harness library: 685 passed, 1 ignored, plus every integration and doc test.
- Qualification: 16,384 forward words and 4,096 training records bit-identical.
- Shared model: 64 accepted steps, gate `[2008113292, 336469910, 0, 0]`, epoch `64`.
- No language, serving or throughput result is claimed.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- `cargo test --locked -p uor-r4-discrete native_vector --lib` - 14 passed.
- `cargo test --locked -p uor-r4-discrete --lib` - 370 passed, 1 ignored.
- CUDA integration with `compute_89` and `UOR_REQUIRE_CUDA=1` - 4 passed on the RTX 4090.
- Package-scoped CUDA Clippy with `-D warnings` - passed.
- Full harness package with `TMPDIR=/var/tmp/uor-harness.KQCj53` - 685 passed, 1 ignored.
- Built-harness `check --only harness.native_space_contract` - 1 rule, 0 failed.
- Release `native_vector_qualify` - passed with the recorded artifact identities.
- `git diff --check` - passed.

## Skills used

- `solution-integrity`: found the partial state mutation and checked the proof before shipping.
- `native-space-cycle`: preserved one atomic state transition and exact raw-word serialization.
- `uor-research-architect`: kept the repair tied to the promoted native artifact.
- `working-discipline`: distinguished unchanged Q31 arithmetic from the corrected state boundary.

## Still open

- The integer serving session still needs to consume the shared serialized model.
- Host-side four-word aggregation remains the next performance boundary.
- No language-model or serving result is claimed.

## Plain summary

The shared Q31 model now refuses overflow or invalid inputs before changing state, so a failed step
is atomic. The final `main` skill graph is restored, all focused and full checks pass, and integer
serving remains the next implementation step.
