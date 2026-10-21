---
id: m1-bvg-k-block-apply-c2
title: Draft the BVG-K Stage T pre-registration sheet (C2 has passed)
milestone: M1
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: []
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-bvg-k-block-apply-c2*, docs/labs/m1-bvg-k-block-apply-c2/**]
---

## Why it matters

BVG-K is the faster discrete learning law that Stage 2 tries first if W2 blames the learning law. It scores 16 coordinates in one GPU launch and commits a block of moves only when the joint result is strictly better. Stage C2 is the last finite qualification before a training run (Stage T) can be registered. See [the discrete learning page](../../mechanisms/discrete-learning.md).

## What is missing

Stage C2 (block propose, verify and apply) is already qualified. The lab record [injective-preflight-and-c2-qualify-2026-10-19](../../../docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) says "BVG-K C2 qualification PASS" (dispatch 6257564516, 2026-10-19, about 13 minutes, roughly $0.40). Its receipt is `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`, SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`. The controls that passed were V0 source identity, RC re-carry parity, the UT/JR/DC/V3 block sweep (decision coverage met), RF refusals and the C1R rerun of C1 parity. All 3,419 updates were on synthetic fixture devices, and `Device::apply` was never called. Stage C1 was qualified earlier: K-coordinate scoring exact (5,130/5,130), sabotage FAILs as required, K=16 scores 16 coordinates per launch at 16.25x throughput.

What remains is the Stage T pre-registration sheet. Stage T (the training run) is not registered and stays held: it needs a FIT review and a typed owner release, and the training hold is still in force. The C2 sheet also lists three things it leaves uncovered: no GPU control compares the BVG-K apply kernel with the ordinary single-move kernel; K=1 identity with today's law (V1) needs a driver transition; and driver-level controls (block-transition accounting, FrozenZeroV1, the all-ineligible no-op batch, V9 checkpoint and resume) are not covered. The Stage T sheet must say how each is covered.

## Done when

Already met: Stage C2 finite controls PASS (block apply exact) and are recorded, with the receipt above.

Still to do: a one-page Stage T pre-registration sheet is posted on the issue, complete enough for a non-author TEST FITNESS: FIT review. Nothing in it runs until FIT and an owner release. Its bar is the Stage 2 bar: "beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds." The estimate to carry is about $12 for two seeds.

## How to start

1. Read `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`, the C2 sheet `docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md` (use its layout), the row 7 entry in `docs/integration/NOW.md`, and `docs/mechanisms/discrete-learning.md`. Also read the K16 outcome record `docs/labs/k16-outcome-call-2026-10-19/README.md` (no KEEP) and the W2 entry in NOW.md, because NOW.md row 7 holds Stage T for the K16 outcome, and row 8 runs the float-credit sheet only if W2 blames the learning law (H_law). NOW.md lists W2 as queued while `docs/integration/project-track.md` marks it done, so confirm W2's status with the PM before drafting.
2. Skim the code the sheet will name: `crates/uor-r4-discrete/src/bvgk_block.rs`, `bvgk_device.rs`, `bvgk_qualification.rs` and `bvgk_source.rs`. Run the host checks with `~/.cargo/bin/cargo check -p uor-r4-discrete`. If the crate does not build without nvcc on your machine, say so on the issue; do not start a GPU run.
3. First deliverable, posted on the issue: a short design note for Stage T. It states the geometric construction (K coordinates scored per launch, a block committed only if `l_joint < l_best`), why it uses no softmax or squashing and no matmul (integer hinge loss, table moves, adds), and the finite controls that already pass. It also says how the three uncovered items above are covered: as extra finite controls before the run, or as checks inside it.
4. Draft the Stage T sheet in `docs/harness/prep/` (for example `m1-bvgk-stage-t-training-<date>.md`). It needs: the data and panel with hashes; the two seeds; the arms; the comparators (unigram, then the admission-matched next-word table); the numeric stop rules and bar; the cost cap; the lease and GPU count; the hold status; and "what this leaves uncovered". Leave any number the sources do not give as "the bar is not yet registered" and ask the PM for it. Do not type an owner release phrase; the owner types it personally.
5. Request a non-author FIT review on the owning issue. Do not launch anything.
6. Open a PR with the sheet that says `Closes #<this issue>` (or `References` if the sheet needs another round), and add one row to `docs/research/notebook/m1-discrete-attention.md` (the M1 notebook page) that records C2 PASS (the receipt SHA-256 above) and the sheet as the next step.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:66`](../../../docs/integration/NOW.md)
- [`docs/start/where-were-going.md:46`](../../../docs/start/where-were-going.md)
- [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`](../../../docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md)
- [`docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`](../../../docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md)
