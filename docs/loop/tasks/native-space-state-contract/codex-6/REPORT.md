# REPORT: native-space-state-contract, codex-6

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `gh pr edit 3968 --base main` failed because the CLI still queries deprecated classic Projects
  metadata. The REST pull-request endpoint changed only the PR base.
- Merging the final control plane produced conflicts in duplicated contract and harness records,
  not in Q31 CUDA source. The final `main` versions were carried forward.
- `loop pr-check --local` then failed because every ORDER in this chain wrapped the roadmap item
  link onto the line after `Item:`; the finalized parser correctly saw no item.
- The first CUDA check used `sm_89`; the build requires a virtual PTX architecture. The successful
  rerun used `compute_89`.

## What changed

PR #3968 is retargeted from the merged control-plane branch to `main`. The CUDA branch merged
`e2175455e` forward at `d12c29a11`, preserving the Q31 CUDA operator and shared-model training
path while replacing older duplicated control-plane files with their final `main` versions.
Claude PRs #3969 and #3970 then advanced `main` to `6434f9ab3`; a second clean merge-forward at
`76f9aa3e2` retained both changes.

All existing ORDERs now put the roadmap item link on the `Item:` line. The tracked PICKUP names
fresh `main`, the merge-forward head, PR #3968's direct `main` base and integer serving as the next
task. The codex-5 REPORT records the merged-head test results.

Size: metadata-only changes across the existing task chain plus this repair record. No model,
kernel or serving code changed.

## Measurements

- RTX 4090 Q31 integration: 4 passed.
- Raw forward words bit-identical: 16,384.
- Raw training records bit-identical: 4,096.
- Shared-model accepted steps: 64.
- Aggregate loss: `17,179,869,184` to `13,373,147,056`.
- Final shared gate: `[2008113292, 336469910, 0, 0]`; epoch `64`.
- Discrete library: 367 passed, 1 ignored.
- Full harness library: 685 passed, 1 ignored, plus every integration and doc test.

No language, serving or throughput result is claimed.

## Checks run at the head

- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo test
  --locked -p uor-r4-discrete --features cuda --test native_vector_cuda` - 4 passed.
- `cargo test --locked -p uor-r4-discrete --lib` - 367 passed, 1 ignored.
- `TMPDIR=/var/tmp/uor-harness.KQCj53 cargo test --locked -p uor-r4-harness` - 685 passed,
  0 failed, 1 ignored, plus every integration and doc test.
- `cargo fmt --package uor-r4-discrete -- --check` and
  `cargo fmt --package uor-r4-harness -- --check` - passed.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --features cuda --test
  native_vector_cuda -- -D warnings` - passed.
- Release `native_vector_qualify` example - passed with 16,384 forward words and 4,096 training
  records bit-identical.
- `uor-harness loop pr-check --local` - PASS, with only the known D76 advisory warnings.
- `git diff --check` - passed.

## Skills used

- `uor-research-architect`: retained the accepted Q31 artifact and advanced its delivery.
- `scope-guard`: limited this repair to ORDER, PICKUP and REPORT metadata.
- `working-discipline`: distinguished failed build-input checks from product results.
- `native-space-cycle`: preserved the one-state, one-transition boundary.

## Still open

- The integer serving session must consume the shared serialized `Q31HamiltonModel` state.
- Host-side four-word aggregation remains a performance boundary, not a parity failure.
- No language-model or serving result is claimed.

## Plain summary

The CUDA branch now sits on fresh `main`, keeps the final control plane, and retains the Q31 CUDA
training implementation. The ORDER metadata is repaired and the tracked handoff points to
integer serving as the next implementation step.
