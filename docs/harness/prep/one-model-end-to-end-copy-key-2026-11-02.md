# Prep sheet: the copy read's two-token key, trained as served (one-model-end-to-end, claude-261, 2 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [claude-261](../../loop/tasks/one-model-end-to-end/claude-261/ORDER.md). Path step 4: one model end to end.

TEST FITNESS: FIT - fresh non-author adversarial review (round 4) of sheet SHA-256 `68e8fe9dcb12f391bb6684e1400119c3761c94f1eb1422396eeaf149b6505852` and `grade.sh` SHA-256 `3cd7663c93d2d5b8b4f59b8b4425a832d3f3be9505a91ad6af222a2fc2796e29`, after rounds 1 to 3's findings were fixed. Its one new minor (row 4(i)'s re-timing could repeat without end) is fixed in this text.

PRE-RUN: PASS - fresh D67 review, round 2, of `job.sh` SHA-256 `7e0d2464a2870d0cf0036967e1108d8cd3ed32272248678a69dd2088631748f0`, `watch.sh` SHA-256 `081aaacdfe4215253badca193490d29da1e3e8a53213de7dfe9d8cb601adc9dd` and this sheet. Round 1 failed on two majors (a run with no arm left waited forever; the seal missed the provenance files), both fixed. Round 2's three minors are fixed in `job.sh`, now SHA-256 `238b8593ce0d071ef879d7996067f2f58f36cf46258500c3acae333b89b043e5`. The changes to this sheet since its FIT (`68e8fe9d…`) are the FIT and PRE-RUN lines, row 4(i)'s end, I-2's cut on multiples of 2,048, and R6's timed input and spread; none moves a bar.

## Question

Does keying the Hamming read's copy heads on the two tokens before a position, in place of one, make the trained-as-served base model reply better on prompts phrased in ways TRAIN never used, at no cost in served speed?

The measured fault (claude-259, `docs/labs/one-model-end-to-end/capacity-2026-11-02/trace/`):
- In conv-a turn 3, at the reply's " is", both copy heads at both depths keep the five positions after an earlier " is" (8, 26, 37, 51, 64).
- They read the mean of what follows those positions, so "your friend is" is served the space before 47.
- Of DEV's strata, only copies (2,002–3,140 of 4,130) moved across claude-259's nine runs.

## Base artifact

This is a training comparison: the candidates are the checkpoints each arm produces. Both arms start from claude-259's (b) recipe, trained as served through main's current `--train-as-served 1`, which carries deepseek-4's forward surrogates (#3565). `train-in-space` and `fbs-snapshot` are built `--locked --features cuda` on every pod from this unit's pushed head (the key is commit `987cc3fe1` on base `eac5c1d90`), and the head and the binaries' SHA-256 values are recorded before the smoke. (Recorded after the pre-run: I-0 on seed 1's first pod, head `af7154c00`, found #3565's activation quantizer refusing on CUDA, F32 times F64. It is repaired in this unit's head, with the values unchanged, and I-0 runs again on every pod. See the ORDER, "Found at I-0".) Outputs go under `results/claude/one-model-end-to-end-copy-key-2026-11-02/sN/`.

## Arms

Both arms use the same command, which differs only in the key:

`train-in-space --data-root DATA --out OUT --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 8192 --eval-every 1024 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --seed S --width 256 --hamming-read 4 [--hamming-copy-key 2]`

| Arm | Copy key | What a copy head's key at `s` names |
|---|---|---|
| (b) control | 1 (flag absent) | `x_{s−1}`, all 64 bits |
| (k2) | 2 | `x_{s−1}` in bits 0–31, `x_{s−2}` in bits 32–63; the query is `x_t` then `x_{t−1}` |

