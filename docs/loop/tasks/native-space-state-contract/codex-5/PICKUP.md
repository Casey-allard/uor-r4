# PICKUP: shared Q31 model state to integer serving

## Premise check

```sh
cd /home/callard/projects/codex-playground/uor-r4-corvuswing
git fetch --prune origin
git -C local/worktrees/codex-cuda-native-train status --short --branch
git -C local/worktrees/codex-cuda-native-train rev-parse HEAD
gh pr view 3968 --json state,isDraft,headRefOid,baseRefName,url
```

Expected: a clean `codex/cuda-native-train` containing the Q31 model work, merge-forward head
`76f9aa3e2`, and the `codex-7` atomic-state repair. PR #3968 is based directly on `main`. The
repository wins over this note.

## Current state

- Fresh `origin/main` is `6434f9ab3415ec99c02da2934643bf98ced56f3a`, including Claude PRs #3969
  and #3970 after the #3967 control plane.
- The CUDA branch merged the control plane at `d12c29a11` and the newer `main` at `76f9aa3e2`;
  all Q31 CUDA source and tests remained intact.
- The finalized ORDER parser required the roadmap item link on the same line; `codex-6` repairs
  that metadata across the existing chain.
- `codex-7` makes every model-step error atomic, checks aggregate loss before conversion and
  restores the final `main` skill dependency.

## Current model path

- `Q31HamiltonModel` owns one gate, step shift and epoch in
  `crates/uor-r4-discrete/src/native_vector.rs`.
- `NativeVectorEngine::train_model_step` runs the AOT CUDA per-item credit and commits the shared
  four-word state.
- RTX 4090: 4,096 training records match Rust word-for-word; 64 accepted shared steps lower
  aggregate L1 loss from `17,179,869,184` to `13,373,147,056`.
- CPU/CUDA gate and epoch match after every shared step; state round-trips through 48 bytes.
- The aggregation is currently host-side. No language, serving or throughput claim is made.

## Next task

Make the integer serving session consume the same serialized `Q31HamiltonModel` state. The next
order must define the shared operator boundary before editing `crates/uor-r4-integer`, preserve
the existing serving integer conventions, and prove a loaded trained gate produces the same raw
words in serving and Rust oracle.

## Off limits

- Do not resume `codex-7` loader symmetry or add more harness layers.
- Do not change Q31 arithmetic/domain without a reproduced defect.
- Do not use another model output or a float master to manufacture the served state.
- Do not touch other labs' work or the protected ASTRA paths.
- Use the RTX 4090 for CUDA checks.

## Standing rules

- `CHARTER.md` wins.
- One persistent native state, one typed operator definition, bit-identical train/serve words,
  lossless serialization (D85).
- Rust model code; AOT CUDA with a Rust oracle; no matmul/GEMM, softmax or squashing.
- Deliver through PR, fresh exact-head review, queue, merge and fresh-main verification.

## Manifest

```text
MANIFEST
- origin/main: 6434f9ab3415ec99c02da2934643bf98ced56f3a  check: git rev-parse origin/main
- CUDA merge-forward: 76f9aa3e2  check: git merge-base --is-ancestor 76f9aa3e2 HEAD
- PR 3968 head: current branch HEAD at push  check: test "$(git rev-parse HEAD)" = "$(gh pr view 3968 --json headRefOid -q .headRefOid)"
- shared model evidence: codex-5/REPORT.md  check: sed -n '/## Measurements/,/## Checks/p' docs/loop/tasks/native-space-state-contract/codex-5/REPORT.md
- CUDA integration tests: 4 passed  check: NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo test -p uor-r4-discrete --features cuda --test native_vector_cuda
- discrete library tests: 370 passed / 1 ignored  check: cargo test -p uor-r4-discrete --lib
- full harness package: 685 passed / 1 ignored  check: TMPDIR=/var/tmp/uor-harness.KQCj53 cargo test --locked -p uor-r4-harness
- CUDA PR: 3968 based on main  check: gh pr view 3968 --json state,isDraft,baseRefName
- next: integer serving consumes the shared Q31 model state  check: top of this file
```
