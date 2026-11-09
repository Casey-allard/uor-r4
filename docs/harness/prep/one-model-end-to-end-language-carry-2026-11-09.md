# Prep sheet: (b64L-d3mwcap6) with the ask heads, the question carry and the found-copy pull (one-model-end-to-end, claude-339, 9 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-339. The run is judged by the served-reply score, under "Yes, pipeline first" (#2028 6709947526).

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 9 Nov 2026). It had 8 findings, all applied below and listed under "FIT changes" at the end.

**Why.**
- **Recall is the gap now.** claude-322's (b64L-d3mwcap6) stops on 109 and 86 of 256 replies without collapsing (9 and 6), but 0 rows are complete at both seeds. Its most common stopped reply names the wrong relation and cuts the value short. The reply's first line holds the asked-for value in only 14 and 2 rows.
- **The recall task's answer is the ask heads with the question carry** (claude-335, B2-aqC at `h = 4`): `R_ask` 1.000 / 0.969 against the 0.50 floor, and its `−C` cut fails at both seeds (0.105 / 0.027). So the carry is what answers.
- **The parts are all in place:**
  - claude-337: the language trainer's pull at its scored rows;
  - claude-336: the integer session serves the ask table and the carry;
  - claude-338: `served-reply-panel --engine integer` scores them, and the CPU-port path refuses an asking artifact.
- **The named risk.** A language window has no question row, so only the scored placement (`--hamming-attract-at scored`, the default) is open. On the recall task that placement (B2-aC) passed at seed 2 and failed at seed 1 (`R_ask` 0.240 / 0.258). It was undertrained and still rising: 0.04 to 0.24 in the last 2,048 steps. The answer here is a reading, not a third seed or more steps:
  - **The checkpoint trend** (Readings): the served first-line count and `pull.jsonl`'s pull distance at steps 8,192, 12,288 and 16,384. A rising L-aC reading at the last step, with no bar met, reads as row 4u ("not learned yet"), and its next step is the longer run.
  - **Why not more steps here:** 16,384 steps keeps the held-out pair against claude-322 and (b64L-D3) at an equal step count, which every earlier sheet on this line kept.

## Question

