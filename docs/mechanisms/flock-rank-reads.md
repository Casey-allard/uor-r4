# Flock / rank-table reads (softmax-free attention)

**The idea (first principles):** A starling in a flock tracks a few neighbours by *rank*, not by a score passed through an exponential. A flock read keeps, per query row, a **sink** (position 0), a **local window** of the last W positions, and the **exact top-K** of the rest by a monotone rank score (Minkowski/Lorentz product, which needs no `arcosh`). It then weights each kept source by its rank, `w_r ∝ 1/(r+1)`, or by a learned per-head table. Ordering is all a rank needs, so the score can be an integer, or a Hamming distance on sign bits (XOR + popcount). Each read touches at most W+K+1 value rows, not t+1. Sources: `crates/uor-r4-training/src/flock.rs` (module docs), `docs/labs/softmax-free-read-2026-10-10/README.md`.

**What it replaces and why that matters:** It replaces the exp-table softmax read. D11 serving allows no floating point, no multiplier in the kernel and no transcendental functions. Today the served stack still uses an exp table, so M4 (#2032) acceptance item 0, "no softmax at runtime", stays open. A rank read is a sort plus a constant table read. It also bounds memory traffic per read, which is what laptop-scale cost needs at long context.

**Where it lives in code:**
- Trainer: `crates/uor-r4-training/src/flock.rs` (`flock_select`, `FlockSpec`, `FlockWeights::{Softmax,Rank}`, `raw_rank_weights`) and `geometric_stack.rs` (`StackModel::set_read_weighting`).
- Flags: `select=flock:W:K`, `read_weighting=softmax|rank|hamming_rank|learned_rank`. The learned variant adds `read.rank_logits [heads, W+K+2]`.
- Serving: `crates/uor-r4-integer/src/stack/flock.rs` (allocation-free insertion top-k), `bitcode.rs` (`BitCode` sign code, SWAR popcount). Artifact schema `uor-r4.lut-stack/3`, Q31 rank tables, audited FULL PASS.

**What has been tried, honestly:** All on the M4 chat recipe (19.93M params, w512 h8 ctx384, 12,207 steps, 2 seeds, held-out 196,608 targets). The softmax baseline is 0.8772 BPB.
- Fixed rank `flock:8:8` (#2140/#2144): **+0.040 BPB**. Seeds agree to 2e-5.
- Hamming-rank: **+0.259 BPB**, and its seeds diverge.
- Learned rank tables (#2153/#2160): **+0.0185 BPB**, 54% of the gap closed, against a 0.01 bar.
- v4 memory was within tolerance for fixed and learned rank (it is noisy at 3–6/40).
- The line hit 3/3 under D21 and **stopped**.

Could these tests judge the idea? Only partly:
- (a) There was one W/K setting.
- (b) Training used a **straight-through gradient equal to the flock-softmax gradient**, so q, k and aux learned for a forward pass the model never runs.
- (c) Hyperparameters (lr 4e-4, warmup) were tuned for softmax.
- (d) The flock ran on the host, at 16–26k tok/s against 163k, which kept the runs small.
- (e) The model is small and the context short, so the bounded-read advantage never shows.

**What success looks like:**
- Own metrics: held-out BPB within 0.01 of softmax at equal or lower served cost, with the D11 read touching ≤W+K+1 rows.
- Milestone: closes #2032 item 0. Served BPB holds as context grows past 384.

**What failure looks like:** With a rank-consistent gradient, a K sweep and long context, rank still trails softmax by more than 0.01 BPB, *and* raising K toward t+1 does not close the gap. That would mean the information lives in score *magnitudes*, which rank discards by design.

**A fair test:**
- Train the rank read in from step 0 with a gradient derived from the rank forward (no softmax surrogate), and re-tune lr/warmup.
- Use a GPU flock kernel.
- Scale: ≥20M params, contexts 384 and ≥1536, K ∈ {8, 32}. Arm W (`flock:32:32` with learned tables) is training on 10 October; its result follows on #2032.
- Use 3 seeds. Seed spread is about 1e-3 BPB, so a 0.01 bar is well resolved.
- Do not use v4 /40 as a gate; its noise is ±3.

**Open design questions:**
1. A rank-consistent surrogate, e.g. a differentiable sort or a temperature-annealed soft rank that ends at the hard table.
2. Score-bucketed tables (rank × quantized margin), which keep some magnitude while staying a table read.
3. Learned W/K per head, or a sink chosen at a geometric address instead of position 0.
4. Hamming over richer codes (multi-bit or H4/icosian codes) instead of 1-bit signs.
