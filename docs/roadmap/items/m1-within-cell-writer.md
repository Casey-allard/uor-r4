---
id: m1-within-cell-writer
title: Improve within-cell token selection with versioned reads
milestone: M1
area: mechanism
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-bvg-k-stage-t-training]
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-within-cell-writer*, docs/labs/m1-within-cell-writer/**, docs/loop/tasks/m1-within-cell-writer/**, docs/mechanisms/discrete-learning.md, docs/research/notebook/m1-discrete-attention.md]
---

## Why it matters

Stage T (#2836) admits 5,009 of 14,287 target cells on T16_fz, but OuterL1V1 selects only 455 top-1. On the same admitted cells, exact TRAIN-built read-code and successor tables score 2,140 and 3,084. Full-vector addressing (#2834) also raised admission while checkpoint-0 top-1 fell. The first bottleneck is selection inside admitted cells, not shortlist reach or vocabulary-scan cost.

## What is missing

A writer that recovers the information already present in the read without confusing admission gains with token-selection gains. #3128 proposes a new snap/relation/frame law; #3129 proposes conditioning interior reads on predicted endpoints. These are versioned mechanism changes, not repairs to the registered old law. Old checkpoints and C8/C10 controls keep their behavior.

The decisive work starts with error decomposition and a zero-update read experiment (B0), not a training campaign. The primary B0 comparison is two-pass endpoint-output framing against nearest-endpoint single-pass framing on exact eight-bit interior-key accuracy over the intersection of admitted rows. Simpler controls include a 16→16 TRAIN-estimated frame remap, frozen/permuted frames, a within-cell read-code reranker, and a permuted-code null. `G`, `T_frozen` and `T_rebuilt` remain distinct TRAIN-only controls.

ReadLawV2 caps SameToken/SameFrame accumulation at the most recent eight matches per edge type per node, resets relation scores per sweep, and checks its integer bound. The 256/256 weighted-snap enumeration is only an arithmetic sanity check; multi-level, negative-gate, residual-mode, entropy and code-coverage controls are required. Old artifacts use retained legacy binaries or deterministic old-law replay; stored hashes are never edited and re-derived snapshots are not called the same checkpoint.

Two passes add no target information, and a four-bit endpoint output cannot recover an eight-bit interior key; a fixture that merely asserts recovery is vacuous. Exactly zero non-inertness refuses the two-pass configuration. T16_fz evidences snap/transport only, while T16_rel is required for relation evaluation; old-checkpoint V2 evaluation is out-of-distribution and fresh training is required for a relation-trained claim. Promotion requires beating both the matched current law and the absolute `G`/T2 comparison on the same population, not merely beating 455.

## Done when

- Saved Stage T outputs decompose the 455 versus 2,140 gap into admission, top-8 candidate coverage, within-cell L1 ranking and DEV read-code coverage, including a TRAIN-only within-cell reranker control.
- B0 compares current single-pass, nearest-endpoint single-pass, two-pass, frozen/permuted frames, a 16→16 frame remap, lookup controls and the within-cell reranker on the same re-derived epoch-0 FullVectorV1 table under OuterL1V1 and frozen DEV rows.
- The primary comparison uses the strict paired rule `d > 24 && d^2 > 4(b+c)` on eight-bit interior-key accuracy over the pre-registered intersection of admitted rows. Total top-1, intersection-admitted top-1, admission, code coverage, non-inertness, comparison counts, resident bytes and throughput are reported separately.
- New-law controls pin bounded arithmetic, negative gates, multiple active levels, `s[12]=1/2`, direct copy succession, frame selection on TRAIN/validation, and old-lineage parity. Every negative gate is shown failing before its implementation passes.
- #3128 and #3129 close only when their acceptance controls and the relevant matched receipts are sealed. A negative closes only the tested configuration. Training requires a positive B0 plus a new sheet, component ablations, throughput projection and distinct data-order or initialization replication. D11 is a separate later unit.

## How to start

1. Read the Stage T record, #2834's missing outer-key arm, `discrete.cu`, `address_cells.rs`, `w2_capacity.rs`, and issues #3128/#3129. Preserve old-law identity and do not touch #3258's or #3264's active branches.
2. Produce the error decomposition first. If the within-cell reranker captures the gap, pursue writer scoring rather than another read law.
3. Write the B0 ORDER and one-page GPU sheet with exact arms, common-support statistics, permuted nulls, stop rules, artifact identities and complete costs. Obtain non-author FIT and the lab's D67 pre-run review before GPU evaluation.
4. Implement only the declared B0 mechanisms with failing host and finite GPU controls first. Refuse inert position modes, block counts other than four, and any identity bypass. Report one primary comparison; secondary readouts do not create repeated opportunities to pass.
5. If two-pass does not beat nearest-endpoint single-pass, or frozen/permuted frames tie it, stop that configuration. Register the exact-memory lookup/reranker successor instead of adding capacity. If it wins, register component-wise ReadLawV2 training before any D11 work.
6. Open a PR saying `Closes #3128` or `Closes #3129` only for the unit that meets that issue's acceptance conditions; otherwise use `References`. Add one notebook row with the measured result and exact uncertainty.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): no softmax/squashing, dense matmul/GEMM, neural grader or floating-point discrete computation.
- **GPU only** (§3.5): training, evaluation and qualification run on GPU. Static host checks and grading of stored outputs may run on the laptop.
- **Pre-registration first** (§4.5): every GPU evaluation needs a one-page sheet and non-author FIT; every run needs the lab's D67 pre-run review.
- **Evidence** (§4.1–§4.6): a negative closes a configuration, never a mechanism. Finite controls are not trained capability. Fresh transfer remains `m1-fresh-panel-transfer`.
- **Rust first and preservation** (§3.11, §4.7): no new Python, no edited artifact hashes, and all negative receipts are preserved.

## Sources

- [`docs/labs/m1-bvg-k-stage-t-training/README.md`](../../../docs/labs/m1-bvg-k-stage-t-training/README.md)
- [`docs/loop/tasks/m1-address-key-full-vector/claude-110/REPORT.md`](../../../docs/loop/tasks/m1-address-key-full-vector/claude-110/REPORT.md)
- [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu)
- [`crates/uor-r4-discrete/src/address_cells.rs`](../../../crates/uor-r4-discrete/src/address_cells.rs)
- [`crates/uor-r4-discrete/src/w2_capacity.rs`](../../../crates/uor-r4-discrete/src/w2_capacity.rs)
- [Issue #3128](https://github.com/UOR-Foundation/uor-r4/issues/3128) and [issue #3129](https://github.com/UOR-Foundation/uor-r4/issues/3129)
