# Prep sheet: recall where the query is not the key (one-model-end-to-end, claude-327, 8 Nov 2026)

TEST FITNESS: FIT (round 4, claude-328, 8 Nov 2026, a fresh non-author reviewer at 3c825bb6aca265df40de1c6d2422f52292d3660e). All eight round-3 findings were confirmed resolved, and the round's 7 minor findings are applied below: interference read at both depths with a slot-1 check, (i-b) without the depth-split terms when that control fails, first match in outcome 2, the per-slot balance test, clipping at B2's slots 1–3 only, the row cut's limits, and I-7's second mutation. The reviewer did not reach `in_space_long.rs`; the lab checked those claims at the same commit (`recall_window` 11810, the `RECALL_*` constants 11788–11797, `recall_hits` 11906, `run_recall` 11936). The history before round 4: Round 1 (a fresh non-author FIT) read NOT FIT, with 2 blockers, 4 majors and 4 minors. Round 2 (another fresh FIT) read NOT FIT, with 1 blocker, 3 majors and 3 minors: the read's second depth lets a copy head answer at `Q(K_j)` and the trunk carry the value forward. Round 3 (claude-328, a fresh FIT) read NOT FIT, with 1 blocker, 2 majors and 5 minors: a carry failure read as a query failure; no B0 control for the depth-split ablations and no position-specific one; no check that the readings restore the weights; an impossible depth-6 case; header interference; the far bucket cut by the window; balance per tail slot; the paired margin's power. Every finding of the three rounds is applied below. A D67 pre-run review comes before any pod.

## The question