- **Seeds** 1, 2 and 3: one 1 × RTX 5090 pod per seed, with both arms sharing GPU 0. Width-256 arms shared a GPU in claude-259's round 2.
- **Then** each arm's last checkpoint is scored: `train-in-space --data-root DATA --score-rows CKPT --out SCORED --samples 1536 --reply-tokens 64 --retain-rates 1 --panel p-w=PW --panel p-v=PV --seed S`. The checkpoint holds its rounded values (`round_if_served`), so this scores the served values. Its saved `hamming_read.safetensors` carries the copy key.
- **Then** `fbs-snapshot CKPT SNAPSHOT.json`, for the export.
- **First, on seed 1's pod only:** deepseek-5's four 256-step runs, exactly as [its sheet](one-model-end-to-end-served-comparison-2026-11-02.md) defines them (plain and served, at seeds 1 and 2), read by that sheet's own bar. They run before both arms, alone on the GPU. That sheet pins "the exact PR head reviewed before the run", which must contain `--served-forward` and #3565's served path. This unit's head contains both, and this sheet's pre-run review is the review of that head for them; their runs leave the copy key at its default 1, the read as it was. deepseek-5's REPORT records them as `NOT_RUN`. Their step 1 is this sheet's I-0, on the same pod. Their outputs go under `results/claude/one-model-end-to-end-copy-key-2026-11-02/s1/ds5/`, with seed 1's `SHA256SUMS`, and are read back before the pod goes down.

## Readings

These are claude-259's readings, with the same files and the same `grade.sh` method, with (k2) in place of the larger arms. The grade is claude-259's `grade.sh` with `ARMS="b k2"` and nothing else changed, kept at `local/claude/claude-261/grade.sh`, which reports an absent (b) as absent where claude-259's stopped, SHA-256 `3cd7663c93d2d5b8b4f59b8b4425a832d3f3be9505a91ad6af222a2fc2796e29`:
- **R1 (decides): the right prefix on p-w.** Per prompt, the number of leading reply tokens equal to the reference's, its end mark included. (k2) is paired with (b) at the same seed: b counts the prompts where (k2)'s prefix is longer, c those where (b)'s is.
- **R2 and R3 (reported, same pairing):** exact replies, and right openings.
- **R4 (reported): the DEV strata**, from `progress.jsonl`'s `dev_nearest` at every eval: openings, copies and the rest. **Copies is the stratum the key acts on.** "Still rising" is as claude-259's sheet defines it: more than 143 hits above the eval at half the steps. "(k2)'s copies rose" means (k2)'s copies hits at the last eval are more than 143 above (b)'s at the same seed.
- **R5 (confirms): R1 on p-v.**
- **R6 (decides speed): served.** Each arm's seed-1 export (`fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT OUT.uorlut`) runs through `uor-chat --protocol 2` on conv-a, conv-b and conv-c, with transcripts and decode tok/s (a 177-token run minus a 1-token run, the median of 5). **The speed condition:** (k2)'s decode tok/s is at least 0.95× (b)'s on each of conv-a, conv-b and conv-c. The key reads one more code per candidate, and nothing else in serving changes. **The timing protocol:** both exports are timed in one session, with no build or other model running on the laptop (the load average is recorded before and after), in five interleaved rounds (b, k2, b, k2, …). Each conversation is timed on its first turn's ids as protocol 2 encodes them (`/tokens` in its transcript, up to and including the `Assistant:` tag), given as `--prompt-ids`. (Recorded after the launch, before any timing: the 177 is claude-259's 14-id prompt. Where a first turn leaves fewer decode steps in the 192-token context, as conv-a's 18 ids may, the long run is the most that fits, found by `uor-chat`'s own refusal, and decode tok/s is (its steps − 1) over the time difference.) Every run is recorded, with each arm's spread, (max − min) / median of its five 177-token runs' wall times, per conversation. If any spread passes 5%, the whole timing is run once more. If it passes 5% again, the speed condition is unread.
- **R7 (reported): the measured case.** conv-a turn 3's reply under each arm's seed-1 export: whether it names Maya.

## Bar

**Per seed**, (k2) against (b) on R1 gets one of three verdicts:
- **better**: b − c > 2·√(b + c), with b + c ≥ 10;
- **worse**: c − b is past the same floor;
- **level**: anything else, including b + c < 10.

An arm is **absent** at a seed when its run there diverged or failed (the stop rule), whatever earlier checkpoint it holds. An absent arm is never level, and the seed gets no verdict. No earlier checkpoint of such a run is scored or put under `A-scored/`.

The rows are read in order, and the first that matches decides.

| Row | Outcome | What it changes |
|---|---|---|
| 1 | (k2) better at all three seeds, on R5 b − c > 0 at every seed, and R6's speed condition holds | **(k2) becomes the base.** The memory-line corpus (the chat's `System: Memory:` turns, which TRAIN never shows) trains on it next. Then long context, then the served memory line. |
| 2 | (k2) worse at all three seeds; both arms measured at every seed; no seed cut; neither arm still rising | (k2) is dropped at this data and recipe, and (b) stays the base. The next copy-key unit is the slot key (entity × attribute atoms), since an ordered two-token key also matches a shared two-token frame. This closes this configuration only (CHARTER §4.1). |
| 3 | (k2) level or worse at every seed, with level at one or more; both arms measured at every seed; no seed cut; neither arm still rising | The two-token key does not move replies at this data. (b) stays the base. If (k2)'s copies rose (R4) at every seed, the key works where it acts and replies are lost elsewhere, and the next lever is the reply's continuation. If not, the next unit is the slot key. |
| 4 | Anything else: better at some seeds only, R5 failing, R6's speed condition failing or unread, a seed cut, an arm absent at a seed, or an arm still rising | Keep and iterate, taking the first of these that applies. (i) If (k2) met every row-1 condition but speed: if speed was unread, the timing is run again in this unit (a re-timing, not a new run), and if that is unread too, go to (ii); if it failed, the next unit makes the paired read as fast, and (k2) becomes the base once it is. (ii) Otherwise, if (k2)'s R1 b − c summed over its measured seeds is above 0, it is the candidate, and the next unit runs three more seeds. (iii) Otherwise (b) stays the base, and the slot key is next. |

