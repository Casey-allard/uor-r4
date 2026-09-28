## 10. Ideas that would ease the research

### 10.1 Technical accelerants, cheapest first

| # | Idea | Why it helps | Cost | Quick falsifier |
|---|---|---|---|---|
| 1 | **Anneal the existing checkpoints**: linear LR decay to 0 | Training used a constant 1e-3 LR and never cooled down (2405.18392) | 1–2 h per arm | Gain in development-tail NLL below 0.02 |
| 2 | **Remove serving overheads**: analytic uniform floor, packed codes, i16 KV, build tables once | 3.36 MB → 0.85 MB touched per token; removes about 8k emulated operations per token | Hours | Bit-identical outputs |
| 3 | **Measure energy with macmon** (a wall meter as ground truth) | The first real joule in the project; decides §11.2 on data | 1 day | Protocol in §5.5 |
| 4 | **Add commutative and no-transport arms plus an A5 probe** to the existing learner | First well-posed geometry test on the real model | CPU-hours | Quaternion arm no better than the commutative arm at ≥4× training length |
| 5 | **Throughput benchmark: incremental vs parallel re-base** | Decides the Phase-1 base by measurement | Days | Re-base below 5× tokens per M1-hour |
| 6 | **Quaternionic-RoPE identity**: the linear quaternion recurrence equals decayed linear attention on frame-rotated q/k | Existing chunked kernels apply after an O(T·d) rotation | Part of #5 | Numerical equivalence (checked: ≤3e-14 float64, ≤9e-6 float32) |
| 7 | **Train continuously, then snap to 2I, then serve as an integer automaton** for tracking lanes | Measured exact to length 4,096 where float32 drifts | Days | Snapped lanes lose tracking relative to float at matched length |
| 8 | **Use the data that already exists**: TinyStories 2–4 epochs; open dialogue and code corpora | Exposure explains about half the gap to #1017 | Data preparation | NLL and coherence at matched parameters |
| 9 | **Logit distillation from #1017, early only** (same tokenizer) | Faster early learning; the teacher (1.57) is weaker than the target | About 14 MFLOP per token of teacher cost | Under 0.05 nats better than the no-KD run at matched tokens |
| 10 | **Polar 600-cell key codebook** for attention and event keys, with lookup-table scores in ½·ℤ[φ] | Satisfies D0-b by design rather than emulation; int3-like quality at 2.73 bits/dim | Days | Must beat int3/PQ at equal bits on real keys |
| 11 | **Exact factored or class-based output softmax** | Cuts the 62.7% of per-token reads in the output head without a learned router | Days | NLL unchanged (exact) at lower bytes/token |
| 12 | **uor-addr κ-labels** for event, record and artifact identity | UOR's own mature standard; retires prime-product identity | Hours | n/a |
| 13 | **3 seeds by default, with bootstrap confidence intervals.** Gate on held-out BPB, a TinyStories-style coherence score and the D6 long-range probe, not on authored 32-prompt panels | Stops n = 1 and small-panel results from steering direction | Small | n/a |

**Quarter-square lookup multiply.** The quarter-square LUT, ab = ⌊(a+b)²/4⌋ − ⌊(a−b)²/4⌋, is a D0-b-legal *speed* fix: 30–45× faster than the current loop.

It has two limits:
- it reads a 1 MiB table, and at Horowitz-scale costs that is likely *more* energy than a hardware multiply;
- it covers only operands up to 16 bits, while some mixture products are 48×15-bit.

Prefer designs that remove activation products, such as codebook scores and snapped rotations.

### 10.2 Process accelerants

1. **One planning object: the work card.** One page per experiment; the D9 work card already has the right fields. Retire the separate step, design, review, result, closeout and budget documents.
2. **Move evidence JSON out of git** into an artifact store addressed by uor-addr hashes. Keep only the hash and a one-line summary in the repository.
3. **Replace echo CI with real `fmt`, `clippy` and `test` on the active crates.**
   - Tests: under 1 minute for integer and tokenizer, about 14 s for training.
   - Builds: about 9 minutes for a cold training-crate build, cacheable (verify F1).
4. **Decide weekly.** Change direction only when a pre-registered kill criterion fires. Put owner checkpoints at decision points, not after the fact.
5. **A fixed weekly budget** instead of raising the ledger limit each cycle. Log one line per run: wall time, model time, tokens, tokens/s.
6. **Housekeeping.** Move the 97% of Rust unused by D8 into a `legacy/` area, rewrite AGENTS.md:122, and cap mandatory reading at about 5k words.
7. **One agent lead at a time.** Reserve multi-model review for promotions and the weekly direction.
