# Prep sheet: the width slope on (b64L) (scale-probe, claude-317, 8 Nov 2026)

Lab: claude. Item: scale-probe (#3472). Order: claude-317. Track B of Casey's 8 Nov decision, "Both in parallel" (#2028 6715318106).

TEST FITNESS: FIT WITH CHANGES, from two rounds. Round 1 FIT WITH CHANGES (a fresh non-author review, 8 Nov). Two blockers, five majors and five minors, all applied below; "Changes from round 1" lists each. Round 2 FIT WITH CHANGES (a second fresh review, 8 Nov): no blockers, two majors, six minors, all applied below; "Changes from round 2" lists each. D67 pre-run review GO WITH CHANGES (8 Nov): one blocker and three minors, all applied; the section at the end lists each, with the scripts' hashes.

## Question

Does the served recipe (b64L) get measurably better on human held-out text as it gets wider, at a fixed head width of 64 coordinates, with everything else fixed?

- claude-295 asked this of the old recipe, and the question could not be read: its training loss did not fall on human text at either width.
- claude-296 classed that as the objective (mechanism).
- (b64L), the recipe that came after, is ahead of D3's T2 at both seeds: +75,038 and +85,590 rows against a band of 10,552 (claude-301).
- So this is the first slope that can be read.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's and claude-315's. `train-in-space` is built on each pod at the pin, with its sha256 recorded.
- **The recipe:** claude-301's (b64L), unchanged except for the width, the heads and the micro-batch:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --batch 64 --eval-every 1024 --steps 16384`, plus `--train-split` D3.
- **The variable is width at a fixed head width of 64:** `--hamming-read` is width / 64. Heads change with width (2, 4, 8), so heads are a co-cause of any width effect, and every reading says so.
  - `HammingRead::new` (`flock_read_hamming.rs` ~989) refuses a head wider than 64, so width 512 cannot keep (b64L)'s 4 heads.
  - Width 128 could keep 4 heads of 32 coordinates. The contrast cell (w128h4) below separates the head count from the width at 128, at one seed.
- **The micro-batch** changes memory and summation order only. claude-300's CUDA test shows micro-batches weighted by their scored rows give the whole batch's step. claude-301's I-e matched the whole batch to about 10⁻⁸ relative at micro-batch 4. Agreement is to rounding, not bit-exact, and I-e checks it at each pod's own width. The batch stays 64 in every arm.
- **`--seed`** seeds both the initial weights (`build_fbs_model_at(arm, config.seed, …)`, `in_space_long.rs` ~1305) and each epoch's window order (`in_space_long.rs` header). So seed s at one width and seed s at another train on the same windows in the same order. That is the basis for pairing by seed (Minimum test power).
- **Not changed:** lr (0.001), steps (16,384), data, context. lr was tuned at width 256, so it is a co-suspect in every row of the outcome map except "rising".

## Data split

claude-301's, unchanged:
- **TRAIN D3:** `tokens.u16` `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`, `response_mask.u8` `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`.
- **The panel:** claude-295's four held-out panels, cut by source id: oasst2, wiki-00000, wiki-00020 and wiki-00040. That is 1,460,697 no-memory rows, row identity `5f88a5b3…`, with each panel's heldout sha256 pair as in `read-301.sh`.
- **The 8-token overlap census** with TRAIN D3 is claude-295's seen set (227,312 seen, 1,228,450 unseen). Every pair reads `clean=` that set. The TRAIN is the same, so the census stands.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- P-dev is not read.

## Arms

| arm | width | heads (`--hamming-read`) | `--micro-batch` | seeds | where |
|---|---:|---:|---|---|---|
| (w128) | 128 | 2 | 16 | 1, 2 | new, pod C |
| (w128h4), contrast | 128 | 4 (32 a head) | 16 | 1 | new, pod C |
| (w256) = (b64L) | 256 | 4 | 16 | 1, 2 | claude-301's seal, not retrained |
| (w512) | 512 | 8 | 8, or 4 (I-m) | 1, 2 | new, pod A (s1) and pod B (s2) |

Width 1,024 is not an arm:
- on the old recipe it did not fit a 32 GB card (claude-295);
- the 80 GB cards are owner-forbidden in `uor-pod`;
- it waits on a 48 GB card unit, which rows 1 and 2 below unlock.

**Memory:** (b64L) at width 256 peaked at 27,179 / 27,276 MiB of the 5090 at micro-batch 16 (claude-301). Memory is expected to grow with micro-batch × width, so (w512) at micro-batch 8 is expected near that same peak. That is why 4 is registered as the fallback.

## Readings

All are read with `scale-probe grade` and `scale-probe pair` at `clean=` D3's seen set, as in claude-301's `read-301.sh`. The wider arm is always the A side.

**The verdict the Bar reads, fixed now:** each pair's `pooled.verdict`, over all held-out rows, as claude-298's and claude-301's Bars read it. `unseen.pooled.verdict` is written beside it as a qualifier ("ahead; unseen inside"), never in its place.

- **R1:** (w512) against (w256).
- **R2:** (w256) against (w128).
- **R3:** each arm against D3's T2 and against D3's unigram.
- **R4, served speed (speed only):**
  - each seal's export, run by `served-reply-panel` at `main` `3d3bbb6a` (claude-316) on `m2-fresh256`, on this laptop, one process at a time: `decode_tokens_per_second` and `tokens_per_second`;
  - training tokens a second from `progress.jsonl`.
  - R4 claims speed only. Served-against-trained quality parity is claude-308's open item and is not claimed here. Speed against context length is not measured, because the scorer has no context option; that part of the item's done-when stays open, and the REPORT says so.
- **R5, the curve (recorded):** pooled top-1 rows for each width and seed. The fitted slope is the least-squares slope of each width's mean over seeds against log₂ width, in rows per doubling, over widths 128, 256 and 512. Whether it is distinguishable from flat is decided only by R1 and R2's verdicts (the Bar), never by the fit alone.
- **R6, the contrast (recorded, no verdict):** (w128h4) seed 1 against (w128) seed 1 and against (w256) seed 1, d only. One seed has no band, so R6 only points to a cause.
- **R7, the fallback span (read only on an I-q failure):** (w512) against (w128), both pairings, the same rules as R1. Both sides come from this unit's build, so it needs no tie to claude-301. It is recorded with its verdict and gives row 6 a slope; it never fills R1 or R2's cell in the Bar.

**A pair needs both seeds on both sides.** With either side at one seed, that reading is "not read", never a one-seed verdict.

## Minimum test power

- **The band.** `pair`'s band for each seed is the larger of the McNemar floor (about 1,100 rows) and the spread of d over the seed pairs (`assess`, `docs/labs/scale-probe/instrument/src/lib.rs` ~922).
  - The 10,552 band was (b64L) against T2, a fixed table. In R1 and R2 both sides vary by seed, so the band adds two arms' seed noise. Expect roughly √2 × 10.5k ≈ 15k rows.
  - `ahead` at two seeds also needs the smaller d to clear the band.
- **The pairing rule, fixed now:** seed s against seed s, the pairing that shares window order. R1 and R2 are also run with the seeds crossed. A verdict counts only if it is the same under both pairings; if they differ, the reading is `mixed` for the Bar.
- **Known limit:** that (w256) and the new arms share window order rests on reading the code (`--seed` seeds the initial weights and each epoch's window order, `in_space_long.rs` ~1305). No test pins it, and I-q ties the build, not the order. The crossed pairing is the guard: if order mattered, the two pairings would disagree and the reading would be `mixed`.

## Bar

Set before any pod. Each reading is `ahead`, `inside`, `behind`, `mixed` or not read, and every combination has one row:

| R1 (512 vs 256) ↓ / R2 (256 vs 128) → | ahead | inside | behind | mixed | not read |
|---|---|---|---|---|---|
| **ahead** | 1 rising | 1 rising at the top | 5 non-monotone | 1 rising at the top | 1 rising at the top |
| **inside** | 2 flat at the top | 3 not distinguishable | 5 non-monotone | 3 | 3 |
| **behind** | 4 falling at the top | 4 falling at the top | 4 falling | 4 falling at the top | 4 falling at the top |
| **mixed** | 3 | 3 | 5 non-monotone | 3 | 3 |
| **not read** | 2b rising below 256 only | 3 | 5 non-monotone below 256 | 3 | 6 nothing read |

## What each outcome unlocks (the outcome map)

1. **Rising, or rising at the top.** Width pays on human text at a fixed 16,384 steps. Heads are a co-cause (R6 points).
   - The next scale-probe unit takes width 1,024 on a 48 GB card (L40S or RTX 6000 Ada, explicit tier) at 16 heads, plus a data cell (D4) at width 512.
   - Track A may adopt the widest width whose served decode stays fast enough.
2. **Flat at the top (R2 ahead, R1 inside).** 512 gains nothing over 256 here. D3 at 16,384 × 64 windows is about 6.8 passes.
   - The co-suspects are the data, the number of updates, and lr (tuned at 256).
   - The next unit tests them at 512, one seed each, after a smoke: lr 0.0005, and 32,768 steps. Then D4 at 512. Width 1,024 waits.
   - **2b, rising below 256 only (R1 not read):** as row 2, with the cause of R1's absence named.
3. **Not distinguishable.** The band is too wide for two seeds; lr is a co-suspect.
   - The next unit adds seeds 3 and 4 at the pair that read `inside` or `mixed`. Nothing is closed (L-007).
4. **Falling.** Before any mechanism call, the instrument suspects are lr and, at the step that changes heads, the head count.
   - The next unit checks lr at the falling width (0.0005 and 0.00025, one seed, a smoke first).
   - Only if that also falls is the result classed mechanism. That closes this configuration, never the mechanism (§4.1).
5. **Non-monotone (R2 behind: 128 beats 256).** The likeliest cause is the head count (2 against 4) or lr, not width.
   - The next unit reads R6 first, then runs an lr check at 256 and 128.
6. **Nothing read.** Instrument. A repair unit, inside the ORDER's bound. If an I-q failure caused it, R7 is reported as the slope across the new build, and the repair unit ties (w256) again before any width call.

## Instrument checks

Registered here, before any pod. They are claude-301's gates, with the changes noted.

- **I-0:** the pin, then `cargo test --release --locked -p uor-r4-float-train --features cuda --lib`, including claude-300's three micro-batch tests by name, as passed.
- **I-3:** each run's `config.json` must show TRAIN D3's two sha256s, the arm's `width`, `hamming_read` and `micro_batch`, `batch` 64, `steps` 16,384, the seed, and claude-301's recipe fields.
- **I-e,** per pod, at the pod's widest arm and the pod's first seed, with `--eval-every 1`. There are three runs: (whole), (micro), and the control (read0), which is (whole) with `--hamming-read 0`.
  - **Pods A and B (width 512):** batch 8, with (micro) as four micro-batches of 2. If (whole) runs out of memory, batch 4 with (micro) as four micro-batches of 1. If that too runs out, I-e is NOT_RUN as an instrument and the pod trains nothing.
  - **Pod C (width 128):** batch 16, with (micro) as four micro-batches of 4. This is claude-301's own I-e.
  - **Pass:** (micro) within 10⁻⁶ relative of (whole) at every step, and (read0) at least 10⁻⁵ from (whole) at the last step.
  - **Power fallback:** claude-301's seed-2 margin was thin, 1.85 × 10⁻⁵ against 10⁻⁵. If at 4 steps (micro) passes but (read0) is under 10⁻⁵, all three runs repeat at 8 steps and the control is read at step 8. Only an underpowered result at 8 steps fails I-e as underpowered.
  - **Fail:** the pod trains nothing, and the unit becomes a diagnosis.
- **I-m,** per arm, from its smoke (128 steps at `--eval-every 32`, `mem.log` every 2 s):
  - the peak is below 90% of the card;
  - (w512) tries micro-batch 8 first, and on an I-m failure or an out-of-memory stop in its smoke it tries 4. Both tries are recorded;
  - if 4 also fails, (w512) is NOT_RUN at that seed, as an instrument.
- **I-f:** the peak after the smoke's step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded, with no cut:** the rate from the smoke's steps 64–128, and W = used + 1.05 × r × steps left + scoring.
  - Every arm trains 16,384 steps; cutting would break the fixed-steps design.
  - The watcher posts the projection on #3472 if the unit's projected spend passes $30.
- **I-q, the build tie (pod C).** Pod C fetches claude-301's seal `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/`, reads it back against its SHA256SUMS, and re-scores (b64L) seed 1's `models/step-016384` on DEV and oasst2 with claude-301's scoring flags.
  - **The seal was listed and read back on the laptop (8 Nov):** `d3-accum-s1.tar.gz` and its list pass `SHA256SUMS`, and the tarball holds `oh/run/b64L-s1/models/step-016384/` (model, config, Hamming read) and all 16 scored row files under `oh/run/b64L-s1-scored/`.
  - **The sealed `proj` hashes** (sha256 of `jq -c '[.positions, .targets, .nearest, .writer]'`, the same in the memory and no-memory files, since memory lies outside the projection):
    - DEV, every panel folder: `488cf911d35be329525e3245ed1535db5e5a79e30c2c3b0cc5660e42776b5601`;
    - oasst2: `95b625931f46eb9eb6e89b9823e8991b2df44a9dc1f118472e0f59b04e1a2c87`.
  - **Pass:** the re-scored DEV and oasst2 row files give exactly these two hashes. Then R1 and R2 compare like with like.
  - **Fail:** R1 and R2 are not read (an instrument); R3 still is, and R7 is read. Pod C trains its three cells whatever I-q reads.
- **I-r:** claude-301's (b64L) grades are re-paired with this unit's laptop `scale-probe` build against D3's T2. They must give claude-301's d, +75,038 / +85,590.
- **I-5:** each panel is scored in its own process, as claude-295's repair did. Each process also scores DEV, and DEV must agree across a cell's four processes.
- **I-x (laptop, before R4):** `cpu-fbs-export --raw-gates --raw-transition` (sha `724a3b78…`) re-exports claude-313's seal byte for byte (claude-315's I-xc). It then exports each new seal, and `served-reply-panel` must load each export and write 256 rows. Those exports are what R4 reads.

## Steps

1. The ORDER at the pin; this sheet's FIT, posted on #3472; the D67 pre-run review of the job, chain, seal and watch scripts, posted on #3472.
2. `uor-pod status`, staying inside 4 pods and $8/h across all labs. claude-315's two pods come down first.
3. Three Vast RTX 5090 pods, each at most $1.15/h, leased, each with a screen watcher:
   - pod A: (w512) s1;
   - pod B: (w512) s2;
   - pod C: I-q, then (w128) s1, (w128) s2 and (w128h4) s1.
   Each pod is launched the moment its own `Ready` line appears (claude-315's lesson).
4. On each pod: setup, I-0, the inputs and I-3, I-e, the smokes with I-m and I-f, I-t, the arms, then the scoring with I-5.
5. Seal each pod to `results/claude/scale-probe-b64l-width-2026-11-08/runs/{A,B,C}/`, read the seal back, then take the pod down.
6. On the laptop: I-r, `grade`, `pair` (R1–R3, both pairings), R5 and R6, then I-x and R4.
7. The REPORT (negatives first), one notebook row in `float-training.md`, and the slope recorded in the pipeline map's gap 9.

## Cost

These are projections; the smokes measure the real rates.
- **(w256)'s rate** was 0.633 / 0.698 s a step (claude-301).
- **(w512):** the old recipe measured ×1.97 a step per doubling (claude-295), and a smaller micro-batch adds kernel launches. Projected 1.25–1.6 s a step, or more at micro-batch 4. That is 5.7–7.3 h of training plus about 0.5–1 h of scoring: up to about 8.3 h a pod.
- **(w128):** projected 0.32–0.40 s a step, so 1.5–1.8 h a cell plus scoring. Pod C runs three cells plus I-q, about 6.5 h.
- **Spend:** about 23 pod-hours at $0.56–1.15/h, so about $13–27. The ORDER's notice line is $30; it is not a cap, and the charter's extend rule applies to a healthy run.
- **An expected overrun:** a (w512) out-of-memory stop late in its run restarts from step 0 at micro-batch 4. That can take one pod to about 15 h and the unit past $30. The notice then fires, and the run goes on.

## Stop rule

claude-301's, with these changes:
- **Any stop seals and takes the pod down at once** (L-004): the chain seals what exists and writes its exit marker, and the watcher downs the pod on that marker.
- An I-0 or I-e failure stops that pod before any training.
- **(w512) at micro-batch 8:** an I-m failure, or an out-of-memory stop in its smoke or mid-run, retries the arm from step 0 at micro-batch 4, once. A failure at 4 makes (w512) NOT_RUN at that seed, as an instrument; R1 is then not read, and the 48 GB card unit is next.
- An out-of-memory stop at width 128 mid-run makes that cell NOT_RUN, and pod C's other cells still run.
- A panel that fails to score is re-scored once, in its own process, from the checkpoint (claude-295's repair). It is NOT_RUN only if that also fails.
- An I-q failure blocks R1 and R2 and opens R7; an I-r failure blocks R1 and R2 against (w256). Neither blocks the new arms' own R3.
- A pod that does not reach SSH within `uor-pod`'s window is replaced once.

## Changes from claude-301's sheet

1. The arms: widths 128 and 512 at width / 64 heads, the (w128h4) contrast, and width 256 as claude-301's seal.
2. `--micro-batch` 8 or 4 at width 512.
3. I-e at each pod's widest arm and batch, with a power fallback.
4. New gate I-q.
5. I-t records only and never cuts steps.
6. New readings R1, R2, R5 and R6, and the speed reading R4 with I-x.
7. The Bar table and the outcome map.
8. The seal path is `results/claude/scale-probe-b64l-width-2026-11-08/`.

## Changes from round 1 (the FIT's findings)

- **B1:** I-e at width 512 runs at batch 8, or 4, and an out-of-memory stop in I-e is registered as an instrument.
- **B2:** the Bar is now a full table, R1 against R2 including "not read". Rows 5 and 6 are new, and a pair needs both seeds.
- **M1:** the band is two-sided, the pairing rule is fixed (seed against seed, shared window order), and both pairings must agree.
- **M2:** the variable is named width at a fixed head width of 64, heads are a co-cause, and the (w128h4) contrast is added.
- **M3:** lr is a co-suspect in rows 2 to 5, and row 2's next unit includes an lr check.
- **M4:** R4 is speed only. Quality parity (claude-308) and speed against context are named as not covered.
- **M5:** the new gate I-q ties the new builds to claude-301's sealed rows.
- **m1:** an out-of-memory stop at micro-batch 8 retries at 4, and a panel that fails to score is re-scored before NOT_RUN.
- **m2:** I-e's power fallback.
- **m3:** "exact" is now "to rounding", with I-e per width.
- **m4:** I-t's spend projection is posted above $30.
- **m5:** the fit is defined, and the notebook row and gap 9 are added to Steps.

## Changes from round 2 (the second FIT's findings)

- **1 (major):** two pairings that disagree now read `mixed`, not `inside`, so a disagreement lands on row 3 and never claims row 2's "512 gains nothing".
- **2 (major):** the Bar reads `pooled.verdict`, as claude-298 and claude-301 did; `unseen.pooled` is recorded beside it.
- **3:** the cell R1 not read, R2 `behind` now maps to row 5, where R6 is read first.
- **4:** R7, (w512) against (w128), is registered and read only on an I-q failure, so row 6 keeps a slope.
- **5:** claude-301's seal was listed and read back, and I-q now names the two sealed `proj` hashes it must match.
- **6:** any stop seals and downs the pod at once, and pod C trains its three cells whatever I-q reads.
- **7:** the micro-batch-4 restart's possible overrun past $30 is noted as expected.
- **8:** the shared window order is named as resting on reading the code, with the crossed pairing as its guard.

## D67 pre-run review (8 Nov)

A fresh-context pre-run review read the whole chain, the trainer source at the pin (micro-batch, heads and checkpoint naming), the store and the five scripts against this sheet. Verdict: GO WITH CHANGES. It found one blocker and three minors; all are applied:

- **1 (blocker):** `up-317.sh` had no cap guard. It now refuses unless the running pods plus one per letter are at most 4, and the running $/h plus $1.15/h per letter is at most $8.00/h. The pods go up only once claude-315's two are down (Step 2).
- **2:** the job had no signal trap, so a killed job wrote no EXIT and the chain's seal refused. A TERM, INT or HUP now ends the job through `end 9`, which writes the seal list and EXIT.
- **3:** the watcher's lost-pod and stall stops left a live pod without a watcher. Both are now one NOTICE line, and the watch goes on.
- **4:** a pod priced above $1.15/h would be left up unwatched. `up-317.sh` now reads each pod's price from its up log and takes a pod above it down at once.

Gates after the fixes: `gt/test-gates.sh` 79 ok, exit 0 (3 new checks for the signal path), and `gt/mutate.sh` 33 of 33 mutations caught (one new: removing the trap).

The scripts as run (SHA-256). `launch-317.sh` checks the first three on each pod before starting the chain:

| Script | SHA-256 |
|---|---|
| `job-317.sh` | `ab2ccaaff1a051807fd344a3cca78dbb844b36533e9d9106c8c3bad79126d86e` |
| `chain-317.sh` | `b284d152bcc7bb08a5704b72d89f4fc00e7d3f04c2dc16672acfd3c1ee3abbf1` |
| `seal-317.sh` | `67ff449798b9fd5e447a8f0d5968b7b0e8c942ea4dcce77a8fcd1a3846902359` |
| `watch-317.sh` | `6dc3313002481448febce4cc5ca7bf79840b020abc58fe2c4fc87644fd79d11b` |
| `up-317.sh` | `b70a8296f8f1dde9b6a469f6084e4c734c978e76bf0c26e88b147f97e9ebb1c9` |

Left unverified by the review, each resting on claude-301's or claude-314's runs at this pin: the trainer's out-of-memory message text (the job's grep), the store's path layout on the pod, the trainer's CLI flag spellings and the `--hamming-read 0` branch.
