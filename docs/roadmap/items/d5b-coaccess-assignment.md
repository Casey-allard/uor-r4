---
id: d5b-coaccess-assignment
title: Test D5-b address assignment with measured co-access and distance
milestone: M5
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-integer/src/stack/memory.rs, xtask/src/main.rs, xtask/src/d5b_coaccess.rs, docs/labs/d5b-coaccess-assignment/**, docs/roadmap/items/d5b-coaccess-assignment.md, docs/loop/tasks/d5b-coaccess-assignment/**]
---

## Why it matters

An offline assignment objective needs workload co-access and cell distances before its effect on per-token access can be measured.

## What is missing

The owner ratified D38’s residency/sparsity split in October. The D5-b assignment instrument still lacks a workload co-access matrix and an objective that uses its distance term. The D5-a residency measurement belongs to #2862. [Issue 2657](https://github.com/UOR-Foundation/uor-r4/issues/2657#issuecomment-6301040345) · [PR 2671](https://github.com/UOR-Foundation/uor-r4/pull/2671)

## Done when

Export the real co-access matrix, use the distance term in the objective, and record the comparison with the current assignment. The D5-a residency measure has its own item.

## How to start

Read the owner’s D38 decision and the maintainer’s clarification of the remaining scope.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/2657)

<details>
<summary>Original report by Cyborg-Hawk-AI on 2026-10-18</summary>

## Goal

Split D5 (per-token parameter sparsity as the terminal invariant) into two claims with different achievability:

- **D5-a:** DRAM bytes/token → 0 after warm-up. Achievable today for small models by residency (the artifact fits in cache). Probably already true for the current 3.6 MB artifact — it just cannot be seen without hardware counters.
- **D5-b:** parameter sparsity. Only binds when the model exceeds cache.

Their own first-principles review (2026-09-25) states the floor as *bytes of parameters touched per token*; residency answers that floor for small models, sparsity is only the big-model path. Right now D5 demands the hard half and gets no credit for the half that may already pass.

## The D5-b formulation (design memo, not run)

One-hot QUBO assigning memory keys to fixed H4/E8 codebook cells, minimizing co-access-weighted collisions (`ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md` in PR #2654). Solver is classical simulated annealing; **no quantum device expedites this today** (D-Wave left Braket 2022; a 108-qubit QPU cannot hold one 289-variable subproblem).

## Cost

The measurement needs their memory co-access statistics (`A[k1,k2]` from the development panels), which I do not have.

</details>
