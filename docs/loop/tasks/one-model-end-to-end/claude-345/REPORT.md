# REPORT: one-model-end-to-end, claude-345

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3911.

This unit began as T0, the memorize test. On 10 Nov the owner redirected it ("T0 is not a track that will advance the project at all"), and it became five runs in a row, each under its own sheet in `docs/harness/prep/`:

| Run | Sheet | What it asked | Row read |
|---|---|---|---|
| T0, two seeds | [memorize](../../../../harness/prep/one-model-end-to-end-memorize-2026-11-09.md) | can today's recipe say back 1,000 dialogues it trained on | row 3, negative |
| Arms m, mk, o | [memory-line](../../../../harness/prep/one-model-end-to-end-memory-line-2026-11-10.md) | does a memory line written before the reply give recall | row 3, negative |
| Arm r, seeds 1 to 3 | [reply-line](../../../../harness/prep/one-model-end-to-end-reply-line-2026-11-10.md) | the copy read under its latest exact match, the line in the reply's wording | row 1 at all three seeds: replicated |
| Arm rk | [mixed-keys](../../../../harness/prep/one-model-end-to-end-mixed-keys-2026-11-10.md) | a two-token key on the second copy head | row 3, negative |
| Arm rt | [memory-chain](../../../../harness/prep/one-model-end-to-end-memory-chain-2026-11-10.md) | a turn memory that also sees the end of the question | row 1 at seeds 1, 2 and 3 |

Words used below. The *copy read* is the model's exact-match read over the conversation (the Hamming copy heads). The *turn memory* is the two tables of TRAIN replies keyed on a turn's tokens. The *line* is `System: Memory: ...`, written before a recall reply; on the panels the corpus writer writes it, on the laptop the chat program's *sieve* writes it from the user's earlier turns.

## What did not work, first

### Arm rt: the three-key turn memory reads row 1, and costs recall where a value is copied

