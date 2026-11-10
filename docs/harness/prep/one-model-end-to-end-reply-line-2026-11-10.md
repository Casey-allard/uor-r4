# Prep sheet: the reply-line retrain (one-model-end-to-end, claude-345, 10 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-345, "The retrain" ([ORDER.md](../../loop/tasks/one-model-end-to-end/claude-345/ORDER.md)); the owner approved it on 10 Nov ("Yes, retrain").

TEST FITNESS: FIT WITH CHANGES, all applied (10 Nov, a fresh-context reviewer that had not seen the plan; it read this sheet and the trainer, the reader, the writer and the exporter at the pin). The changes: the corpus has three abstaining replies, so the reader takes all three and a new check (I-n) counts them on the panel; row 3 needs at least 200 matched pairs and row 4 takes the case under 200; rows 1 and 3 say that the float panel shows copying of an oracle line, and leave "says the value" to the laptop; a row-1 result is provisional if the rule's GPU tests did not run; one edge of the key rule is named under "Base artifact".

PRE-RUN REVIEW: PASS WITH CHANGES, all applied (10 Nov, a fresh-context reviewer over the pod kit against the pin and this sheet; it found nothing that would fail a healthy run). Applied: the launch script's default price ceiling is this sheet's $1.60/h; the job checks both options in the run's `config.json`; the rule's test step requires all six matching tests to pass.

**Why.**
- The memory-line run ([sheet](one-model-end-to-end-memory-line-2026-11-10.md)) trained the talking recipe on a TRAIN that has the memory line. Arm m talks, and the line barely moves the value: over 407 matched pairs it was exact on 7.6% with the line and 6.1% without it (D = 0.015), and on the laptop it said 1 of 4 stated values.
- Two serving switches on arm m's own artifact then showed two causes. On a 7-check conversation it said 1 of 7 values as exported; 4 of 7 with the copy read taking its latest exact match and the line typed in the reply's wording; 6 of 7 with the turn memory also switched off.
  1. **The copy read averages.** Each copy head reports the mean of its 16 nearest matches, ties to the earlier, so a value named twice is blended with the older one.
  2. **The turn memory overwrites.** Its reply table stores the commonest next token of a reply, and at serving that stored token replaces a copied value from its second token on.
- A third fact follows from the first: with the line written as `Memory: Maya.` the copy read has to find the value inside other wording. With the line written as the reply itself, the reply is a token-by-token copy of the line.

## Question

Trained under the rule it is served with (latest exact match), with the turn memory leaving to the copy read every key the copy read already answers, and with the memory line written as the reply, does the served model say the value its memory line names, with no serving switch?

## Base artifact

- **Recipe:** arm m's, unchanged (`--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 192 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48`, 16,384 steps, seed 2), plus two trainer options that are off by default:
  - `--hamming-copy-latest 1`: each copy head reads one match, the latest earlier position whose key equals the row's exactly, at full weight, and reads nothing when there is none. Under it the copy heads' rotations and margins receive no gradient: an exact match does not depend on them.
  - `--turn-memory-leave-copied 1`: a key of the opening table or the reply table is not stored when the copy read's latest exact match names the target on more than half of that key's TRAIN rows. The rule is per key, not per row, so a key's minority rows cannot take the key and override the copy. One edge is not pinned by a test: where a token is repeated immediately, the vote's own reading of the latest match may differ from the read's by one position. It can only move a key whose rows are near half and half.
- **The pin:** `fd71a02785f951fb7f07c93051a0722eee7b76e6`. The trainer and `fbs-snapshot` are built at the pin on the pod. The reader, the exporter and the chat program run on the laptop at the branch head; the trainer and the writer are not changed after the pin.
- **How the rule reaches the laptop:** the checkpoint records it for the float scorer; the run's `config.json` records `hamming_copy_latest: true`; `fbs-export --run-config` writes it into the artifact's Hamming record; the session serves the recorded rule. No `--copy-rule` flag is used.
- **Compared with, not rerun:** arm m and arm o of the memory-line run, sealed under `results/claude/one-model-end-to-end-memory-line-2026-11-10/run/`.

## Steps

The GPU tests of the rule, a 64-step smoke on the same inputs, then 16,384 training steps at `--eval-every 2048` from a fresh start. `fbs-snapshot` then reads `models/step-016384`.

