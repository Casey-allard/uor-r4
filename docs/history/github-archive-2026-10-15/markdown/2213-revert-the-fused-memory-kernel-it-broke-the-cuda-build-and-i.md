# #2213 — Revert the fused-memory kernel: it broke the CUDA build and its rate claim was not the kernel's (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T21:17:45Z
- **Updated:** 2026-10-11T21:27:06Z
- **Closed:** 2026-10-11T21:23:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2213
- **Merged:** 2026-10-11T21:23:44Z
- **Merge commit:** 18133a9c37cdb099e7b595752e72ad30481c2eb2
- **Base / Head:** main / UOR-Foundation:deepseek/revert-fused-kernel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Two errors in one merged PR ([#2212](https://github.com/UOR-Foundation/uor-r4/pull/2212)), found within the hour, both reverted here.**

**1. The kernel broke the CUDA build.** A pod bootstrap at the merged sha fails with three errors in `uor-r4-training` under `--features cuda` — `E0308` (the launcher's `Option<Forward>` used where the `cuda_fwd` path expects the tuple) and `E0603` (a private item). **The local non-CUDA build and CI both passed because neither compiles the CUDA feature**, so a CUDA-only regression reached `main` that would have broken every CUDA build of the training stack.

**2. The rate claim was not attributable to the kernel.** That cycle's pod was bootstrapped with `--ref <main sha>`, and the tool fetches that sha **from GitHub** — so the binary contained `main` **without** the kernel edits. The "≈12× step improvement" is the **fat-versus-lean codebook** difference (1.19 s/step against 0.175 s/step), two configurations already known to differ, and the CPU/CUDA agreement (~1e-9) compared **one binary with itself**.

## What this PR does

- **Reverts** the kernel change, restoring the CUDA build.
- **Restores the record with a retraction banner** rather than deleting it, and adds a correction record: how the errors were found, and the practice changes below.
- **Prepends a retraction entry** to `current-state.md`.
- The fused forward's source remains in the reverted commit as a **starting point**, not a result.

## Practice changes, effective immediately

1. **A kernel change is not verified until a pod built from the branch that contains it runs it** — `--ref` must name the branch head sha, and the compile output is read rather than assumed.
2. **CUDA-only paths get a CUDA compile before merge** (the pod bootstrap *is* that compile; it takes minutes).
3. **Every rate claim names its binary's ref and the configurations compared**, so a configuration difference can never be read as a kernel effect.

Line: fused address kernel · count 0/3 · headline **unchanged** — the retracted claim is withdrawn, not replaced

References #2029
PR; sleep 240; gh pr checks 2>&1 | grep -v skipping | head -2; gh pr merge --squash --delete-branch=false 2>&1 | tail -1


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