At (b64L)'s recipe and pin, on claude-322's TRAIN (D3 + the reply-capped `mw-balp/train` × 6), with the ask heads on the 4 Hamming reads, and pulled at the scored rows:
- **does the carry put the asked value into the served reply?** That is the first-line count and the panel's complete rows, against the no-carry arm (L-a), against the no-ask line (claude-322) and against its own `−C` cut;
- **does stopping hold** (EOS stops and collapse, against claude-322's 109 / 86 and 9 / 6)?
- **what does it cost on the held-out human text,** against (b64L-D3) and claude-322?

## Base artifact

- **Recipe:** claude-301's (b64L), unchanged. `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16`, for 16,384 steps at `--eval-every 1024`.
- **The pin:** `fbf15e56f3ab1961b30610893392e7a97fd0ab18` (claude-338's merge). It holds claude-337's pull, claude-336's served asks and claude-338's scorer. The trainer is built at the pin on each pod; the scorer and `fbs-export` are built at the pin on the laptop.

## Data split

- **TRAIN:** claude-322's mix, unchanged, from the store at `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`:
  - tokens `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`;
  - mask `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`;
  - manifest `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`.
  - 417,421 usable TRAIN windows.
- **DEV, the held-out panels and m2-fresh256:** claude-322's, at its pins (its sheet's Data split). Nothing is regenerated.

## Arms

| Arm | flags beyond the recipe | what it isolates |
|---|---|---|
| **L-aC** | `--hamming-ask 1 --hamming-carry-question 32 --hamming-attract 1` | the full mechanism, pulled at the scored rows |
| **L-a** | `--hamming-ask 1 --hamming-attract 1` | the no-carry control (L-010): the ask heads and the pull, with no carry |
| L-aC `−C` | L-aC's checkpoint, served with its carry removed (below) | the carry's share, at the same weights |
| claude-322 | none (sealed: artifacts `86e51d5e…`, `8838cd10…`; grades `24d69af0…`, `4407e2c2…`) | the no-ask line, at the same recipe and TRAIN |

- **The carry's reach is 32, not claude-335's 4.** The carry adds the last `h` rows' points at the readout read, each weighted by a learned per-token gain that starts at 0.
  - On the recall task the question sat four rows before the answer. Here it does not.
  - Measured on m2-fresh256 with the pin's tokenizer (`token-counts`, mode text):
    - the question (`query_ids`) is 6 to 13 tokens (median 10);
    - the `\nAssistant:` header after it is 6 tokens;
    - a reply's lead-in before the value (" Your current job is") is 7 tokens.
  - So at the last prompt row, the question's key word ("job") is 8 rows back, and the question's first token up to 19 rows back. At the row that emits the value, it is up to 26 rows back.
  - `h = 4` would carry only the header, so 32 covers the panel's worst case with room to spare. The per-token gains let the model weight the question's words and leave the filler at 0.
  - **TRAIN's distances are not counted.** The mix's `mw-balp` part comes from the same generator family as the panel; D3's part is not counted. The REPORT names this if the carry reads as unused (row 4c).
  - **Cost:** claude-335 measured +1% for doubling `h` from 4 to 8. Three more doublings would be about +3% more by that slope; that is an extrapolation, and I-t records the real value.
  - claude-335's B2-aqC8 (`h = 8`) also passed at both seeds, so a longer reach is not untested.
- **Seeds 1 and 2** for L-aC and L-a: four runs, one RTX 5090 pod each.
- **`−C` is a data cut, not a new run.** `jq` removes `carry` from every `hamming_ask` entry of L-aC's snapshot, and `fbs-export --run-config` exports it again. The question table and every other section are unchanged. That is checked by I-cut.

## Readings

At seeds 1 and 2, on each arm's served `step-016384` (and `−C`), from `served-reply-panel --engine integer` on m2-fresh256, the same measures as claude-322's sheet:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows, the count of the most common reply text, beside the number of rows it is over. Also distinct reply texts over all rows.
- **The first-line count**, by `firstline.jq` (`b15c778d…`): the asked-for value, the other case's value, and both. This is the recall reading. Here it is the deciding count for rows 2 and 3, not descriptive (see Bars).
- **Reply length** (median), and EOS stops per stratum.

**The checkpoint trend,** for L-aC and L-a: the first-line count and the complete rows at `step-008192`, `step-012288` and `step-016384`, each served the same way, beside `pull.jsonl`'s pull distance at those steps.

**Held out:**
- `scale-probe grade` of `step-016384` with `train=` the mix, on all 1,460,697 rows, from the pod's no-memory `--score-rows` output.
- `scale-probe pair` against claude-301's sealed (b64L-D3) grades (`e17a1354…`, `3170c779…`): its pooled verdict and mean d, as in claude-322's sheet.
- Descriptive: against claude-322's grades (`24d69af0…`, `4407e2c2…`).

**Recorded only:** `pull.jsonl` per checkpoint, the dev curve, the last step's loss, the step time, and the carry gains' extremes on the snapshot (claude-335's sign split).

## Bars

Fixed before any run.
- **The panel's bar:** at least 104 complete and 52 good pairs, at both seeds. Every model so far has 0.
- **Recall moved,** the bar this sheet adds: at both seeds, L-aC's first-line asked count is **at least 16 above** both L-a's at that seed and claude-322's at that seed (14 and 2), **and** at least 16 above its own `−C` cut.
  - 16 rows is about 6% of 256. At claude-322's rates (about 3% of rows), one count's binomial spread over 256 is about 2.7 rows, so the spread of a difference of two counts is about 3.9. 16 is about 4 of those.
  - Seed-to-seed spreads have been far larger (claude-321: 25 and 76), which is why both seeds must meet it.
  - **The paired control is `−C`:** the same checkpoint served without its carry, so seed and weights are shared exactly. L-a and claude-322 are guards against a gain that comes from the ask heads or the pull alone. A seed number does not pair separate runs.
- **Stopping held:** at least 64 EOS stops at each seed (claude-322's lower seed, 86, less about one claude-322 seed spread), and no collapse (one text on 64 or more EOS-stopped rows).
- **The held-out cost**, a decision rule, not a bar: claude-322's ceiling, a mean pooled d against (b64L-D3) below −58,212.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | an arm NOT_RUN at a seed after its rerun, for an instrument fault | **`blocked`.** Repair and rerun that arm and seed under this sheet. |
| 0b | an arm NOT_RUN after its rerun, for a mechanism cause (a non-finite loss with the ask heads on) | **`diagnostic stop`** on that arm. Next: a decision review in the REPORT. |
| 0c | I-xg, I-xa, I-cut or I-c fails | **`blocked`.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds on L-aC | **The served model answers.** Next: the outcome map's next scale, under its own sheet. |
| 2 | recall moved and stopping held on L-aC | **The carry puts the asked value into served replies on language.** Next: the panel's complete rows. The REPORT reads why replies hold the value but are not complete (the wrong relation, the value cut short, no EOS), and the next lever follows that. |
| 3 | recall moved, stopping not held | **The carry carries the value, at the cost of stopping.** Next: a decision review between a stronger EOS share in TRAIN and a decode-time stop, with this arm as the base. |
| 4c | recall not moved, and L-aC's `−C` cut reads within 4 of L-aC itself at both seeds | **The carry is not used on language at this recipe.** Next: a decision review between a question-row placement on chat windows (the `both` placement that passed at both seeds) and TRAIN's distance count (above), which says whether 32 rows reach the question in training. |
| 4u | recall not moved, and at both seeds L-aC's rise in first-line count from `step-012288` to `step-016384` is at least 8 above L-a's rise at the same seed | **Not learned yet** (B2-aC's shape), and the rise is the carry's, not the shared recipe's. Next: L-aC alone at 32,768 steps, two seeds, under its own sheet and cost. That run is a different LR schedule: `--lr-end 0` decays linearly over the whole run, so the step-16,384 point of a 32,768-step run is at half the learning rate, not at its end. That sheet reads its trend against its own steps, not this run's. |
| 4 | recall not moved, otherwise | **No recall gain from the ask heads and the carry at 16,384 steps,** for this configuration only. Next: a decision review in the REPORT, reading the trend, the `−C` cut and the pull distance together. |

A seed that is NOT_RUN reruns once, sealing to `runs-2/` (L-002), on a fresh pod: the job refuses an existing `run/`, and the seal an existing `oh-seal/` (pre-run review, minor 2).

## Instrument checks

Registered before any pod. claude-322's, with these changes:
- **I-0, I-p, I-3, I-1, I-m, I-f, I-t, I-5, I-4, the watcher:** as claude-322's, at the new pin, TRAIN unchanged (417,421 windows).
  - Each `config.json` must name `hamming_ask` true, `hamming_attract` 1, and `hamming_carry_question` 32 (L-aC) or none (L-a).
  - **The watcher's notice time** is set on each pod from I-t's measured step time: 1.5 × (step time × 16,384), and never under 28,800 s. It stops nothing (D67).
- **I-5a, the asks in the no-memory scoring:** `--score-rows` serves the asks it was trained with. Checked in code at the pin:
  - `score_rows` calls `load_reads` (`in_space_long.rs:8912`);
  - `load_reads` restores `HammingReads::load`, which loads each read's `d{depth}.ask` (`flock_read_hamming.rs:1417`) and the readout read's `d{depth}.carry` with `carry_rows`.
  - The run records the carry as loaded: `step-016384`'s snapshot, which `fbs-snapshot` takes through the same `HammingReads::load`. Its readout entry's `carry.rows` must be 32 (L-aC), or the entry has no carry (L-a).
- **I-pull, from the smoke:** `pull.jsonl` exists after the smoke and its mean copy rows per step is above 0. A pull that never runs is an instrument NOT_RUN.
- **I-xg, the export control, once on the first pod before its full run:** the served export path, run on claude-322's sealed seed-1 `step-016384`, must give claude-322's artifact `86e51d5e…`. This shows that the export path this run uses serves what claude-322's did.
  - **As first written** the path was `fbs-snapshot` on the pod, then `fbs-export --run-config --raw-gates --raw-transition` on the laptop. Its first reading **failed** (9 Nov, 14:17Z): `9b01ed62…`. The repair is under "I-xg repair (row 0c)" below.
  - **As repaired**, the path is the laptop snapshot. `cpu-fbs-export`, claude-322's own path, snapshots the trunk and the base Hamming reads on a copy of the checkpoint whose `hamming_read.safetensors` keeps the base inventory only (`hamming-base-copy`). The ask entries are the pod snapshot's `hamming_ask`. Then `fbs-export` serves it.
  - **The check, at every served step:** the pod snapshot's own export has the same header and the same bytes in every `hamming.*` section. The trunk sections may differ, and their count is recorded.
- **I-xa, the ask export, on the smoke's `step-000064`:** its snapshot has a `hamming_ask` entry per read; the readout's entry has `carry.rows` 32 (L-aC) or none (L-a); `fbs-export` accepts it; and `served-reply-panel --engine integer` scores it while the default engine refuses it by name.
- **I-cut:** the `−C` artifact differs from L-aC's in the carry sections only.
  - **The sections:** every artifact section other than `hamming.d{depth}.carry` is byte-equal between the two, compared section by section. `read_hamming_uorlut01` on both gives the same question tables, and carry `None` on the cut.
  - **The snapshots:** every key other than the `carry` field of each `hamming_ask` entry is unchanged (`jq` equality after deleting that field from both).
  - **Provenance, recorded beside the cut:** the input snapshot's sha256, the `jq` filter's text and sha256, and the cut snapshot's and cut artifact's sha256.
  - **Where the carry comes from:** `fbs-export` takes the carry from the snapshot's `hamming_ask` entries (`quantize_hamming_ask`, `fbs-export.rs:219`). It does not compare it with the run config's `hamming_carry_question`, which still says 32 for the cut. So the snapshot, not the config, is what the cut changes, and I-cut checks the artifact.
- **I-c, on the laptop before any reading:** claude-322's two artifacts score, under `--engine integer`, to claude-322's sealed reports without the timing fields (claude-338's parity, repeated at the pin).
  - **Result (9 Nov, before any pod): PASS at both seeds.** The scorer was built at the pin (`c6204f66…`). The stripped reports are equal: s1 `5a27775c…` (claude-338's value), s2 `7eb8dbf5…`.

**Why the export moves to the pod:** `cpu-fbs-export` (DeepSeek's CPU exporter, which claude-322 served through) does not read ask heads. Exporting an asking checkpoint through it would drop them silently, the fault claude-338 closed in the scorer. `fbs-snapshot` reads the routes through the model's CUDA projection, so it runs on the pod. `fbs-export` (CPU) then runs on the laptop.

## Steps

1. The ORDER, this sheet's FIT, then the D67 pre-run review of the scripts.
2. **On the laptop:**
   - claude-322's job, chain, seal, watch, seed and read scripts, as claude-339's, with only these changes:
     - the arm flags;
     - the pin;
     - the `config.json` checks;
     - I-pull;
     - `fbs-snapshot` of `step-008192`, `step-012288` and `step-016384` on the pod, sealed beside the run;
     - I-xg on seed 1's L-aC pod;
     - the read script's serve step through `fbs-export` and `--engine integer`, plus `−C` and I-cut.
   - Their gate tests and mutations, and `bash -n`.
   - I-c, and the scorer and `fbs-export` built at the pin.
3. `uor-pod status`. The caps must hold with four pods: at most 4 and $8/h. Then one RTX 5090 pod (at most $0.70/h) per arm and seed, each with its watcher before any script ships.
   - **The order, so I-xg comes before any full run:**
     - L-aC seed 1's pod starts first. After its build it snapshots claude-322's seed-1 checkpoint and waits, before its own full run, for the laptop's verdict.
     - The laptop exports that snapshot and compares the artifact with `86e51d5e…`, then writes PASS or FAIL on the pod.
     - The other three pods start only after a PASS. A FAIL stops all four before any full run (row 0c).
4. **On each pod:** I-p, I-0, the build, the inputs, I-3, the smoke, I-m, I-f, I-pull, I-xa's snapshot, I-t. Then 16,384 steps. Then the three snapshots, and the no-memory scoring of `step-016384` with I-5.
5. **Seal** to `results/claude/one-model-end-to-end-language-carry-2026-11-09/runs/{L-aC,L-a}/s{1,2}/`, each root empty first (L-002). Read back, then take the pod down.
   - The rerun seals to `runs-3/`, and to the laptop while the store is full. See [the ask-range repair](#ask-range-repair-row-0a).
6. **On the laptop:** I-xg, I-xa, I-c. Then serve every snapshot, cut `−C`, I-cut, the panel, the first-line counts and the trend. Then grade and pair.

## Minimum test power

- **The first-line count** is the deciding reading. claude-322's 14 and 2 set the floor, and claude-321's 25 / 76 the seed spread. The bar is 16 rows, about 4 spreads of a difference (3.9), against the paired `−C` cut and both guards, at both seeds.
- **The trend's 8 rows** (row 4u) is half the bar, over 4,096 steps, on the difference of two rises (L-aC's against L-a's).
- **Stopping and collapse** as claude-322's: within 32 of the floor is "near".
- **The held-out pair** uses `scale-probe pair`'s band, as before.

## Cost

- **Step time, an estimate:** claude-335 measured the ask heads at about half the no-read speed (43,852 against 89,873 context tokens a second), and the carry at about 3% more at `h = 4`.
  - That was on the recall task's own baseline, not on (b64L)'s 4-read language recipe. So the factor of 2 is carried over, not measured here.
  - At that factor, each run takes about 2 × claude-322's 8,046 s, about 16,100 s or 4.5 h, plus `h = 32`'s few percent.
  - I-t records the real value on each pod, and the watcher's notice is set from it.
- **Four pods at $0.563–0.70/h × about 4.9 h** (with setup and scoring): **$11 to $14.** The budget notice, not a cap, is 4 × 6 h × $0.70 = $16.80.
- The snapshots add three `fbs-snapshot` runs per pod, a few minutes each.

## Stop rule

claude-322's, with:
- **I-pull failing:** NOT_RUN for that arm and seed (instrument).
- **I-xg failing:** stop before any full run. The export path is the fault (row 0c).
- **A snapshot failing** at an intermediate step: the trend loses that point, and the REPORT says so. At `step-016384` it is row 0c.
- **A table past its stored range:** row 0a. The ask tables are held at their range since `721a4784f`; see [the ask-range repair](#ask-range-repair-row-0a).

## Changes from claude-322's sheet

1. **The arms:** L-aC and L-a, with the ask heads, in place of the one no-ask arm. claude-322 is the no-ask line, read from its seals.
2. **The pin** is `fbf15e56f`.
3. **The export** is `cpu-fbs-export`'s snapshot of the trunk and the base reads, with the pod `fbs-snapshot`'s ask entries spliced in, then `fbs-export` on the laptop, with I-xg and I-xa. As first written it was the pod snapshot alone; I-xg's first reading sent it back to the laptop for the trunk (row 0c repair, below).
4. **The scorer** runs with `--engine integer`.
5. **The first-line count decides** rows 2 to 4. A new recall bar is added; the stopping floor is lowered to 64 against this TRAIN's own 86.
6. **The checkpoint trend and the `−C` cut** are new readings.
7. **Four pods,** and a cost about four times claude-322's.
8. **The carry's reach** is 32, from the panel's measured question and header lengths.

## FIT changes (round 1, all applied)

1. **The carry's reach:** measured on the panel; `h` raised from 4 to 32; TRAIN's distances named as not counted; row 4c reworded.
2. **Row 4u** compares L-aC's rise with L-a's rise, and row 4c now comes before it.
3. **I-5a added:** the no-memory scoring loads the asks and the carry, checked in code, and `carry_rows` is recorded on the pod.
4. **I-cut** compares every non-carry section byte for byte and every non-carry snapshot key, records the cut's provenance, and names that `fbs-export` takes the carry from the snapshot.
5. **Row 4u's next unit** names the different LR schedule of a 32,768-step run.
6. **The "same-seed" justification is dropped:** `−C` is the paired control, and L-a and claude-322 are guards.
7. **The spread of a difference** is about 3.9, so 16 is about 4 spreads, not 6.
8. **The step-time factor** is called an estimate, and the watcher's notice is set from I-t.

## Pre-run review

[claude] D67 pre-run review, 9 Nov 2026: three fresh reviewers (delegate:reviewer, Opus), each given the sheet and the scripts only, in `local/claude/claude-339/`.

- **Round 1** read job-339's config check (arm_want against the trainer's Config at fbf15e56, keys left out at 0 or false), I-pull (`pull.jsonl` is written at every checkpoint, so the smoke at eval-every 32 has four lines), the snapshot steps (EVERY 1,024, so 8,192, 12,288 and 16,384 are checkpoints), the I-xg handshake, seed-339's $0.70/h refusal and pin check, and seal-339's root guard (four distinct roots). It found two minor issues, both fixed: the no-file I-pull test now checks its reason, and a `runs-2` rerun goes on a fresh pod (Stop rule above). It ran out of context before the rest.
- **Round 2a** (watcher, launcher, job's remaining parts, chain): **PASS**, with three minor findings.
  - M2 is fixed: a failed `uor-pod run` is now reported, not hidden by the pipe.
  - M1 is declined: I-t's projected wall is used + 1.05 × rate × 16,384 + 300, so wall − used is the remaining projected time. The watcher's time notice, the time already up + 1.5 × that, is the sheet's rule.
  - M3 is declined: renewing while ssh is down would keep a dead pod billing (inherited from claude-322).
  - It found that `score_one` scores through the CUDA `train-in-space --score-rows`, which loads the asks and the carry, so the CPU scorer's refusal does not apply.
- **Round 2b** (`read-339.sh`): **FAIL** on its serve mode.
  - **B1:** bash 3.2 treats an empty array as unbound under `set -u`, so the default-engine call stops `serve`.
  - **M1:** `score/` is not fresh, and a scorer refusal can read an old report.
  - **M2:** I-cut does not check that the cut's header shows no carry.
  - **M3:** a missing step-016384 still exits 0.
  - Its `ixg` mode, the only part used during the runs, was reviewed sound: the config pin, the refusal to overwrite, and the comparison with `86e51d5e…`.
  - So the serve-mode findings gate the readings, not the launch. They are fixed and tested before any reading; the record of that goes in the REPORT.
- **Tests:** test-gates 81/81, test-read 28/28, test-seal and test-watch pass. The watch mutations are 10/10 caught. In the job mutations, ipull-no-file-ok and ixg-no-timeout are now caught, after the I-pull reason check and an 8 s watchdog on the no-verdict wait. The full mutation run is re-run before the REPORT.

**Pins** (SHA-256 of the shipped scripts, the same files both rounds read; `seed-339.sh` refuses any other):
- job-339.sh `f553a7659106b8f09410fd8774549ce6aff6311ec440d85751c3df66e03029df`
- chain-339.sh `2e4a9c66afa3f4876ab03719f63a75add3f1769eb6289e0878d172a5d1b88a52`
- seal-339.sh `9594f63f720dd7c1cedb92df539b8282939719c6f5c1dad886c4e7affca5cb92`

The registered `runs-3` round ran later versions of the three scripts, pinned the same way: job-339.sh `4e1272b8e48f9bc8dafee98ac53888c71d7a5a9a60861b42fbee371bc4e46d36`, chain-339.sh `5eb466019a48f6b4693367cb90ec0599b94b7e52213bba0c9ea6fa54897851a7`, seal-339.sh `e1ca7e1b7806b6fbaf3f2c79b0d90ffceec688a51b4711c7157352b032d378c9`.

**Verdict: PASS for the launch.** The whole chain is present: the pin fbf15e56 is in the repo, the TRAIN mix, panels and claude-322's seal are in the HF store, and claude-322's laptop export matches. A pass unlocks rows 1 to 4u; a fail sends the cell to rows 0a to 0c. The FIT is above; the lease stays inside the caps (4 pods at no more than $0.70/h each against $8/h). L-aC seed 1 goes first; the other three cells start after its I-xg PASS.

## I-xg repair (row 0c)

[claude] 9 Nov 2026. The negative first.

**The first I-xg failed.**
- **The launch:** L-aC seed 1's first two pods never answered SSH; `uor-pod` deleted each. The third (`vast-59897413`) built, ran the smoke and I-xa's snapshot, and wrote I-xg's snapshot.
- **The reading:** the laptop's export of that snapshot gave `9b01ed62…`, not `86e51d5e…`. The job stopped before its full run, as the stop rule says, and sealed to `runs/L-aC/s1/` (NOT_RUN: "I-xg, no PASS from the laptop"). The watcher took the pod down.

**Classified: an instrument fault in the export path, not a mechanism result.**
- **The two snapshots differ in four quantized entries.** The pod's snapshot and claude-322's laptop snapshot of the same checkpoint are equal in every key but `layers`. In `layers`, four quantized entries (8 values) differ, among them one projection's `rec_out` atom (31 for 30) and its state (42 for 7). In the artifacts, four trunk sections differ: `l1.rec_out.fbs`, `l4.mlp_down.fbs`, `l4.rec_out.fbs` and `l5.rec_gate.fbs` (fresh review, finding 3, corrected).
- **Why:** `fbs-snapshot` loads the model on the GPU (`Device::new_cuda(0)`) and builds the trunk tables from it there. claude-322's served snapshot was built on the laptop by `cpu-fbs-export`. Where two nearest atoms are almost tied, the two hosts' arithmetic can choose differently.
- **The laptop path reproduces exactly.** `cpu-fbs-export` built at the pin's crates gives claude-322's snapshot `c9fe1d91…` and artifact `86e51d5e…` byte for byte.

**The repair: the laptop snapshot,** with I-xg's bar unchanged.
- **The tool:** `cpu-fbs-export` refuses an ask checkpoint ("Hamming tensor inventory differs"). So `hamming-base-copy` (new, in this PR) writes a copy of `hamming_read.safetensors` with the base inventory only. It keeps `heads`, `copy_key` or `copy_keys`, and the five reads per depth, byte for byte. It leaves out `carry_rows`, `d{depth}.ask` and `d{depth}.carry`, and refuses any other name.
- **The splice:** the ask entries come from the pod snapshot. `fbs-snapshot` builds them through `HammingReads::load(dir, &Device::Cpu)`, not on the GPU.
- **The check:** at every served step (`read-339.sh`'s `laptop_snap`), the pod snapshot's own export must have the same header and the same bytes in every `hamming.*` section as the served artifact, and every served trunk section must equal `cpu-fbs-export`'s own artifact, so the splice touched no trunk number. The count of trunk sections that differ from the pod's is said. The served pair is written under staging names and moved into place only after every check passes, so a refused artifact never sits under the served name, where the `−C` block would read it.

**Shown before any rerun, on the first pod's seals:**
- **claude-322's checkpoint, with the first pod's real GPU snapshot:** the repaired I-xg passes, `86e51d5e…`, and records the pod path's `9b01ed62…`.
- **L-aC's smoke `step-000064`** (asks on both reads, carry 32 rows): the served artifact equals the pod path's byte for byte (`c0c2edd4…`), with 0 trunk sections differing. The served snapshot keeps the pod's `hamming_ask` unchanged, and I-xa passes end to end (integer engine scores it, the default refuses it by name).
  - The pod snapshot's JSON prints its floats through f64 when it carries asks (`serde_json::to_value`). So a `jq` comparison flags every float as different. The artifacts, which read f32, are equal.
- **The negatives:** a pod snapshot with one base read changed fails the check, at I-xg and at `laptop_snap`.
- **Tests:** test-read 45/45, test-seal passes; `hamming-base-copy`'s unit test passes. The negative gate: with the trunk check removed, a fake `cpu-fbs-export` that moves one embed number passes, and both its tests fail.
- **Fresh review** (delegate:reviewer, spec and diff only): PASS with minors. Findings 1 (a refused artifact under the served name), 3 (this section's diagnosis) and 4 (no served-trunk check) are fixed above. Finding 2: `serve` reads one ROOT, so the four seals are fetched into one assembled local root, `ROOT/ARM/sS/oh/run`, with L-aC seed 1 from `runs-2/` and the other three from `runs/`; the NOT_RUN `runs/L-aC/s1` stays sealed and is named in the REPORT. The nits (`UP_FORCE=0` still forces, `seed-339.sh` overwriting `ixg/`) are left: no launch sets either.

**What changes:**
- `read-339.sh`'s `ixg` mode and every served export (I-xa, the trend, `−C`) use the laptop snapshot. The `−C` cut is taken from the served snapshot of `step-016384`.
- `seed-339.sh` takes the seal root, so L-aC seed 1 reruns into `runs-2/` (L-002) on a fresh pod, the stop rule's one rerun. It also takes `UP_FORCE=1` for `uor-pod up --force-retry`, after the two SSH failures tripped the circuit breaker.
- The pod scripts and their pins (job, chain, seal) are unchanged.

**The order is unchanged:** L-aC seed 1 first, and the other three after its I-xg PASS.

**The rerun (9 Nov):** L-aC seed 1 relaunched on `vast-59899989` ($0.563/h), sealing to `runs-2/`. I-xg passed at 14:39:34Z on the repaired path: the new pod's GPU snapshot is byte-identical to the first pod's (`6a424749…`), the pod path again gives `9b01ed62…` (recorded only), and the laptop snapshot gives `86e51d5e…`. L-a seed 1, L-aC seed 2 and L-a seed 2 then launched on three more pods at $0.563/h each, four pods and $2.24/h in all.

**Pins added:**
- `cpu-fbs-export` `6748a09f4c253d7d0af5144ade990024d373d5d0756b30a2d44895e76bd19865`
- `hamming-base-copy` `b085a8a7d2f0fabe92ebb3c129f3fd565f63ab58dec0887bef05c856539a9da3`
- claude-322's checkpoint files, pinned in `ixg` mode.

## Ask-range repair (row 0a)

[claude] 9 Nov 2026. The negative first.

**All four full runs stopped before step 16,384,** each on the same refusal, so no cell has a `step-016384`. Each sealed or was copied as NOT_RUN (EXIT 6), and none is a reading of the arms.

| Cell | Seal | Train exit | Last checkpoint | Train seconds | The refusal |
|---|---|---|---|---|---|
| L-aC s1 | `runs-2/` | 1 | `step-014336` | 7,619 | `d6.ask[51596] = -8.000233` |
| L-a s1 | `runs/` | 1 | `step-014336` | 7,443 | `d6.ask[4235] = -8.000174` |
| L-aC s2 | `runs/` | 1 | `step-014336` | 7,713 | `d2.ask[4266] = 7.9999156` |
| L-a s2 | laptop (the store was full) | 1 | `step-014336` | 7,519 | `d6.ask[4240] = -8.000165` |

**Classified: a format-range fault, not a mechanism result (row 0a, L-007).**
- **What it is:** training as served rounds every table to its integer each step. An ask table is Q12 in `i16`, so it stores −8 to 32767/4096. The float weights the optimizer holds drift, and a few reach the end of that range. The first to pass it stops the run.
- **Measured** on L-aC s1 (1,048,576 weights per ask table): `d6` rms 1.337 at step 12,288 and 1.410 at 14,336. Weights at |v| ≥ 7: 783, then 2,045. At |v| ≥ 7.9: 0, then 18.
- **It is not the carry:** the no-carry arm stopped the same way, at the same step.
- **Only the ask tables are near their range.** On L-a s2's `step-014336`, the largest |v| per table: ask 7.92 to 7.93 (limit 8); weight 6.13 (limit 128); margin 19.2 (limit 32,768); rotations 1.24 (limit 128). The trunk's tables reached 16,384 in claude-322, which had no asks.

**The repair (commit `721a4784f`):** training as served holds each ask weight inside the range its export can store, before it rounds (`hold_hamming_ask_in_exported_range`).
- A weight past the end sits at the end, and is served and trained as that value. The served model is what the export stores.
- This is the format's own range, the same saturation any fixed-point store has. It is not a squashing function: inside the range every value is unchanged.
- The export and the snapshot still refuse a value past the range, since a saved table should never hold one.
- A test pins it (`an_ask_weight_past_the_served_range_is_held_at_its_end`), and fails with the hold set aside, on this same refusal.

**What changes for the rerun:**
- **Pin:** the job, chain and launcher pin `721a4784f265efea6b490d5c3100779e4c4b1607`. Against the old pin, `crates/` differ by the hold, its test and `hamming-base-copy` (added for I-xg, row 0c, and used on the laptop only).
- **All four cells rerun,** under this sheet and these bars, unchanged. Row 0a is "repair and rerun that arm and seed". A run that resumed from `step-014336` would be a different training path from the one registered, so each restarts from step 0.
- **Seal root:** `runs-3/` for all four (L-002: `runs/` and `runs-2/` hold the NOT_RUN seals and stay).
- **The order is unchanged:** L-aC seed 1 first, the other three after its I-xg PASS. I-xg's bar and pin (`86e51d5e…`) are unchanged.
- **Cost:** a full run is about 7,600 s × 16,384 / 14,700 ≈ 8,500 s, plus setup and scoring, about 3 h. Four pods at $0.563 to $0.70/h: **about $7 to $8.50.** The first round cost about $5.20 on its four full pods.

**The store is full (a second fault, an instrument one).** At 17:13Z `uor-store put` was refused: "Private storage limit reached". Reads still work, so pods still start.
- **The seal step while that holds:** the chain ends `seal=4`, with the seal complete on the pod and the pod up. `pull-seal-339.sh` copies `oh-seal/` to `local/claude/claude-339/seal-laptop/ROOT/ARM/sS/`, checks `SHA256SUMS` and the tar's list, and only then takes the pod down. If any check fails, the pod stays up. `fetch-339.sh` reads a laptop seal in place of the store's.
- **If the store has room again by then,** the seal goes to the store and reads back as Step 5 says.
- The laptop seals go up to the store once it has room. The REPORT names which seals are on the laptop only.

**The rerun's fresh review (9 Nov):** PASS with minors, from a non-author reviewer who read the code. The fixes, before launch:
- **The seal pull now starts by itself** (`autopull-339.sh`, started by the launcher in its own screen), so a pod whose seal the store refused doesn't sit idle waiting for the session. It reacts only to `seal=4` on its own pod, and tries three times.
- **The margin's low side is measured.** The rounding refuses a margin at or below 0. The smallest margin across the four runs at `step-014336` is 9.44, down from 9.83 at `step-012288` on the same table. At that rate it stays near 9 at 16,384.
- **The launcher needs its seal root named,** so no cell can seal to an old root by default.
- **A failed copy is set aside** rather than blocking a retry.
- **The REPORT counts the held weights:** the entries at −32,768 and 32,767 in each `step-016384` ask table, so the saturation is visible.

**How the rerun started (9 Nov, about 17:50Z to 18:17Z).** The negatives first.
- **The first relaunch never answered SSH.** `uor-pod` deleted `vast-59924022` at 900 s. Nothing ran on it.
- **The second relaunch failed I-xg because of my own launcher (instrument).** On `vast-59926331`, the laptop's `read-339.sh` refused to write to the I-xg folder the first round had left, so it compared nothing and reported FAIL. Each launch now names its own `IXG_OUT`.
  - That FAIL made the job seal. The NOT_RUN seal it put in `runs-3/L-aC/s1` is moved byte-identical to `runs-3-void-ixg-collision/L-aC/s1`, so `runs-3/` holds only the rerun's own runs. It is not a reading of any arm.
- **One more launch failed before SSH.** L-a seed 2's first try ended `up exit=1`, with its pod (`vast-59928452`) stopped while loading. The launcher tried again, and the stopped pod is deleted.
- **The store has room again.** With the owner's approval (#2028 6747419800), 784 files (15.4 GB) were deleted and the history squashed. Writes worked again about 18:05Z, so the seals go to the store and read back as Step 5 says. `autopull-339.sh` stays armed for a `seal=4`.

**What runs:** all four from step 0 at the pin above, sealing to `runs-3/`. L-aC seed 1 (`vast-59927559`) passed I-xg against `86e51d5e…` at 18:12:56Z, and the other three started after it: L-a seed 1 (`vast-59928444`), L-aC seed 2 (`vast-59928474`), L-a seed 2 (`vast-59928489`).

## Read (9 Nov, row 4c)

**All four runs reached `step-016384` and sealed (`job=0 seal=0`), and the read's instrument checks passed. The outcome is row 4c, a negative for this configuration.** L-aC's first-line asked count is 0 and 2, its `−C` cut reads the same at both seeds, no cell has a complete reply, and EOS is 2 and 0. The numbers and hashes are in the [REPORT](../../loop/tasks/one-model-end-to-end/claude-339/REPORT.md).