[claude-326's assessment](../../loop/tasks/one-model-end-to-end/claude-326/ASSESSMENT.md) (Q4, Part B) asks whether the trained Hamming read can **choose a value by a query that is not the key**, the way `m2-fresh256`'s "What is my job?" chooses the value after "I am a … by trade".
- **What the 27 Oct probe showed:** it recalled 0.984–0.994 ([sheet](in-space-trunk-to-text-hamming-recall-2026-10-27.md), notebook row 109).
- **Why it does not answer this:** its query writes the key again (`in_space_long.rs:11803`), so the copy head's exact match on the current token does all the work.
- **Why the panel cannot answer it:** claude-327's Part A shows that on claude-322's, claude-323's and claude-325's served artifacts the reply's first token is one of 2–3 frame words across all 256 rows, whichever relation is asked. So the panel's first-target rank reads format, not choice.

**Two routes to the value exist in this read, and the sheet separates them:**
- **(i) Table alignment.** The input table is trained (`fix_input_table: false`), so a copy head can pull `Q(K)`'s sign code onto `K`'s and match `K` exactly. This works only at a position whose current token is the query token.
- **(ii) A learned query.** The type heads' query is built from the residual, so it can carry the query forward and pick `K`'s fact from any position: **(ii-a)** at the scored position, or **(ii-b)** at `Q(K_j)` at depth 2, with the trunk carrying the result, as (i-b) below.

**The read sits at two depths** (`HAMMING_DEPTHS = [2, 6]`), and both run at every position. So route (i) has two forms:
- **(i-a)** a copy head at the scored position, which needs the query token to be the current token there;
- **(i-b)** a copy head at the query token `Q(K_j)`, which fetches the value there and writes it into the stream; the trunk then carries it to the scored position. Its gradient comes from the scored position's loss, so it trains. Only depth 2 can do this: depth 6 reads the final norm's output, the readout, so nothing it brings back reaches a later position.

In the panel, the current token at the answer position is the header's last token (`:`), the same for every question. So (i-a) cannot answer the panel, and (i-b) and (ii) can. B2's header recreates that: it closes (i-a) and leaves (i-b) and (ii), which the readings below tell apart. Either is a positive for this read on the panel's form; they lead to different next steps.

## Steps: the arms

**Common to every arm**, as row 109's sheet:
- the FBS sheet trunk, `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`;
- context 192;
- seeds 1 and 2, with seed 3 if the two seeds split, as row 109;
- held-out queries decoded as the whole table's nearest token.

| Arm | Task | Read |
|---|---|---|
| **B0-h** | `--recall 4`, unchanged: the query writes the key again | `--hamming-read 4` |
| **B0-n** | the same | none |
| **B1-h** | two-class, query at the scored position | `--hamming-read 4` |
| **B1-n** | the same | none |
| **B2-h** | two-class with a 4-token header | `--hamming-read 4` |
| **B2-n** | the same | none |

**The two-class task** (new, `--recall-query 1`):
- **Episodes:** each 192-token window holds **4 episodes**, each over 2 relation tokens. That is 8 distinct relations per window, drawn from a 16-token relation range.
- **Facts:** each relation is **written once**, as `K v`, with a fresh value.
- **The query:** each episode asks one of its two relations, `j`, drawn uniformly and independently of everything else. The query writes `Q(K_j)`, where `Q` is a fixed bijection from the 16 relation tokens to a disjoint 16-token query range, the same in every window and seed. Each `Q` token occurs at most once in a window.
- **Targets:** the scored target is `K_j`'s value, written into the input at the next position, as `--recall` writes its targets (teacher forcing). These 4 are the window's only scored targets.
- **Signal:** 4 targets × 16 windows × 4,096 steps is 262,144 scored targets, row 109's count at its saturation step, 1,024 × 16 × 16.
- **Distance** is counted from a fact's value position to its episode's `Q(K_j)` position, in both B1 and B2. The asked fact's distance falls in one of the far buckets, 33–96 or 97–180, two episodes per bucket per window. **The other fact's distance is drawn uniformly from the same bucket**, independently, so within each bucket the asked fact is the nearer one half the time.
- **Placement:** the 4 queries sit at fixed positions at the end of each window, after every fact: the last 8 positions in B1, each episode's tail being `Q(K_j) v`, and the last 24 in B2, each tail being `Q(K_j) H1 H2 H3 H4 v`. The four tails are the window's **tail slots** 1–4, in position order. In B2 the queries sit at 168, 174, 180 and 186; in B1 at 184, 186, 188 and 190.
- **Buckets to slots:** each window draws a uniform assignment of its four buckets (two 33–96, two 97–180) to the four slots, independently of `j`.
- **The window cuts the far bucket:** position 0 is BOS, as in `--recall`, so a fact `K v` starts at position 1 or later and a slot's largest distance is its query position less 2. That cuts the 97–180 bucket only at B2's slots 1–3 (166, 172, 178); B2's slot 4 (184) and every B1 slot (182–188) are not cut. Each distance is drawn uniformly over its bucket's range clipped at that largest distance.
- **Collisions:** the facts sit at the positions their drawn distances name, with filler between, and every fact sits before the first tail slot. A draw that collides with another fact is redrawn from the same generator.
- **Order:** which of an episode's two facts is written first is drawn uniformly, so pairing and order cannot be read from position.
- **The REPORT gives** the realized distance distribution of each bucket in each slot.
- **Ranges:** values, relations, queries, header and filler are disjoint.
- **Recording:** each held-out query records whether nearest names the asked value (`R_ask`), the paired relation's value (`R_oth`), another of the window's values (`R_any`), or none, with its tail slot. `R_ask` is reported per bucket and per tail slot.

**The header** (`--recall-header 4`): four fixed tokens `H1 … H4`, from their own range, follow each `Q(K_j)`. The value `v` is written 5 positions after `Q(K_j)`, and it is predicted at `H4`'s position, the scored position. In B1 the value follows `Q(K_j)` directly and is predicted at `Q(K_j)`'s position.
- The current token at every scored position is then `H4`, whichever relation is asked, so (i-a) cannot tell the relations apart.
- The header occurs 4 times a window, so a copy head that matches `H4` finds earlier headers. Their next tokens are the other episodes' targets, which are scored "another value", not `R_ask`. This is noise, not a shortcut, but it grows with the tail slot (slot 4's `H4` has three earlier headers, slot 1's none), so `R_ask` is read per slot. It may pull the copy heads' combine weight down in B2, and the REPORT says whether it did. The outcome map reads it as header interference when turning off the depth-6 copy heads raises B2's `R_ask` (below).

**Held-out:** 256 windows for every arm.
- B1 and B2: 1,024 queries, 512 per far bucket, each query's bucket being its asked fact's distance.
- B0: `--recall 4` scores 4 queries per bucket per window, so 1,024 per far bucket.

**Eval-time readings at every checkpoint, for B1-h and B2-h** (and on B0-h for the controls):
- **The trace,** taken at two positions, the query token `Q(K_j)` and the scored position (in B1 they are the same position). For each copy and type head at each depth, it records whether the asked fact's value position, the paired one, or neither is **kept with weight above 0** among its 16 kept positions, and that weight. A position kept at weight 0 counts as not kept.
- **The chance rate** beside each kept rate: 16 over the number of earlier positions, the rate a head that keeps 16 positions at random would show.
- **Five ablations,** each recomputing `R_ask` in the readings only, not in training. Each sets some combine weights to 0 or cuts some rows, and restores them before training resumes:
  - `R^{−c2}`: the copy heads at depth 2 off;
  - `R^{−c6}`: the copy heads at depth 6 off;
  - `R^{−c}`: the copy heads at both depths off;
  - `R^{−t}`: the type heads at both depths off;
  - `R^{−cQ}`: the copy heads at both depths cut at the 4 `Q(K_j)` rows only, through the read's row cut (`set_row_cut` with the copy scope): the limit is 0 at each `Q(K_j)` row (no candidate, so the copy heads find nothing there) and `time` at every other row of the `batch · time` vector. In B1 the `Q(K_j)` row is the scored row.

**The code this needs** (built in this unit, claude-328, in `crates/uor-r4-float-train`):
- `recall_window` gains the two flags, default off;
- a held-out window count setting, since `RECALL_EVAL_WINDOWS` is 128;
- the outcome counts, per bucket and tail slot, and the realized distance distributions;
- the trace at both positions, with weights;
- the five eval-only ablations, restoring the weights and row cuts after each reading.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

`R_ask`, `R_oth`, `R_any` and the ablated `R`s are counted over the held-out queries of each bucket.

| Arm | Passes when |
|---|---|
| **B0 (instrument)** | `R(B0-h) ≥ 0.95` and `R(B0-n) ≤ 0.02`. Otherwise the run is an instrument fault and nothing else is read |
| **Ablation controls (on B0-h)** | `R^{−c}(B0-h) ≤ 0.10`, since the copy heads solve B0, and `R^{−t}(B0-h) ≥ 0.95`, since the type heads are not needed there. Otherwise the ablations are not a valid reading, and outcomes 2–4 are read from the trace alone |
| **Depth-split controls (on B0-h)** | `max(R^{−c2}(B0-h), R^{−c6}(B0-h)) ≥ 0.90`: one depth's copy heads alone solve B0, so turning one depth off is not by itself a fault. Otherwise `R^{−c2}` and `R^{−c6}` are not used, and depth is read from the trace and `R^{−cQ}` alone |
| **Row-cut control (on B0-h)** | `R^{−cQ}(B0-h) ≤ 0.10`: in B0 the query row is the scored row, so cutting the copy heads there must remove the answer. Otherwise `R^{−cQ}` is not used |
| **B1 (choice)** | `R_ask(B1-h) ≥ 0.50`, `R_ask(B1-h) ≥ R_oth(B1-h) + 0.25`, and `R_ask(B1-h) ≥ R_ask(B1-n) + 0.10` |
| **B2 (carry)** | the B1 rule for B2-h against B2-n |

An ablated reading **keeps the bar** when it passes its arm's rule with `R_ask` replaced by the ablated `R`.

**Kept**, for a head at a position, means: the asked value's position is kept with weight above 0 in at least 0.5 of the queries, and at least 0.25 more often than the paired value's position.

**What each rule rejects:**
- A copier that picks one of the episode's two relations at random scores about 0.5 on `R_ask` and on `R_oth`, so the bar asks for selectivity.
- A copier of the nearer fact scores about 0.5 on each within a bucket, since the other fact's distance is drawn from the same bucket.
- A copier over the window's 8 values scores about 1/8.

## Outcome map (read in order)

0. **B0 fails:** an instrument fault. Repair it and rerun; nothing else is read.
1. **Undertrained.** If `R_ask` of an arm still rises by more than 0.05 between its last two checkpoints and misses its bar, the arm is undertrained: extend it to 16,384 steps under this sheet. That is not a mechanism result.
2. **B2 passes:** the read answers a question whose answer position does not show the question. Attribute it, using each ablation only if its B0 control passed, first match wins:
   - **`R^{−c}` keeps the bar:** the type heads carry it. Under (ii-a) the trace at the scored position shows a type head keeping the asked position. Under (ii-b) the trace at `Q(K_j)` shows a depth-2 type head keeping it and none keeps it at the scored position: a learned query at the question plus a carry by the trunk.
   - **`R^{−c2}` fails, `R^{−c6}` keeps the bar, `R^{−cQ}` fails, and the trace at `Q(K_j)` shows a depth-2 copy head keeping the asked position:** route (i-b), a copy at the query plus a carry by the trunk. All four must hold before (i-b) is named. `R^{−cQ}` is the one that ties it to the query row. If the depth-split control failed, (i-b) is named on `R^{−cQ}` failing and the trace alone, without the `R^{−c2}` and `R^{−c6}` terms. This is a positive for this read on the panel's form, not an instrument fault: the panel's question word is a current token too.
   - **Any other pattern:** read from the trace at both positions; the REPORT names the heads and depths that carry it. It is a positive for B2, with the attribution open.

   **Next, either way:** claude-325's bare-answer recipe, larger or with a curriculum from B2's form to the panel's. Under (i-b), the next sheet also checks that the panel's question word can align with its fact's key word in the trained table.

**Read before outcomes 3 and 4, when B2 fails — header interference:** if `R^{−c2}(B2-h)` or `R^{−c6}(B2-h)` is at least `R_ask(B2-h) + 0.10`, each counted only if its B0 control passed, the copy heads' matches on earlier headers cost B2 its answer. A depth-2 copy at `H4` writes an earlier episode's value into the stream, so either depth can do it. That is a reading of B2's form, not a mechanism negative; the per-slot `R_ask` shows whether it grows with the slot. Outcomes 3 and 4 are still read and reported, marked as under interference. Next: B2 with each episode's header drawn from a 16-token header range independently of `j`, so `H4` has no earlier match, under an addendum to this sheet with its own FIT.

3. **B2 fails, B1 passes:**
   - **`R^{−c}` fails, `R^{−t}` keeps the bar, and the trace shows a copy head keeping the asked position at the scored position:** B1 is carried by the copy heads (route i-a at B1's scored row, which is its `Q(K_j)` row). Read as row 4, whose first test is the carry.
   - **Otherwise,** the read chooses at the query but does not carry the choice four tokens. Next: the carry, a read at the query token whose result is written forward. Tested on B2 first.
4. **B2 fails, and B1 fails or is carried only by copy heads at the scored position,** split by the trace at B2-h's two positions, first match wins:
   - **The trace at B2-h's `Q(K_j)` shows a depth-2 copy or type head keeping the asked position:** the read chooses at the query, and the trunk does not carry the value to `H4`. A carry failure, read as outcome 3's Otherwise. Next: the carry.
   - **No type head keeps the asked value's position at either position, and B2-h's slot-1 `R_ask` (no earlier header) also misses the bar's 0.50:** a query failure. If slot 1 passes while the whole arm fails, the failure is read as header interference, as above, not a query failure. This read's configuration (type-head rotation query, read 4, width 256) does not learn the choice on this task; that is mechanism-side for the configuration (§4.1, §4.2). Next: lever (b) of the assessment, a question-keyed read, under its own sheet. It must pass B2 here before any language run.
   - **A type head keeps it, but `R_ask` fails:** a payload or combine failure. The type heads' payload, the residual `v_s` added on the readout, is the form that failed the 27 Oct probe (module doc, steps 5–6). Next: a type-head payload that brings back the kept position's table point, as the copy heads' payload does. Tested on B2 first.

**Seeds:** two seeds that split run a third; at least 2 of 3 must agree. Every cell, margin, kept rate, chance rate and ablation is reported either way.

## Instrument checks

- **I-1:** with neither new flag, `recall_window` writes windows whose SHA-256 equals a fixture hashed from `main`'s output at the pin. The fixture covers seeds 1 and 2, training indices 0–127, and the held-out seeds `seed ^ RECALL_EVAL_SEED`. The test fails if the default path moves.
- **I-2 (two-class windows):**
  - the query token is never a relation token;
  - `Q` is the same in every window, and each `Q` token occurs at most once a window;
  - each relation is written once;
  - the target is the asked relation's value;
  - exactly 4 targets are scored;
  - the ranges are disjoint;
  - both of an episode's facts have distances inside the asked fact's bucket, clipped at the slot's largest distance, and every fact sits before the first tail slot;
  - over 10,000 windows, in each bucket × tail slot cell (about 5,000 queries, where 2% is about 2.8 standard deviations), `j` is balanced against which fact is nearer and against which fact is written first, each within 2%;
  - each tail slot receives a 33–96 bucket in 0.5 ± 0.02 of windows.

  Each is a unit test with a mutation that makes it fail.
- **I-3 (header):** each value sits 5 positions after its `Q(K_j)` and is scored at the position 4 after it (`H4`); the 4 episode tails fill the window's last 24 positions; the header tokens are fixed. Tested the same way.
- **I-4:** every read arm's step-0 predicted points and step-0 loss equal its no-read twin's bit for bit, since the read's weights start at 0. The Hamming combine weights move off 0 by step 512, by `hamming_weight_mean_abs`. The read's three negative gates, run before the timed run, each fail their test with code 101: the margin's sign flipped; a row allowed to keep itself; picks that lose their window's offset.
- **I-5:** the trace and each of the five ablations match a hand-built two-episode window in a unit test, at both positions and both depths. Ablating a head that found the value drops `R_ask` on it, the row cut's limits are 0 at the `Q(K_j)` rows and `time` elsewhere, and a position kept at weight 0 counts as not kept.
- **I-6 (finite CUDA):** a 32-step smoke of every arm on the pod, at finite losses, before the timed run.
- **I-7 (the readings leave training alone):** 1,024 steps of B2-h with the readings on (every 512 steps) and off give bit-identical losses at every step and bit-identical weights at the end. Two mutations each make the test fail: one skips the weight restore (or the class mask's reset) after an ablation; the other skips `set_row_cut(None)` after `R^{−cQ}`, leaving the thread-local cut on the training thread.

## Probe ladder

B0 first on the pod, with its ablation, depth-split and row-cut controls. If B0 misses its bar, stop and repair; B1 and B2 do not run on a failed instrument. If only a control fails, B1 and B2 run, and the readings that control guards are replaced by the trace, as the bar table says.

## Data split (generated, none from the store)

Every window is generated from `(seed, index)`. Held-out windows use `seed ^ RECALL_EVAL_SEED`, as the 27 Oct sheet, so they never equal a training window at these run lengths.

## Base artifact

None. Every arm trains from step 0 at the pin the next unit names.

## Minimum test power

- **B1 and B2, near 0.5:** 512 held-out queries per bucket. One arm's standard deviation is about 0.022, and a difference of two arms about 0.031, so the 0.10 margin against the no-read twin is about 3.2 standard deviations.
- **The 0.25 margin is paired:** `R_ask − R_oth` is taken over the same queries, and the two outcomes exclude each other. At `R_ask` 0.5 and `R_oth` 0.25 its standard deviation is about 0.037, so the margin is about 6.8 standard deviations.
- **Per slot,** 128 queries per bucket and slot: those readings are descriptive, not part of the bar.
- **B0, near 0.985:** 1,024 per bucket. Its standard deviation is about 0.0038, so its 0.95 bar sits about 9 standard deviations under row 109's seed-1 reading.
- **Kept rates:** over 512 queries, the 0.25 margin is about 8 standard deviations.

## Cost

Row 109 ran at about 12,187 context tokens a second with the read, so one arm of 12.6M positions takes about 17 minutes with the read and about 12 without. Six arms at two seeds is about 3 hours of one RTX 5090, about $1.5–2.5, inside the caps. The readings are eval-only and add minutes. The undertrained extension (outcome 1) would add about 4× for the arms it covers.

## Results and stop rule

- **Results:** written to `results/claude/one-model-end-to-end-query-recall-2026-11-08/` in the HF store with a `SHA256SUMS`, and read back before any pod is brought down.
- **Stop** at B0's failure, or at any non-finite loss.
- **A pod that does not answer SSH** is retried once on another offer.
- **A watcher is armed on every pod** (L-004).

## Pre-run review (D67, claude-328, 8 Nov 2026)

**[claude] Verdict: PASS**, from a fresh non-author reviewer who read the sheet, the six pod scripts, the laptop gate outputs and the recall structs at the pin `659b2286b56afcd3e9056539d1f5d72d6c403c35`. It found no blocker and no major. Two of its minors were fixed before launch:
- **Minor 1:** a loss written as `null` after step 0 now marks the arm diverged, so the stop rule acts on it whatever stderr says.
- **Minor 3:** the watcher renews the lease to 1 h every 25 minutes, so a stalled pod's lease lapses within the hour.

**Deviations the REPORT names:**
- **I-7's first mutation is compound.** It deletes the class mask's reset and also reverses the ablation order. Alone, the deleted reset is an equivalent mutant: the last ablation, `R^{−cQ}`, sets an empty mask itself, so the test could not fail. Both code reviewers and the author found this independently.
- **I-4's step-0 record** compares the twins' step-0 readings: rows, hits and the two-class counts, which are the nearest picks. It does not compare the predicted points and the step-0 loss, because `run_recall` logs neither at step 0.
- **I-4's three gates** run the whole `flock_read_hamming::` module. `gates.txt` names the tests that failed.
- **A gate that does not fail stops nothing.** The binary is built first. Such a gate's guard is read as unproven.
- **Not scripted here:** seed 3 (if seeds 1 and 2 split) and outcome 1's 16,384-step extension. Each gets its own scripts and review.

**Laptop gates, all green:**
- the job's checks on fixtures: 32 cases, and 15 of 15 mutations caught;
- the six source mutations: each applies at the pin with its exact diff and builds;
- the seal and chain test, and the watch and chain test (5 of 5 mutations caught);
- each arm's flags reach the CLI's CUDA refusal, not a flag error.

**Scripts** (in `local/claude/claude-328/`), SHA-256:

| Script | SHA-256 |
|---|---|
| `job-328.sh` | `a49075ca016b2a7735989762ab3882a165ca8a7fba4cd19225272e37c683c465` |
| `chain-328.sh` | `aa610c745effc5343284475bda00993bdb71a35d8efc20224f33cc6ca52eda48` |
| `seal-328.sh` | `021aa1a3a320f25739b6d8b0cb4d0febb52572637427700c38fa87db90e2cb30` |
| `mutate-328.sh` | `6c6647e4b7882d39db24e189e2a3b29ca595317394b9d9e707e26bd20f6745c8` |
| `watch-328.sh` | `5af0d07165b4d163c867147012a187ec40e483abb621b69c0d2915fee8bfcc8f` |
| `seed-328.sh` | `5928ba2bc53413c16213cb715f64850667fe991453ccf5327fdbea775007902e` |

**Compute:** two Vast RTX 5090 pods, one seed each, at most $0.70/h each, inside 4 pods and $8/h. The reviewer's estimate is 2.5–3.5 h per pod, about $4–5 in all.
