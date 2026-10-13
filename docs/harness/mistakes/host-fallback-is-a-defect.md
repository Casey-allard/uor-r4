---
id: host-fallback-is-a-defect
title: A host-side fallback presented as a cost, not as a defect
date: 2026-10-12
severity: block
class: compute
trigger: [host fallback, host-side fallback, CPU training, ran on the laptop, cuda_covered, device=cuda, step time]
paths: [crates/**, docs/labs/**, docs/integration/DECISIONS.md]
milestones: [m1, m4]
mechanism: starling-read
guard: prlint.no_cpu_model_compute
evidence: [docs/labs/softmax-free-read-2026-10-10/README.md, docs/labs/no-cpu-gpu-only-2026-10-12/README.md, docs/integration/DECISIONS.md]
status: active
---

## What happened

The softmax-free read record wrote, in its own mechanism description, that the read runs
"CPU, and on CUDA through the host fallback, which every flock-selected read already
uses". Measured at the same time, the flock-gated read was slow in step time. D24 was
written on 12 October because a host fallback had been reported as a cost of the
mechanism rather than as a defect in it, and because the same configuration had been
executed under `cuda_covered() == false` while being scored as if device-resident.

## Symptom

A record or PR reports a host/CPU execution path next to model numbers, or quotes a step
time, throughput or quality figure that came from a run whose operator fell back to the
host. The number is not usable as mechanism evidence: both the arithmetic and the step
time are wrong.

## Lesson

Model compute never runs on the CPU, and a host-side fallback is a defect to fix with a
kernel - not a cost to report, and not mechanism evidence. The M1 invariant-addressed
router campaign was cancelled mid-flight for exactly this (2 of 3 arm-A seeds, pod
deleted) and recorded as a configuration cancellation, not a negative.

## How to apply

- Before quoting a step time, confirm the operator ran on the device: no
  `cuda_covered()` false path, no host copy of the forward.
- If a path lands on the host, write the blocker and the kernel it needs; do not open a
  result PR around it.
- State the CPU/GPU split in the record: data preparation and grading are CPU work on the
  owner's laptop; training, evaluation and every part of the forward that scoring or the
  value mix touches are GPU work through `scripts/pod/uor-pod`.
- `python3 scripts/harness/cli.py lint check` raises `prlint.no_cpu_model_compute` on a
  body that presents CPU or host-fallback model compute as an accepted result.
