# PICKUP: native vector geometry and Q31 CUDA delivery

This is the tracked mirror of `local/codex/PICKUP.md`. The repository is the authority if the two
disagree.

## Premise check

```sh
cd /home/callard/projects/codex-playground/uor-r4-corvuswing
git fetch --prune origin
git rev-parse origin/main
gh pr view 3967 --json state,isDraft,headRefOid,url
git -C local/worktrees/codex-native-space-cycle status --short --branch
git -C local/worktrees/codex-cuda-native-train status --short --branch
```

The previous control-plane merge head is `74deb0a6b`; this tracked note is updated after it.
`origin/main` is `085ff55d92c31ded738913ae8907251e4dcd8a21`; PR #3967 is open and ready for
review; the control-plane branch is `codex/native-space-cycle`; and
`codex/cuda-native-train` is at `48298622d`, with draft PR #3968 stacked on it. Report any
disagreement before acting.

## Off limits

- Do not resume `codex-7` loader symmetry. The owner redirected work to the vector-geometry CUDA
  path.
- Do not touch another lab's branch, worktree, claim or uncommitted files.
- Preserve `/var/tmp/sc-direct-flowmux.iptmjZre/evidence/`, `host-7b224540/`,
  `source-c07.zjcaw7Q7/`, and `source/.mvm-test/target/`.
- Do not change Q31 arithmetic or its closed domain without a reproduced defect.
- Use the RTX 4090, not the 5090.
- Training or evaluation needs its pre-registration/FIT and GPU lease; finite kernel tests do not.
- Model code is Rust plus AOT CUDA. No Python, matmul/GEMM, softmax or squashing.

## Current state

PR #3966 is merged as `f40f06abe82447b3174418ffcd2f7245125e5c8f`. PR #3967 contains the
native-space control plane. Its second review repair added the active ORDER's owner approval,
attributed findings to the failing source, accepted the roadmap lifecycle, aligned the float
exception with D85, and added `crates/uor-r4-discrete/**` plus
`crates/uor-r4-integer/src/discrete_d11.rs` to the next item's scope.

Focused native-space tests pass 8. The clean full harness package run with `TMPDIR` outside the
checkout passed 685 tests with 0 failures and 1 ignored. Formatting and `git diff --check` pass.
The remaining gate is the fresh exact-head review and queue on the merged-forward head.

`codex/cuda-native-train` carries the actual Q31 vector operator in
`crates/uor-r4-discrete/src/native_vector.{rs,cu}` and `native_vector_device.rs`, with
`native_vector_qualify.rs` and `native_vector_cuda.rs`. Recorded RTX 4090 evidence: 4,096 Q31
quaternion pairs / 16,384 raw `i64` words bit-identical to the Rust oracle on `compute_89`,
CUDA 13.1; noncommutative Hamilton checks; no float/tensor/matrix/transcendental PTX; ptxas 40
registers for forward and 44 for training, both with 0 spills. The Q31 training step matched 4,096
records word-for-word. One shared serializable gate/epoch model now aggregates those CUDA records;
64 accepted model steps lower aggregate L1 loss from `17,179,869,184` to `13,373,147,056`, with
CPU/CUDA gate-and-epoch parity after every step. The branch is pushed and stacked on the control
plane as draft PR #3968; serving does not consume the shared state yet.

## Next task

1. Merge PR #3967 through the protected queue and verify fresh `origin/main`.
2. Retarget PR #3968 to `main`, merge current `main` into `codex/cuda-native-train`, and merge
   the CUDA training PR.
3. Run the serving wiring so the integer session consumes the shared serialized
   `Q31HamiltonModel` gate/epoch transition.

Cancel if the compiled CUDA source no longer matches the reviewed arithmetic, the 4090 does not
reproduce the Rust words bit-for-bit, or serving needs a numerically different operator.

## Orders already carried out

- Merged and verified #3966 at `f40f06abe82447b3174418ffcd2f7245125e5c8f`.
- Created and repaired PR #3967's exact native-space contract.
- Built and locally qualified the integer Q31 Hamilton CUDA operator on the RTX 4090.
- Made invalid `Q31VectorState` values unconstructible.
- Added the bounded Q31 CUDA training step and one shared serializable Q31 gate/epoch model.
- Preserved the ASTRA paths above.

## Standing rules

- `CHARTER.md` wins.
- One persistent native state, one typed operator definition, bit-identical train/serve words
  and lossless serialization (D85).
- Serve integers; floats are reference or diagnostic only.
- Rust model code; CUDA is AOT and has a Rust oracle.
- Deliver through PR, fresh exact-head review, queue, merge, fresh-main verification and cleanup.
- Re-read this note at every commit before changing direction.

## Running

No GPU lease or training run is active. Review session `99896` is stale because it observed a
moving worktree before this tracked repair commit; it is not evidence for the new head.

## Manifest

```text
MANIFEST
- origin/main: 085ff55d92c31ded738913ae8907251e4dcd8a21  check: git rev-parse origin/main
- PR 3967 head: current branch HEAD at push  check: test "$(git rev-parse HEAD)" = "$(gh pr view 3967 --json headRefOid -q .headRefOid)"
- focused native-space tests: 8 passed  check: cd local/worktrees/codex-native-space-cycle && CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness native_space_contract
- full harness package: 685 passed / 0 failed / 1 ignored  check: docs/loop/tasks/infra-loop-harness/codex-3/REPORT.md
- CUDA branch head: 48298622d  check: git -C local/worktrees/codex-cuda-native-train rev-parse HEAD
- CUDA branch tree: 3331bd3023156fc874ff50a8f610cf7dfe5aad4a  check: git -C local/worktrees/codex-cuda-native-train rev-parse HEAD^{tree}
- CUDA PR: #3968 draft stacked on #3967  check: gh pr view 3968 --json state,isDraft,baseRefName
- RTX 4090 Q31 parity: 16,384 raw i64 words bit-identical  check: UOR_REQUIRE_CUDA=1 cargo test -p uor-r4-discrete --features cuda --test native_vector_cuda
- next action: make integer serving consume the shared Q31 model state  check: sed -n '/## Next task/,/## Orders/p' docs/loop/tasks/infra-loop-harness/codex-2/PICKUP.md
```
