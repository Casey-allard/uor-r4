# Softmax-free reads for the served stack: rank-table and Hamming-rank flock reads — October 10

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032), acceptance item 0 (no softmax at runtime). Pre-registration: [#2032 comment 6093484433](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433) (arms A–D: 18:29Z and 20:35Z cards of 9 October).

**Status: preparation 1 of 2.** This record covers the trainer mechanism, which arm group 1 (A, B, D) trains with. The result, KEEP or REJECT against the pre-registered bar, is added by the result PR. The integer serving port of the two reads is preparation 2.

## The question

The served stack reads still weight their sources with an exp-table softmax. Can a read whose weights come from a fixed rank table (no exponential) replace it without a measurable loss?

## The mechanism (trainer, `geometric_stack.rs`)

`StackModel::set_read_weighting(ReadWeighting)`, saved as `config.json` field `read_weighting`, absent for softmax:

- **`rank` (arm B):** each read row keeps the flock support of `select=flock:W:K`: the sink at position 0, the last `W` positions, and the top `K` of the rest by score (the shared `flock_select`).
  - The NoRead slot is ranked among the kept sources and loses ties.
  - With `m` = kept + 1, the weights are `w_r = (1/(r+1)) / Σ_{i<m} 1/(i+1)` by rank. Unkept sources get exactly 0.
  - This is the starling rule: each source is weighted by its rank among the nearest few, not by an exponential of its score.
- **Gradient:** straight-through. The query, key and auxiliary gradients are exactly those of the flock-softmax read on the same scores. The value gradient uses the forward rank weights. Both passes use the same selection.
- **`hamming_rank` (arm D):** the projected query and key are binarized by a straight-through sign (±1, `sign(0) = +1`) before the read, then weighted by rank.
  - For ±1 vectors the L2 read's squared distance is `4 · popcount(q ⊕ k)` (the Lorentz excess is `2 · popcount`), so the score is a Hamming rank.
  - The served kernel can therefore compute it with XOR and popcount.
- **Where it runs:** CPU, and on CUDA through the host fallback, which every flock-selected read already uses.
- **What it refuses:** bf16; no flock; and a geometric address, latch, lineage or span, which replace the ordinary read.
- **CLI:** `geometric-stack train` and `dialogue-train` accept `select=flock:W:K` and `read_weighting=softmax|rank|hamming_rank`. They record both in the report and lineage, refuse a resume with a different weighting, and copy the weighting to data-parallel replicas.

## The run this prepares (arm group 1)

- **Recipe:** the M4 chat stack's recipe, the argv of `chat-served2-20261008/model-a1`: `train`, rrarrarr, w512 h8 ctx384, read l2, 19.93M params, chat-v0-p2 train, batch 32, lr 4e-4, warmup 500, 12,207 steps.
- **Arms:** seeds 1 and 2 for each of A (softmax), B (`select=flock:8:8 read_weighting=rank`) and D (`... hamming_rank`).
- **Bar:** B or D within 0.01 BPB and 2 v4 points of A, at equal or lower served cost.
- **Arm A** ran at `main` 5b241d89c (softmax path unchanged).
  - Float held-out NLL at 512 windows, 196,608 targets: **s1 1.7250754 (0.877118 BPB), s2 1.7253881 (0.877277 BPB)**, on the stream basis 2.837427 B/token.
  - s1 reproduces the recorded original run's 1.7250755 to 1e-7.
