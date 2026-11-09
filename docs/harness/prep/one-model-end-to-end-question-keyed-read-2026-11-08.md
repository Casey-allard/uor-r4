# Prep sheet: a question-keyed read (one-model-end-to-end, claude-329, 8 Nov 2026)

TEST FITNESS: NOT_FIT after two rounds, by fresh non-author reviewers (claude-329's stop rule applies; no code or run under this unit).
- **Round 1, at `3c7a10f9a`:** one blocker, the byte-identical I-R; one major, no pull at B2's question row; four minors. All six are applied.
- **Round 2, at `59b69fad4`:** every round 1 finding is confirmed fixed. One blocker remains: I-R gated hits against `rows` at every checkpoint, which step 0 fails. There are also two minors. The reviewer called the sheet "fit" apart from that wording. All three are applied below.
- **Owed:** a fresh FIT of this text, in the next unit (claude-330), before any code.

TEST FITNESS: FIT, by a fresh non-author reviewer in claude-330, at `e4b2977e6` (this text as merged by claude-329), before any code. No blocker and no major; six minors, all applied below. The reviewer's one negative gate for the orchestrator (detach `a` in the pull path; I-A2's gradient-reaches-`a` assertion must fail) is in claude-330's mutation script as `ia2-detach-ask`. The verdict is recorded in `local/claude/claude-330/fit-report.md` and in claude-330's REPORT.
- **Minor 1:** I-R's loss tolerance had no evidence behind it. `train_loss_mean` is now reported, with the sealed step-3,584 to step-4,096 change beside it for scale, and not gated. I-R gates on `rows` and hits.
- **Minor 2:** outcome 4's first branch is a configuration result for `λ` when I-A2 and I-A3 pass.
- **Minor 3:** B2-a and B2-aq differ in the pull's placement (at the `Q` row and not at `H4`), not only in a signal at `Q`. Outcome 3 says so.
- **Minor 4:** diagnosis point 2 now names the true limit: the shared lane-wise rotation and the margin gating. The residual is not detached.
- **Minor 5:** `a` is one table per depth.
- **Minor 6:** outcome 2's attribution names an ask head at `Q(K_j)` at either depth.
- **Deviation, recorded:** I-1's "fixture hashed from `main` at the pin" is built as a printed digest. The test `i1_digest::ask_default_forward_digest`, between the `// I1-DIGEST BEGIN` and `// I1-DIGEST END` markers, prints `I1-DIGEST <hex>`. The pod appends that block to the base commit's `flock_read_hamming.rs` and to the head's, runs both, and requires the two lines to be equal. No hash is committed, so the fixture cannot drift from the base it names.

## The question

[claude-328](../../loop/tasks/one-model-end-to-end/claude-328/REPORT.md) read outcome 4 of [its sheet](one-model-end-to-end-query-recall-2026-11-08.md), a query failure, on both seeds:
- **B0 works.** The trained Hamming read answers 0.98–0.996 when the question repeats the key.
- **B1 and B2 fail.** Where the question token `Q(K_j)` is not the key `K_j`, the read answers at most 2 of 512 queries in a bucket, as its no-read twin does. No type head keeps the asked position above chance.