## Data split

Written by `m-world-recall-corpus`, world v2, protocol 2, every other argument at its default. `recall=reply` writes each memory line as the turn's own reply (`Memory: Your friend is Maya.`), an abstaining reply included. The writer has three abstaining replies: "I don't know. You haven't told me yet.", "You haven't told me that yet." and "I'm not sure, you didn't tell me.".

| Set | Writer arguments | Size | tokens.u16 SHA-256 | response_mask.u8 SHA-256 |
|---|---|---|---|---|
| TRAIN, line as the reply | `conversations=60000 seed=13 recall=reply` | 8,127,216 tokens; 39,088 lines (34,460 values, 4,628 abstentions) | `9681a1270dd0125da1fb8ab22bb6925e9811d4a882f5c46614c9bbc5576db16e` | `d046f92d8fcc338adb2b80d3dafabf6e067f00ebee936e91385cb8956414c4e3` |
| Panel, line as the reply | `conversations=1000 seed=14 recall=reply` | 134,323 tokens; 641 lines (562 values, 79 abstentions) | `db48d045b8e4c80b65cf8022080ccfc969e9bcde2713eef1ee3de50bb55964f9` | `83e51b2f101668d8a2945f34d1199f0e05601684d1607b0017d4b1b6991999dd` |
| Panel without the line (the memory-line run's, unchanged) | `conversations=1000 seed=14 recall=off` | 121,894 tokens; no lines | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |

- In the store: `results/claude/one-model-end-to-end-reply-line-2026-11-10/corpus/` and `panel-line/`, and `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/`.
- The same conversations as the memory-line run's: TRAIN has 215,017 turns and 2,459,896 reply tokens, each panel 3,586 turns and 40,719 reply tokens. Only the line differs.
- The panels are held out by the writer's seed (14 against 13). They share the world and its templates with TRAIN, so they test copying the line on unseen conversations of the same kind, not open text.
- The line is the oracle's: the writer knows the reply. At serving the line comes from `uor-chat`'s sieve, which is changed in this unit to write a reply sentence around the value it finds: `Memory: It's Maya.`, one of the writer's own reply forms, and `Memory: You haven't told me that yet.` when it finds nothing. (This sheet first said the sieve would flip the person of the stored statement; the shorter form was chosen before the run because it reuses the sieve's tested value and cannot repeat a token of the statement.) That change is laptop-side, has its own tests, and is not in the pin. In TRAIN "It's VALUE." follows the key-value questions, not the relation questions, so on a relation question the model is asked to copy a line form it saw after other questions; the REPORT shows every line the sieve wrote.
- In-run DEV is `ft/dev` under the data root, as in arm m. Its curve is recorded and decides nothing.

## Arms

- **r:** the recipe with both options on the TRAIN above, seed 2, one RTX 5090 Vast pod.

Not run, and what that leaves unsaid: the three changes move together, so this run cannot say which of them did what. No single-change arm, no twin of arm r trained without the line, no other seed, and no two-token copy keys. They are run only if a result needs them.

## Readings

**On the pod, in float** (`train-in-space --score-rows CK --panel p=<panel> --sample-opening last --samples 1200 --context 192 --reply-tokens 48 --retain-rates 1`): two passes on the last checkpoint, one per panel. Each pass writes the greedy replies with the checkpoint's turn memory and with it taken out.

- **Recall replies, matched pairs, E, D, completion:** as defined in the memory-line sheet, read by `recall-pairs` with `none=` set to the writer's three abstaining replies, so the abstaining lines are counted and left out of the pairs as the `none` lines were. Arm m: 562 recall replies, 407 matched pairs, E 0.076 with the line and 0.061 without; arm o on the no-line panel: 0.061 over the same 407.
- **G = E(r, line) − E(o, no line):** arm o's sealed pass on the same no-line panel file is the reader's `control`. Arm o never saw a line and is served without one (L-010's control).
- **E over every recall reply** (`E_line_recall`): exact replies over all recall replies of the line panel, paired or not. Added to the reader for this run because the longer lines move more window boundaries, so fewer replies may find a partner.
- **The same E with the turn memory taken out:** what the copy read does without the tables. Reported; no bar.
- Not read on this panel: the names-the-value share. The line is the whole reply, so that share would repeat E.
- **What the float panel can and cannot show.** On this panel the line is the reference reply, token for token. A high E, D or G shows that the model copies the line before it into its reply. It does not show that the right value was found, and it does not score abstention. Finding the value is the sieve's part, and only the laptop readings test the two together.

**On the laptop, as the integer artifact** (`fbs-export --raw-gates --raw-transition --run-config config.json`, then `--context 8256`, served by `uor-chat --stack ... --protocol 2` with `/memory auto` and no other switch), for arm r beside arm m as exported:

- **L:** the memory-line sheet's fixed 12-line script: of its four direct questions, those whose reply contains the stated value as a whole word and no other stated value. Arm m: 1 of 4.
- The 7-check conversation of the serving switches (arm m as exported: 1 of 7) and the 14-turn conversation of 10 Nov, word for word in the REPORT with each line the sieve wrote. This is the reading the owner judges by.

Not measured here: tokens per second, other seeds, open text, reworded questions beyond the three in the script.

## Bars

Fixed before the run. They route the next step; none stops the line of work.

- **3 of 4:** L at or above 3. This is the product reading and it leads.
- **0.10:** D at or above 0.10 and G at or above 0.10.
- **200:** D and G are not read from fewer than 200 matched pairs; `E_line_recall` is still reported.

## What each outcome unlocks

Rows are checked from the top, and the first that matches owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | a job step fails, a GPU test of the rule fails, an input hash differs, the smoke leaves no reply key to the copy read, there is no last checkpoint or config, or the export fails | NOT_RUN, an instrument failure | repair it and rerun under a new root; the panel passes alone can be rerun from the sealed checkpoint |
| 1 | L at or above 3, and D and G at or above 0.10 | the float model copies an oracle line, and the served model says the value the sieve's line names (provisional until the rule's GPU tests have run, if they did not on the pod) | seeds 1 and 3; then the next gaps in this order: reworded questions (the sieve's reach), lines that repeat a token (a two-token key under the latest rule), then the exact address memory writes the line and the recipe moves to human text |
| 2 | L at or above 3, and D or G under 0.10 or under 200 pairs | the laptop shows the value and the panel does not | the panel is the suspect: the matched samples are read by hand before the cell is classed; the conversations stand as shown |
| 3 | L under 3, at least 200 matched pairs, and D and G at or above 0.10 | the float model copies an oracle line and the laptop does not show it | serve the same script with the line typed by hand in reply wording (`/memory TEXT`): if that works the sieve's line is repaired, if not the export path |
| 4 | L under 3 otherwise (D or G under 0.10, or under 200 matched pairs) | this configuration does not make the served model say its line's value | read 20 line-panel replies token by token against their lines; where a reply leaves its line (the first token, a repeated token, the end marker) names the repair, and the pass with the turn memory taken out says whether the tables or the read lose it |

A row-4 reading closes this configuration only (CHARTER §4.1): it is classed as instrument or mechanism from those 20 replies before it is called.

**Prediction, written before the run and before the fitness review** (a calibration record, not a bar): E on the line panel between 0.45 and 0.85 over the matched pairs (arm m: 0.076), D above 0.40, G above 0.40; L = 3 or 4; at least 5 of 7 on the 7-check conversation with no switch; row 1. Where I expect it to fail: the end marker (at the line's full stop the copy read names the turn separator, so the end rests on the reply table and the trunk), lines that repeat a token, and a sieve line whose wording the model never saw as a reply. What I cannot size: whether wording on turns without a line suffers from the keys left to the copy read.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on the pod.
- **I-g:** on the pod, before the smoke, `cargo test --release -p uor-r4-float-train --features cuda --lib -- latest` runs the rule's tests on the GPU (two of the four are GPU-only: the sort takes the later of two exact matches; a copy head under the rule reads what followed the latest copy, where the mean blends two). A failed test is row 0. If the test build itself does not compile on the pod, that is flagged, the tests are recorded as not run, and the run goes on: the smoke and its scoring pass still exercise the rule, and the outcome is then read with that flag.
- **I-k:** the smoke prints how many opening keys and reply keys were left to the copy read. The job stops (row 0) when either line is missing or no reply key was left. Both counts go in the REPORT.
- **I-3:** TRAIN's and both panels' files match the hashes above before training; the tokenizer's and DEV's hashes are written to `inputs.sha256`.
- **I-e:** the line is encoded as served. Done on the laptop, 10 Nov: the new TRAIN holds the served marker's ids (`201 53 91 353 970 28 379 970 2852 28`) 39,088 times, the writer's own count of lines.
- **I-n:** the reader finds the abstaining lines. Done on the laptop, 10 Nov: the three abstaining replies, as the reader encodes them after the marker, are followed by `\nAssistant:` 21 + 34 + 24 = 79 times in the panel and 1,536 + 1,519 + 1,573 = 4,628 times in TRAIN, the writer's own counts. After the run the reader must report 79 abstaining lines less those outside the sampled windows, and no more than 562 recall replies; otherwise the pair counts are not read.
- **I-w:** the writer's source is unchanged between the commit the sets were written at (`abd7ad86c6519006d63d291314f468ec4ee0c2c0`) and the pin, and the panel rewritten at the pin has the same two hashes.
- **I-c:** the run's `config.json` has `hamming_copy_latest: true` and `turn_memory_leave_copied: true` and otherwise equals arm m's field by field, apart from the TRAIN hashes.
- **I-s:** a 64-step smoke on the same inputs, then one 8-sample scoring pass on its checkpoint. A smoke that does not train is row 0. A smoke scoring pass that fails is flagged and does not stop the run. Its numbers are never read.
- **I-x:** the artifact exported with the run's config has `copy_latest` in its Hamming record, and equals the same export without the config in everything but that header field (the unit test `the_copy_rule_is_recorded_only_when_latest` holds the general case).
- **I-t:** the run ends exit 0 with `models/step-016384`, a snapshot and its `config.json`.
- **Seal:** the pod's `run/` is sealed with a file list and SHA-256 sums, put in the store under `results/claude/one-model-end-to-end-reply-line-2026-11-10/run/r-s2/<try>/` and read back. A run is never written into an existing root (L-002).

## Minimum test power

One seed: a first reading, and no milestone claim rests on it. A row-1 result is rerun at seeds 1 and 3 before the notebook records it as more than one seed. D and G rest on at least 200 matched pairs. L has four questions and is not a powered reading: it is the product reading, shown word for word, and the panel carries the count.

## Cost

One pod at no more than $1.60/h (the memory-line run's pods cost $0.56 to $1.60/h on 10 Nov, and its three arms took about 50 minutes each, not the 5.5 hours its sheet expected). Expected: about 65 minutes with the test build, under $2. Ceiling: 3 hours, $4.80. One of the four pods and at most $1.60/h of the $8/h cap.

## Compute and stop rule

- One pod is launched and watched until it reaches SSH and starts its job. The watcher is started before anything ships (L-004). It renews the lease and never cancels a healthy run; time, spend and stall are notices only.
- The job seals `run/` on every exit, a signal included. Scoring passes are capped at 40 minutes each and never fail the job.
- The pod goes down when its own four checks pass: the chain ended with its seal, the required files are in the seal's list, the store's sums read from the laptop equal the pod's, and no GPU process is left. A failed job's pod goes down once its logs are sealed.

## Addendum, 10 Nov: seeds 1 and 3

Written after seed 2's result and before either seed runs.

Seed 2 is row 1: E on the line panel 0.759 over 399 matched pairs (303 exact), D 0.759, G 0.697, L 4 of 4, and the rule's six GPU tests ran. As this sheet states, a row-1 result is rerun at seeds 1 and 3.

- **What runs:** arm r at seeds 1 and 3, one pod each, with the same pin (`fd71a027`), kit, inputs, bars and rows as seed 2. Nothing in the trainer, the data or the reader changes.
- **How the three seeds are read:** each seed is classified by the rows above. The result is *replicated* if seeds 1 and 3 are both row 1, *mixed* if one is, and *not replicated* if neither is.
- **Laptop reading:** the same three scripts, word for word, with the chat binary built at `bd2532281`, as for seed 2. A reading with the later sieve (`b94edae8d`) is shown beside it and is not the registered one.
- **Not a bar here:** seed 2's negative on the hand-written 14-turn script (three general turns where the copy read intrudes, one wrong acknowledgment) is read on each seed's conversations and reported; it does not move a seed's row.
- **Cost:** two of the four pods, at most $1.60/h each and $3.20/h of the $8/h cap; about 55 minutes and under $1 each at seed 2's price ($0.78 in all).