## Instrument checks

- **I-0, the read's own job (probe a), on every pod before training:** `cargo test --release --locked --features cuda -p uor-r4-float-train --lib -- --test-threads=1`, the whole CUDA library suite (deepseek-5's sheet asks for it too). It passes only if its output shows at least one test run, none failed, and the line `copy_key_two_tells_two_facts_after_the_same_token_apart ... ok`. The suite includes claude-261's two-token test, `copy_key_two_tells_two_facts_after_the_same_token_apart`: on a window with two facts after the same last token, the copy-key-1 heads tie the two positions and add their mean, and the copy-key-2 heads keep only the right one and add its value. A failure stops the pod before any training: it is an instrument fault.
- **I-1, the smoke (probe a), on every pod:** 256 steps of both arms side by side. Each must run `run_long`; its `config.json` must hold `"train_as_served": true`, and (k2)'s must hold `"hamming_copy_key": 2`. Each must score with `--samples 4` and snapshot; (k2)'s snapshot JSON must hold `"copy_key": 2`. If (k2) fails, the code is repaired in this unit before any long run. If (b) fails, the pod stops.
- **I-2, throughput (probe b):** each arm's smoke `train_context_tokens_per_second` projects 8,192 steps. If either passes 90 minutes, both are cut to the largest multiple of 2,048 steps the slower one fits, so that R4's eval at half the steps exists, and `steps.txt` records `CUT`. Ninety minutes is claude-259's limit for this recipe: its (b) and (h) took 33–52 minutes sharing a GPU, and an hour would leave the shared pair little margin. The ORDER's "about an hour" is amended to match. Under 2,048 steps, the pod stops.
- **I-3, parity, on the laptop:** each arm's seed-1 export goes through `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64`, on DEV's prompts taken from the arm's own `samples-memory.json`. At most 2.9% of first tokens may disagree with the float file (claude-243's bar). Past that, or if the export refuses, the arm's R6 and R7 are instrument faults, repaired in this unit before R6 and R7 are read; R1 to R5 stand.
- **I-4, pairing:** both arms at a seed decode the same prompts in the same order. `grade.sh` stops on a mismatch of `position` or `prompt`.
- **I-5, the export:** (k2)'s artifact header holds `"copy_key": 2` in its `hamming` record; (b)'s holds no `copy_key`.

