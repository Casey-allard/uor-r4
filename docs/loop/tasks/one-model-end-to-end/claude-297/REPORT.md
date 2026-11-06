# REPORT: one-model-end-to-end, claude-297 (steps and data for the nearest objective)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Pin: `92ea7ccdf45d7924f6cd3cac1498d262ee6ce692`. PR #3774.

Eight runs were trained, scored, sealed and graded: the sheet's 2×2 of TRAIN D1 or D3 at 8,192 or 32,768 steps, (near)'s recipe at width 256, seeds 1 and 2. The prep sheet is [`one-model-end-to-end-steps-data-2026-11-06`](../../../../harness/prep/one-model-end-to-end-steps-data-2026-11-06.md), with its FIT and its D67 pre-run review applied. The seals are `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s1/steps-data-s1.tar.gz` (sha256 `1c7a09781c60475067b4fc2319db92a16ef0f589dcf355db1d09f35edcc4ec69`) and `runs/s2/steps-data-s2.tar.gz` (sha256 `dd3cc8885cf1b35aa840da9f7d029d8a8dc6097a8321d963e4abdfcc8ce5c467`), each with its `SHA256SUMS`, read back on the pod and checked again on the laptop.

**Claim check.** This is step 3 of Casey's one track (#2028 6675163438, 6 November: "Yes, one track"): "whatever it finds, until the bar is met". It tests whether the shipped trunk's recipe, (near), meets the bar with more steps or more data, and if not, which lever moves it.

## What did not work, first

- **The one-track bar is not met.** No cell is `ahead` of its own TRAIN's T2 at both seeds.
  - (n3L), D3 at 32,768 steps, reads `inside` against its T2 (d +16,207 / −21,768, band 37,975). It is ahead at seed 1 (272,867 against 256,660) and behind at seed 2 (234,892). On unseen rows it is `inside` too (d +7,250 / −24,231, band 31,481).
  - The other three cells are `behind` their T2 at both seeds (table below).
- **The long runs are not stable across seeds.** (n3L)'s two seeds land 37,975 rows apart, and (n1L)'s 6,529 apart, against 4,183 and 7,014 for the 8,192-step cells. The band that decides the bar is that spread, so at two seeds a long run cannot show a lead smaller than about 38,000 rows.
  - The spread is the optimizer's, not the measurement's (§4.2, instrument against mechanism): seed 2's training loss reproduced exactly across hosts. At step 31,744 its lost first run (Quebec) and its rerun (Czechia) both read 0.9592743465327658 for (n3L) and 0.9693796455394477 for (n1L), to every printed digit. The lost run kept no models, so seed 2's top-1 itself was not reproduced, only the trajectory. Seed 1's (n3L) was already lower at step 8,192 (0.9601 against 0.9791) and stayed lower to the end (0.9306 against 0.9594).
- **Steps alone do not clearly move it.** The steps lever at D1, (n1L) against (n1S), is `mixed` (d +15,425 / +1,882, band 13,543): ahead at seed 1, not distinguishable at seed 2.
- **(n1L)'s loss and its top-1 disagree across seeds.** Seed 2 ends at a lower loss (0.9695 against 0.9754) and fewer hits (225,571 against 232,100). Loss is read only within an arm (the sheet), and this is why.
- **Seed 2's first run was lost to a host fault** at step 31,744 of 32,768 and rerun from step 0, because the trainer at the pin has no resume (Departures).
- **The spend passed the ORDER's $14 cap: about $15.17** (Timings). The sheet's D67 cap was $16, and nothing was cut.
- **Not read:** the prediction mix on wiki-00000. The sheet lists it as a reading, and no bar rests on it.

## The readings

