# REPORT: one-model-end-to-end, claude-368 (an expert speaks with the weight of what it knows, and a sure expert refuses a reply it never saw)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3959.

The order asked whether claude-367's pair experts answer more everyday turns when each expert's vote is weighed by how surely it knows its answer and a sure expert may refuse a reply it never saw. Both were built behind a setting whose default is claude-367's vote, and a grid of 128 settings was read on the seed-23 and seed-24 panels, held out. **The bar reads "Does not add":** one setting of the 128 is inside the order's guards on all six readings and it is claude-367's own, so by the order no seed-25 panel was written. Outside the guards the best setting gains 16 good replies over the six readings, under 3 a reading, and loses twice as many good ones. The code does not join `main`; it is kept as a patch. Reading the conversations for this report showed a larger fault than the one the order was after, found on the way and not under a bar: with the experts off, in the four recorded conversations that hold a cut reply (p-s1 and p-s2), every later turn is cut again or repeats the reply before it, 35 turns of 35.

## What did not work, first

1. **The weighed vote loses replies at every setting of the grid.** At claude-367's keep and floor it gives 1,031 good everyday replies over the six held-out readings against 1,061 (−30); the best weighed setting gives 1,037. On the seed-24 panel the three models go from 170, 171, 170 of 239 to 167, 165, 163 ([why24.tsv](why24.tsv)).
   - **Why.** TRAIN words one fact several ways ("It's Beijing." and "The capital of China is Beijing."). The pair that holds the key word has seen each wording a few times, so its largest count is a small share of its total and the weighing shrinks exactly its vote. The top vote then falls under the floor and the experts do not answer: replies left to the model go from 172 to 341 over the six readings, replaced ones from 347 to 263. On p-s1, seed 24, 17 replies that were replaced and 10 that stood are now left alone, and replies that do not end go from 10 to 17.
   - A lower floor does not mend it: weighed, keep 1/32, floor 500 gives 1,029 and loses 15 to 18 good rows a reading.
2. **The refusal fires rarely and does not pay.** At claude-367's keep and floor it refuses 17 replies over the six readings (1, 2, 1 on seed 23; 5, 4, 4 on seed 24) and gives 1,059 good replies against 1,061, losing 6 good rows on each seed-24 reading where claude-367 loses 4.
   - Its larger effect is one the order did not intend to be useful: the floor is read on the top reply the sure experts allow, so 13 replies of p-s1 that stood are instead left to the model. What is said does not change.
   - The count `n` an expert must be seen with changes nothing: for every weigh, keep and floor, the grid lines for `n` of 2, 4 and 8 are the same on all six files. No expert seen fewer than 8 times changed a verdict.
