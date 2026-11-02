# Prep sheet: the capacity sweep, trained as served (one-model-end-to-end, claude-259, 2 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [claude-259](../../loop/tasks/one-model-end-to-end/claude-259/ORDER.md). Path step 2, "a model that replies". Round 2, after the first FIT's five blockers.

## Question

Does a larger trained-as-served model reply better on prompts phrased in ways TRAIN never used, and at what cost in served speed? The sealed (r-drop) run's DEV top-1 stops rising by step 2,048 (11,770 → 11,841 of 14,287 by step 16,384) while its train loss keeps falling. So more steps at width 256 buy nothing. The model's size has not been varied since claude-153 (28 Oct, before the turn memory).

## Arms

Every arm is claude-211's (r-drop) recipe, trained as served (claude-258), with a size change and a shorter run:

`train-in-space --data-root DATA --out OUT --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 8192 --eval-every 1024 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --seed S --width W --hamming-read H`

| Arm | `--width` | `--hamming-read` (heads) | Head width | GPU |
|---|---|---|---|---|
| (b) the base | 256 | 4 | 64 | 0 |
| (h) more heads | 256 | 8 | 32 | 0 |
| (w) wider | 512 | 8 | 64 | 0 |
| (W) widest the writer serves | 1,024 | 16 | 64 | 1, alone |

- **Pods:** one 2 × RTX 5090 pod per seed (seeds 1, 2 and 3), with the job `local/claude/claude-259-capacity/job.sh` run under `uor-pod run --gpu 0,1`.
- **Steps:** every arm runs the same count, 8,192: four times the step where (r-drop)'s DEV reading stopped rising, read every 1,024 steps. The only exception is I-2's cut, which applies to all arms equally.
- **After training,** each arm's last checkpoint is scored:
  `train-in-space --data-root DATA --score-rows CKPT --out SCORED --samples 1536 --reply-tokens 64 --retain-rates 1 --panel p-w=PW --panel p-v=PV --seed S`
  This decodes greedily with the memory on and with it out, on DEV's prompts, p-w's 1,202 and p-v's 393. The checkpoint is saved at its rounded values (`in_space_long.rs`, `round_if_served`), so this scores the served values.
- **Then** `fbs-snapshot CKPT SNAPSHOT.json`, for the export.

## Readings

R1, R2, R3 and R5 are read from `samples-{panel}-greedy-memory.json` by `local/claude/claude-259-capacity/grade.sh` with `jq`; R4 from each arm's `progress.jsonl` with `jq`; R6 on the laptop. M2's panels are left out: `m2-fresh256` is sealed behind credentials (HTTP 401, `docs/roadmap/items/cpu-training-pilot.md`), and `m2-reply512` is graded by the old native learner's own evaluator (`docs/roadmap/items/m2-fresh-heldout-panel.md`), which does not read this model.
- **R1 (decides): the right prefix on p-w.** For each prompt, the number of leading reply tokens equal to the reference's, its end mark included. An arm X is paired with (b) at the same seed on the same prompts: b = prompts where X's prefix is longer, c = where (b)'s is longer.
  - **Power:** on claude-220's two decodes of p-w, R1 pairs 339–384 prompts a seed against a floor of 37–39. So R1 detects a net shift of about 3% of the 1,202 prompts, where exact replies (12 of 1,202 for (r-drop) s1) could not decide.
- **R2 and R3 (reported, same pairing):** exact replies (the reply opens with the whole reference) and right openings (the first token).
- **R4 (reported): DEV strata.** Openings, copies and the rest, from `progress.jsonl`'s `dev_nearest` at every eval. An arm is **still rising** if its last eval's DEV nearest hits exceed those at the eval half its steps earlier by more than 143 (1% of 14,287 rows). Under a cut, that earlier eval is the last one at or before half the cut's steps.
- **R5 (confirms): R1 on p-v.** Used for row 1's chosen arm against (b).
- **R6 (reported): served.** Each arm's seed-1 export (`fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT OUT.uorlut`) is served on the laptop.
  - Conversations: conv-a, conv-b and conv-c from `m4-export-uorlut01-cli/claude-259`, through `uor-chat`, with transcripts.
  - Decode tok/s, that unit's protocol: a 177-token run minus a 1-token run, the median of 5. This is the arm's cost. The shared-GPU training rate is not.
  - The arm that ships is served at all three seeds.