Panel top-1, no memory, on the 1,460,697-row human panel (row identity `5f88a5b3…`), against each cell's own TRAIN's T2 and its unigram (`scale-probe pair`, `clean=` its own TRAIN's seen set). TRAIN's unigram gets 37,421; T2 gets 253,018 on D1 and 256,660 on D3.

| cell | top-1 s1 / s2 | vs T2 (all rows) | vs T2 (unseen) | vs unigram | T2's share s1 / s2 |
|---|---|---|---|---|---|
| (n1S) D1, 8,192 | 216,675 / 223,689 | behind (−36,343 / −29,329, band 7,014) | behind (−35,114 / −29,460, band 5,654) | ahead (band 7,014) | 28.2% / 32.4% |
| (n1L) D1, 32,768 | 232,100 / 225,571 | behind (−20,918 / −27,447, band 6,529) | behind (−21,773 / −26,967, band 5,194) | ahead (band 6,529) | 33.8% / 31.2% |
| (n3S) D3, 8,192 | 236,892 / 241,075 | behind (−19,768 / −15,585, band 4,183) | behind (−23,055 / −19,022, band 4,033) | ahead (band 4,183) | 37.7% / 38.7% |
| (n3L) D3, 32,768 | 272,867 / 234,892 | **inside** (+16,207 / −21,768, band 37,975) | inside (+7,250 / −24,231, band 31,481) | ahead (band 37,975) | 47.3% / 36.0% |

T2's share is the cell's hits minus the pair's `b` (rows the cell gets and T2 misses, since d = b − c), so the rows both get, over T2's hits (`pair/<cell>-vs-t2.json`).

**By the sheet's Bar:** (n3L) reads "Not distinguishable from T2 at these seeds. The next unit adds seeds to that arm before any other change." The other three read "Not met for that configuration only" (§4.1). Under `pair`'s band rule, added seeds cannot turn (n3L) ahead (Still open), so that next step is set aside.

### The levers

The D1 cell is the A side, so a D1 `behind` means the D3 cell is ahead.

| lever | pair | verdict | d s1 / s2 (band) |
|---|---|---|---|
| steps at D1 | (n1L) − (n1S) | mixed | +15,425 / +1,882 (13,543) |
| **data at 8,192 steps** | (n1S) − (n3S) | **behind: D3 is ahead** | −20,217 / −17,386 (2,831) |
| data at 32,768 steps | (n1L) − (n3L) | mixed | −40,767 / −9,321 (31,446) |
| both | (n1S) − (n3L) | mixed | −56,192 / −11,203 (44,989) |

The data lever is the only one decided at both seeds: D3 moves the trunk, by 17,000–20,000 rows at 8,192 steps, beyond the band at both seeds and on unseen rows (d −15,160 / −13,867, band 1,293). Under "What each outcome unlocks", this is "None does, and data moves it": the next unit trains longer at D3, or on more human text.

**Record only:** (n1S) reproduces claude-296's sealed (near) exactly at both seeds (`record-n1S-vs-296near`, d 0 / 0), across the two pins; the trainer binary is the same file (`528a2634…`).

## Departures from the sheet

- **Seed 1 ran on the sheet's one-card fallback, not on a two-card pod.** Two two-card pods for seed 1 never answered SSH within 900 s, and `uor-pod` deleted each before anything ran: vast-59383672 at 07:35Z and vast-59385113 at 07:50Z. Both were offers on the Quebec host where seed 2's first pod ran healthy, and `uor-pod up` has no option to exclude a host.
  - Its circuit breaker then refused a third `up`. The one-card request ranks a different host first, so it was retried with `--force-retry`, recorded in PLACE.md before the call.
  - Seed 1 ran on vast-59386600 (1×5090, $0.975/h, Maryland), its four cells in turn, longest first, under the fallback's limits: MAXSEC 21,600, HOURS 6, BUDGET $6.
  - That is the sheet's Step 2 fallback and its one named exception to the 4-hour cap. The recipe, the pin and the cells are unchanged.
- **Seed 2's first run was lost to a host fault and rerun from step 0.** Its Quebec host (machine 153046) went offline at 09:23Z, with both long cells past step 31,744 of 32,768, and came back at 09:52Z after a reboot, with the job dead.
  - The trainer at the pin has no resume, and its checkpoints hold no optimizer state. Finishing from step 31,744 would have reset the optimizer late in a decaying schedule, a different configuration, so those cells are NOT_RUN on that pod.
  - Seed 2 was rerun from step 0 on the one-card fallback, vast-59396995 (1×5090, $0.986/h, Czechia), with the same limits as seed 1.
  - A two-card replacement (vast-59396933, $2.191/h) was over the sheet's $2.00/h ceiling and was deleted unused after 185 s.
  - The lost run's logs (no models) are kept locally, sha256 `baaa1becfa8fc85333edf7eadf327e450a77afa80fc86825b364a7adde6c6418`. Its last losses, at step 31,744, were 0.9693796455394477 for (n1L) and 0.9592743465327658 for (n3L). The rerun read the same at that step (`progress.jsonl`) and ended at 0.9695 and 0.9594 at step 32,768.
- **The two seeds' binaries were built on different pods,** as the sheet's Base artifact allows. Both read `train-in-space-92ea7ccd`, sha256 `528a26342dbd83b05fee3b410b880cd6cf2827c5b4f5bce95a89679cd31fb98a` (`binaries.sha256`).

## What changed

- **The prep sheet** at `docs/harness/prep/`, with its FIT and D67 changes.
- **This REPORT and one notebook row** in `docs/research/notebook/float-training.md`.
- **The roadmap item's sequence for the one track,** with three leads from a read-only review of lexlean-r4 (below).
- **No code changed.** The runs used the pin's trainer and grader.
- **Off the repository:** the seals in the store; the job, chain, seal, watch, reattach and reading scripts in `local/claude/claude-297/` (uncommitted).

## The lexlean-r4 review (Casey's request, 6 Nov)

Four read-only agents read `~/lexlean-r4`'s work since 28 October, with file and line evidence. Negatives first:
- Its "2.085B-parameter" UORLUT01/8 model is counted table entries, not learned weights: n-gram counts over hand-written replies held as string literals in `gpu/train_uorlut_v8.rs` (lines 148–465), "pick the highest-frequency next_tok per ngram" (line 1345), and `total_parameters()` sums the tables (lines 1485–1490).
- Its `TernaryAssocBank` is set-only bigram and trigram bits (lines 1060–1067, 1173–1179). No Hebbian rule exists in its code.
- Its UR4GLM "GPU training" is a greedy keep-if-not-worse coordinate search on an integer L1 and Hamming cost over about 10,000 rows, 170 ms in all (`gpu/general_glm.cu`, `gpu/rtx5090_training_receipt.json`). It has no next-token objective.
- It has no held-out next-token grade. Its "held-out" check is 23 prompts by substring match, whose expected text is the training replies.
- Its serving reader of our lut-stack format uses an exponential lookup table and an RMS norm (`crates/wasm/src/lut_stack_exec.rs`, lines 1605 and 1639), which CHARTER §3.2 bans, so it is a format cross-check only.

Three leads were put on the roadmap, in order: a centroid refit of the readout on this unit's sealed checkpoints (from `train_uorlut_v8.rs` lines 1001–1032), an address-margin term in the objective (`crates/wasm/src/lexlean_generated.rs` lines 223–292), and, after the bar, a strict-improvement guard for train-equals-serve on the integer table (`docs/LEARNING.md` lines 46–93). One proposal was rejected: folding T2's successor table into the trunk's weights would grade the trunk on T2's own answers.

## Measurements

### Instruments

- **I-0:** `cargo test -p uor-r4-float-train --features cuda --lib` passed on both pods at the pin: 382 passed, 0 failed, 12 ignored (`tests.out`).
- **I-1:** all eight 64-step smokes exited 0 with a finite loss (`smoke/*.train.exit`).
- **I-3:** D1's and D3's tokens, masks and manifests, the tokenizer and DEV matched the sheet's shas on both pods (`i3-*.txt`); all eight runs' `.i3` read `ok`.
- **I-4:** all eight graded checkpoints score the same 1,460,697 rows, identity `5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.
- **I-5:** every scored checkpoint's `.i5` reads `same` on every panel and on DEV (40 of 40 lines), and DEV's rows agree across the four per-panel processes (`*.dev-agree`).
- **I-s:** the grades and pairs ran with claude-295's seen sets for D1 and D3 (`local/claude/claude-295/seen/d{1,3}`), and `pair` accepted them; every pair names its `clean_seen_sha256`.
- **Every train, score and chain exit is 0.** That is 56 of 56 exit files, 28 a seed: 8 train (the smokes included), 4 per-cell scores and 16 per-panel scores. Each pod's `oh/run/EXIT` reads 0, and both chains ended `job=0 seal=0`.

### Timings and the pod

- **Training:** 1,466–1,599 s for the 8,192-step cells and 5,893–6,396 s for the 32,768-step cells, each alone on a card: about 0.18–0.20 s a step (`*.train.seconds`).
- **Seed 1:** vast-59386600, up 07:52:33Z, bootstrap 18 s (a cache hit), chain `job=0 seal=0`, down 12:23:18Z: $4.40.
- **Seed 2, first run:** vast-59383676 (2×5090, $1.775/h, Quebec), up 07:20:17Z, lost at 09:23Z, down after its logs were kept: $4.99.
- **Seed 2, rerun:** vast-59396995, up 09:34:38Z, bootstrap 28 s, chain `job=0 seal=0`, down 14:25:52Z: $4.78.
- **Unused pods:** vast-59383672 and vast-59385113 about 15 minutes each at $1.775/h (about $0.89), and vast-59396933 for 185 s at $2.191/h (about $0.11).
- **Spend:** about $15.17 in all, past the ORDER's $14; the sheet's D67 cap was $16.

## Checks run at the head

- **The head** is this PR's final commit. No code changed after the pin.
- `loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-297/ORDER.md` and `loop pr-check --local` are run at the head and recorded on the PR.
- `cargo xtask roadmap check` fails on two problems in Kimi's `infra-desktop-client-tauri` item (from #3675), which predate this PR and are not in its scope.
- **The pods** are all down, after both seals were read back.

## Skills used

None applied.

## Still open

- **The one-track bar** is not met.
- **More seeds cannot move (n3L)'s verdict, so the decision review sets the sheet's seeds rule aside here** ([claude], 6 Nov). `pair` gives every seed a band of at least the spread of d across all the seeds given, and `ahead` needs every seed beyond its band (`docs/labs/scale-probe/instrument/src/lib.rs` lines 919–953). It also takes at most three seeds.
  - Seed 2's d is −21,768. For the lowest seed to be ahead, its d would have to exceed max d − min d, which is at least 0. A seed with a negative d can never do that, so no added seed can turn (n3L) `ahead`.
  - The Bar's line for an `inside` arm ("adds seeds to that arm before any other change") was written without that case in mind. Spending GPU time on it would buy a reading that cannot change.
  - The sheet's pre-registered branch that does apply is "None does, and data moves it": (n3S) against (n1S) is decided at both seeds. Its next unit trains longer at D3, or on more human text.
  - The seed spread itself is the second target. (n3L)'s seeds separate by step 8,192 (loss 0.9601 against 0.9791), so a recipe whose seeds agree is part of meeting the bar.
  - This choice is logged for Casey, who can reverse it.
- **The centroid refit** (the roadmap's step 2) can ride on this unit's sealed checkpoints, which hold the models.
- **The trainer has no resume.** A host fault late in a 32,768-step run costs the whole run. Saving optimizer state in the checkpoints would let a run continue exactly.

## Plain summary

This unit trained the model longer and on three times as much human text, at two random seeds, to see whether it now beats the "commonest follower" table (T2) at guessing the next word.

More text clearly helps: at both seeds it adds about 17,000–20,000 correct guesses out of 1.46 million. Training four times longer helps less clearly.

The best setting, three times the text and four times the steps, beat T2 at the first seed (272,867 against 256,660 correct) but not at the second (234,892). The two seeds landed in different places while training, and the difference is real, not a measuring fault: rerunning the second seed from scratch on another machine retraced its training exactly.

So the bar is not met yet. More seeds would not help: the grading rule needs every seed to be ahead, and the second seed already is not. The next step is more training on the larger text, with a recipe whose seeds land in the same place. GPU time cost about $15.

References #3407
