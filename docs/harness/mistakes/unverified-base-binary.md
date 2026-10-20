---
id: unverified-base-binary
title: A rate attributed to a binary that did not contain the change
date: 2026-10-11
severity: block
class: evidence
trigger: [12x step improvement, fused kernel, retracted, --ref main sha, pod bootstrap, binary did not contain]
paths: [docs/labs/**, scripts/pod/**, docs/labs/compute.md, docs/labs/*/README.md]
milestones: [m1, m4]
mechanism: exact-addressed-memory
guard: none: verifying a pod's binary is a run-time act; the record is the only witness
evidence: [archive/labs/fused-memory-kernel-2026-10-11/README.md, archive/labs/context-curve-2026-10-11/README.md]
status: active
---

## What happened

The fused product-key memory record announced that the memory arm's training step fell
from ~2.2 s to 0.175 s (about 12x). It was retracted the same hour: the pod had been
bootstrapped with `--ref <main sha>`, which fetches from GitHub, so the binary contained
`main` **without** that cycle's kernel edits. The improvement was the difference between
the fat memory configuration measured earlier and the lean one measured here - a codebook
size, not a kernel. The CPU-versus-CUDA agreement (~1e-9) was two runs of the same
binary, so it said nothing about the kernel either. The kernel had also broken the CUDA
build, which the local non-CUDA build and CI could not see, and the change was reverted.

## Symptom

A rate, a speed-up or an agreement claim comes from a pod whose bootstrap reference was
`main` (or any ref that does not contain the change under test), or from a build whose
feature set was never compiled. The number looks clean and is about the wrong artifact.

## Lesson

Bind every measurement to the artifact that produced it. A number is a measurement of the
change only if the binary contained the change; a pod fetched from a ref is evidence
about that ref and nothing else. The same failure hides a second one: CI and the local
build passed while every CUDA build of the training stack would have failed, because
neither compiles the CUDA feature.

## How to apply

- Before a pod run meant to measure new source, verify the built binary contains it:
  bootstrap from the branch or sha that holds the change, and record that ref in the
  record next to the number.
- Compile the feature you are measuring (`--features cuda`) at the exact head, or state
  plainly that it was not compiled.
- If the ref turns out to be wrong, retract the number at the top of the record rather
  than editing it away - `archive/labs/fused-memory-kernel-2026-10-11/README.md` is the
  pattern to copy.
- Every rate comparison names the artifact, the ref, the configuration and the
  comparison, including the codebook size that this retracted 12x actually measured.
