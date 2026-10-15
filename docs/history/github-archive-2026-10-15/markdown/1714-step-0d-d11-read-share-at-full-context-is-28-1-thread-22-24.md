# #1714 — Step 0d: D11 read share at full context is 28% (1 thread) / 22-24% (4 threads) on 96M (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T03:49:25Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-05T03:57:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1714
- **Merged:** 2026-10-05T03:57:25Z
- **Merge commit:** 578c3073c858db5d5e7ed57b3201ad980cc44758
- **Base / Head:** main / UOR-Foundation:codex/step0d-d11-read-share
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This is the Step 0d instrument from the [5 October barrier assessment](docs/research/barrier-assessment-2026-10-05/final-assessment.md). It is training-free. It measures the share of a D11 serving step that goes to the read when the 96M artifact's context is full.

**Decision-rule outcome:** KV sparsity / D5 admission work is closed; D5 effort goes to weight-row routing.

At full context, the read is **28.3%** of a 1-thread step and **22.3–24.1%** at 4 threads, both under the frozen 30% threshold. The margin at 1 thread is thin. See the limits.

## Setup

- **Artifact:** `~/uor-r4-local/d11-chat/c100-export/model.lut`, sha256 `abdadaf7…fbafd97`. It is the 96M pointer model: width 1024, 16 heads, pattern `rrarrarrarrarr`, L2 read, context 384.
- **Rebuild:** `geometric-stack` was rebuilt at `416e25a3`, which includes the #1691 pool, and served with `lut-chat engine=d11` at `threads=1` and `threads=4`.
- **Prompts:** 4 single-turn prompts, each joining open `heldout-200-a`/`-b` user turns. Exact protocol-2 histories are **378, 380, 373 and 374 ids**. With `max_new_tokens=4`, the served histories reach 382, 384, 377 and 378.
- **Bit identity:** the replies at 1 and at 4 threads are identical, 16 of 16 ids.
- **Profiling:** macOS `sample` at 1 ms. The new `examples/d11-read-share.rs` builds the prompts and times every step unprofiled. After a marker it replays positions 370 to 383 sixty times from a saved session, so the sampler sees only full-context steps.
- **Categories** (`step0d/sample_shares.py`): the first kernel recognized when walking up from the leaf.
  - Weight maps include the read layers' q/k/v/null/out projections.
  - Read is the scan over cached positions: L2 distance, softmax tables and mixture.

## Results

Sampled shares of busy samples, full context:

| view | weight maps | read | recurrence | pointer | other |
|---|---|---|---|---|---|
| 1 thread, hot | 63.7% | **28.3%** | 6.5% | 0.1% | 1.4% |
| 4 threads, hot, all workers' CPU | 68.9% | **24.1%** | 4.7% | 0.1% | 2.3% |
| 4 threads, hot, critical path (the `stack_step` thread, join waits charged to the phase) | 67.8% | **22.3%** | 6.5% | 0.2% | 3.1% |
| 1 thread, whole lut-chat run (prefill from 0, then 4 ids) | 73.0% | 17.3% | 7.6% | 0.1% | 1.9% |
| 4 threads, whole lut-chat run | 75.7% | 15.8% | 5.7% | 0.1% | 2.7% |

Unprofiled step time against position (median over the 4 prompts, Theil-Sen fit):

| threads | step at position 0 | per position | step at 383 | position-dependent share | context where it reaches 30% |
|---|---|---|---|---|---|
| 1 | 35.2 ms | 36.4 µs | 49.1 ms | **28.4%** | ≈414 |
| 4 | 16.8 ms | 13.4 µs | 21.9 ms | **23.3%** | ≈539 |

At 1 thread and full context, the read splits into these kernels:

| kernel | share of the whole step |
|---|---|
| `stack_square` | 14.3% |
| `stack_mix_row` | 4.9% |
| `stack_isqrt` | 4.3% |
| `stack_l2_distance` | 3.9% |
| softmax tables | ≈0.7% |

## Limits

- **The margin is thin.** At 1 thread the read is 28%. It grows linearly with position and would pass 30% at a context of about 414. Any context extension past 384 reopens this decision.
- **#1691's figure of about 10% read was from short everyday-32 contexts.** Averaged over a filling prefill the read is 17%, and at full context it is 28%.
- **Most of the read cost is the exact L2 arithmetic, not KV volume.** It is a table-built square for each element plus an `isqrt` for each score, about 22.5 of the 28 points. A cheaper exact L2 kernel would cut it without any sparsity, for example ‖q‖² − 2q·k + ‖k‖² with the existing query tables and cached key norms. That is a kernel task, not D5 admission.
- **Shared machine.** The load average was 7 to 15 on 8 cores, so the 4-thread wall times are noisy. The sampler slows the pool: profiled lut-chat took 68.6 s at 4 threads against 65.9 s at 1 thread. Treat the throughput figures in the `chat.json` files as profiled, not as speed results.
- **Scope.** One artifact and 4 prompts. The shares measure CPU time, not energy.

## Artifacts

- **Report root:** `~/uor-r4-local/step0/d-profile/`, claimed exclusively.
  - It holds the sealed `prompts-3/`, `steps-t1/`, `hot-t1/`, `hot-t4/`, `chat-t1/` and `chat-t4/`, the `sample-*.txt` profiles and `analysis/`.
  - `SHA256SUMS.txt` lists 53 files and has sha256 `b9b81c59…e4397`.
  - Two earlier attempts are kept: `prompts/`, which was too short at 350–369 ids, and `prompts-2/`, which ended in error.
- **Record:** `docs/research/barrier-assessment-2026-10-05/step0d/README.md`.

## Checks at this head

- `cargo fmt --check -p uor-r4-training`: pass.
- `cargo test --release -p uor-r4-training --example d11-read-share`: 2 passed.
- `python3 scripts/check_claim_wording.py`: pass.
- Both modes of the new example were run end to end on the real artifact, above.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5987827824 — 2026-10-05T03:57:02Z — Casey-allard

Exact-head review at a8b9fe726f91bdec21d9e57cebfe9c1d818f29da: **approved**.
- **Scope:** a new example, `d11-read-share`, plus the step0d write-up and two small tabulation scripts. No model or serving code changes.
- **Checks:** `cargo fmt --check` passes, and `cargo check --example d11-read-share` passes at this head.
- **Decision outcome:** KV sparsity and D5 admission work are closed at context 384; the thin margin is noted (it reopens past a context of about 414).

Queuing.


---
_Comment/review ids in this file: 5987827824_

