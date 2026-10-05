# Step 0d: D11 read share at full context (96M)

Status: measured behavior on one laptop under shared load. Training-free. References #820.

## Question and frozen rule

What share of a D11 serving step goes to the read (the scan over cached
positions) when the context is full? The rule in
[final-assessment.md](../final-assessment.md) (Step 0d) was frozen before the run:

- If the read share is under 30% at full context, KV-sparsity and D5-admission
  work is closed, and D5 effort goes to weight-row routing.
- Otherwise, read cost matters at 384.

## Setup

- **Artifact:** `~/uor-r4-local/d11-chat/c100-export/model.lut`, sha256 `abdadaf7…fbafd97`.
  The 96M pointer model has width 1024, 16 heads, MLP 704, pattern
  `rrarrarrarrarr` (4 read layers, 10 recurrence layers), L2 read, context 384
  and vocabulary 4096.
- **Engine:** `geometric-stack lut-chat engine=d11`, rebuilt at `416e25a3`,
  which includes the #1691 pool.
- **Prompts:** 4 single-turn requests, each made by joining open
  `heldout-200-a`/`-b` user turns. Exact history lengths under protocol 2 are
  378, 380, 373 and 374 ids. Generation is capped at 4 ids, so the served
  histories reach 382, 384, 377 and 378.
- **Bit identity:** the lut-chat replies at 1 thread and at 4 threads are
  identical, 16 of 16 ids.
- **Profiler and tool:** `/usr/bin/sample` at 1 ms intervals. The new
  `examples/d11-read-share.rs` builds the prompts and times every step on its
  own (unprofiled prefill). After a marker it replays positions 370 to the end
  of each history 60 times from a saved session, so a sampler attached at the
  marker sees only full-context steps.
- **Categories:** `step0d/sample_shares.py` walks each sample from the leaf
  upward and takes the first kernel it recognizes.
  - **Weight maps:** `stack_gemv*`, the pair/activation tables and the
    `stack_map_*` tasks. This includes the read layers' q/k/v/null/out
    projections.
  - **Read:** `stack_heads`, L2 distance (`stack_square`, `stack_isqrt`), the
    softmax tables (`stack_exp_neg`) and `stack_mix_row`.
  - **Recurrence:** `stack_recurrence`, `stack_lanes` and the rotations.
  - **Pointer:** `stack_pointer`.
  - **Other:** everything else.
  - **Critical view:** counts only the thread that runs `stack_step`, and
    charges that thread's waits at a join to the phase it waits in.

## Results

Sampled shares of busy samples at full context (positions 370 to 383):

| view | weight maps | read | recurrence | pointer | other |
|---|---|---|---|---|---|
| 1 thread, hot | 63.7% | **28.3%** | 6.5% | 0.1% | 1.4% |
| 4 threads, hot, all workers' CPU | 68.9% | **24.1%** | 4.7% | 0.1% | 2.3% |
| 4 threads, hot, critical path | 67.8% | **22.3%** | 6.5% | 0.2% | 3.1% |
| 1 thread, whole lut-chat run (prefill 0 to 380, then 4 ids) | 73.0% | 17.3% | 7.6% | 0.1% | 1.9% |
| 4 threads, whole lut-chat run | 75.7% | 15.8% | 5.7% | 0.1% | 2.7% |

Unprofiled step times, from the median step per position over the 4 prompts
and a Theil-Sen fit:

| threads | step at position 0 | per position | step at 383 | position-dependent share at 383 | context where it reaches 30% |
|---|---|---|---|---|---|
| 1 | 35.2 ms | 36.4 µs | 49.1 ms | **28.4%** | about 414 |
| 4 | 16.8 ms | 13.4 µs | 21.9 ms | **23.3%** | about 539 |

The read's own kernels at 1 thread, full context, as a share of the whole step:

| kernel | share |
|---|---|
| `stack_square` (L2 distance) | 14.3% |
| `stack_mix_row` | 4.9% |
| `stack_isqrt` (one per score) | 4.3% |
| `stack_l2_distance` | 3.9% |
| softmax tables | about 0.7% |

## Decision

The read share at full context is under 30% at both thread counts. The
sampled share and the timing fit agree: 28.3% and 28.4% at 1 thread, and
22.3% to 24.1% against 23.3% at 4 threads.

**Rule outcome:** KV sparsity / D5 admission work is closed; D5 effort goes to weight-row routing.

## Limits

- **The margin is thin.** At 1 thread the read is 28%, two points under the
  threshold. Read cost grows linearly with position, so at 1 thread it would
  pass 30% at a context of about 414. Any context extension past 384 reopens
  this decision.
- **#1691 understated the read.** Its figure of about 10% came from
  everyday-32, whose contexts are short. Averaged over a filling prefill the
  read is 17%, and at full context it is 28%.
- **The read cost is arithmetic, not KV volume.** About 22.5 of the 28 points
  are the exact L2 distance: a table-built `stack_square` for each element,
  plus one `stack_isqrt` for each score. A cheaper exact L2 kernel would shrink
  the read without any sparsity. One option is ‖q‖² − 2q·k + ‖k‖², using the
  existing query tables and cached key norms.
- **Load.** The machine was shared, with a load average of 7 to 15 on 8 cores.
  The 4-thread wall times are noisy. Under the sampler, the 4-thread lut-chat
  run took 68.6 s against 65.9 s at 1 thread. The sampler also inflates
  wall-clock time, so the throughput fields in the `chat.json` files are not
  speed results.
- **Scope.** One artifact, 4 prompts. The shares measure CPU time, not energy.
- **Report root:** `~/uor-r4-local/step0/d-profile/`. Its manifest
  `SHA256SUMS.txt` has sha256 `b9b81c59…e4397`.
  - `prompts/` was an earlier, shorter build (350 to 369 ids).
  - `prompts-2/` ended in error.
  - Both are kept.
