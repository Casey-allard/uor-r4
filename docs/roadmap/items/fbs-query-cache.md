---
id: fbs-query-cache
title: Cache teacher-forced FBS queries for CPU readout experiments
milestone: research
area: evaluation
status: done
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

Freezing teacher-forced queries makes it possible to compare readout candidates without repeating trunk inference.

## What is missing

The cache, exact replay, matched centroid/shuffled scoring and corruption checks shipped as development instrumentation. The separate two-seed capture package records its own execution requirements. [PR 3803](https://github.com/UOR-Foundation/uor-r4/pull/3803) · [PR 3805](https://github.com/UOR-Foundation/uor-r4/pull/3805) · [PR 3806](https://github.com/UOR-Foundation/uor-r4/pull/3806) · [PR 3807](https://github.com/UOR-Foundation/uor-r4/pull/3807)

## Done when

The optional capture preserves the normal scorer, binds the checkpoint and every split, rejects corrupt or overwritten input, reproduces frozen decisions and reports paired centroid and shuffled-control top-1/top-8. Rank-aware candidates are optional.

## How to start

Read the implementation, refusal tests and registered capture package for the exact instrument boundary.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3801)

<details>
<summary>Original report from 2026-11-06</summary>

## Goal

Emit an immutable, checkpoint- and split-bound cache of teacher-forced FBS query points and their targets, then evaluate centroid and matched output-table candidates on the workstation CPU without rerunning the trunk.

## Boundary

This is development instrumentation. It does not train the trunk, alter optimizer or micro-batch behavior, change serving defaults, or qualify a model.

## Done when

- `--score-rows` can optionally emit the cache without changing existing scoring behavior.
- The CPU evaluator verifies the complete cache identity and reproduces the frozen original decisions.
- Centroid, target-shuffled, and optional rank-aware candidates preserve cached queries and report top-1/top-8 on the same panel.
- Tests cover malformed, reordered, overwritten, mixed-checkpoint and changed-query inputs.

</details>
