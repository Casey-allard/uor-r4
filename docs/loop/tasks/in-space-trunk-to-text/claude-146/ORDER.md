# ORDER: in-space-trunk-to-text, claude-146 (Hamming flock attention, recall first)

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `5bc6f6522`.

## Goal

**Give the trunk real retrieval: several cheap geometric look-backs per layer, with graded credit and position, tested on recall before language.**

This unit is the owner's direction of 27 Oct (#2028, comment 6444239112), quoted: "Yes: after the content-read run is read and merged, take the Hamming attention design as the next unit, with an MQAR-style recall probe first."

claude-144 and claude-145 each closed one single-read configuration. Both landed in cell D at most seeds, with reach at most `0.0195` against `0.05`.

**The read, `flock_read_hamming.rs` (new), switched by `--hamming-read H`, default 0.**
1. **Heads:** `H` (4), each on 16 lanes (64 coordinates). Reads after layers 2 and 4.
2. **Points:**
   - the query, the key and the value are the residual rotated per lane by learned unit quaternions;
   - keys and queries are spun by position, with SpiralCore's per-lane rotation by position (RoPE's role; the sign codes are not exactly offset-invariant, since rotations keep L2, not signs);
   - half the heads carry the successor's point as payload: a one-step induction head.
3. **Match:** each head's 64 sign bits, compared by `popcount(q XOR k)`. The forward pass uses the ±1 codes; the backward pass passes the distance's gradient straight through the codes to the rotated points.
4. **Choose:** the 16 nearest earlier positions, hard and detached.
5. **Weigh:** `max(0, m_h − d) / 16`, with a learned margin per head. No exponential, and no normalisation by a learned sum.
6. **Add:** through a weight that starts at 0.

## Cancel probe (run first)

Cancel if a Hamming or popcount attention exists in the float crate, or another open order changes the runner.

Result (2026-10-27T15:40Z, `origin/main` at `5bc6f6522`): `git grep -i -e popcount -e count_ones -e hamming origin/main -- crates/uor-r4-float-train` finds nothing; no open PR touches the crate. **Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/src/flock_read*.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `docs/harness/prep/in-space-trunk-to-text-hamming-*`
- `docs/research/notebook/float-training.md` (rows)
- `docs/loop/tasks/in-space-trunk-to-text/**`

## Done when

1. **An MQAR-style recall task in the runner.** It follows the existing bench's definition (`crates/uor-r4-training/examples/mqar-bench.rs`):
   - a key at `p` and its value at `p + 1`, then the key again at `p + d`, where the target is the value;
   - distance buckets; disjoint key, value and filler ranges; fresh pairings per sequence.

   It is scored by recall per distance bucket. It has a host test of the generator.
2. **The read builds, and its CUDA tests are green.** They cover the bit-code match against the L1 surrogate, causality, the weights, a zero weight leaving the trunk unchanged, and each head's credit sign with a negative gate.
3. **The recall probe:** read against no read against claude-145's content read, on the recall task. It is read by its own sheet's bars, which a light gate reviews.
4. **Only if recall passes:** the language run under the claude-145 bars.
5. **The REPORT** puts negatives first and leads with tokens/s, both training and integer decode.

## Stop and decide (never stop and wait)

- No recall above the no-read arm: classify instrument or mechanism, and record it. No language run.
- A missing number gives no cell: rerun that step.