- **Recall fell, and stayed above the bar.** E is 0.712 (284 of 399 matched pairs exact), against arm r's 0.759 (303), and against 0.784 (313) for the same checkpoint with the turn memory taken out. Recall replies exact: 398 of 562 (arm r 418). V, the registered loose count, is 0.752 (300); the strict count is 284.
- **Where it fell: inside a copied value and right after it.** The turn memory makes 55 panel replies wrong that the same checkpoint gets right without it (arm r's table: 19), and makes 192 right (arm r's: 122). Of the 55, 19 run on after the full reply ("It's brostfar." becomes "It's brostfar is Lisbon."), 14 change a number ("Your lucky number is 735." becomes "7335", "634" becomes "64"), 8 are another greeting and 14 are other. Arm r has 1 and 2 of the first two shapes.
- **On the laptop the same fault is one reply.** "What is my lucky number?" gets "It's 40." where 47 was stated, in both scripts. With the turn memory off the same artifact says "It's 47." So the 18-turn script's recall is 3 of 4 where arm r's is 4.
- **The cause, read off the code and not yet counted.** The table gives a key to the copy read when the copy read names more than half of that key's TRAIN rows, and a wider key yields only where a TRAIN row falls past such a key. A reply whose value TRAIN never held has no such key. Its row falls through to a wider key that ends on a common token (a digit, a word piece), and the table names a token there. Which of the three keys answers at each of the 33 replies was not counted.
- **N passes by 2 replies:** 246 of 579, where the bar of 0.42 needs 244. The row does not rest on it. The row's other condition, the 18-turn script's general questions, reads 10 of 10.
- **What the 10 of 10 is.** The script's ten general questions were written to TRAIN's wording. They were not matched against TRAIN's text here (the laptop holds TRAIN as token ids), and the panel's held-out conversations come from the same writer as TRAIN. The count shows the table saying back TRAIN's reply inside a conversation TRAIN never held. Wording TRAIN does not hold is the row's next step and is not measured here.
- **With the turn memory off the same artifact answers 0 of the 10** general questions and all 4 recall questions. Recall is the copy read's; everything else is the table's.
- **Colors stayed at 0 of 8 on the panel,** where the sheet's prediction said above 0. The rest of the prediction held: row 1, recall within 0.05 of arm r's E (0.047 lower), N above 0.42, sums above 0 (11 of 13).
- **Statements:** "I have a dog called Rex." (the L and 7-question scripts) gets the previous statement's reply, "Got it, you grew up in Lisbon!", at every seed. On the 18-turn script "My dog is called Rex." gets that same reply at seed 2, and at seeds 1 and 3 the statement comes back word for word ("My dog is called Rex."). Three of four statements are acknowledged right at every seed on the L and 18-turn scripts; the dog statement is the miss each time.
- **"Where am I from?" still declines** a stated fact; the sieve writes no line for it (listed under "Still open").
- **Seeds 1 and 3 read row 1 on the script's general questions alone.** N is 238 and 243 of 579 where the row's other condition needs 244, and the 18-turn script's general questions read 9 of 10 at both. Both answer "What is 2 plus 3?" with "2 plus 2 is 2.", where seed 2 says 5.
- **The copied number breaks at every seed.** "What is my lucky number?" gets "It's 40." at seeds 1, 2 and 3 when the sieve writes the line, and "47." when the line is typed. The fault does not move with the seed: 23 "Your" recall replies are inexact at each seed, and their first wrong token has the same count at every position (5 to 12) at all three.
- **The turn memory costs recall at every seed:** E is 0.707, 0.712 and 0.692 with it and 0.777, 0.784 and 0.767 without, a loss of 28, 29 and 30 pairs. Under the strict count V equals E at every seed (282, 284 and 276 pairs); the registered V of 0.747, 0.752 and 0.732 is the loose count.
- **Seeds 1 and 3 ran before arm r's result was published.** Their jobs started on the pods at 19:22:50Z and 19:26:58Z (the first line of each job's log) on this lab's pre-run review, as the sheet's row 1 names. At about 20:07Z an instruction in this lab's chat said to publish arm r's three-seed result before any new arm and to hold the memory-chain approval until then. Both runs had sealed by then. They are reported here as run, and this unit opens nothing further (the ORDER's hold).

### Arm rk: a two-token copy key lost recall

- **Row 3: recall was not kept.** E fell from arm r's 0.759 to 0.406 (162 of 399 matched pairs exact) and V to 0.434. On the laptop L is 0 of 4 and the four recall questions of the 18-turn script are 0 of 4; its general questions are 1 of 10.
- **Which reading fell and where:** recall, at the reply's first token. Of the 72 inexact recall replies that should open with "Your" or "You", 64 are wrong at token 0.
- **The cause, shown by one switch.** With the turn memory taken out of the same checkpoint, E is 0.724 (289 of 399). The copy read still copies; the turn memory writes over it. The leave rule of this arm asks every copy head, and at a reply's first token the two-token head has no match, so 158 opening keys were left to the copy read where arm r leaves 6,040. The stored opening then replaces the line's first word.
- **What it closes:** the leave rule that needs both heads, with keys 1 and 2. It does not close a two-token key (CHARTER §4.1).
- **Not done for this arm at the registered chat program:** the 7-question script (C3) was not run and check I-h was not looked at on its own. The row does not rest on either: recall fell on the panel and on L. With the chat program at this branch's head the script reads 2 of 7, shown word for word below.
- **The sheet's prediction missed.** It said row 2 was the likeliest and row 3 possible, for a different reason (a turn whose line says the value is absent). Row 3 was read, and the cause was the opening key, which the sheet did not foresee.

### Arm r is the base, and four things about it are negative

- **General questions are wrong on the laptop.** On the fixed 18-turn script 2 of 10 general questions are right at seed 2 and 3 of 10 at seed 1: "What is 2 plus 3?" gets "2 plus 2 is 2 is 2 is". Two causes were read off the tokens: the one-token copy key fires on common tokens inside a general question, and the turn memory's key is the first 8 tokens of the turn, which end before the word a general question turns on.
- **The turn memory does nothing for recall.** With it taken out, E is 0.764, 0.757 and 0.787 at seeds 2, 1 and 3, against 0.759, 0.749 and 0.782 with it. The copy read does the recall. What the turn memory adds is replies that are not recall: 188, 189 and 183 of 579 exact with it, 82, 120 and 66 without.
- **About one recall reply in four is still not exact** (E 0.749 to 0.782), and every seed gets the fourth statement wrong on the laptop: "I have a dog called Rex." gets "Got it, your dog is Maya." (seed 1: "May").
- **Seed 1 drops the end of a name.** It says "May" for "Maya" wherever the name is said, so its 7-question script reads 3 of 7 and its 18-turn recall 3 of 4, and with the line typed as "47" it answers "4, your dog is my." Seeds 2 and 3 do neither. The three seeds are side by side under "The replies, word for word".
- **The panel shows copying, not finding.** On the panel the line is the reference reply, token for token, and the writer knows the answer. Finding the value is the sieve's part, and only the laptop scripts test the two together. The scripts are 4, 7 and 18 turns, written by this lab.
- **The sieve changed after the sheet was registered.** The registered laptop reading uses the chat program at `bd2532281` (the 7-question script: 5 of 7). Two later sieve fixes (`b94edae8d`: the sieve needs the question's subject; `ad81e080b`: no line for a general question) give 6 of 7 on the same script and are shown beside it, not in place of it.

### Arms m, mk and o: a memory line alone did not give recall

- **Row 3 at all three arms.** Over 407 matched recall pairs, exact replies with the line and without: arm m 7.6% and 6.1% (D 0.015), arm mk 6.4% and 5.2% (D 0.012), arm o, trained without a line, 6.6% and 6.1%. Replies that name the value: m 20.4% and 16.7%, mk 18.9% and 13.5%, o 15.2% and 14.7%. L is 1, 1 and 0 of 4.
- **Why, found the same day by two serving switches on arm m unchanged.** The copy read reported the mean of its 16 nearest matches, so several matches blend into a wrong token; and the turn memory's stored TRAIN replies write over a copied value from its second token on. With both switched on arm m, and the line typed by hand in the reply's wording, the script's six value questions went from 1 right to 6 right. That was a hand-fed reading on weights never trained with either change, so it was a reason to retrain and not a result. Arm r is the retrain that puts both changes into training.

### T0: the recipe did not memorize one reply

- **This recipe at 4,096 steps did not memorize one reply.** Trained 231 times over 1,133 windows of 1,000 short dialogues, it reproduced 0 of 1,072 replies at seed 1 and 0 of 1,072 at seed 2. That is row 3 of the sheet at both seeds.
- **It says one sentence to every prompt.** Seed 1 opens every one of its 1,117 decoded replies with "Got it, your dog is". Seed 2 opens every one with "Got it, your friend is". The 1,117 true replies are 818 different sentences.
- **The miss is at the first word of the reply, and it is there under teacher forcing too.** With the true text fed in, the first reply token is right 36 times in 1,117 at both seeds (3.2%). None of the 1,117 true replies opens with "Got". The 36 are replies that happen to share its first token, " G": "Goodbye! Have a great day." (15), "Glad I could help." (13), "Good night." (2) and six names. All 36 leave the reference at the next token. So the decode is not what loses the reply; the model never has it.
- **What it did learn is the template, not the set.** Token fit on the trained set is 0.602 and 0.607. On DEV, a file it never trained on, the same count is 7,819 and 7,816 of 14,287 (0.547). After 231 passes it fits its own training text barely better than unseen text of the same kind. DEV's mix of dialogue families is not the set's, so the two shares are an observation, not a controlled comparison.
- **The loss was still falling when the run ended.** It went from 0.997 to 0.717 (seed 1) and 0.998 to 0.739 (seed 2), with the last 512 steps moving it 0.013 and 0.011 as the learning rate reached 0. So this closes this recipe at 4,096 steps on this set, and nothing more (CHARTER §4.1). It does not show that a longer run cannot fit.
- **Two recorded-only items were not done.** The sealed checkpoints were not exported and typed into `uor-chat --stack`; the decoded replies below come from the float scorer. The output-side table gradient was not re-measured (the sheet says why).

### Slips in the work itself

- **35 minutes were lost after the owner's "Run it"** (memory-line arms). Three launches went out in one background loop and all failed unseen: every cheap offer was on one host that sat loading past the tool's 900 s wait. Pods `vast-60018714`, `vast-60020631`, `vast-60023450` and `vast-60023616` were deleted by the tool or by a status read during bootstrap; none trained. The arms then ran as try-2 and try-3 on other hosts.
- **The memory-line sheet's price ceiling was passed.** It says $0.70/h; arms mk and o ran on pods at up to $1.60/h to get off the stuck host. The three arms cost $0.61, $1.10 and $1.09, under the sheet's $9.40.
- **Arm rt's first pod never took the key.** `vast-60106351` answered on its SSH port and refused the key for 900 s. The launcher had been given the wait and network settings under the wrong variable names. try-2 was started on another host while it waited.
- **Commit `abd7ad86c` broke another crate's build.** It added a field to an integer snapshot struct that `uor-r4-cpu-train` builds by literal. `fd71a0278` moved the value to the run config and removed the field.
- **A fault in arm rt's kit was caught before launch, not by review.** The copied job checked for a config field that arm r's config does not have; it would have failed the run after training.
- **One planned check could not run.** The trainer refuses to train without a GPU, so the export route for the new table was checked in the pod's smoke (I-r) and not on the laptop first.
- **The sheets failed the harness's shape rule twice.** DeepSeek repaired T0's sheet (`c4449e7da`); this lab then wrote the mixed-keys and memory-chain sheets without the same headings, CI failed, and `6c631f694` added them. No bar changed in either repair.
- **T0's seed 1 first pod never came up.** `vast-59995065` was deleted by the pod tool after 900 s with no SSH. A plain relaunch got `vast-59997533`, which ran clean.
- **Both claims were refused by the harness.** `loop claim` refused this unit ("scope Cargo.lock overlaps Cargo.lock of item cpu-training-pilot (PR #3910)") though it touches no `Cargo.lock`, and refused claude-346 because this unit held the item. Both ORDERs were written by hand. For Casey: the claim check blocks a second unit on one item even when the two share no file.

## What held

- **Arm r reads row 1 at three seeds.** E 0.749, 0.759 and 0.782 over 399 matched pairs (arm m: 0.076), G 0.687, 0.697 and 0.719 against a model that never saw a line, L 3, 4 and 4 of 4 at seeds 1, 2 and 3. By the sheet's addendum the result is replicated. On the laptop the four recall questions of the 18-turn script are 3, 4 and 4 of 4.
- **Each cause was shown by switching it,** not argued: the mean against the latest exact match, the turn memory on and off, the leave rule's head count.
- **The laptop count of the three-key table matched the pod's build to the row** (five counts, below), and an artifact made before the table exports byte for byte with the new exporter.

## What changed

- **T0:** no code.
- **The reader:** `crates/uor-r4-training/src/bin/recall-pairs.rs`, a new binary that reads the matched recall pairs of two panel passes.
- **Serving** (`crates/uor-r4-integer`): the copy rule an artifact declares (`mean`, `latest-exact`, `latest-nearest`), written by `fbs-export --run-config` from the run's config; `uor-chat --copy-rule` and `--no-turn-memory` as switches; the sieve writes its line as a reply sentence and writes none for a general question; a `memory_chain` record under schema `uor-r4.lut-stack/9` and the three-key read. An artifact without the record is read as before.
- **Training** (`crates/uor-r4-float-train`), every option off by default: `--hamming-copy-latest`, `--turn-memory-leave-copied`, the leave rule per copy head, `--turn-memory-tail`, and three laptop counts (`--reply-bound-train`, `--reply-bound-dev`, `--reply-chain-tail`).
- **The corpus writer:** `m-world-recall-corpus recall=reply`.
- **In the repository:** the aligned design ([DESIGN.md](DESIGN.md)), five sheets, this REPORT, the notebook rows.
- **In the store,** under `results/claude/`: `one-model-end-to-end-memorize-2026-11-09/`, `...-memory-line-2026-11-10/`, `...-reply-line-2026-11-10/`, `...-mixed-keys-2026-11-10/` and `...-memory-chain-2026-11-10/`.

## Measurements

### Arm r at seeds 1, 2 and 3: replicated

The reply-line sheet's addendum was written on 10 Nov after seed 2 was read and before seeds 1 and 3 ran. It reads each seed by the sheet's rows and calls the result *replicated* if seeds 1 and 3 are both row 1, *mixed* if one is and *not replicated* if neither is. Row 1 is L at or above 3 with D and G at or above 0.10.

**Seeds 1 and 3 are both row 1: the result is replicated.** Seed 2's reading is the one registered before they ran and is unchanged.

The registered reading of each seed: the reader at the pin, the turn memory on, 1,141 decoded line-panel replies against 1,108 no-line replies.

| Reading | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| E_line: exact replies over the matched pairs | 0.749 (299) | 0.759 (303) | 0.782 (312) |
| E_noline: the same pairs served with no line | 0.000 (0) | 0.000 (0) | 0.000 (0) |
| D, E_line minus E_noline | 0.749 | 0.759 | 0.782 |
| G, E_line minus arm o's 0.063 (25 pairs; no line in training or serving) | 0.687 | 0.697 | 0.719 |
| Matched pairs | 399 | 399 | 399 |
| Pair replies that name the line's value | 300 | 306 | 315 |
| The same under the strict count (the reader at this branch's head) | 300 | 304 | 313 |
| Pair replies that end on the end marker | 389 | 390 | 383 |
| Line-panel replies that end on the end marker, of 1,141 | 1,085 | 1,084 | 1,046 |
| L: the 4 recall questions with the line typed, value named (exact) | 3 (2) | 4 (3) | 4 (3) |
| Row | 1 | 1 | 1 |

The replies that do not end on the end marker ran to the decode budget. On the laptop the end marker closed 9, 8 and 8 of the 9 replies of the L script, all 11 replies of the 7-question script at every seed, and 16, 17 and 18 of the 18 replies of the 18-turn script. The transcripts of all three seeds are under "The replies, word for word".

**Where the seeds differ.** These are model results, and none moves a row.

- Seed 1 drops the last piece of the name: "May" for "Maya" in every reply that says it. Its 7-question script reads 3 of 7 where seeds 2 and 3 read 5, and its 18-turn recall 3 of 4 where they read 4.
- Seed 1 misses the fourth L question: with the line typed as "47" it says "4, your dog is my."
- All three seeds answer "What is my cat called?" with "It's Rex." under the registered sieve, decline "Where am I from?", and acknowledge "I have a dog called Rex." with another fact's value ("Got it, your dog is Maya."; seed 1: "May"). On the 18-turn script "My dog is called Rex." gets "Got it, you grew up in Lisbon." at seeds 2 and 3 and "I'm a helpful assistant that runs on your computer." at seed 1.
- General questions on the 18-turn script: 3, 2 and 2 of 10.

**Instrument record, kept apart from the model results.** No step of the three runs failed. Each ran once (try-1), exited 0 after 16,384 steps, passed the rule's six tests on the GPU and its 64-step smoke, and was sealed; the three seals' sums were checked again on the laptop for this report. Each last checkpoint exported to an artifact of 14,186,496 bytes that `uor-chat` loaded. The run config differs between seeds in the `seed` field alone. Two instrument matters sit outside the runs and are listed where they happened: the sieve changed after the sheet was registered (the registered reading keeps the `bd2532281` program), and commit `abd7ad86c` broke another crate's build before the pin (`fd71a0278` repaired it).

**The frozen configuration.** "Arm r" from here on means exactly this.

- **Trainer pin:** `fd71a02785f951fb7f07c93051a0722eee7b76e6`. The pod built the same programs at all three seeds: `train-in-space` SHA-256 `08021329aac3b2dcf2ffa9f215a7621532f125cb7d3987b484816959b9d51bdb`, `fbs-snapshot` SHA-256 `9a9ec5dafd011b060d820c38d8e9d4bbf3cc4fb56c3c56fcc244781efb75dc06`.
- **Chat program of the registered laptop reading:** built at `bd2532281d9375fa433a5d93d06119bf086a485b`, with no serving switch beyond the `/memory` lines each script types.
- **Recipe:** arm r of the [reply-line sheet](../../../../harness/prep/one-model-end-to-end-reply-line-2026-11-10.md), 16,384 steps from a fresh start on one RTX 5090.
- **Inputs, the same at all three seeds** (the job refuses to train when one differs):

| Input | Store path | SHA-256 |
|---|---|---|
| TRAIN tokens | `results/claude/one-model-end-to-end-reply-line-2026-11-10/corpus/train/tokens.u16` | `9681a1270dd0125da1fb8ab22bb6925e9811d4a882f5c46614c9bbc5576db16e` |
| TRAIN reply mask | `.../corpus/train/response_mask.u8` | `d046f92d8fcc338adb2b80d3dafabf6e067f00ebee936e91385cb8956414c4e3` |
| Line panel tokens | `results/claude/one-model-end-to-end-reply-line-2026-11-10/panel-line/train/` | `db48d045b8e4c80b65cf8022080ccfc969e9bcde2713eef1ee3de50bb55964f9` |
| Line panel reply mask | the same folder | `83e51b2f101668d8a2945f34d1199f0e05601684d1607b0017d4b1b6991999dd` |
| No-line panel tokens | `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/train/` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| No-line panel reply mask | the same folder | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| Tokenizer | `w2-inputs/2026-10-25/driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| DEV tokens | `w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV reply mask | `w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |

**Per seed:** the run config, the last checkpoint (step 16,384), the integer artifact exported from it, and the seal. Each seal is in the store under `results/claude/one-model-end-to-end-reply-line-2026-11-10/run/r-s<seed>/try-1/`.

| Seed | File | SHA-256 |
|---|---|---|
| 1 | `r-s1/config.json` | `6a0803f7c9745223f7e13bdc9fea5f9437389dac9c5881f5727d103ac7078b04` |
| 1 | `r-s1/models/step-016384/model.safetensors` | `d27d07747e87c6010a4e50de91b283ffad7b55bc7bccc6240aed1baead1e8b81` |
| 1 | `r-s1/models/step-016384/hamming_read.safetensors` | `02cbf5b95071417e8f3973bdd022aa76bfae9d15a39408a5725e9ed57f3aad4f` |
| 1 | `r-s1/models/step-016384/turn_memory.safetensors` | `a6d0018b8682eb088ffe5236cedcfa10a3dea0aad37c8ce96496458a387eca0f` |
| 1 | `r-s1/models/step-016384/turn_memory_reply.safetensors` | `1ce1b474e86219f7a9b5a6de9c5eed295a4a4ae6d51e69532944a98ea23ccff3` |
| 1 | `r-s1-c8256.uorlut`, the integer artifact, 14,186,496 bytes | `f052d8069ed2e9ddbb87fd2feaace2ef2bd45b07580015f49fb4deefb87362c5` |
| 1 | `reply-line-r-s1-try-1.tar.gz`, the seal | `306664947cf7f52add535b41090b50194df737626fd346963e22fe9bbe6e631f` |
| 2 | `r-s2/config.json` | `0854cf9406729f702e7842dd449f2b82ce802b383684ea8a390f063251d81417` |
| 2 | `r-s2/models/step-016384/model.safetensors` | `ca1893e7e49bb00a983a27a017cdb0044e1b7a5288b8f3bbee467b0a8a3cff0f` |
| 2 | `r-s2/models/step-016384/hamming_read.safetensors` | `f48eb90993c28b740b76e53e1a2e980b05074136e8b4d3ac9a59775eb0ed124d` |
| 2 | `r-s2/models/step-016384/turn_memory.safetensors` | `675afdc866d5bc2540d5c439b9c3b6722fe0fecac4afaf3209018e83af60bfb0` |
| 2 | `r-s2/models/step-016384/turn_memory_reply.safetensors` | `8f7ac6ccaabc3ac8aaafa6694093116f68efad25a0f69b9cbe0822d5e65906fa` |
| 2 | `r-s2-c8256.uorlut`, the integer artifact, 14,186,496 bytes | `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0` |
| 2 | `reply-line-r-s2-try-1.tar.gz`, the seal | `9d1deb681492395f4469a055be7019a1f2f5ab4dc37a5a0508a2809c05d8a388` |
| 3 | `r-s3/config.json` | `32916c158e02e3db013314ffcec7d65bb6397ff4202bbe49f624b83a1d7943e4` |
| 3 | `r-s3/models/step-016384/model.safetensors` | `c4b3616c819b49527a09ef8c232e33872d653c829ea3eac9d13c21bea33107bd` |
| 3 | `r-s3/models/step-016384/hamming_read.safetensors` | `4c33a38981abb1039698bb7b9f679e1698adbec5475685c029fe60f75943b07f` |
| 3 | `r-s3/models/step-016384/turn_memory.safetensors` | `851894389dfe94d8826a4e09c0e9df19a43bb40e4c90645d098fc187bb51638f` |
| 3 | `r-s3/models/step-016384/turn_memory_reply.safetensors` | `672578ef389027375ced2ca23f0adfcab968861bc2f0fa10f17a2754d9244905` |
| 3 | `r-s3-c8256.uorlut`, the integer artifact, 14,186,496 bytes | `d229cf1b32c9eb7deed6698be98a3370ce4f18108063126f69b70f5aebcd309a` |
| 3 | `reply-line-r-s3-try-1.tar.gz`, the seal | `627dd1b472b93af91746703e85864dbb0f69e2fa25e3d9759740c065a8d357f9` |

### The line panel, arm r onward

One panel for every arm from r on: 1,141 decoded replies, 562 of them recall replies, 399 matched pairs, 579 replies that are not recall. E is exact replies over the matched pairs, V the share that names the line's value, G is E minus the same count for arm o (no line in training or serving), N the exact replies among the 579. "Off" is the same checkpoint with the turn memory taken out (the no-memory control, L-010).

| Arm, seed | E | V | G | Recall exact of 562 | All exact of 1,141 | N of 579 | E, off | All exact, off | N of 579, off |
|---|---|---|---|---|---|---|---|---|---|
| r, 1 | 0.749 (299) | 0.752 | 0.687 | 413 | 602 | 189 | 0.757 (302) | 537 | 120 |
| r, 2 | 0.759 (303) | 0.767 | 0.697 | 418 | 606 | 188 | 0.764 (305) | 503 | 82 |
| r, 3 | 0.782 (312) | 0.789 | 0.719 | 429 | 612 | 183 | 0.787 (314) | 498 | 66 |
| rk, 2 | 0.406 (162) | 0.434 | 0.343 | 217 | 351 | 134 | 0.724 (289) | 467 | 65 |
| rt, 1 | 0.707 (282) | 0.747 | 0.644 | 396 | 634 | 238 | 0.777 (310) | 507 | 80 |
| rt, 2 | 0.712 (284) | 0.752 | 0.649 | 398 | 644 | 246 | 0.784 (313) | 509 | 78 |
| rt, 3 | 0.692 (276) | 0.732 | 0.629 | 385 | 628 | 243 | 0.767 (306) | 507 | 88 |

**On the laptop,** the integer artifact in `uor-chat`, the sieve writing the line. L is the 12-line script with the value typed as the line; C3 the 7-question script; C4 the 18-turn script (4 statements, 4 recall questions, 10 general questions, a general reply counted right only when it is one of TRAIN's own replies with the right answer).

The registered readings, each with the chat program its sheet names (`bd2532281` from arm r on). These decide the rows.

| Arm, seed | L of 4 | C3 of 7 | C4 recall of 4 | C4 general of 10 |
|---|---|---|---|---|
| m, 2 | 1 | 1 | not in its sheet | not in its sheet |
| r, 1 | 3 | 3 | 3 | 3 |
| r, 2 | 4 | 5 | 4 | 2 |
| r, 3 | 4 | 5 | 4 | 2 |
| rk, 2 | 0 | not run | 0 | 1 |
| rt, 1 | 4 | 5 | 3 | 9 |
| rt, 2 | 4 | 5 | 3 | 10 |
| rt, 3 | 4 | 5 | 3 | 9 |

Arm rt's registered program is the one built at its pin (`05e50006b`), because the `bd2532281` binaries do not read a memory chain; its row above and its row below are the same run.

The same two scripts through every seed-2 artifact with one chat program, the one at this branch's head (the sieve of `ad81e080b`), so the replies can be read side by side below. A description, not a bar: the sieve here is later than the registered one, which is why arm r's C3 is 6 and arm rk's general count is 3.

| Artifact | C3 of 7 | C3 statements of 4 | C4 recall of 4 | C4 general of 10 |
|---|---|---|---|---|
| a7r, the 29 Oct talking model | 1 | 1 | 0 | 2 |
| m | 1 | 1 | 1 | 2 |
| r | 6 | 3 | 4 | 2 |
| rk | 2 | 3 | 0 | 3 |
| rt | 5 | 3 | 3 | 10 |
| rt, turn memory off | 6 | 0 | 4 | 0 |

**Exact replies by kind** on the line panel, a description and not a bar (a `jq` filter over the decoded samples; its totals agree with the reader's for arm r and differ by 1 or 2 for arm rk).

| Kind (replies) | r, 2 | r, 2 off | rk, 2 | rk, 2 off | rt, 2 | rt, 2 off |
|---|---|---|---|---|---|---|
| other (657) | 251 | 212 | 132 | 175 | 269 | 222 |
| recall, line form (203) | 160 | 142 | 71 | 151 | 168 | 143 |
| recall, "Your" form (158) | 137 | 138 | 97 | 141 | 135 | 140 |
| greeting (68) | 40 | 4 | 40 | 1 | 40 | 4 |
| legs (16) | 6 | 0 | 1 | 0 | 6 | 0 |
| sums (13) | 0 | 0 | 0 | 0 | 11 | 0 |
| yes or no (9) | 9 | 7 | 9 | 1 | 9 | 0 |
| color (8) | 0 | 0 | 0 | 0 | 0 | 0 |
| opposite (8) | 3 | 0 | 2 | 0 | 8 | 0 |

Arm rt's kinds sum to 646 where the reader counts 644 exact. At seeds 1 and 3 the same filter reads: sums 10 and 11 of 13, opposites 8 and 8 of 8, legs 6 and 6 of 16, greetings 40 and 40 of 68, colors 0 and 0 of 8, recall in the "Your" form 135 and 135 of 158, recall in the line's form 168 and 165 of 203.

**What the turn memory changes on one checkpoint** (panel replies right with it off and wrong with it on, and the reverse):

| Arm, seed | Made wrong | Runs on after the full reply | A number changed | Another greeting | Other | Made right |
|---|---|---|---|---|---|---|
| r, 2 | 19 | 1 | 2 | 7 | 9 | 122 |
| rt, 2 | 55 | 19 | 14 | 8 | 14 | 192 |

Arm rt's other seeds, counted the same way and not split by shape: seed 1 makes 53 replies wrong and 184 right, seed 3 makes 62 wrong and 185 right.

**The tables.** Arm r's leave rule left 187,254 of the reply table's 327,257 keys (57%) and 6,040 of the 13,454 opening keys to the copy read, and kept 140,003 reply keys. Arm rt's three-key table, counted on the laptop before the run and built on the pod:

| Count | Laptop, before the run | Pod |
|---|---|---|
| Reply keys left to the copy read | 944,018 of 1,591,205 | 944,018 of 1,591,205 |
| Keys that yield to a copied key | 16,254 | 16,254 |
| Reply keys kept | 630,933 | 630,933 |
| TRAIN rows named right | 1,439,251 of 2,244,879 (arm r's table: 1,345,704) | 1,439,251 of 2,244,879 |
| Opening keys left to the copy read | 6,040 of 13,454 | 6,040 of 13,454 |

On the panel's rows the same laptop count gave: rows the copy read owns, 12,923, of which the table names 925 (arm r: 911); other rows, 24,210, of which the table names 21,387 right (arm r: 19,856, so 1,531 more).

**Instrument checks, arm rt, seed 2.** All pass.

- The seal's sums match on the laptop and all 168 files in it check; the job ended `EXIT 0` after 16,384 steps with a snapshot of its last checkpoint.
- I-c: the run's config differs from arm r's in one field, `turn_memory_tail: 9`.
- I-g: the rule's 13 tests passed on the GPU before training; the 64-step smoke run took 33 s and its export was 55,424,640 bytes.
- I-k: the pod's table counts equal the laptop's, in the table above.
- I-m: the exported artifact is 55,424,640 bytes, schema `uor-r4.lut-stack/9`, with `memory_chain` tail 9 (SHA-256 `39672ffaad41051b87b81dec2e1f7c859cf8230939628056828486f19fd86d01`); the chat program reports 7,414 opening keys and 630,933 reply keys, the table's.
- The reader's control pass holds a partner for every matched pair, with no repeated window.
- The no-memory control (L-010) was scored on the pod for both panels and run on the laptop for both scripts.

**Instrument or model?** Model. The float panel and the integer artifact agree, each with its own memory-off control: on the panel the turn memory moves N from 78 to 246 and E from 313 to 284; on the laptop it moves the general questions from 0 to 10 and recall from 4 to 3, and the one recall loss is a number, the panel's second-largest shape. The code review left one thing unverified, that a served chain key might be formed differently from the trained one once a prompt has left its row's block. Nothing here shows it: the laptop's general replies are the table's. It is not excluded for turns longer than these scripts'.

### Arms m, mk and o

The memory-line panel: 1,137 decoded replies, 562 recall replies, 407 matched pairs. Arm m: E 0.076 (31 of 407), D 0.015, V 0.204. The other arms' shares are in the negatives above. All three arms ran 16,384 steps at seed 2, about 50 minutes each on one RTX 5090.

### T0

**The row read, with each bar at its lower seed: row 3.** R2 is 0.602, under 0.95. R1 is 0 and R2r is 0, both under 0.90. Row 0 does not fire: every instrument check passed at both seeds.

| Reading | Bar | Seed 1 | Seed 2 |
|---|---|---|---|
| R1, exact replies over complete samples | ≥ 0.90 | 0 of 1,072 (0.000) | 0 of 1,072 (0.000) |
| R2, token fit under teacher forcing | ≥ 0.95 | 24,343 of 40,419 (0.602) | 24,521 of 40,419 (0.607) |
| R2r, replies whole under teacher forcing | ≥ 0.90 | 0 of 1,072 (0.000) | 0 of 1,072 (0.000) |
| Complete samples | ≥ 800 | 1,072 | 1,072 |

**Instrument checks, both seeds.** I-p, I-0 (465 tests passed), I-3 (every hash; 1,133 usable windows), I-s, I-t and I-r pass. The counts are the sheet's: 1,117 samples, 1,072 complete, 45 cut by the window, 40,419 scored rows, and every reference position of every complete sample found in the rows file.

**Instrument or model?** A model result, for four reasons.
1. The counts above match the sheet's, taken from the set before any run.
2. R1 and R2r agree by two paths: free-running and teacher-forced both give 0.
3. The first reply token is a scored row in every sample (no sample's first position is missing from the rows file), and the trainer builds its loss rows the way the scorer builds its rows: both call `assemble`, which takes each window's `scored_offsets` (`in_space_long.rs` at the pin, lines 2017 to 2021 for the training step and 2458 for the scorer). The loss is a mean over those rows. So the first token was trained on and not learned. What the model writes there agrees: "Got" opens 32 of the 128 true first replies and none of the last ones, so it is a reply opening the model took from trained rows of this kind, not the output of a row that was never a target.
4. The loss moved every 512 steps, so the optimizer was reaching the values.

**Where the replies leave the reference** (complete samples).

| | Seed 1 | Seed 2 |
|---|---|---|
| At token 0 | 1,036 | 1,036 |
| At token 1 | 36 | 36 |
| Later, or only at the end marker | 0 | 0 |
| Exact | 0 | 0 |

**Teacher-forced hits by reply token index** (the last-opening samples). The first token is the one that must come from the context; the later ones follow from the tokens just before them.

| Index | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Seed 1 | 36/1117 | 501/1117 | 548/1116 | 604/1105 | 546/1100 | 497/1038 | 491/963 | 455/852 | 393/727 | 342/628 |
| Seed 2 | 36/1117 | 509/1117 | 548/1116 | 594/1105 | 528/1100 | 448/1038 | 489/963 | 451/852 | 392/727 | 329/628 |

**First tokens.** The true first tokens of the 1,117 samples: " It" 161, " Your" 124, " That" 114, " I" 105, " You" 99, " The" 45, and others. The model's first two tokens at seed 1: " G" then "ot", 1,117 times.

**Recorded only.**
- Distinct replies, each cut at its end marker, over all 1,117 samples: the model writes 116 (seed 1) and 152 (seed 2); the truth has 818 (823 if the 45 window-cut references are counted uncut). The commonest model reply is "Got it, your dog is." at seed 1, 428 times in the 1,117 samples and 408 in the 1,072 complete ones, and "Got it, your friend is." at seed 2, 462 and 448 times.
- Replies holding an end marker: 997 and 1,039 of 1,117. The rest run to the 48-token cap, most of them repeating one token.
- The first-opening pass (128 windows' first reply): 0 of 128 exact at both seeds. Here 32 of the 128 true replies do open with "Got it", and the model's first token is right on exactly those 32. It still gets none of them whole: it names the wrong thing and gives no value.
- DEV at step 4,096: nearest 7,819 and 7,816 of 14,287; reply openings 161 of 1,226 at both seeds, where the commonest single opening is 161. The model's DEV openings are one opening.
- Training: 1,807 s and 1,802 s for 4,096 steps (0.44 s a step). Scoring: 603 s and 579 s for the deciding pass.

**The prediction held.** Written before the run: row 3 at both seeds, R2 between 0.60 and 0.90, most inexact replies leaving at token 0. Read: row 3 at both seeds, R2 0.602 and 0.607, and 1,036 of 1,072 leave at token 0.

## The replies, word for word

### The 18-turn script on the laptop

Seed 2 of each arm, the integer artifact in `uor-chat` with `/memory auto`, the chat program built at arm rt's pin. One conversation, in this order. a7r is the 29 Oct talking model, which was never trained with a line. The last column is arm rt's artifact served with `--no-turn-memory`, the control.

| Said or asked | a7r | m | r | rk | rt | rt, turn memory off |
|---|---|---|---|---|---|---|
| My lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, you grew up inst legs. |
| My best friend is Maya. | Got it, your friend is 47. | Got it, your friend is 47. | Got it, your friend is Maya. | Got it, your friend is Maya. | Got it, your friend is Maya. | Got it, you grew up inst legs. |
| I grew up in Lisbon. | Got it, you grew up in Liswood. | Got it, you grew up in Lismouth. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon! | I grew up in Lisbon. |
| My dog is called Rex. | Got it, your friend is 47. | Got it, your friend is 47. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon! | Got it, you grew up in Lisbon. |
| What is the opposite of hot? | The opposite of hot is 47. | The opposite of hot is 47. | The opposite of hot is the opposite of hot is the opposite of hot is | The opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is the opposite of hot is | The opposite of hot is cold. | The day afterot it, you grew up in Lisbon. |
| What is 2 plus 3? | 2 plus 4 is . | 2 plus 4 is 4. | 2 plus 2 is 2 is 2 is | The opposite of hot is 2 plus 3? | 2 plus 3 is 5. | I grew up in Lisbon. |
| What color is a lemon? | It'son. | It's 47. | 2 plus 2 is a lemon? | It's yellow. | It's yellow. | It's great for. |
| What is my friend's name? | Your friend is 47. | Your friend's name is 47. | It's Maya. | Your. | It's Maya. | It's Maya. |
| Tell me a word that means the opposite of big. | The opposite of hot is . | The opposite of hot is 47. | The opposite of big is my friend's Maya. | The opposite of big is my friend's name? | The opposite of big is small. | Tell me a word that means the opposite of big. |
| How many legs does a spider have? | It has six legs. | It has six legs. | It has two legs. | It has six legs. | It has eight legs. | A duck interesting place place. |
| What is my dog called? | Your dog is named Got it. | Your dog is named. You haven't told me yet. | It's Rex. | Your. | It's Rex. | It's Rex. |
| Give me the opposite of fast. | The opposite of hot is . | The opposite of hot is 47. | The opposite of fast is my dog called? | The opposite of fast is my | The opposite of fast is slow. | Give me the opposite of fast. |
| What is 4 plus 4? | 4 plus . | 4 plus 4 is 11. | 4 plus 4 is 1. | 4 plus 4 is 11. | 4 plus 4 is 8. | The day afterot it, you grew up in Lisbon. |
| Where did I grow up? | You grew up in Lismouth. | You grew up in Lisstemfield. | It's Lisbon. | You're welcome! | It's Lisbon. | It's Lisbon. |
| What color is milk? | It's white. | It's white. | It's white. | It's white. | It's white. | I grow up? |
| What is the opposite of hard? | The opposite of hot is . | The opposite of hot is 47. | The opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of of of | The opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is | The opposite of hard is soft. | I grow up? |
| What is my lucky number? | Your lucky number is . | It is 47. | It's 47. | The opposite of hard is my lucky number? | It's 40. | It's 47. |
| Do cats bark? | No. | No. | No. | No. | No. | Hi lion. |

### The 7-question script on the laptop

The same artifacts and chat program. Four statements, then seven questions; the last question has no stated answer, so the right reply declines.

| Said or asked | a7r | m | r | rk | rt | rt, turn memory off |
|---|---|---|---|---|---|---|
| My lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, you grew up inst legs. |
| My best friend is Maya. | Got it, your friend is 47. | Got it, your friend is 47. | Got it, your friend is Maya. | Got it, your friend is Maya. | Got it, your friend is Maya. | Got it, you grew up inst legs. |
| I grew up in Lisbon. | Got it, you grew up in Liswood. | Got it, you grew up in Lismouth. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon! | I grew up in Lisbon. |
| I have a dog called Rex. | Got it, your dog is 4. | Got it, your dog is 47. | Got it, your dog is Maya. | Got it, your dog is Maya. | Got it, you grew up in Lisbon! | I'm sorry you grew up in Lisbon. |
| What is my friend's name? | Your friend is 47. | Your friend's name is 47. | It's Maya. | Your-. | It's Maya. | It's Maya. |
| Where did I grow up? | You grew up in Lismouth. | You grew up in Lisstemfield. | It's Lisbon. | You're welcome! | It's Lisbon. | It's Lisbon. |
| What is my dog called? | Your dog is named Got it. | Your dog is named. You haven't told me yet. | It's Rex. | Your-. | It's Rex. | It's Rex. |
| What is my lucky number? | Your lucky number is 47. | It is 47. | It's 47. | It's 47. | It's 40. | It's 47. |
| Who is my friend? | Your friend is Zititsss | Your friend's name is 47. | It's Maya. | Your-. | It's Maya. | It's Maya. |
| Where am I from? | Your friend is Zitsss | I don't know. You haven't told me yet. | You haven't told me that yet. | You haven't told me that yet. | You haven't told me that yet. | You haven't told me that yet. |
| What is my cat called? | Your friend is Zitsss | I'm not sure, you grew up in Lisbon. | You haven't told me that yet. | You haven't told me that yet. | You haven't told me that yet. | You haven't told me that yet. |

### Arm r at seeds 1, 2 and 3, the registered program

The three fixed scripts through each seed's integer artifact, in `uor-chat` built at `bd2532281`, word for word. These are the registered laptop readings behind arm r's rows; nothing is switched on beyond what each script types.

**The L script.** Four statements, then each question with its value typed as the line by `/memory`.

| Said or asked | r, seed 1 | r, seed 2 | r, seed 3 |
|---|---|---|---|
| My lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. |
| My best friend is Maya. | Got it, your friend is May. | Got it, your friend is Maya. | Got it, your friend is Maya. |
| I grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. |
| I have a dog called Rex. | Got it, your dog is May. | Got it, your dog is Maya. | Got it, your dog is Maya. |
| (line typed: Maya) What is my friend's name? | Maya. | Maya. | Maya. |
| (line typed: Lisbon) Where did I grow up? | Lisbon. You You's name? | Lisbon. You remember remember remember | Lisbon. Youededed |
| (line typed: Rex) What is my dog called? | Rex. | Rex. | Rex. |
| (line typed: 47) What is my lucky number? | 4, your dog is my. | 47. | 47. |
| (line typed: none) What is my cat called? | none. | none. | none. |

**The 7-question script.** The sieve writes the line (`/memory auto`).

| Said or asked | r, seed 1 | r, seed 2 | r, seed 3 |
|---|---|---|---|
| My lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. |
| My best friend is Maya. | Got it, your friend is May. | Got it, your friend is Maya. | Got it, your friend is Maya. |
| I grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. |
| I have a dog called Rex. | Got it, your dog is May. | Got it, your dog is Maya. | Got it, your dog is Maya. |
| What is my friend's name? | It's May. You're welcome! | It's Maya. | It's Maya. |
| Where did I grow up? | It's Lisbon. | It's Lisbon. | It's Lisbon. |
| What is my dog called? | It's Rex. | It's Rex. | It's Rex. |
| What is my lucky number? | It's 47. | It's 47. | It's 47. |
| Who is my friend? | It's May. You're welcome! | It's Maya. | It's Maya. |
| Where am I from? | You haven't told me that yet. | You haven't told me that yet. | You haven't told me that yet. |
| What is my cat called? | It's Rex. | It's Rex. | It's Rex. |

**The 18-turn script.** The sieve writes the line.

| Said or asked | r, seed 1 | r, seed 2 | r, seed 3 |
|---|---|---|---|
| My lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. | Got it, your lucky number is 47. |
| My best friend is Maya. | Got it, your friend is May. | Got it, your friend is Maya. | Got it, your friend is Maya. |
| I grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. |
| My dog is called Rex. | I'm a helpful assistant that runs on your computer. | Got it, you grew up in Lisbon. | Got it, you grew up in Lisbon. |
| What is the opposite of hot? | The opposite of hot is the opposite of | The opposite of hot is the opposite of hot is the opposite of hot is | You haven't told me that yet. |
| What is 2 plus 3? | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is | 2 plus 3 is 5. |
| What color is a lemon? | 2 plus 2 is a lemon | 2 plus 2 is a lemon? | It's 5. |
| What is my friend's name? | It's May. You haven't told me that yet. | It's Maya. | It's Maya. |
| Tell me a word that means the opposite of big. | The opposite of big is is is | The opposite of big is my friend's Maya. | The opposite of big is my friend's Maya. |
| How many legs does a spider have? | The opposite of big is is | It has six legs. | It has six legs. |
| What is my dog called? | It's Rex. | It's Rex. | It's Rex. |
| Give me the opposite of fast. | The opposite of fast is slow. | The opposite of fast is my dog called? | The opposite of fast is my dog |
| What is 4 plus 4? | 4 plus 4 is 1ay. | 4 plus 4 is 1. | 4 plus 4 is 1. |
| Where did I grow up? | It's Lisbon. | It's Lisbon. | It's Lisbon. |
| What color is milk? | It's white. | It's white. | It's white. |
| What is the opposite of hard? | It's of of | It's of fast. | It's of fast. |
| What is my lucky number? | It's 47. | It's 47. | It's 47. |
| Do cats bark? | No. | No. | You haven't told me that yet. |

### T0, from the float scorer

Every 45th complete sample, starting at the 8th. "Asked" is the user's last line; the model saw the whole conversation before it.

| # | Asked | True reply | Seed 1 wrote | Seed 2 wrote |
|---|---|---|---|---|
| 1 | Thank you! | Anytime! | Got it, your dog is? | Got it, your friend isat. |
| 2 | What is weldoul? | Weldoul is 3740. | Got it, your dog isttl. | Got it, your friend is G'm'm'm'm'm'm likeuc |
| 3 | Do you remember what groudpas was? | It's 260. | Got it, your dog is. | Got it, your friend is! didn fourant is! |
| 4 | Do you remember what pidweel was? | That's 20. | Got it, your dog is. | Got it, your friend is! |
| 5 | What is my friend's name? | Your friend's name is Bresfainddrees. | Got it, your dog is? | Got it, your friend is. |
| 6 | What was virkpikbrud again? | Virkpikbrud is jofekmai. | Got it, your dog is P to to! it is is is is is is is is i... | Got it, your friend is. |
| 7 | Thanks! | Happy to help! | Got it, your dog is. | Got it, your friend is . |
| 8 | What is my favorite color? | You love blue. | Got it, your dog is? | Got it, your friend is. |
| 9 | I finally have a guitar. | Oh, a guitar! How exciting. | Got it, your dog is. | Got it, your friend isnyde later. |
| 10 | Do cats bark? | No. | Got it, your dog is. | Got it, your friend is. |
| 11 | Do you remember what nuplaik was? | It's 41. | Got it, your dog is. | Got it, your friend is. |
| 12 | Can dogs fly? | No. | Got it, your dog is P to gets'reingingingingingingingingi... | Got it, your friend is T about about about |
| 13 | What do I work as? | You haven't told me that yet. | Got it, your dog is number isel | Got it, your friend is G'm |
| 14 | Do you remember what drindsturk was? | Drindsturk is sandtud. | Got it, your dog is number isel | Got it, your friend is. |
| 15 | Do you remember what mamfor was? | It's vurkmud. | Got it, your dog is today. | Got it, your friend is! didn telling me,,,, |
| 16 | What is my name? | You're Toumgrunnail. | Got it, your dog is. | Got it, your friend is G'm |
| 17 | What's my code word? | I don't know. You haven't told me yet. | Got it, your dog is. | Got it, your friend is. |
| 18 | Write a sentence that has the word rain. | The rain fell on the roof all night. | Got it, your dog is. | Got it, your friend isat. |
| 19 | Tell me what drestdast was. | Drestdast is hairshoust. | Got it, your dog is number isel | Got it, your friend is . |
| 20 | Repeat exactly: kulbrus geekgaingrek vunshorkbail runound... | kulbrus geekgaingrek vunshorkbail runound varhestnoust la... | Got it, your dog is? | Got it, your friend is P sorry you,,,,,,,,,,,,,,,,,,,,,,,... |
| 21 | Do you remember what dutkimsound was? | Dutkimsound is tultouszoum. | Got it, your dog is Pesday tell day a is  is a is bigger. | Got it, your friend is! didn four bigger. |
| 22 | What is houtfar? | It's zomban. | Got it, your dog is? | Got it, your friend is! |
| 23 | What is my lucky number? | I'm not sure, you didn't tell me. | Got it, your dog is. | Got it, your friend is G'm |
| 24 | What colour is the sky? | It's blue. | Got it, your dog is today. | Got it, your friend is. |

## What this unlocks

**Arm r is replicated at three seeds and frozen.** By the reply-line addendum seeds 1 and 3 are both row 1. Its pin, inputs, per-seed configs, last checkpoints, artifacts and seals are listed by hash under "Arm r at seeds 1, 2 and 3: replicated". It is the configuration ready for the next decision.

**Arm rt reads row 1 of its sheet at seeds 1, 2 and 3:** recall kept (E 0.707, 0.712 and 0.692; V 0.747, 0.752 and 0.732; L 4 of 4; the 18-turn script's recall 3 of 4) and general replies arrive (the script's general questions 9, 10 and 9 of 10; N 238, 246 and 243 of 579, over the 244 it needs at seed 2 alone). By the row the three-key turn memory would become the base recipe and the next unit would ask questions and statements in wording TRAIN does not hold. That step is not taken here: this unit opens no further arm (the ORDER's hold of 10 Nov), and what follows is a proposal for the next decision.

Arm rt is the first artifact of this unit that recalls stated facts and answers general questions in one conversation on the laptop: 12, 13 and 12 of the 18-turn script's 14 questions at seeds 1, 2 and 3, where arm r answers 6 at each seed.

Two things the reading found would go ahead of new wording in a next unit:

1. **The table answers inside a copied value.** The two sealed passes of seed 2 already say what silence there is worth. With the table silent on the 562 recall replies and asked on the rest, they sum to 431 + 246 = 677 exact replies of 1,141, against 644 with the table asked everywhere and 606 for arm r; E would be the memory-off 0.784. That is a sum over two passes and not a run. The proposed change is that rule on both sides (the table is not asked on a turn whose line carries a value), served on the laptop and scored, with the count of which key answers at the 33 broken replies beside it. The same sum at seeds 1 and 3 is 427 + 238 = 665 and 419 + 243 = 662, against 634 and 628.
2. **The sieve's five gaps** under "Still open", as their own change with their own test set, so a new-wording script measures the model and not the sieve.

T0's row 3 named the address memory's first test on the same set as its next step. The owner's redirect of 10 Nov set that aside: T0's follow-on steps are not started, and its result stays as the record of that configuration. The address memory's code is claude-346 (PR #3913).

## Evidence

- **Sealed runs,** each read back from the store with its sums checked on the laptop:
  - T0: `results/claude/one-model-end-to-end-memorize-2026-11-09/runs/T0/s1/` and `.../s2/`.
  - Arms m, mk, o: `results/claude/one-model-end-to-end-memory-line-2026-11-10/run/m-s2/try-2/`, `mk-s2/try-3/`, `o-s2/try-2/`.
  - Arm r: `results/claude/one-model-end-to-end-reply-line-2026-11-10/run/r-s1/try-1/`, `r-s2/try-1/`, `r-s3/try-1/`.
  - Arm rk: `results/claude/one-model-end-to-end-mixed-keys-2026-11-10/run/rk-s2/try-1/`.
  - Arm rt: `results/claude/one-model-end-to-end-memory-chain-2026-11-10/run/rt-s2/try-2/`, `.../run/rt-s1/try-1/` and `.../run/rt-s3/try-1/`. Seeds 1 and 3 passed the same instrument checks as seed 2: seal sums, 168 files, exit 0 after 16,384 steps, 13 rule tests on the GPU, a smoke export of 55,424,640 bytes, a config that differs from arm r's in `seed` and `turn_memory_tail` alone, and table counts equal to the laptop's. Artifact SHA-256: seed 1 `a85581464e0814b3958fa2b9eb8a0b0fc24865cba4c9a8b3dabdf3bece35f5e3`, seed 3 `9dbca8836acb052261d2b013ed4dfacf51389c867fc8093188af91ec4e16291b`.
- **Pods and cost.** T0: `vast-59997533` $0.52, `vast-59995068` $0.47. Arms m, mk, o: `vast-60023071` $0.61, `vast-60024244` $1.10, `vast-60023459` $1.09. Arm r: `vast-60092861` $0.83 (seed 1), `vast-60086378` $0.78 (seed 2), `vast-60093171` $0.94 (seed 3). Arm rk: `vast-60098847` $0.37. Arm rt: `vast-60107071` $0.80 (seed 2), `vast-60112858` $0.78 (seed 1), `vast-60113008` $0.75 (seed 3). `uor-pod status` at 20:08Z on 10 Nov showed no pod running. Each pod went down by itself after its four checks passed. Pods that never trained are listed under the slips above.
- **Reviews before each run:** every sheet carries its FIT (a fresh non-author review) and this lab's pre-run review. The three-key table had two fresh code reviews, one per side, before its sheet; both approved with low items, and five were fixed at the pin `05e50006b9b1dcd1799166df478e08139c0a9e48`.
- **Left from the two code reviews, not fixed:** the slot build is quadratic in the turn's length per decode step; no test covers a chain across a second turn in one session, or a chain stack with a point key.

## Done-when

1. T0's FIT and pre-run review before any pod: done, recorded in the sheet.
2. T0's two seeds trained, scored, sealed and read back: done.
3. Negatives first, the row read, the counts at both seeds, at least twenty decoded replies: this REPORT (24).
4. One notebook row for T0: in `docs/research/notebook/float-training.md`.
5. The memory-line sheet's FIT and pre-run review before any pod: done.
6. Arms m, mk and o trained, sealed and read back: done.
7. The same conversations through today's artifact (a7r) and each arm, word for word, with the panel counts and the row read: the two reply tables under "The replies, word for word", and the panel tables above. Arm o has no line to serve and is not in the reply tables.
8. The reply-line sheet's FIT and pre-run review before any pod: done.
9. Arm r trained, sealed and read back, at three seeds: done.
10. The mixed-keys and memory-chain sheets' FIT and pre-run review before any pod: done.
11. Arms rk and rt trained, sealed and read back, the row named, and for a recall loss which reading fell and where: arm rk above; arm rt read row 1 at seeds 1, 2 and 3, and where its recall fell is in the negatives above.
12. A fresh review of each side of the three-key table before its run, and an older artifact exporting byte for byte: done (arm r's artifact, SHA-256 `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0`, from both exporters).
13. Arm r's three seeds classed by the reply-line addendum and frozen by hash, with the three seeds' transcripts: done, replicated.
14. No pod running and no new arm opened by this unit: `uor-pod status` at 20:08Z showed 0 pods.

## Checks run at the head

- **At the commit that publishes arm r's three seeds** (documents only, on top of `9503749ae`): the prep check and the three ratchets were run again and read the same; arm r's three seals and arm rt's two new seals check against their sums; PR #3911's checks at `9503749ae` read 10 passed, 13 skipped, none failed.
- **The fresh review of that commit** (`7431aa4ad`) was split across three reviewers, each of which stopped at its context budget with part unread. Read and passed: the code fix of `9503749ae`; arm r's table against the reader output, its classification against the sheet's addendum, and every reply cell of the three transcript tables with their end-marker counts (the tables hold 114 reply cells: 9, 11 and 18 rows at three seeds; a line-by-line diff of each seed's column against its chat output, run afterwards, read 114 cells and no difference); arm rt's two panel rows, its laptop rows counted again from the replies, and its row against the memory-chain sheet. Found: two sentences about the dog statement that held for one seed or one script only (the arm rt "Statements" bullet and the arm r seed-difference bullet). Both are corrected in the commit after it, and the "Still open" line now names the two other replies. The built trainer was also run with `--reply-chain-tail 9`, with no `--reply-bound` and with `--reply-bound 0`: it exits 1 with the refusal both times. The parts no reviewer reached at `7431aa4ad` (the hash tables against their source files, the counts by kind, and the notebook rows) and the verdict are in the PR's review record.
- `uor-harness check --only prep`: exit 0, no error, 26 warnings, none on this unit's sheets or files. The full check was not run here; CI runs it.
- The softmax, tensor-core and discrete-float ratchets pass.
- Every count in this REPORT for arm rt comes from the binaries built at the run's pin, kept aside before the review's fixes were made. The strict value counts and the made-wrong, made-right table come from the reader at `f2bff1b37` and a `jq` filter over the saved samples.

- **Fresh-context code review at `6c631f694a7a8164d3663a00c2e3337736347e36`:** five reviewers, each given the ORDER's scope lines and one slice of the diff, none of the reasoning. The trainer's long-context file: APPROVE. The read kernels and the trainer's options: APPROVE. The integer serving format and read: CHANGES. The panel reader and the corpus writer: CHANGES. The chat program, the sieve and the serving tests: CHANGES.
- **Fixed in `f2bff1b37351a87288d998c0319853b7fa0f3966`:**
  - The forced replay and the trace picked positions under the mean rule on an artifact whose own rule is the latest exact match; they now pick by the rule in force. With the fix set aside its test fails.
  - The memory-chain record's check added to a header value before comparing it, so the largest tail wrapped in a release build. With the fix set aside its test fails.
  - `uor-chat --copy-rule mean` could not override an artifact that declares the latest exact match, and the sieve worded its line for the switched rule and not the artifact's own.
  - The reader printed G when the control pass did not hold a partner for every matched pair. Every saved reading of this unit has full cover, so no G above changes. G is now withheld without it.
  - The trainer ignored `--turn-memory-tail` without `--turn-memory` and `--reply-chain-tail` without both bound folders; both are now refused.
- **Second round, on the fix commit alone** (a fresh reviewer, the spec and the diff only): CHANGES, two items, both fixed in `9503749aef84a86293863c5485b548231e0b5f68`.
  - The refusal of `--reply-chain-tail` sat inside the reply-bound mode's own block, so the option was still ignored without `--reply-bound 1`. It now runs with the other option checks. On the laptop the option is refused alone, with `--reply-bound 1` only, and with both folders only; with all three the trainer goes on to its next check.
  - A doc comment had moved to the wrong function.
  - The reviewer found the other fixes correct, read the two set-aside tests as pinning their fixes, and listed what has no test (under "Still open").
- **V is a loose count, and stays the registered one.** The reader counts a reply as naming the value when it holds the line's ids, last id dropped, as one run. A longer word that starts with the value's ids passes, and so does a reply that lists several values. V can only be too high. It decided no row here: recall is kept only when E is also at least 0.60, and an exact reply always names its value. The reader now prints a strict count beside it (`V_closed`: the run followed by the line's closing id or the end marker). Re-read on the saved passes at `f2bff1b37`, strict against registered, of 399: arm r 300 against 300 (seed 1), 304 against 306 (seed 2), 313 against 315 (seed 3); arm rk 173 against 173; arm rt 284 against 300. Arm rt's 16 are replies that run on after the value, which the loose count passes and the strict one does not; its strict share, 0.712, is still above the bar of 0.60. The fixed reader gives the registered E and G on every saved pass, and counts no repeated window and no pair on a taken partner.
- **What the reviewers checked and found holding:** with every new option off, the trainer, the kernels and the served bytes are as on main; the latest rule takes the latest exact match, never the row itself or a later one, and reads nothing without a match; the leave rule asks every head; a wider key yields to a copied key and not the reverse; the printed counts count actual removals; exact is id for id with the end marker; the reader's denominators are the stated ones; the sieve reads only the user's turns, never the model's reply or a reference; no softmax-family function, no dense matrix product and no float in the serving read.
- **The switch, run once at `f2bff1b37`:** `uor-chat --copy-rule mean` on arm r's seed-2 artifact prints "Copy heads read under 'mean', not the 'latest-exact' this artifact was exported and scored" and serves; without the option no notice is printed. This is the only check of the option's parsing.
- **Tests at `f2bff1b37`, on the laptop:** `recall-pairs` 11 passed; `uor-r4-integer` with `serving-wasm`, the copy-rule, forced-replay and memory-chain tests, 11 passed; `uor-chat` 6 passed; `uor-r4-float-train` compiles (its GPU tests ran on the pod at the pin, I-g). The softmax, tensor-core and discrete-float ratchets pass.
- **T0's second review round** returned CHANGES in this session and was not posted to the PR. Its findings were not carried into this report; the review above read the PR's code afresh, and the final review at the head reads this REPORT's T0 sections again.
- T0, kept from the first report: I-p, I-0 (465 tests), I-3, I-s, I-t, I-r and both seal read-backs passed at both seeds; both scored runs were checked against their sealed `SHA256SUMS`.
- **Fresh-context review of T0, round 1,** at `1fb08d4e583575443693e96784a965ba10b53f4a`: two reviewers, both CHANGES; every finding and what changed is in the first report's record (commit `ea9d3990d`) and in [DESIGN.md](DESIGN.md) section 10.11.
- **Another lab's commit is in this PR.** DeepSeek pushed `c4449e7daed7fd4d6876f13637d0524f9b4c0562` onto this branch on 10 Nov. It added the report sections the shape rule requires and gave T0's sheet the headings the prep rule requires, under a dated note that names the commit the pods ran from. This lab read every added line against the sealed runs and the harness source.

## Skills used

None beyond the project loop: the registered sheet, FIT, pre-run review, sealed pod run, store read-back and report checks.

## Still open

- **Arm rt's table answers inside and right after a copied value** (33 panel replies, and "It's 40." on the laptop). Which key answers there was not counted. This is the next unit's first step.
- **Wording TRAIN does not hold** is not measured for any arm. The 18-turn script's general questions were written to TRAIN's wording and were not matched against TRAIN's text.
- Colors are 0 of 8 on the panel for arms r, rk and rt; the cause was not looked for.
- A statement about a dog is acknowledged wrong by arms r and rt ("Got it, your dog is Maya.", "Got it, you grew up in Lisbon!"); on the 18-turn script arm r at seed 1 answers it with "I'm a helpful assistant that runs on your computer." and arm rt at seeds 1 and 3 repeats the statement back.
- Seed 1 of arm r writes "May" for "Maya" in one reply; the cause was not looked for.
- Arm r, seed 2, answers the statement "My name is Maya." with "Got it, your lucky number is Maya." (seen once, in the switch check above; the recall reply after it, "It's Maya.", is right). The acknowledgement of a statement is not counted by any reading here.
- **The sieve is narrower than its description, found by the code review and not fixed here.** Each of these writes a wrong line or none, read off the code and not yet run:
  - A question made only of common words declines a stated fact: "I grew up in Lisbon." then "Where am I from?" writes "You haven't told me that yet", and the model then says so.
  - A general question that shares a word with a stated fact gets a value line: "My lucky number is 47." then "What is a lucky number?" writes 47.
  - The subject is checked for name, when and age questions only: "My favorite color is blue." then "What is my sister's favorite color?" writes blue.
  - A possessive is read as its owner: "My best friend is Maya.", "My friend's dog is Rex.", then "Who is my friend?" writes "It's dog is Rex".
  - A statement that opens with "Tell" or "Say" is skipped as a request.
  The three laptop scripts hold none of these cases except the first, so the laptop counts above stand for those scripts and not for the sieve in general.
- **No test, from the second review round:** the chat program's `--copy-rule` handling (run once by hand); the reader's refusal when no reference holds the end marker; the reader passing full cover on to G (the helper is tested, the wiring is not); the `turn` bound of the memory-chain record.
- **Tests the review asked for and this unit does not have:** `--no-turn-memory` taking out the turn memory and nothing else; a chain key carried across a second turn in one session; a general question that shares a word with the history; the trainer's key forms against the served ones where the prompt has left a row's block.
- **Under the latest rule the copy heads' key codes get no gradient from the copy read.** The one position read is at distance 0, where the distance has no slope. The value path still trains. A copy head whose nearest key is one bit off reads nothing and is not pulled toward it. Stated here because no sheet says it.
- The latest rule reads at full weight only when the head's margin is at least one bit; a head with a smaller margin reads less, and none at a margin of 0. The margins of the sealed arms were not listed.
- The slot build's cost per decode step and the two missing chain tests, from the code reviews.
- T0: this negative closes the 4,096-step configuration only; the output-side table gradient remains unmeasured for that recipe; its sealed checkpoints were not exported.

## Plain summary

Five runs under five sheets. Three are negatives: the memorize test, a memory line written in the corpus's own words, and a two-token copy key. Arm r, where the copy read takes its latest exact match and the line is worded as the reply, recalls stated facts at three seeds and answers general questions wrong. By its sheet's addendum that result is replicated, and its pin, inputs, checkpoints and artifacts are frozen by hash in this report.

Arm rt adds a turn memory that also sees the end of the question. It reads row 1 at all three seeds. On the laptop it answers 12, 13 and 12 of the 18-turn script's 14 questions, where arm r answers 6: nine or ten of the ten general questions and three of the four recall questions.

It pays for that in recall. The table now also answers inside a copied value, so 47 comes back as 40, and 33 held-out replies break that the same model gets right with the table off. With the table off the model answers none of the general questions: recall is the copy read's and everything else is the table's.

Nothing further is opened here. On 10 Nov an instruction in this lab's chat said to publish arm r's three-seed result before any new arm. Arms rk and rt had already run by then; their results are reported above as run, and the next change is a proposal for the next decision.

Not measured for any arm: questions in wording TRAIN does not hold. No milestone moves. The next unit is the table's silence inside a copied value, the sieve's gaps, and then new wording.