## Decision table

**Per seed**, an arm X against (b) on R1 is one of three verdicts:
- **better**: b − c > 2·√(b + c), with b + c ≥ 10;
- **worse**: c − b past the same floor;
- **level**: anything else, including b + c < 10.

X **carries** when it is better at all three seeds. The rows are read in order; the first that matches decides.

| Row | Outcome | What it changes |
|---|---|---|
| 1 | Some arm carries, and the chosen arm passes R5 | **Choosing it:** start at the cheapest arm that carries, in cost order (h), (w), (W). Move to the next costlier carrying arm only if it is better than the current arm at every seed (R1, paired on those two). Stop when none is. **R5:** the chosen arm's p-v b − c against (b) is above 0 at every seed. **Unlocks:** that arm becomes the base. The next unit trains it to its plateau and ships it, with R6's tok/s cost stated. `scale-probe` gets this curve. |
| 2 | No arm carries, every arm is level or worse at every seed, every arm was measured at every seed (none dropped at I-1 or stopped by a fault), no seed was cut (I-2), and no arm is still rising (R4) | Size is not the ceiling at this data. Width stays 256 with 4 heads. The next lever is the reply itself (path step 4: copies and the reply's continuation, fixed with geometry), as claude-155 read it. This closes these four configurations only (CHARTER §4.1). |
| 3 | Anything else: an arm better at some seeds only, an arm absent at a seed, a seed cut, an arm still rising, or row 1's R5 failing | Keep and iterate. The candidate is the arm with the largest R1 b − c summed over seeds, if that sum is above 0; else the still-rising arm with the largest sum, if any; else (b). The next unit runs it to its plateau at 3 more seeds before anything ships. An absent arm is reported as absent, never as level. |

## Instrument checks

- **I-1 smoke (probe a):** on every pod before its main run, 256 steps of every arm side by side, sharing the GPUs as in the main run. Each must:
  - run `run_long`, with `config.json` holding `"train_as_served": true`;
  - score with `--samples 4`;
  - snapshot.

  An arm that fails is dropped and recorded (the ORDER's stop rule). If (b) fails, the pod stops.
- **I-2 throughput (probe b):** each arm's smoke `train_context_tokens_per_second` projects 8,192 steps.
  - If any arm passes 90 minutes, every arm at that seed is cut to the most whole thousands of steps the slowest arm fits, and `steps.txt` records `CUT`.
  - A cut bars row 2. Under 2,048 steps, the pod stops.
- **I-3 parity:** on the laptop, each arm's seed-1 export through `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64`, on DEV's prompts taken with `jq` from the arm's own `samples-memory.json`.
  - At most 9 of 310 first tokens may disagree with the float file (claude-243's bar), scaled to the prompts the file holds: at most 2.9%, its count recorded.
  - Past that, or if the export refuses, the arm's R6 is an instrument fault, not a reading. R1 to R5 stand: they are read from the GPU's files, which score the served values.
- **I-4 pairing:** every arm at a seed decodes the same prompts in the same order. `grade.sh` stops on a mismatch of `position` and `prompt` taken index by index.

## Data

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`. The code pins them and refuses any other at load (`crates/uor-r4-float-train/src/hamilton_sheet.rs`, the `PINNED_*` constants); the job records the sums.

  | File | sha256 |
  |---|---|
  | `mw-balp/train/tokens.u16` | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
  | `mw-balp/train/response_mask.u8` | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
  | `ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
  | `ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
  | `tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

- **p-w:** `inputs/claude/heldout-panel-p-w1200-2026-10-31` (seed 2121). `tokens.u16` is `038b0085e6888d988d4b0b6ceb7846015f21f1c40f26ec3286ddcc049c17d217` and `response_mask.u8` is `797fc23474231595ae42b9cfbf7930ae2021b13d23d8b8b3d4fb3c7dda06ec16`.
- **p-v:** `inputs/claude/heldout-panel-p-v-2026-10-30` (seed 2111). `tokens.u16` is `3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0` and `response_mask.u8` is `c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023`.
- Both panels are checked against their own `SHA256SUMS` and these values. No size was ever chosen on either.
- **The binaries:** `train-in-space` and `fbs-snapshot`, built on each pod from this unit's head (which holds PR #3559), `--locked`, with sha256s recorded before the smoke.

## Cost and stop rule

- **Cost:** three 2 × RTX 5090 pods on Vast, at about $0.70–1.40/h each, for about 3 h. That is at most $4.20/h and about $12.60 in total, inside 4 pods and $8/h with what other labs hold at launch (`uor-pod status`, with the Vast adapter built). The 90-minute projection bounds training only; the build and the scoring are logged, each lease is taken for 4 h and renewed, and a healthy run is extended, never cancelled.
- **Stop rule:**
  - A diverged loss (non-finite) or an export refusal is an instrument fault: that arm stops, and the fault is repaired in its own unit, never counted.
  - Outputs go to `results/claude/one-model-end-to-end-capacity-2026-11-02/sN/` with `SHA256SUMS`, read back, before any pod goes down.
  - A watcher is armed for every terminal state (`EXIT`), and pods go down after their save.

## Amendment, 2 Nov 2026, 16:20Z: round 1's smoke and the layout of round 2

I-1 did its job on round 1. On seed 2 (pod `vast-58846821`, started 16:14:12Z), every arm passed its step-0 evaluation, then:
- (W) ran out of GPU memory at 49 s, alone on GPU 1, a 32 GB RTX 5090 (`CUDA_ERROR_OUT_OF_MEMORY`). So (W) does not fit this card at batch 16.
- (b) and (w) ran out of GPU memory together at 16:16:05Z, sharing GPU 0 with (h). GPU 0 held 27.7 GB at the last sample before that.
- (h), then alone on GPU 0, passed: 256 steps, about 8.7 GB.

Round 1 ended at exit 5, since the base arm failed its smoke. It is kept as it was in `/workspace/capacity`, and it is saved with the results.

Round 2 changes the layout only. Arms, steps, readings and the table are unchanged:
- **(b) and (h) share GPU 0.** Both are width 256.
- **(w) has GPU 1 alone.**
- **(W) is dropped,** under the ORDER's stop rule. It is **absent**, never level: it was not measured. So row 2 is barred, and the size question at 1,024 stays open for a larger card or a recipe change, which this unit does not make.
- **Scoring runs one arm at a time on each GPU.** The two GPUs still score side by side.
- **Round 2 writes to `/workspace/capacity-r2`** (L-002), and its smoke repeats I-1 for this layout.
- **The job** is `job.sh` at sha256 `71fc10fe82234b9c48552b271c5a7788da1180ca152830ba3a1bb550ac1c265d`.

**Cost.** Vast's 5090 price rose from $0.563 per GPU-hour at the review to $0.89–1.17 at launch:
- The pods are $2.33/h (seed 1), $1.775/h (seed 2) and $2.171/h (seed 3), or $6.28/h in all. That is inside the $8/h cap but above the $4.20/h projected.
- Seed 1's first pod (`vast-58845589`) never opened SSH, and `uor-pod up` deleted it after 900 s, about $0.45.
- The watcher's budget is $24. A healthy run is extended past it, never cancelled.
