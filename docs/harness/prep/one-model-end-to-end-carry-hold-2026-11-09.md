# Prep sheet: the carry lever, a held stream write (one-model-end-to-end, claude-331, 9 Nov 2026)

TEST FITNESS: NOT_FIT after two rounds, by fresh non-author reviewers (claude-331's stop rule applies: no code or run under that unit).
- **Round 2, at `6aec906df`:** every round 1 major is confirmed fixed in the text. Two new majors and two minors, all applied below:
  - **Major A:** (r1u) by one scalar `|w_y|_mean` did not undo a lane-wise `w_y` with mixed signs. It now ranks `u` against the written value points `w_y ⊙ p_v` at the best non-negative scale, and so does (r1h).
  - **Major B:** outcome 4's third branch read the whole stream at `H4`, which `H4`'s own point can swamp. It is now split on (r1h), with a branch of its own for a held term the residual swamps.
  - **Minor 1:** I-R2 names the trace's fields exactly.
  - **Minor 2:** the cost counts the cut and rank passes.
- **Owed:** a fresh FIT of this text, in the next unit (claude-332), before any code.

Round 1, at `35776f884`: NOT FIT, with no blocker, three majors and four minors, all applied below.
- **Major 1:** the value rank at the `Q` row is now read three ways: on the payload `y`, on the write `u` by direction, and on the whole stream. The whole stream mixes in the residual and `w_y` starts at 0, so on its own it could read a right payload as wrong.
- **Major 2:** two readings at `H4`, of the held term and of the stream after the read plus the hold, and an eval-time cut `−H` (`g := 0`) beside `−aS`. Outcome 4's split is now conditional on them.
- **Major 3:** I-R2 also matches claude-330's question-row trace, since its hits sit at chance and cannot check the question placement.
- **Minors:** the hold's zero start gradient, `|w_y|` logged, and a hold that never grew read as undertrained; `g` for the value tokens; the refusals as a new clause; a no-read digest in I-1 behind the reuse of B1-n.

## The question

[claude-330](../../loop/tasks/one-model-end-to-end/claude-330/REPORT.md) read outcome 3, second branch, of [its sheet](one-model-end-to-end-question-keyed-read-2026-11-08.md), on both seeds:
- **B1 passes exactly.** The ask heads, trained by the found-copy pull, choose a value by a question that is not its key: 512 / 512 in both far buckets.
- **B2 stays at chance.** B2-a and B2-aq answer ≤ 2 / 512, and B2-n ≤ 5 / 512.
- **B2-aq's ask heads choose at the question.** They keep the asked position at `Q(K_j)`'s row in 376–512 of 512 rows, and nothing at the scored row `H4`.

So the value is chosen at the question and lost before the scored row. [claude-326's map](../../loop/tasks/one-model-end-to-end/claude-326/ASSESSMENT.md) (Q4) names the lever for this branch: "the carry: a read at the question token whose result is written forward".

**Where the value can go, read from the code at `35f55d334`** (`crates/uor-r4-float-train`):
1. **Two reads.** `HAMMING_DEPTHS = [2, 6]` (`flock_read_hamming.rs:73–75`).
   - The **stream read** runs after layer index 1 of 6 (`geometric_stack_native.rs:1684–1701`). It adds `w_c ⊙ c + w_y ⊙ y` to the stream (`flock_read_hamming.rs:1713–1738`).
   - The **readout read** runs after the final norm, after every trunk layer (`geometric_stack_native.rs:1728–1743`). It moves the row's own final point.
2. **The readout read at `Q(K_j)` cannot reach `H4`.** A row's prediction is that row's own final point. The readout read at `H4` cannot choose either: its ask query is `point(H4) + a[H4]`, the same for every question.
3. **The only route from `Q(K_j)` to `H4`** is the depth-2 ask write at the `Q` row, carried by trunk layers 3 to 6.
   - The trunk's `composed_recurrence` (`geometric_stack_native.rs:1962–2145`) is causal: a width-4 depthwise time convolution (shifts 0–3), and a quaternion scan with fixed dyadic decays (`RATE_TABLE`).
   - The MLP works row by row.
4. **B2's window** (`recall_query_window`, `in_space_long.rs:12026–12155`). Each tail slot is `Q H1 H2 H3 H4 v`, with the header `[48, 49, 50, 51]`. The scored row is `H4`, with target `v`.
   - **The trap:** the header repeats in every slot. So a head at `H4` finds the previous slot's `H4`, whose next token is the previous slot's answer.

This sheet asks two things:
- (A) **The diagnostic:** is the value lost in the stream write at the `Q` row, or in the trunk's carry?
- (B) **The lever:** does a held stream write, which delivers the `Q` row's ask write straight to the next rows, carry the value to `H4`?

## The mechanism: the held ask write

Under `--hamming-hold h` (requires `--hamming-ask 1`), the **stream read's ask write is held forward** for `h` rows.
- Let `u_t = w_y ⊙ y_t` be the ask heads' stream write at row `t` (zero where no ask head found a candidate).
- The stream at row `t` gains `Σ_{s=1..h} g[x_{t−s}] · u_{t−s}`, over rows `t − s` of the same window only.
- `g` is a new learned table: **one scalar per token, for the stream depth only, starting at 0.** It is saved as `d2.hold`.
- At `g = 0` the forward equals the forward without the hold, bit for bit.
- **What it learns.** The scored loss at `H4` reaches `g[Q(K_j)]` through `u` at the `Q` row. It also reaches `a[Q(K_j)]` and `w_y` through the held term. The header tokens' `g` is pushed away from what hurts, since their writes carry the previous slot's answer.
- **It starts with no gradient.** `g = 0`, `w_y = 0` (`flock_read_hamming.rs:1226`) and `a = 0` (`:832`). The gradient to `g` is `⟨∂L, u⟩`, so it is zero until the direct path moves `w_y` off 0. claude-330's ask arms moved it to 0.017–0.394 by step 512. `|w_y|` and `g` are logged at every checkpoint, so a hold that never moved is read as such (outcome 1), not as a carry failure.
- **`h = 8` is pre-registered.** It covers B2's 4-token header with room to spare, so the hold does not encode the header's length. No other `h` is run in this unit.
- **No dense matrix multiplication:** a per-token scalar table, a shifted causal sum and a lane-wise scale.
- **No squashing:** `g` is a raw scalar, with no bound and no normalization.
- **Refused:** the decode step and the integer export each refuse a read with a hold, through a new clause. Today `flock_read_hamming.rs:1557–1561` refuses only instrument cuts in the decode step (`from > 0`), and `fbs_export.rs:389` refuses unexported read forms. This unit serves nothing.

**The cut `−aS` (the ask stream write off).** A new ablation with `classes() = [[false, true], [false, false]]`. It drops the depth-2 ask heads at every row, and so the hold, through the existing class mask (`flock_read_hamming.rs:99–106`, `1717–1723`). It is chained after the existing readings like `−aQ`, so runs without ask heads write identical readings.

**The cut `−H` (the hold off).** An eval-time cut that sets `g := 0` on a held checkpoint and leaves the direct write. `−aS` removes the direct write too, so only `−H` attributes a result to the hold alone.

**The carry readings (continuous).** Recall cannot fall below chance, so the cuts need readings that can. For every held-out query, a **value rank** is the rank of the asked value's table point among the 512 value tokens' table points, by Euclidean distance from a vector, ties broken by token id. 1 is best and 256.5 is chance. The vectors:
- **At the `Q` row, the depth-2 read's write, three ways** (the stream is `x + w_c ⊙ c + w_y ⊙ y`, and `x` holds `Q`'s own point plus two layers, so the whole stream can hide a right payload):
  - (r1y) the ask payload `y`, the mean of the kept table points (`flock_read_hamming.rs:1579–1585`, `1660`);
  - (r1u) the write `u = w_y ⊙ y`, against the **written value points** `q_v = w_y ⊙ p_v` (the type weight row, lane by lane, `flock_read_hamming.rs:1717–1737`). Each value's distance is `min_{α ≥ 0} |u − α q_v|`, with the closed form `α* = max(0, ⟨u, q_v⟩ / ⟨q_v, q_v⟩)`. So neither a small `w_y` nor a lane-wise scale with mixed signs reads as a wrong value;
  - (r1s) the whole stream just after the write.
- **At `H4`:**
  - (r1h) the held term `Σ_s g[x_{H4−s}] · u_{H4−s}`, which equals `w_y ⊙ Σ_s g[x_{H4−s}] · y_{H4−s}`, ranked against the same `q_v` with the same best non-negative scale (held arms only). A held term with a negative net weight on the value reads as not carrying it, since it moves the stream away from the value;
  - (r1hs) the stream just after the depth-2 read plus the hold;
  - (r2) the final point, before the readout read.

Each is reported as the mean rank per checkpoint, with and without `−aS` and `−H`.

## Steps: the arms

**Common to every arm**, as claude-330's sheet:
- `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0 --steps 4096 --eval-every 512 --recall-eval-windows 256 --recall-readings 1`;
- context 192;
- seeds 1 and 2, with seed 3 if the two seeds split.

The forms and the ask arms' flags are claude-330's, unchanged. The ask arms are `--hamming-read 4 --hamming-ask 1 --hamming-attract 1`.

| Arm | Form | Read |
|---|---|---|
| **B0-a** | B0 | ask (instrument) |
| **B1-a** | B1 | ask (the reproduction check, I-R1) |
| **B2-aq** | B2 | ask, `--hamming-attract-at question` (the reproduction check, I-R2, and the cut's base) |
| **B2-aqH** | B2 | B2-aq with `--hamming-hold 8` |
| **B2-aH** | B2 | ask, pull at the scored rows (as claude-330's B2-a), with `--hamming-hold 8` |
| **B0-n, B2-n** | B0, B2 | none (the no-read twins, L-010) |

**Readings**, as claude-330's, plus:
- `R^{−aS}`, `R^{−H}` (held arms) and the carry readings, with and without each cut, on B2-aq, B2-aqH and B2-aH;
- `g` per checkpoint for the 16 question tokens, the 4 header tokens and the 512 value tokens (each as mean, minimum and maximum), and the mean of the rest;
- `|w_y|` per depth per checkpoint.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

| Arm | Passes when |
|---|---|
| **B0 (instrument)** | `R(B0-a) ≥ 0.95` and `R(B0-n) ≤ 0.02` |
| **I-R1 (instrument)** | B1-a passes claude-330's B1 rule at both seeds: `R_ask ≥ 0.50`, `R_ask ≥ R_oth + 0.25`, `R_ask ≥ R_ask(B1-n) + 0.10`. B1-n is not rerun: claude-330's sealed B1-n line at the same seed is the twin |
| **I-R2 (instrument)** | B2-aq's step-4,096 line, against claude-330's sealed `runs-2/s{seed}/…/B2-aq-s{seed}/recall.jsonl` at the same seed: the same `rows`; each bucket's `hits` within `0.02 · rows`; and, since those hits sit at chance, **the question-row trace too**: in `readings.trace[]`, for every entry with `position == "query"` and `copy == false` (the ask heads), keyed by `(depth, head, bucket)`, `kept_ask` is within `0.05 · rows` of the sealed entry's at the same key (claude-330 seed 1, depth 2: 392 and 376 for head 2, 512 and 512 for head 3). The timing fields are ignored |
| **B2 (carry)** | the B1 rule for B2-aqH, and separately for B2-aH, each against B2-n |

## Outcome map (read in order, first match wins)

0. **B0, I-R1 or I-R2 fails:** an instrument fault. Repair it and rerun; nothing else is read.
1. **Undertrained:** a held arm misses its bar and either its `R_ask` still rises by more than 0.05 between its last two checkpoints, or the hold never grew (the mean `|g|` over the 16 question tokens is below 0.01 at step 4,096). Extend it to 16,384 steps under this sheet.
2. **B2-aH passes:** the hold carries the value, and the scored loss alone trains the choice through it. Attribute it with `R^{−H}` and `R^{−aS}` (each expected to fail the bar) and `g` (expected highest on the question tokens). **Next:** a language run with ask heads and the hold on claude-325's bare-answer recipe, under its own sheet.
3. **B2-aqH passes, B2-aH fails:** the hold carries the value once the choice is trained at the question row, but the scored loss does not train the choice through the hold. **Next:** the pull at every row (the language form, where both placements are present) on B2, under its own sheet.
4. **Both held arms fail.** Split by the carry readings on B2-aqH, first match wins:
   - **(r1y) is near chance:** the ask payload at the `Q` row is not the value, though the trace keeps its position. That is a payload fault, instrument-side until the trace and the payload are reconciled. Repair it and rerun.
   - **(r1y) is near 1 and (r1u) is near chance:** the payload is right, and the write's weight `w_y` loses it, in this configuration (§4.1, §4.2). **Next:** the stream combine, tested on B2 first.
   - **(r1u) is near 1, and (r1h) at `H4` is near chance:** the write is right, and the hold does not deliver it to `H4`. Read `g`: a hold that stays small, or that is dominated by the header and value rows' writes, is a configuration result for `h = 8` and this `lr`. **Next:** the hold's form (a carried register, overwritten only by a found write), tested on B2 first.
   - **(r1h) is near 1, and (r1hs) at `H4` is near chance:** the held term carries the value, and `H4`'s own stream swamps it. **Next:** the hold's scale against the residual (the stream combine), tested on B2 first.
   - **(r1hs) is near 1, and (r2) at `H4` is near chance:** the value reaches `H4`'s stream, and the trunk layers after the read lose it. **Next:** move the stream read later (to the last depth), or hold into the readout read, tested on B2 first.
   - **Any other pattern** is named from the readings.

"Near 1" means mean rank ≤ 8; "near chance" means mean rank ≥ 128. Both are read at step 4,096 at both seeds. The same readings are reported for B2-aq, the unheld base, whichever outcome lands.

**Seeds:** two seeds that split run a third; at least 2 of 3 must agree. Every cell, rank, `g` value and ablation is reported either way.

## Instrument checks

- **I-1 (default paths):** claude-330's printed digest (`i1_digest::ask_default_forward_digest`) is equal at the base and the head. Two new digests are equal at the base and the head too: one with `--hamming-ask 1` and no hold, and one of a no-read forward. The no-read digest is what lets I-R1 reuse claude-330's sealed B1-n as the twin.
- **I-H1 (the hold starts off):** with `g = 0`, the forward and the gradient with `--hamming-hold 8` equal the forward and gradient without it, bit for bit.
- **I-H2 (the hold's rows):** on a hand-built input with one nonzero ask write at row `t`, the added term is `g[x_t] · u_t` at rows `t + 1` to `t + h`. It is zero at row `t` itself, at `t + h + 1`, before `t`, and in every other window of the batch.
- **I-H3 (the gradient):** a loss at row `t + 4` alone gives nonzero gradients to `g[x_t]` and `a[x_t]`, and none to `g` of tokens that appear only after `t + 4`.
- **I-H4 (save, load and refusals):** `g` round-trips through save and load. The decode step and the export each refuse a read with a hold, with an error that names it.
- **I-H5 (the cuts):** `−aS` removes every depth-2 ask write and every held term, leaves the copy heads' picks unchanged, and leaves the readout read's ask heads on. `−H` removes every held term and leaves the direct write bit for bit.
- **I-V (the value ranks):** on hand-built vectors, each rank equals a brute-force sort of the 512 distances, ties broken by token id. Two cases are pinned:
  - a residual `x` that dominates the stream: (r1y) and (r1u) read 1 while (r1s) reads far from 1;
  - a `w_y` with mixed signs and uneven lane sizes: (r1u) and (r1h) still read 1.
- **I-4, I-6, I-7:** as claude-330's sheet, with I-7 run on B2-aqH.
- **I-R1, I-R2:** as in the bar.

Each check is a unit test (I-6 and I-R excepted). Each has at least one mutation that makes it fail, run on the laptop where it can be and on the pod for the CUDA tests.

## Probe ladder

B0-a, B1-a and B2-aq first on each pod. If any instrument check misses, stop and repair. The held arms do not run on a failed instrument.

## Data split

As claude-330's sheet. Every window is generated from `(seed, index)`, and held-out windows use `seed ^ RECALL_EVAL_SEED`.

## Base artifact

None. Every arm trains from step 0.

## Minimum test power

As claude-330's sheet, with 512 held-out queries per bucket: the 0.10 margin against the no-read twin is about 3.2 standard deviations near 0.5, and the paired 0.25 margin about 6.8.

## Cost

claude-330 measured these on one RTX 5090: about 6 minutes for an ask arm and 2.5 minutes for a no-read arm. The cuts (`−aS`, `−H`) and the six value ranks add evaluation passes on three arms, counted here at ×1.3 on those arms. Per seed, five ask arms and two no-read arms is then about 41 minutes, plus a build of about 10 minutes. Two pods in parallel, one per seed, about $0.50 each, about $1 in all. That is inside the caps and one GPU unit. Outcome 1's extension adds about 4× for the arms it covers.

## Results and stop rule

- **Results:** written to `results/claude/one-model-end-to-end-carry-hold-2026-11-09/` in the HF store with a `SHA256SUMS`, and read back before any pod is brought down.
- **Stop** at a B0, I-R1 or I-R2 failure, or at any non-finite loss.
- **Watchers:** one is armed on every pod (L-004), with the lease renewed by it.
- **Scripts:** pinned by SHA-256 in a D67 pre-run review section added before any pod.
- **Builds:** each checkout builds into its own cargo target, and the job checks the "Compiling … (<checkout>/crates/uor-r4-float-train)" line before trusting a test verdict (claude-330's fix).