3. **The best setting outside the guards is not worth a panel.** Even vote, keep 1/4, floor 1500, refusal on: 1,077 good replies (+16 over six readings, +2.7 a reading) with 30 good rows lost against 15, 8 on each seed-24 reading where the pick's guard allows 4. It is picked from the same two panels it is scored on, so a fresh panel could read lower.
4. **The three faults claude-367 named are still there.** In the new fact conversation, held out, "Can birds fly?" is answered "No." on all three models (replaced), "A dog says woof." stands for the cow on p-s2, and "What is 7 + 9?" is left as "7 plus 7 is 11." on all three.
5. **Where the held-out replies are still wrong** (seed 24, claude-367's setting, of 239): replies that need no word of the turn 13, 12, 11 wrong of 114; a word of the turn (first letter, spell, repeat) 15, 15, 16 of 39; a fact about the word 15, 15, 16 of 51; work on the numbers 26, 26, 26 of 35. Number work is the largest group and the experts move it from 8 good to 9.
6. **With the experts off, the recorded conversations do not get past a cut reply (found on the way, not under a bar).** Of the nine recorded conversations with the experts off, four contain a reply the short-cycle rule cut, all on p-s1 and p-s2. In all four, every later turn is cut again or is the same reply as the one before: 35 of 35 turns ([cut-replies.tsv](cut-replies.tsv)). On the new fact conversation p-s1 and p-s2 answer the first turn and nothing after; p-s3, which has no cut reply, answers 7 of 13.
   - The cut reply stays in the chat's history without an end mark and the next reply copies it. Two hand runs outside the order's plan ([chat-facts-2-less-one-turn.txt](chat-facts-2-less-one-turn.txt)): with the first cut turn left out of the script, p-s1 answers 6 of 12 and goes on past a second cut reply after one damaged turn; p-s2 is cut on the next question and repeats it to the end.
   - In the same run p-s1 answers the colour of a banana with "It's slin.", its own wrong reply to the capital of China two turns before: a reply that ended wrong is read back too.
   - With the experts on, the two conversations that hold a cut reply go on (0 of 8 later turns cut or repeated), because the experts' reply is what enters the history. Where the experts are unsure after a cut reply nothing protects the next turn.
7. **The code was built before the order was committed,** while claude-367's PR waited in the merge queue for a runner. The order says so. No grid was read before the order's commit (16:11:28 local time; the first grid file is 16:12:47). One reading was made before it, the replay of claude-367's seed-24 files at claude-367's own setting.
8. **This branch starts at claude-367's head, not at `main`.** PR 3958 was still in the merge queue when this report was written: from 20:36 UTC GitHub was slow to give the repository's runs a runner (one docs run of 21:11 UTC waited 11 minutes for one; the merge group's one job, queued at 21:10 UTC, had none at 21:30 UTC). PR 3958 has since merged as `88b7bd5c7951713f403bf2831c58b4c47c8499cd`, and this branch has taken `main` by a plain merge: its difference from `main` is this unit's files alone.
9. **Not run:** the crate's whole suite on the code that was set aside. The module's tests, the two tools' tests, the stack chat tests and the replay ran (below).

## What changed

- **Nothing in the crate.** The head's `crates` tree is the order commit's, which is claude-367's.
- **The code is kept as [code-archived.patch](code-archived.patch),** 1,168 lines, SHA-256 `5d286ddfa1f76706a5e2d88714d5e2184a282083b5c9af9248493d77d419bfda`. Applied at `90bc67c84f7d6959584aee8116114dc11b621a66` it rebuilds the `crates` tree of `75c335c7578635f78f1d6fd576e763db9a485007`. It holds:
  - `pair_experts.rs`: `Weigh::Even` (claude-367's share, `(c << 20) / t`) and `Weigh::Sure` (`((c * m) << 20) / (t * t)`); an expert is sure at `t >= n` and `8 * m >= 7 * t`; a reply no count of a sure expert holds does not stand; the experts' answer is their top reply among those every sure expert holds; the refusal is void when the sure experts share no reply with a vote. 21 tests.
  - `place-asks experts-check`: `--weighs` and `--refusals` as two more sides of the grid, a `refused` column and verdict word.
  - `uor-chat`: `--pair-weigh` and `--pair-refuse`, named in the transcript's record.
- **The setting** ([settings.json](settings.json)): the pick is claude-367's (`pick_is_base`).
- **One new fact conversation** ([chat-facts-2.json](chat-facts-2.json)), committed with the setting in `dd9d65365b8f0d5acbc51bd2bb110b209d1f4f66` before it was served.
- No trained model, counted table or reader source is changed. No pod and no training.

## Measurements

All readings are held out: each row's own TRAIN entries are taken out of the table first. "Good" is claude-366's count: the reply ends and is the reference or a reply TRAIN gives the same turn. The models are the three saved ones (p-s1, p-s2, p-s3); the table is claude-367's (`21d458fc374a88cb8b367a4edcf0c524ba73562bb420e6e7ba11271a714a6ad0`).

### The grid and the pick

128 settings a file: weigh (even, weighed) × keep (1/32, 1/8, 1/4, 1/2) × floor (500, 750, 1000, 1500) × refusal (off, `n` of 2, 4, 8), cap off. Files: [grid23.tsv](grid23.tsv), [grid24.tsv](grid24.tsv); the pick script's output is [pick.txt](pick.txt).

| Setting | Good, seed 23 of 235 (p-s1, p-s2, p-s3) | Good, seed 24 of 239 | Sum | Good rows lost a reading | Left to the model | Replaced | Refused | Inside the guards |
|---|---|---|---|---|---|---|---|---|
| Experts off | 145, 144, 142 | 137, 135, 134 | 837 | | | | | |
| **claude-367's:** even, 1/32, 1500, off | 185, 182, 183 | 170, 171, 170 | 1,061 | 1, 1, 1, 4, 4, 4 | 172 | 347 | 0 | yes, the only one |
| The refusal alone: even, 1/32, 1500, on | 185, 183, 183 | 169, 169, 170 | 1,059 | 2, 2, 2, 6, 6, 6 | 274 | 358 | 17 | no (6 lost) |
| The weighing alone: weighed, 1/32, 1500, off | 179, 180, 177 | 167, 165, 163 | 1,031 | 1, 1, 1, 7, 7, 7 | 341 | 263 | 0 | no (7 lost) |
| Both: weighed, 1/32, 1500, on | 179, 180, 177 | 168, 166, 166 | 1,036 | 1, 1, 1, 7, 7, 7 | 410 | 265 | 2 | no |
| Best weighed: weighed, 1/8, 1500, on | 179, 180, 177 | 169, 166, 166 | 1,037 | 1, 1, 1, 7, 7, 7 | 435 | 266 | 1 | no |
| Best of all 128: even, 1/4, 1500, on | 187, 187, 186 | 173, 172, 172 | 1,077 | 2, 2, 2, 8, 8, 8 | 290 | 397 | 5 | no (8 lost) |
| Worst: weighed, 1/4, 1000, on | | | 876 | up to 41 | | | 9 | no |

The guards, from the order: on every reading at most 3 told-value rows and 2 never-told rows replaced, at most 4 good everyday rows lost. No setting replaces more than 1 told-value row or any never-told row; every setting but claude-367's loses more than 4 good rows on at least one reading.

### The bar

| | Reading |
|---|---|
| The pick | claude-367's own setting |
| Seed-25 panel | not written, by the order |
| **Bar** | **Does not add** |

### Seed 24 by what the reply needs, at keep 1/32 and floor 1500

Good replies of 114 (no word of the turn), 39 (a word of the turn), 51 (a fact about the word), 35 (work on the numbers). File: [why24.tsv](why24.tsv); its off rows are claude-367's reading of the same panel with the experts off (`local/claude/claude-367/read24/<model>.class.json`).

| Model | Arm | Good of 239 | No word | A word | A fact | Numbers | Do not end | Stands | Left to the model | Replaced | Refused |
|---|---|---|---|---|---|---|---|---|---|---|---|
| p-s1 | off | 137 | 104 | 9 | 16 | 8 | 30 | | | | |
| p-s1 | claude-367's | 170 | 101 | 24 | 36 | 9 | 10 | 146 | 31 | 58 | 0 |
| p-s1 | refusal | 169 | 100 | 25 | 36 | 8 | 10 | 128 | 45 | 62 | 5 |
| p-s1 | weighed | 167 | 97 | 24 | 37 | 9 | 17 | 127 | 58 | 50 | 0 |
| p-s1 | both | 168 | 97 | 25 | 37 | 9 | 17 | 118 | 67 | 50 | 0 |
| p-s2 | off | 135 | 104 | 8 | 15 | 8 | 32 | | | | |
| p-s2 | claude-367's | 171 | 102 | 24 | 36 | 9 | 8 | 144 | 29 | 62 | 0 |
| p-s2 | refusal | 169 | 101 | 24 | 36 | 8 | 9 | 127 | 43 | 65 | 4 |
| p-s2 | weighed | 165 | 97 | 23 | 36 | 9 | 15 | 127 | 58 | 50 | 0 |
| p-s2 | both | 166 | 97 | 24 | 36 | 9 | 15 | 118 | 67 | 50 | 0 |
| p-s3 | off | 134 | 104 | 9 | 13 | 8 | 18 | | | | |
| p-s3 | claude-367's | 170 | 103 | 23 | 35 | 9 | 10 | 139 | 32 | 64 | 0 |
| p-s3 | refusal | 170 | 102 | 24 | 36 | 8 | 10 | 122 | 46 | 67 | 4 |
| p-s3 | weighed | 163 | 97 | 22 | 35 | 9 | 14 | 124 | 62 | 49 | 0 |
| p-s3 | both | 166 | 97 | 24 | 36 | 9 | 14 | 113 | 71 | 51 | 2 |

How p-s1's verdicts move from claude-367's setting: under the weighing 17 replaced replies and 10 standing ones are left to the model and 9 standing ones are replaced; under the refusal 5 standing replies are refused, 13 standing ones and 1 replaced one are left to the model.

### The new fact conversation, thirteen turns

Good turns of 13, read by hand against the plain answer. Every reply, word for word, is in [chat-facts-2.replies.txt](chat-facts-2.replies.txt); the kit files' sums are in [chat-facts-2.sha256](chat-facts-2.sha256).

| Model | Experts off | claude-367's setting, held out | As served |
|---|---|---|---|
| p-s1 | 1 | 10 | 12 |
| p-s2 | 1 | 8 | 10 |
| p-s3 | 7 | 10 | 12 |

p-s1, word for word:

| Turn | Experts off | Held out | As served |
|---|---|---|---|
| Hello! | Hello! How can I help you today? | Hello! How can I help you today? | Hello! How can I help you today? |
| Tell me the capital of China. | It's slin. | The capital of China is Beijing. | The capital of China is Beijing. |
| Which day is after Tuesday? | The day after Tuesday is after after after (cut) | It's Wednesday. | The day after Tuesday is Wednesday. |
| What is the opposite of early? | The day after Tuesday is is the opposite of of of (cut) | The opposite of early is late. | The opposite of early is late. |
| What is the color of a banana? | The day after Tuesday is is the color of of of (cut) | It's yellow. | It's yellow. |
| What letter does dog start with? | the same reply (cut) | dog starts with the letter D. | dog starts with the letter D. |
| What sound does a cow make? | the same reply (cut) | A cow says moo. | A cow says moo. |
| How many legs has a bird got? | the same reply (cut) | It has two legs. | It has two legs. |
| How do you spell cat? | the same reply (cut) | tree is spellednesday. | cat is spelled c-a-t. |
| Which is bigger, 4 or 11? | the same reply (cut) | 11 is bigger. | 11 is bigger. |
| Can birds fly? | the same reply (cut) | No. | Yes. |
| What is 7 + 9? | the same reply (cut) | 7 plus 7 is 11. | 7 plus 7 is 11. |
| Thank you so much! | the same reply (cut) | You're welcome! | You're welcome! |

p-s2 held out differs from p-s1 on three turns: "The day after Tuesday is Wednesday." (stands), "A dog says woof." for the cow (stands, wrong) and "It has four legs." for the bird (stands, wrong); as served it keeps the last two. p-s3 held out is p-s1's, reply for reply.

### Cut replies in the recorded conversations

A reply is cut when the short-cycle rule stops it; it has no end mark. File: [cut-replies.tsv](cut-replies.tsv), over claude-367's fifteen conversations and this unit's nine.

| Conversations | With a cut reply | Turns after the first cut reply | Of those, cut again or the same reply as the one before |
|---|---|---|---|
| Experts off (9) | 4 | 35 | 35 |
| Experts on, held out or as served (15) | 2 | 8 | 0 |

## Checks run at the head

| Check | Result |
|---|---|
| The head's `crates` tree against the order commit's (`git diff 90bc67c84f7d6959584aee8116114dc11b621a66 HEAD -- crates/`) | empty |
| `git apply --check code-archived.patch` at the head; applied, the tree against `75c335c7578635f78f1d6fd576e763db9a485007`; then taken out again | applies; equal; clean |
| On the patch's code: the module's tests (`--lib pair_experts`), with `--features serving-wasm` and in the default build | 21 passed; 21 passed |
| The stack chat tests (`--lib stack_chat`), the table tool's (`--bin place-asks`), the chat program's (`--bin uor-chat`) | 24 passed; 11 passed; 13 passed |
| `cargo check -p uor-r4-training` | builds |
| `rustfmt --edition 2021 --check` on `pair_experts.rs` | clean |
| Negative gates on the vote and the refusal, each rule set aside in turn and restored: the weighed share; a sure expert refuses; sure at exactly 7/8; not sure under 7/8; under `n` nothing is refused; the floor reads the allowed top; void with no shared reply; void when the shared reply has no vote | each test fails, as it must (8 of 8) |
| Negative gates on the two tools: the refused count, verdict word and the line's two new columns; the grid passing the weighing and the refusal; a weighing that is neither refused; the chat's flags reaching the setting, needing the table, and named in the record and the transcript | each test fails, as it must (15 of 15) |
| The default setting: the patch's build replays claude-367's seed-24 panel, three models, held out and as served | all 12 files (replies and rows) byte for byte claude-367's |
| The pick script on claude-367's setting | its six readings are claude-367's report's (185, 182, 183; 170, 171, 170) |

**Review.** Found while testing, before the grid: when the only reply the sure experts shared had no vote, a reply could be "replaced" by itself and counted as refused. The agreed replies now count only replies with a vote, with a test and its gate. The report was read by a fresh reviewer (`delegate:reviewer`, no part in the work) against the order and the evidence files. It recomputed the pick, the headline sums, the refusal lines and the conversation counts from the committed files and found every number to match and the bar to follow from the order. It asked for changes of wording, all taken: the cut-reply finding was stated as "never recovers", which the hand run in item 6 contradicts, and was not marked as outside the bar (three places, now "in the four recorded conversations ... 35 of 35" and "found on the way, not under a bar"); "a fresh panel would read lower" was a guess (now "could"); the off rows of the by-class table were not in `why24.tsv` (added, with their source). Three checks it left were run in the main session: the patch applies at the order's commit and gives the `crates` tree of `75c335c75`; no banned word in the added lines; p-s3's conversation counts (7, 10, 12) match the word-for-word file.

Kit (`local/claude/claude-368/`): the table tool that read the grid, SHA-256 `422020cf732d1b5c21db26c48766daefd1c70345cc9ac0f15206d0a8c90b570f`, and the chat program that read the conversations, `f5d36c979b8eb5d7b5015ffe2d1247e67dd06901d1593680883d095dfd5ae53e`, both built from the patch's code. The six grid files' sums are in the kit.

## Skills used

None.

## Still open

- **A reply that does not end must not stay in the conversation.** Item 6 is the next unit: the chat keeps a cut reply out of its history (the turn is shown as cut and the next turn starts from the history before it), read on the recorded conversations with the experts off and on, and on a longer conversation. It needs no training.
- **A wrong reply that ends is read back as if it were told** ("It's slin." twice). That is the copy read working on the model's own words, and it needs its own reading.
- **Train the pair signal.** The count-and-check has given what it has: about +35 of 239, and this weighing (sureness as `m/t`) and this refusal add nothing on these two panels. Rows addressed by the turn's pairs and written by the output's error in the float trainer are the trained form; they need a GPU run with a prep sheet.
- **Number work** (26 of 35 wrong) needs a mechanism that works on the operands; no stored reply can.
- **The owner readings** claude-367's report lists (a table counted from TRAIN served in the product, §3.1; §3.6).

## Plain summary

The last unit added a table that checks each reply against the words of the user's turn. This unit tried two ways to make that table smarter: let a pair of words that always gets the same answer count for more, and let such a pair veto a reply it has never seen. Neither helped in the form tried here, on these two panels. The training text says the same fact in several wordings, so the pair that knows the fact looks unsure of itself and is turned down; the veto fires on a handful of rows and costs as many good replies as it saves. By the rule fixed before the numbers were read, the setting stays as it was, and the new code is kept as a patch and not merged.

What the model says is unchanged from the last unit: with the table on, a thirteen-turn conversation of fact questions is answered on 10 to 12 turns as served and 8 to 10 when the table has never seen the question's wording.

Reading those conversations with the table off showed something more important. In the four recorded conversations where one reply broke into a loop and was cut, every later turn is cut again or repeats the reply before it, 35 turns of 35. The broken reply stays in the conversation and the model copies it. In one run by hand a model did get past a second cut reply, so this is what the recordings show, not a law. The next unit keeps a cut reply out of the conversation.
