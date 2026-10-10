# Softmax-free reads for the served stack: rank-table and Hamming-rank flock reads — October 10

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032), acceptance item 0 (no softmax at runtime). Pre-registration: [#2032 comment 6093484433](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433) (arms A–D: 18:29Z and 20:35Z cards of 9 October).

**Status: preparations 1 and 2 of 2.** The trainer mechanism (#2140) and the multiplier-free integer serving of both reads (this record's second section) are in. The result, KEEP or REJECT against the pre-registered bar, is added by the result PR.

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

## Serving (preparation 2): the D11 engine serves both reads

- **Artifact:** schema `uor-r4.lut-stack/3`. `/2` already means a pointer stack, and the frozen D10 engine's exact-schema check refuses `/3`, so no older engine can serve a rank artifact as softmax. Its `shape` adds `read_select {window, k}` (the sink is always position 0), `read_weights "rank"` and `read_binary`.
  - The exporter writes `/3` only for a flock model with a rank or hamming-rank weighting. Every other refusal still applies: flock with softmax, a non-zero sink, U(1) transport (a review finding, pinned by a test).
- **Rank read:** per head, the engine's own scores plus age go through the allocation-free integer flock selector, now a bounded insertion top-k. The library `select_nth_unstable_by` and `sort_unstable_by` compiled to `madd`/`mul`, which the audit flagged as reachable callees.
  - NoRead is ranked among the kept sources and loses ties. The weights are the Q31 `1/(r+1)` tables precomputed at load for each support size.
  - Each read touches at most `window + k + 1` value rows (17 for `flock:8:8`), against `t + 1` for the softmax read.
- **Hamming-rank read:** `BitCode` is a sign-bit code of up to 256 lanes with XOR binding. Its Hamming distance is a shift-and-add SWAR popcount: `count_ones` lowers to NEON `cnt` plus `fmov` on arm64, and to a multiply on x86 without POPCNT.
  - A per-head table indexed by `h` holds the engine's own dense score of ±ONE vectors (ONE = 2^16) that differ in `h` coordinates. A score is one XOR, a popcount per 64 lanes and a table read, plus age.
  - BitCode is a similarity code. It is not an exact identity: BLAKE3 and prime addresses are.
- **Checks at the PR head:**
  - `uor-r4-integer` lib: 282 passed.
  - Cross-engine oracle `stack_d11_oracle`: 13/13 (schema /1 and /2 serve bit-identically).
  - `stack_softmax_free_export`: 3 passed. Random 4-bit weights, D11 against float: rank −0.039 nats, hamming −0.012 nats, top-1 agreement 1.0.
  - `stack_export`: 17 passed. Frozen `uor-r4-lut`: 25 passed.
  - `audit_zero_matmul_serving.py --stack`: **FULL PASS**, 91 reachable functions. A negative gate with the library sorts restored fails on exactly those callees.
- **Evaluation:** `geometric-stack d11-evaluate ... reference=none model=ROOT/model` scores D11 against the float model on the same windows, for artifacts D10 cannot read.
