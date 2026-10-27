# ORDER: in-space-trunk-to-text, claude-144 (a float-trainable Flock read on the in-space trunk)

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `1d78d3d3e`.

## Goal

**Give the in-space trunk a geometric read over its own context, so that a reply opening can use the prompt. Then test whether it does.**

claude-142 (#3327) found that the trunk's recurrence learns short memory. Its best checkpoint reads about 8–12 tokens back (`R16 = 0.011`). Three ways of keeping lanes long stopped training. The project's own answer to reading context is the Flock read ([flock-read](../../../../mechanisms/flock-read.md)): look at a few nearest earlier positions, carry them into the query's frame, and move toward their consensus. It exists only as an integer kernel in the discrete crate.

**The read, `crates/uor-r4-float-train/src/flock_read.rs` (new).** It sits on top of the trunk's hidden output, so `StackModel`, its saved format and every recorded run stay unchanged.

1. **Points.**
   - **Query and key:** each position's hidden point is rotated lane by lane by a learned unit quaternion, one for queries and one for keys. These are Hamilton products per 4-lane block, so there is no matmul.
   - **Payload:** what an earlier position `s` offers is the shared table's point for the token after it, `e(x_{s+1})`. That token is a known input when `s + 1 ≤ t`. This is "what followed a similar state", the copying route the [relation read](../../../../mechanisms/relation-read.md) names.
2. **Choice.** For each position `t`, keep the `K` earlier positions `s < t` whose keys are nearest the query in mean L1. The choice is hard, exact and detached: repeated argmin, with no softmax. Fewer than `K` candidates means fewer kept, never padding.
3. **Consensus.** The plain mean of the kept payloads, weighted 1/K. Nothing exponential, nothing normalized by a learned score.
4. **Add.** The query point becomes `h_t + w ⊙ c_t`, where `w` is a learned per-coordinate weight starting at 0. At step 0 the read changes nothing.
5. **Credit to the keys.** Each kept neighbour's distance enters the consensus as a straight-through factor, `1 + (d − d.detach())`, which is exactly 1 in the forward pass. The gradient then pulls the keys of neighbours whose payload helped toward the query, and pushes away those that hurt. It is Flock's "align with the flock" carried into training.
6. **Its own optimizer.** An AdamW in candle tensor operations, on the GPU, with the trunk's constants.

The runner `train-in-space` takes `--flock-read K`, with 0 meaning off and the default. It applies the read in training, in DEV scoring and in greedy replies. The read's parameters are saved beside each checkpoint.

## Cancel probe (run first)

Cancel if a float Flock read already exists, or another open order changes the float-train runner.

Result (2026-10-27T11:45Z, `origin/main` at `1d78d3d3e`):

- `git grep -n -i flock origin/main -- crates/uor-r4-float-train` finds nothing.
- The open PRs touch no file in this scope: #3330 is this unit, and `gh pr list --state open` lists no other float-train change.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/src/flock_read.rs` (new)
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `docs/harness/prep/in-space-trunk-to-text-flock-*`
- `docs/research/notebook/float-training.md` (one row)
- `docs/loop/tasks/in-space-trunk-to-text/**` (this task folder)

## Done when

1. The read builds, and its CUDA tests are green on the pod. They cover:
   - the choice: the nearest `K`, causal, never the position itself;
   - `w = 0` leaving the query point unchanged;
   - the straight-through factor being 1 in the forward pass;
   - the optimizer's step matching a hand-computed AdamW step.
2. A probe-ladder run shows the read trains: a few thousand steps on one pod, read against no read.
3. The decisive run, under its own prep sheet with a FIT and a D67 pre-run review: the read against no read at the same settings. It is read by that sheet's cells, recorded as a notebook row, with the replies shown.
4. The REPORT puts negatives first, leads with tokens/s, and says whether the item's done-when is met.

## Stop and decide (never stop and wait)

- If the read stalls training the way the three long-memory variants did, classify the cause before calling it, and record it.
- A missing number gives no cell: rerun that step.
