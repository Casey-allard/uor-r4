# Arm S (SoftSort rank-consistent training): stopped by owner direction — October 11

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032), acceptance item 0 (no softmax at runtime). Pre-registration: [#2032 comment 6102148887](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102148887), amended at [6102170500](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102170500), `TEST FITNESS: FIT` at [6102348077](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102348077).

**Outcome: STOPPED by the owner before the decisive pair.** There is no KEEP or REJECT. The stop closes this configuration, SoftSort-trained `flock:8:8` with a learned rank table and an L2 score, as an owner decision, not as a measured negative (D22 §1). The headline is unchanged: softmax at runtime yes, served 0.886838 BPB.

## Why it stopped

The owner, in chat, about 00:10Z on 11 October (recorded on #2032):
- **Not geometric enough.** A rank or SoftSort replacement for softmax still scores by an L2 distance between learned vectors. It *"retranslate[s] it back to essentially binary at every layer"* and gains nothing over transformers.
- **Too costly for what it answers.** Runs should answer stated questions, not *"brute force guess and check"*.
- **Direction:** a read built on ring products, with the Z/256 byte ring as the target because it is cheapest in hardware. The design is posted for owner review at [#2032 comment 6103724735](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6103724735).

## What ran

| run | binary | steps reached | result |
|---|---|---:|---|
| retune ×4 (seed 1, 3,000 steps) | f07d2ff01 | < 1,000 | stopped: no step-1,000 eval after 45 min; one CPU core at 100 % |
| retune ×4 | 6a9a24f6 (#2182) | < 1,000 | stopped: 5,315 tok/s, still too slow |
| retune ×4 | 1b1d11f50 (gather fix) | 1,000 | stopped by owner direction. Dev NLL at step 1,000 on 64 windows: lr 4e-4/wu 500 **2.726**, lr 4e-4/wu 1500 **2.956** |
| **A1536** (softmax control, ctx 1536, seed 1, 12,207 steps) | f07d2ff01 | 12,207 | float held-out NLL **1.4916751** on 512 windows × 1536 (786,432 targets), stream basis 2.837427 B/token. Served BPB and v4 were not run |

The A1536 model, the data copy and the partial retune roots are on the network volume under `/workspace/uor-r4/claude/softsort-s-20261010/`. A laptop copy of the A1536 model is kept under the lab's local work root.

## Tooling found and fixed (blockers, not negatives)

1. **Serial host selection.** `softsort_learned_read` picked each row's flock support in a serial loop of about 98k rows per read layer per step. It now runs under rayon with identical output: [#2182](https://github.com/UOR-Foundation/uor-r4/pull/2182), merged as `d97fe3989`. The same PR adds `evaluate softsort_tau=T max_blocks=N` for the soft-versus-hard gap.
2. **candle 0.9.2's CUDA `index_add`** (the backward of `index_select`) gives each thread one output column and loops over every index (`candle-kernels/src/indexing.cu:133-137`). For the kept-value fetch that is 64 threads over 1.77M ids. The fix gathers along time and makes the rank-table row a one-hot matmul: ×3.3 measured (5,315 → 17,415 tok/s, 50 steps, 5090). It was not merged, because its run stopped. It is archived as [`claude_softsort-gather-20261011.patch`](../../history/branch-archive/claude_softsort-gather-20261011.patch).
3. **A related candle bug:** the backward of `ScatterAdd` builds its mask with `scatter(indexes, zeros_like(init))`, which fails whenever the index width differs from the destination's. A dense-scatter version of the read failed the finite-difference gradient test for this reason.

## Cost

The Claude lab's pods `jg1ztimihpt7m5` (2×5090) and `0ped5ywrvpmdbj` (1×5090) ran from about 21:48Z to 00:10Z. That is about 2.4 h × $3.57/h, roughly **$8.50**, of which A1536 used about $1.80. Both pods are deleted.

## Lesson for the next design

Each arm on this line asked "does read variant X land within 0.01 BPB of A?" and needed hours of GPU to answer. None asked why a read fails. The [ring-read design](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6103724735) is a ladder:
- rungs at $0 on the laptop: exact arithmetic, then retrieval trainability on `mqar-bench`, then the recall cost of exact addressing;
- then a $2 language check calibrated against these runs;
- a full run only after that.
