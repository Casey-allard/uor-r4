---
id: icosian-snap-parity
title: Build a lawful GPU comparator for the historical S1.4 icosian snap replay
milestone: research
area: evaluation
status: ready
open_to: anyone
needs_gpu: true
depends_on: []
scope: [docs/harness/prep/icosian-snap-parity*, docs/labs/icosian-snap-parity/**, docs/roadmap/items/icosian-snap-parity.md, docs/loop/tasks/icosian-snap-parity/**]
---

## Why it matters

The September 29 extract reports 2,477 root mismatches in 1,177,878 selections and top-1 agreement of 0.991943359375 across 4,096 positions in 16 windows. The source directory lacked a `report_output` manifest. The extract marks it `UNSEALED` and records separate SHA-256 hashes for `attempt.json` and `snap-parity.json`. These are the historical values an independent measurement can check. [Hash-bound S1.4 extract](../../../archive/evidence/s1-4-snap-parity-2026-09-29.json)

## What is missing

**First deliverable.** Build and review a separate GPU comparator for the root selections and unnormalized top-1 choices. The public inputs include the model, held-out tokens and `SHA256SUMS`. The legacy Rust example loads `Device::Cpu` and computes log-softmax during scoring. Its program and original invocation are historical evidence. A lawful comparator must execute the model comparison on the GPU, exclude banned calculations from its compiled and loaded paths, and preserve the registered selection and tie rules. [Legacy implementation](../../../crates/uor-r4-training/src/stack_snap_parity.rs) · [Contributor handoff](https://github.com/UOR-Foundation/uor-r4/issues/2655#issuecomment-6391158177) · [Public inputs](https://huggingface.co/datasets/caseyallard/uor-r4-data/tree/main/s1-4-parity-2026-09-29)

## Done when

A reviewed GPU comparator reproduces the named measurements on the pinned inputs under its own FIT-reviewed sheet. Report the selections, mismatches, top-1 agreement, input and comparator hashes, and hardware. Record any difference from the hash-bound historical counts using the registered comparison.

## How to start

Start with the comparator repair in a separately built Rust/CUDA instrument under `docs/labs/icosian-snap-parity/`. Check root selection, ties, and raw-score argmax on finite controls. An independent replay can proceed only after the comparator passes review, its sheet has a non-author FIT, and the applicable GPU pre-run review passes. The original CPU command is outside this registration. [Charter §§3.2–3.5 and §4.5](../../../CHARTER.md)

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/2655)

<details>
<summary>Original report by Cyborg-Hawk-AI on 2026-10-18</summary>

## Goal

Serve the 2I icosian transport snap (the S4-adopted main line's transport) as a D11 kernel, with parity against the sealed snap-parity record.

## What exists (measured)

The snap path (`stack_snap_select`, `stack_snap_rotation`) is Z[φ] shift-add argmax over the 120 roots + the 120×120 group table (`uor-r4-core/src/native_geometric/learner/group_table.rs`), with the exact Z[φ] tie-break. On x86_64 it audits R2-clean:

- `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack` = **FULL PASS** (0 forbidden instructions, 44 symbol ranges).

## Frozen gate (pre-registration)

Parity against `docs/evidence/s1-4-snap-parity-2026-09-29.json` (the sealed record, top1 agreement 0.9919, root mismatches 2477/1177878). Pre-registering the run before any fit, per the rules.

## Cost

The parity run needs the S4-arm artifact and the chat-v0 held-out set, which I do not have. The kernel and the native audit are in PR #2654.

</details>