[claude-326's assessment](../../loop/tasks/one-model-end-to-end/claude-326/ASSESSMENT.md) names the next lever, (b): **a read whose query is built from the question.** This sheet asks whether such a read can learn to choose by the question on claude-328's B0, B1 and B2 forms.

**Why the type heads could not learn it** (read from the code at `308e25aad`, `crates/uor-r4-float-train/src/flock_read_hamming.rs`):
1. **The choice carries no gradient.** `choose_sorted` takes `query.detach()` and `key.detach()` (`:1720–1721`). A position is pulled closer only if it is already among the 16 kept picks and inside the margin, through `relu(m − d) / max(m, 1)` (`:1370–1374`). In B1 the asked position is among the picks about `16 / (t − 1)` of the time, about 0.09, which is the chance rate claude-328's trace measured.
2. **The query cannot name a token.** A type head's query is `rotate(query_rotation, residual_t)` (`:1181`): one lane-wise rotation, the same for every token. Asking for `K_j` from `Q(K_j)` needs the residual at `t` to carry `K_j`'s direction already. The residual is not detached (`:1157`), so a gradient can reach the embedding, but only through candidates already kept and inside the margin. The limit is the shared lane-wise rotation together with that margin gating, not the table's detach in the read (`:1163`).
3. **The signal is scaled by a weight that starts at 0.** Every gradient to the type heads passes through `w_y` (`:1044`), which reached only 0.075–0.104.
4. **The payload is the form that failed before.** A type head brings back `v_s`, the rotated residual. The module doc (`:50–52`) records that this form "failed the recall probe".

## The mechanism: ask heads

Under `--hamming-ask 1`, each read's **type heads become ask heads**. Everything else in the read is unchanged, the copy heads included.
- **Query:** `rotate(key_rotation, point(x_t) + a[x_t])`, then the sign code.
  - `point` is the shared table's point, detached, as the copy heads use it.
  - `a` is a new learned table of per-token **ask offsets**, one point per token, starting at 0. There is one table per depth (saved as `d{depth}.ask`), not one shared by both depths.
  - At step 0 an ask head's query equals a copy head's query bit for bit. So the read starts as an exact-match read and learns where to look by moving `a`.
- **Key:** the copy heads' key, `rotate(key_rotation, point(x_{s−1}))`.
- **Payload:** the copy payload, the table point of the token at the kept position `s`.
- **Combine:** the copy form. On the readout it moves toward the mean kept point, with its own weight `w_y`; on the stream it adds `w_y ⊙ y`. Ask heads have their own margins, initialized as the copy heads' (`head_width / 4`).
- **No dense matrix multiplication:** a table lookup, a lane-wise quaternion rotation and Hamming top-k.
- **No squashing:** the sign code with the identity backward, as the read already uses.

**The training signal, the found-copy pull** (`--hamming-attract λ`, the geometry's own distance):
- Take a row `t` that carries a training target `y_t`. Let `S_t` be the earlier positions whose token is the target: `1 ≤ s < t`, `x_s = y_t`.
- If `S_t` is not empty, each ask head `h` at each depth adds `λ · bits_h(t, s*) / head_width`. Here `bits_h` is the read's own `hamming_bits` between the ask head's code at `t` and its key code at `s`, and `s*` is the position of `S_t` with the nearest key (a hard minimum, no gradient through the choice).
- The key code is detached, and in the pull's own query path `key_rotation` is detached too, so the pull moves only the ask offsets `a`.
- **What the pull leaves alone, and what it does not:** the pull never moves the copy heads. The *task loss* through the ask heads does reach `key_rotation`, which the ask and copy heads share, just as the type heads' task loss reached `query_rotation` before. That sharing is part of the design, not hidden by it.
- The term is averaged over the rows with a match, heads and depths, and added to the training loss.
- **`--hamming-attract-at question` (B2-aq only):** in a two-class recall window, each scored episode's pull is placed at its `Q(K_j)` row instead, with that episode's scored target `v_j`. The row's task loss is unchanged (it stays unscored). This is a probe arm that only the recall task can express. It asks whether the choice at the question can be trained in B2's form when a signal reaches it.

In B1, `y_t` at `Q(K_j)`'s row is `v_j`, and `S_t` is the fact's value position, whose key is `K_j`. So the pull moves `a[Q(K_j)]` until the ask code of `Q(K_j)` reads `K_j`. Because `Q` is the same in every window, one offset per question token serves every window. In language the same rule pulls a word's ask code toward the key of whatever preceded its next token's earlier occurrence: a learned "what to look for" per token.

**What it cannot do, named in advance.** An ask head's query depends only on the current token. At B2's scored position the current token is always `H4`, so ask heads there cannot choose. B2 is passable only if the depth-2 ask head at `Q(K_j)`'s row fetches the value into the stream and the trunk carries it four tokens: route (i-b) of claude-328's sheet. **In B2-a the pull never acts at `Q(K_j)`.** The training loss covers only scored rows (`assemble` returns `(ids, scored, targets)`), and in B2 the `Q(K_j)` row is not scored. So `a[Q]` gets only the task loss through the four-token carry: B1-a0's cold start plus the carry. B2's scored rows give the pull at `H4`, a token that asks for a different key in each episode, so the pull there averages over keys. B2-aq is the arm that puts the pull at `Q(K_j)`, so a B2 failure can be split into "the choice at the question was never trained" and "the choice was made, and the carry failed" (outcome 3). In language every row has a next-token target, so this gap belongs to the recall task's scoring, not to the read.

## Steps: the arms

**Common to every arm**, as claude-328's sheet:
- `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0 --steps 4096 --eval-every 512 --recall-eval-windows 256 --recall-readings 1`;
- context 192;
- seeds 1 and 2, with seed 3 if the two seeds split.

The forms are claude-328's, unchanged:
- **B0:** `--recall 4`;
- **B1:** `--recall 4 --recall-query 1`;
- **B2:** `--recall 4 --recall-query 1 --recall-header 4`.

The ask arms add `--hamming-read 4 --hamming-ask 1`.

| Arm | Form | Read |
|---|---|---|
| **B0-a** | B0 | ask, `--hamming-attract 1` |
| **B1-a** | B1 | ask, `--hamming-attract 1` |
| **B1-a0** | B1 | ask, `--hamming-attract 0` (the offsets learn from the task loss alone) |
| **B2-a** | B2 | ask, `--hamming-attract 1` |
| **B2-aq** | B2 | ask, `--hamming-attract 1 --hamming-attract-at question` (the pull at `Q(K_j)`'s row) |
| **B0-h** | B0 | claude-328's read, `--hamming-read 4` (the reproduction check, I-R) |
| **B0-n, B1-n, B2-n** | B0, B1, B2 | none (the no-read twins, L-010) |

`λ = 1` is pre-registered. The pull is at most 1 per head (`bits ≤ head_width`); the task loss and the pull are both logged at step 0 and at each checkpoint, so their scales are read, not assumed. No other `λ` is run in this unit.

**Readings**, as claude-328's, with "type" read as "ask":
- `R_ask`, `R_oth` and `R_any` per bucket and tail slot;
- the trace at the question and scored positions;
- the five ablations: `−t` turns the ask heads off;
- **new cut `R^{−aQ}`:** the ask heads at both depths cut at the 4 `Q(K_j)` rows only, through the row cut with a new ask scope (`ReadScope::Ask`, the ask heads' counterpart of claude-328's copy-scope `R^{−cQ}`). It is read on B1-a, B2-a and B2-aq;
- **new:** the mean pull distance `bits / head_width` over the held-out queries' rows, per checkpoint, per depth.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

The rules are claude-328's, applied to the ask arms:

| Arm | Passes when |
|---|---|
| **B0 (instrument)** | `R(B0-a) ≥ 0.95` and `R(B0-n) ≤ 0.02`. Otherwise nothing else is read |
| **I-R (instrument)** | B0-h's `recall.jsonl` is compared, line by line, with claude-328's sealed `runs/s{1,2}/…/B0-h-s{seed}/recall.jsonl` (in the store, checked 8 Nov). The timing fields `train_seconds` and `train_context_tokens_per_second` are ignored. Passes when, **at the step-4,096 line only**: `rows` are equal to the sealed line's; and each bucket's `hits` is within `0.02 · rows` of the sealed line's `hits`, so `|hits − sealed hits| ≤ 0.02 · rows`. `train_loss_mean` is reported beside the sealed line's, with the sealed step-3,584 to step-4,096 change for scale, and is not gated: no repeat run measures its run-to-run drift. The earlier checkpoints are compared and reported but not gated, since hits still rise fast between 512 and 2,048 steps and two runs that are not bit-identical can differ there. A broken default path fails it plainly: claude-328's `−c` reading drops B0-h to 0 / 0, against sealed hits of 0.98–0.996. Otherwise the build moved the default path, and nothing else is read. Byte identity across pods is not assumed: the CUDA kernels are not shown to be deterministic. I-1 is the bit-exact check, on the laptop |
| **B1 (choice)** | `R_ask(B1-a) ≥ 0.50`, `R_ask(B1-a) ≥ R_oth(B1-a) + 0.25`, and `R_ask(B1-a) ≥ R_ask(B1-n) + 0.10` |
| **B2 (carry)** | the B1 rule for B2-a against B2-n |
| **B2-aq** | the B1 rule for B2-aq against B2-n; it decides only the outcome 3 split |
| **B1-a0** | descriptive: the B1 rule is read and reported, and it decides no outcome by itself |

**Kept** means what it meant in claude-328's sheet: weight above 0 for the asked position in at least 0.5 of queries, and at least 0.25 more often than for the paired position.

## Outcome map (read in order, first match wins)

0. **B0-a fails, or I-R fails:** an instrument fault. Repair it and rerun; nothing else is read.
1. **Undertrained:** an arm's `R_ask` still rises by more than 0.05 between its last two checkpoints and misses its bar. Extend it to 16,384 steps under this sheet.
2. **B2-a passes:** the question-keyed read chooses, and the value reaches the scored position, with no pull at the question. Attribute it from the trace and `R^{−aQ}`:
   - an ask head at either depth kept at `Q(K_j)`, with `R^{−aQ}` failing the B2 bar, means a fetch at the question plus a carry by the trunk; the trace names the depth;
   - any other pattern is named from the trace.

   **Next:** a language run with ask heads on claude-325's bare-answer recipe, under its own sheet.
3. **B1-a passes, B2-a fails.** B1-a0 is read alongside: if it passes too, the task loss alone trains the offsets; if it fails, the pull is the working signal. Then split on B2-aq, first match wins:
   - **B2-aq passes:** the choice at the question can be made, and the trunk carries its value, once a signal reaches the `Q` row. B2-a failed for want of the pull's placement at the `Q` row (B2-aq pulls at `Q` and not at `H4`; B2-a pulls at `H4` only), which is a property of the recall task's unscored question row. Language rows carry the pull at every row, so both placements are present there. **Next:** an ask-head language run on claude-325's recipe, under its own sheet, since every language row is scored. The carry lever is not needed yet.
   - **B2-aq fails, and an ask head is kept at `Q(K_j)` in B2-aq's trace:** the read chooses at the question, and either its write to the stream (`w_y ⊙ y` at the `Q` row) or the trunk's four-token carry fails. This sheet does not separate the two: B1 can pass through the readout move alone, so B1-a passing does not show the stream write works. **Next:** the carry lever (claude-326, Q4), tested on B2 first, under a sheet that adds an ask-stream cut to split the stream write from the carry.
   - **B2-aq fails, and no ask head is kept at `Q(K_j)`:** the choice fails under B2's form even with the pull there. Read the header-interference rule (claude-328's, with `−c2`/`−c6` on B2-aq) first; otherwise name it from the trace and the pull distance at `Q`, under outcome 4's first two branches as instrument checks.
4. **B1-a fails,** split by the readings, first match wins:
   - **The pull distance at step 4,096 is above 0.25 of its step-0 value** (about 0.5 at `a = 0`): the signal does not move the offsets far enough. If I-A2 or I-A3 failed, that is an instrument-side fault in the gradient path: repair it and rerun; it is not a mechanism result. If both passed, it is a configuration result for `λ = 1` (too weak against an `lr` that decays to 0, or the task loss moving the shared `key_rotation` under the codes). **Next:** a `λ` step under its own sheet.
   - **The distance falls, and no ask head keeps the asked position:** the picks do not follow the codes, a fault in the choice or key path, also instrument-side. Repair it and rerun.
   - **An ask head keeps the asked position, and `R_ask` fails:** a payload or combine failure, mechanism-side for this configuration (§4.1, §4.2). **Next:** the combine, tested on B1 first.

**Seeds:** two seeds that split run a third; at least 2 of 3 must agree. Every cell, margin, kept rate, chance rate, ablation and pull distance is reported either way.

## Instrument checks

- **I-1 (default paths):**
  - with no new flag, a fixed-seed forward of `HammingRead` on a fixed input gives outputs whose SHA-256 equals a fixture hashed from `main` at the pin;
  - claude-328's I-1 recall-window fixture still passes.
- **I-A1 (ask heads start as copy heads):** with `a = 0`, each ask head's query code equals the copy query code of the same token, bit for bit, at both depths.
- **I-A2 (the pull's targets):** on a hand-built window:
  - `S_t` is exactly the earlier positions whose token is the target, with `s ≥ 1` and `s < t`;
  - rows without a match add 0;
  - `s*` is the nearest key;
  - the attraction's gradient reaches `a` and is zero for the table, `key_rotation` and the copy heads' margins.
- **I-A3 (the pull trains):** on a toy window, gradient steps with `λ = 1` bring `bits(t, s*)` to 0 for a question token whose bijection partner is the key, within a pre-set step count. With `λ = 0` and no task loss, `a` does not move.
- **I-A4 (payload and combine):** on a hand-built window, an ask head that keeps the value position moves the readout toward that value's table point, as a copy head does. `−t` removes the move.
- **I-A5 (the question placement and the ask cut):**
  - under `--hamming-attract-at question`, on a hand-built B2 window, the pull's rows are exactly the `Q(K_j)` rows, each with its episode's `v_j`, and no scored row carries a pull. The rows are found from the training window itself: each scored offset is `query_pos + 4` with target `ask_value` (`recall_query_window`'s `scored_offsets` and `scored_targets`), so the `Q` row is the scored offset minus 4, with the same target. Each episode's `S_t` is its one value position, since a window's 8 values are distinct;
  - `R^{−aQ}`'s row cut removes the ask heads' candidates at the `Q(K_j)` rows only, and leaves the copy heads' picks unchanged.
- **The ask path is covered by I-A1 to I-A5, not by B0-a.** At `a = 0` the ask heads equal the copy heads, and in B0 the copy heads alone solve the task, so B0-a passing does not show the ask path works.
- **I-4 (from claude-328):**
  - every ask arm's step-0 picks equal its no-read twin's, since the weights start at 0;
  - the ask weights move off 0 by step 512;
  - three negative gates each fail their test: the margin's sign flipped; a row allowed to keep itself; **picks that lose their window's offset** (`choose_sorted`'s per-window position offset dropped). That last is the gate claude-328 owed, written against the picks this time, not against the copy key.
- **I-6 (finite CUDA):** a 32-step smoke of every arm on the pod, at finite losses, before the timed run.
- **I-7 (the readings leave training alone):** claude-328's test on B1-a, plus claude-328's laptop guard test.
- **I-R:** as in the bar.

Each check is a unit test (I-6 and I-R excepted). Each has at least one mutation that makes it fail, run on the laptop where it can be and on the pod for the CUDA tests.

## Probe ladder

B0-a and the I-R arm first on each pod. If either misses, stop and repair. B1 and B2 do not run on a failed instrument.

## Data split

As claude-328's sheet. Every window is generated from `(seed, index)`, and held-out windows use `seed ^ RECALL_EVAL_SEED`.

## Base artifact

None. Every arm trains from step 0.

## Minimum test power

As claude-328's sheet, with 512 held-out queries per bucket:
- the 0.10 margin against the no-read twin is about 3.2 standard deviations near 0.5;
- the paired 0.25 margin is about 6.8 standard deviations;
- B0's 0.95 bar is about 9 standard deviations under claude-328's B0-h readings.

## Cost

claude-328 measured these on one RTX 5090:
- **Run times:** about 6 minutes for a read arm and 2.5 minutes for a no-read arm.
- **Per seed:** six read arms and three no-read arms is about 44 minutes, plus a build of about 10 minutes.
- **Total:** two pods in parallel, one per seed, about $0.50 each, about $1 in all.

That is inside the caps and one GPU unit. Outcome 1's extension would add about 4× for the arms it covers.

## Results and stop rule

- **Results:** written to `results/claude/one-model-end-to-end-question-keyed-read-2026-11-08/` in the HF store with a `SHA256SUMS`, and read back before any pod is brought down.
- **Stop** at a B0 or I-R failure, or at any non-finite loss.
- **Watchers:** one is armed on every pod (L-004), with the lease renewed by it.
- **Scripts:** pinned by SHA-256 in a D67 pre-run review section added before any pod.

## Pre-run review (D67, claude-330, 8 Nov 2026)

**[claude] Verdict: FIT TO RUN**, from a fresh non-author reviewer who read this sheet, the pod scripts, the laptop gate outputs and the code at the pin `7275fb01e0c457fea3731ff3291c3a0efb1f3e01`. It found no blocker. Before the review, the author found one gap and fixed it: I-7 on B1-a had no test, so the pin adds `the_recall_readings_leave_ask_training_alone`, which runs claude-328's I-7 body with the ask heads and the pull on.

**Its findings, and what was done:**
- **Three gates unproven (major):** the reviewer could not confirm that `ia4-combine`, `ia2-depth-mean` and `i4-window-offset-keys` are seen by their tests.
  - **`ia4-combine`:** the author read it: it adds the type mean onto the residual instead of moving the residual to it, so I-A4's window-0 row reads `before + v`, not `v`.
  - **`ia2-depth-mean`:** each depth's count is spans × ask heads, which is 2 on I-A2's rows, so the undivided sum is twice the mean.
  - **`i4-window-offset-keys`:** window 1's question meets window 0's keys in the three-window copy test.
  - All three are read from `gates.txt` before any guard is claimed.
- **Minor, fixed:** the sealed loss line moved out of `ir-lines.jsonl` into `ir-loss.txt`, and the gate count is no longer hard-coded.
- **Minor, fixed:** the eight host gates were rerun once at the pin on a quiet tree (`host-gates-pin.out`).

**Deviations the REPORT names:**
- **The CUDA gates run on seed 1's pod only.** They test the code, not the seed.
- **The host gates** (I-A1's margin, I-A2's window start, I-A3's λ = 0, I-A5's three, the ask refusals) ran on the laptop.
- **I-1 at the base** appends the pin's `I1-DIGEST` block to the base commit's read and compares the two printed digests.
- **A seed-3 pod, if seeds 1 and 2 split, has no I-R.** claude-328 sealed seeds 1 and 2 only, so a seed-3 pod reads B0's bar alone and records I-R as `NOT_AVAILABLE`.
- **A gate that does not fail stops nothing.** The binary is built first, and that gate's guard is read as unproven.

**Scripts, by SHA-256** (in `local/claude/claude-330/`; `mutate-328.sh` is claude-328's, unchanged):

| Script | SHA-256 |
|---|---|
| `job-330.sh` | `8d4f86a78cb2e6e8e929a3e030943d4dc0c8cecc719c54ffdb18e6df512e80d6` |
| `chain-330.sh` | `110145dde3e26e6c90c08f906948950071a86ac08c2efdf4986d11831b974251` |
| `seal-330.sh` | `302257b9a16168199c71204d64c9f00fea9f17ee7ca8d901289039a6320575c1` |
| `mutate-330.sh` | `746fd39d32fc8c781c9a45dfe70e1c8192d207aa667f042e925bb83c5ba9424f` |
| `mutate-328.sh` | `6c6647e4b7882d39db24e189e2a3b29ca595317394b9d9e707e26bd20f6745c8` |
| `watch-330.sh` | `3ebc07a18f2dc85b2edd4b11fd4d307126db27feb255833e7db299dac6619e4b` |
| `seed-330.sh` | `2926e5a7dfd6f64a1ceffc2f52599c84b84b432db1a463f5092dc0693a7f8f86` |

**[claude] First launch, and the fix (8 Nov 2026).** Both pods stopped at I-0 with `EXIT 4 I-0: ask_heads_start_as_copy_heads_bit_for_bit did not pass`. Both sealed to `runs/s1` and `runs/s2` and went down. The two pods cost about $0.12 together.
- **The cause was an instrument fault, not the mechanism.** I-1 built the base checkout into the same target directory the pin used. Cargo keeps a workspace member's absolute path out of its artifact hash, so it counted the base's binary as fresh for the pin. It finished in 0.06 s and ran the base's tests: 442 of them, 426 passed, with the same binary hash, `092ee3c6f32e4c6e`, in `i1-base.err` and `tests.err`. The pin's new tests did not exist in that binary.
- **The fix:**
  - The base builds into its own target, `/root/build/target-base`.
  - Two new guards stop the job if a step runs another checkout's binary. The base's digest step must show the crate compiling from the base's checkout. I-0 must show it compiling from the pin's.
  - `seed-330.sh` now takes the store root and allows only `runs-2`. `runs/s1` and `runs/s2` stay sealed as they are (L-002).
  - Only `job-330.sh` and `seed-330.sh` changed. Their old hashes were `a1a60323fe1d0702feb433c8a493ede8769d6f96c8c68aac74fe8ffbbf9c2d5b` and `a9cc06f41c4ee2c32390a9f0b2bd351b3146591bcd8a269ed73ed37ef752cf30`, and the table above now shows the new ones.
- **The rerun** goes to `runs-2/s1` and `runs-2/s2`, at the same pin, with the same bars.