## Data split

TRAIN, DEV, the tokenizer, p-w and p-v are exactly claude-259's ([its sheet](one-model-end-to-end-capacity-2026-11-02.md), Data split), pinned in the code (`hamilton_sheet.rs`'s `PINNED_*`) and checked against their `SHA256SUMS`:

| File | sha256 |
|---|---|
| `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
| `mw-balp/train/response_mask.u8` | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
| `ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| `ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| `tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| p-w `tokens.u16` (`inputs/claude/heldout-panel-p-w1200-2026-10-31`) | `038b0085e6888d988d4b0b6ceb7846015f21f1c40f26ec3286ddcc049c17d217` |
| p-w `response_mask.u8` | `797fc23474231595ae42b9cfbf7930ae2021b13d23d8b8b3d4fb3c7dda06ec16` |
| p-v `tokens.u16` (`inputs/claude/heldout-panel-p-v-2026-10-30`) | `3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0` |
| p-v `response_mask.u8` | `c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023` |

No key was chosen on either panel.

## Steps

1. Build the pinned binaries on each pod, and record their SHA-256 values.
2. Run I-0. On seed 1's pod, run deepseek-5's four runs and their measurements.
3. Run I-1's smoke for both arms, and project I-2.
4. Train both arms for 8,192 steps (or I-2's equal cut).
5. Score each checkpoint (memory on and memory out; DEV, p-w and p-v), then snapshot it.
6. Save every output under the store root with `SHA256SUMS`, read it back, and only then take the pod down.
7. On the laptop: export, I-3, I-5, R6 and R7, then apply the table in order.

## Minimum test power

Three seeds, with 1,202 paired p-w prompts and 393 p-v prompts per seed. claude-220's two decodes of p-w pair 339–384 prompts a seed against a floor of 37–39, so R1 detects a net shift of about 3% of the panel.

## Cost and stop rule

- **Cost:** three 1 × RTX 5090 Vast pods at about $0.6–1.2/h each. Each takes about 2.8 h: about 15 minutes to build, test and smoke, about 50 minutes of shared training (claude-259's (b) and (h) took 1,994–3,101 s together), and about 80 minutes of scoring. claude-259's scorer ran at 0.77 s a greedy sample, and here both arms score side by side on the one GPU. An arm whose scoring runs out of GPU memory scores again alone after the other; if both must, scoring takes about 160 minutes, and each pod about 4 h ($8–14 in all). Seed 1 adds about 40 minutes for deepseek-5's runs (their sheet: 10 minutes of training and 20 of evaluation, with margin), and I-0's whole suite about 10 minutes on every pod. In all, about $6–10, at most about $3.60/h with three pods: inside 4 pods and $8/h with what other labs hold at launch (`uor-pod status`). Each lease is taken for 4 h and renewed. A healthy run is extended, never cancelled.
- **Stop rule:**
  - An export refusal is an instrument fault: it is repaired, never counted.
  - A diverged loss (non-finite) stops that arm, and is classified when it happens (CHARTER §4.2). If (b) at the same seed diverges too, or the divergence traces to a fault in the code, it is an instrument fault: repaired, never counted. If only (k2) diverges, it is a (k2) result at that seed: (k2) is absent there, read under row 4. If only (b) diverges, with no code fault traced, the control failed: (b) is trained again at that seed, alone on a fresh pod, and the seed is read once it has a checkpoint. If it diverges again, (b) is absent at that seed (row 4).
  - Added during the run, on claude's decision review after seed 2's (k2) ran out of memory (the ORDER's "Found in the main run"):
    - An arm that runs out of GPU memory while it shares the card is an instrument fault, never counted.
    - That arm trains again at its seed, alone on a fresh pod, at the paired arm's step count, and the seed is read once it has a checkpoint.
    - If it runs out alone too, it is absent at that seed (row 4).
  - Outputs are read back from the store before any pod goes down.
  - A watcher ends on every terminal state (`EXIT`, a fault file, error lines), with a 90-minute stall window for the scorer's silent passes.
