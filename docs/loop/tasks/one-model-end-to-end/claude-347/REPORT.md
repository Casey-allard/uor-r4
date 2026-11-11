# REPORT: one-model-end-to-end, claude-347

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3924.

Five laptop readings on the replicated arm r model, asked for by an outside review. No pod, no training, no bar. Every claim below is [claude]'s.

| Reading | State |
|---|---|
| 1. Float against integer, row by row | done, turn memory on and off |
| 2. Does the model pick the fact itself | done, in a changed form (ORDER, amendment 2) |
| 3. General turns in wording TRAIN does not hold | done |
| 4. What the model holds, in full | done |
| 5. The BPB scorer's floor | done |

## What did not work, first

- **Reading 3: neither arm answers more than one of ten general questions in wording TRAIN does not hold.** Each of the ten general questions was reworded and checked absent from TRAIN's token stream. Asked one at a time in a fresh conversation, arm r answers 0, 0 and 0 of 10 and arm rt 1, 0 and 1 of 10 (seeds 1, 2, 3); the one right reply is "No." to "Can a cat bark?". In TRAIN's wording the same programs answer 4, 4 and 5 (arm r) and 9, 10 and 9 (arm rt). The usual reply to a reworded question is the question said back.
- **Reading 3: in the 18-turn script the reworded questions take recall down too.** With the four facts stated and the sieve writing the memory line, recall is 0 of 4 at every seed for arm r and 1 of 4 at every seed for arm rt, against 3, 4, 4 and 3, 3, 3 in the same script with TRAIN's wording. The second reworded question draws "2 plus 2 is 2 is 2 is", and arm r then gives that same reply to all twelve turns after it, recall questions included.
- **Reading 2: with the sieve's line removed, neither arm answers any recall question.** The same 18-turn script with the sieve off: recall is 0 of 4 at every seed on both arms, against 3, 4, 4 and 3, 3, 3 with it on. So in this script recall depends on the rule's line. The reading does not separate picking the fact from reading a turn that far back. The general turns do not move (3, 2, 2 and 9, 10, 9 either way).
- **Reading 2 was not run as the ORDER first described it.** The state-distance count was withdrawn before any run (ORDER, amendment 2): the laptop files carry no fact labels, and a miss on that count would have closed nothing.
- **Reading 1's first number was an instrument reading.** `uor-chat --prompt-ids` steps raw token ids and never tells the session where the reply begins (`mark_reply_start`, `crates/uor-r4-integer/src/stack/session.rs`); only the text chat does. So on that path the integer artifact answers with its turn memory off, and against the float replies scored with the memory on only 678, 691 and 644 of 1,141 rows were equal. The comparison below was made with a reader written for it, `raw-id-replies`, which marks the reply start as the text chat does.
- **The float model and its integer export still differ on some rows.** Turn memory on for both: 68, 46 and 31 of 1,141 rows (seeds 1, 2, 3). Turn memory off for both: 160, 119 and 93, of which 146, 94 and 93 are rows without a memory line.
- **The samples do not name which rows are recall rows.** `reference_named` is 0 on every row. The split used here is read from the prompt itself: a row is a line row when its prompt holds the memory line's opening ids (`53,91,353`). 641 rows are line rows and 500 are not, at every seed.
- **Reading 4 cannot give a count of learned values for the trunk.** The trunk's tables are stored packed (`*.fbs`), so the artifact gives stored entries and bytes, not learned values.
- **The item's claim was refused.** `uor-harness loop claim` answered that claude-346's worktree holds the item; the ORDER records it.
- **The first queue attempt was refused.** At `2c57f87230a973dec1610d925e2b868b9dbce7fe` `loop queue` found two faults. This REPORT lacked three required headings. And the float scan (D32) counted two sites in the reader: each was a tuple index, `.0`, that the formatter had put at the start of a line, which the scan reads as a float literal. The reader holds no float. It now takes the pair apart by name, and the scan counts none.

## What held

- **The integer export is right as often as the float model, within 2 rows of 1,141 with the memory on.** Turn memory on for both, replies that equal the reference word for word: float 602, 606, 612; integer 601, 607, 614. Right in float only: 5, 0, 1. Right in the integer only: 4, 1, 3. With the memory off: float 537, 503, 498; integer 532, 503, 497.
- **With the memory on, the two say the same thing on 1,073, 1,095 and 1,110 of 1,141 rows.** On the 641 line rows: 631, 631 and 639.
- **The reader's control held on every row.** With no reply mark its replies equal `uor-chat --prompt-ids` on 1,141 of 1,141 rows at each seed.
- **No row failed to run.** 1,141 rows a seed on each path, no error line, empty error files.

## What changed

- **The reader:** `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`, a new instrument program behind a new `integer-reader` feature in the instrument's `Cargo.toml`. No other program's build changes, and `Cargo.lock` is unchanged.
- **In the repository:** the ORDER with its two amendments, this REPORT, and five rows in the float-training notebook.
- **Not changed:** any model, artifact, training code, serving code, sheet or bar.
- **On the laptop, not committed:** the evidence under `local/claude/claude-347/`.

## Measurements

### Reading 1: float against integer, the same prompt, row by row

Each of the 1,141 panel prompts in arm r's sealed float score went through the exported artifact: `uor-chat-bd2532281 --stack r-s<seed>-c8256.uorlut --tokenizer tokenizer.json --prompt-ids <the row's prompt> --max-tokens 48`, greedy `serve_step`. The two replies are compared up to and including the first end-of-reply id; the float score keeps 48 ids and runs on past it.

**Turn memory on for both**, with the reply mark set as the text chat sets it. Float: `samples-p-greedy-memory.json`. Integer: `raw-id-replies --mark-reply 1`, built from this branch ([source](../../../../labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs)), which steps the same ids through `serve_step` and marks the reply start before the prompt's last id, as `StackChat::feed` does.

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Rows | 1,141 | 1,141 | 1,141 |
| Reader control: unmarked replies equal `uor-chat --prompt-ids` | 1,141 | 1,141 | 1,141 |
| Replies equal | 1,073 | 1,095 | 1,110 |
| Replies different | 68 | 46 | 31 |
| First difference at reply token 0 | 2 | 2 | 1 |
| First difference at tokens 1 to 5 | 27 | 22 | 18 |
| First difference at token 6 or later | 39 | 22 | 12 |
| Float reply equals the reference | 602 | 606 | 612 |
| Integer reply equals the reference | 601 | 607 | 614 |
| Right in float only | 5 | 0 | 1 |
| Right in the integer only | 4 | 1 | 3 |
| Line rows (641), replies equal | 631 | 631 | 639 |
| Line rows, float equals the reference | 487 | 492 | 503 |
| Line rows, integer equals the reference | 487 | 492 | 503 |
| Rows without a line (500), replies equal | 442 | 464 | 471 |
| Without a line, float equals the reference | 115 | 114 | 109 |
| Without a line, integer equals the reference | 114 | 115 | 111 |

**Is it like for like?**

- The float side's "memory" is the checkpoint's turn memory: `score_rows` (`crates/uor-r4-float-train/src/in_space_long.rs`) writes the greedy samples once with the turn memory as loaded and once with it taken out (`set_turn_memory`). The exported artifact carries that memory as its opening and reply tables, and the reply mark is what switches them on in the integer session.
- No option was passed on any path: no `--copy-rule`, so each side reads under the artifact's own rule, and no `--content-readout`. The artifacts are SpiralCore/FBS stacks.
- The reader has no short-cycle stop and no reply cap below 48; the float score has neither. Every float reply is 48 ids long, so a reply with no end-of-reply id is compared on all 48.
- Not checked in code: that the float scorer's greedy pick and the integer `serve_step` pick are the same read. The row agreement is the evidence for it.
- The reader is built from this branch's library, not from arm r's pin. Its unmarked control against the pinned chat program is exact; its marked path has no second program to be checked against on raw ids.

**Turn memory off for both.** Float: `samples-p-greedy-no-memory.json`. Integer: `uor-chat-bd2532281 --prompt-ids`.

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Rows | 1,141 | 1,141 | 1,141 |
| Replies equal | 981 | 1,022 | 1,048 |
| Replies different | 160 | 119 | 93 |
| First difference at reply token 0 | 70 | 51 | 32 |
| First difference at tokens 1 to 5 | 53 | 47 | 40 |
| First difference at token 6 or later | 37 | 21 | 21 |
| Float reply equals the reference | 537 | 503 | 498 |
| Integer reply equals the reference | 532 | 503 | 497 |
| Right in float only | 14 | 2 | 3 |
| Right in the integer only | 9 | 2 | 2 |

Split by whether the prompt holds a memory line:

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Line rows | 641 | 641 | 641 |
| Line rows, replies equal | 627 | 616 | 641 |
| Line rows, float equals the reference | 495 | 479 | 490 |
| Line rows, integer equals the reference | 490 | 479 | 490 |
| Rows without a line | 500 | 500 | 500 |
| Without a line, replies equal | 354 | 406 | 407 |
| Without a line, float equals the reference | 42 | 24 | 8 |
| Without a line, integer equals the reference | 42 | 24 | 7 |

The instrument reading, kept so it is not taken for a model reading: against the float replies scored with the memory on (`samples-p-greedy-memory.json`), 678, 691 and 644 rows are equal, and the integer reply equals the memory-off float reply but not the memory-on one on 307, 332 and 404 rows. The reverse case is 4, 1 and 0 rows.

**Instrument or model?** The 678, 691, 644 gap was the instrument: the chat program's raw-id path does not switch the turn memory on. The differences that remain once both sides run the same way are between the float model and its export. Most fall where neither side is right: 59, 45 and 27 of the 68, 46 and 31 with the memory on, and 137, 115 and 88 of the 160, 119 and 93 with it off. With the memory on the export loses 5, 0 and 1 right replies and gains 4, 1 and 3, a net of −1, +1 and +2. With it off it loses 14, 2 and 3 and gains 9, 2 and 2, a net of −5, 0 and −1.

The chat program printed a median of 1,009, 1,015 and 1,010 generated tokens per second a row. These are short prompts (17 to 192 tokens), each in a fresh process; it is not a reading at context 8,256.

### Reading 2: does the model pick the fact itself?

Read from what the model says (the ORDER's second amendment gives the reason the state count was withdrawn). The registered 18-turn script, `conv-4.txt`, with its first line `/memory auto` removed: the four facts are stated, nothing writes a memory line, and the four recall questions are asked as before.

| | Arm r, seeds 1, 2, 3 | Arm rt, seeds 1, 2, 3 |
|---|---|---|
| Sieve on: recall of 4 | 3, 4, 4 | 3, 3, 3 |
| Sieve off: recall of 4 | 0, 0, 0 | 0, 0, 0 |
| Sieve on: general of 10 | 3, 2, 2 | 9, 10, 9 |
| Sieve off: general of 10 | 3, 2, 2 | 9, 10, 9 |

Seed 2 with the sieve off, word for word:

| Asked | Arm r | Arm rt |
|---|---|---|
| What is my friend's name? | Okay, you grew up in Lisbon. | The opposite of hot is my friend's name? |
| What is my dog called? | It's name? | The opposite of big is my dog called? |
| Where did I grow up? | Where did I grow up? | Where did I grow up? |
| What is my lucky number? | The opposite of | I grow up? |

**Instrument or model?** Model, for this configuration. One line of the script differs, and the sieve-on column is the registered reading. With the sieve's line removed, neither arm answers any recall question at any seed, so in this script recall depends on the rule's line. The reading does not separate selection from reading earlier turns: without the line the fact is four or more turns back, and a model that could pick the fact but not read that far would score the same. The general turns do not move, so they do not depend on the line.

### Reading 3: general turns in wording TRAIN does not hold

**The check that the wording is new.** Each question was encoded as the chat program writes a turn (`User: <question>`, a newline, `Assistant:`) with the tokenizer's `token-counts` example, and the id run was searched for in TRAIN's 8,127,216 ids (`mwr-train/train/tokens.u16`, 60,000 conversations). The ten original questions are the positive control.

| Original question | Times in TRAIN | Reworded | Times in TRAIN |
|---|---|---|---|
| What is the opposite of hot? | 31 | Hot has an opposite. What is it? | 0 |
| What is 2 plus 3? | 9 | Add 2 and 3 for me. | 0 |
| What color is a lemon? | 162 | A lemon is which color? | 0 |
| Tell me a word that means the opposite of big. | 30 | Which word is the opposite of big? | 0 |
| How many legs does a spider have? | 221 | A spider has how many legs? | 0 |
| Give me the opposite of fast. | 46 | The opposite of fast is what? | 0 |
| What is 4 plus 4? | 10 | How much is 4 plus 4? | 0 |
| What color is milk? | 185 | Milk is what color? | 0 |
| What is the opposite of hard? | 33 | Name the opposite of hard. | 0 |
| Do cats bark? | 146 | Can a cat bark? | 0 |

A reply counts as right when it holds the expected word (cold, 5, yellow, small, eight or 8, slow, eight or 8, white, soft, no; for recall Maya, Rex, Lisbon, 47). Arm r runs in `uor-chat-bd2532281`, arm rt in the chat program at its own pin, each on its own three artifacts.

**One question a conversation** (`--say`, nothing before it):

| | Arm r, seeds 1, 2, 3 | Arm rt, seeds 1, 2, 3 |
|---|---|---|
| TRAIN's wording, right of 10 | 4, 4, 5 | 9, 10, 9 |
| New wording, right of 10 | 0, 0, 0 | 1, 0, 1 |

**The 18-turn script** (`/memory auto`, four facts, then ten general and four recall questions mixed; `conv-4.txt` is the registered script, `conv-5.txt` the same with the ten general lines reworded):

| | Arm r, seeds 1, 2, 3 | Arm rt, seeds 1, 2, 3 |
|---|---|---|
| TRAIN's wording: general of 10 | 3, 2, 2 | 9, 10, 9 |
| TRAIN's wording: recall of 4 | 3, 4, 4 | 3, 3, 3 |
| New wording: general of 10 | 0, 0, 0 | 1, 0, 0 |
| New wording: recall of 4 | 0, 0, 0 | 1, 1, 1 |

The TRAIN-wording rows repeat the earlier readings of arm rt (9, 10, 9 general; recall 3 of 4 with "47" said as "40"), so the scripts and programs are the ones used before.

**Word for word, seed 2, one question a conversation, new wording:**

| Asked | Arm r | Arm rt |
|---|---|---|
| Hot has an opposite. What is it? | Hot has an opposite. | Hot has an opposite. |
| Add 2 and 3 for me. | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| A lemon is which color? | A lemon is which color? A | A lemon is which color? |
| Which word is the opposite of big? | Which word is the opposite of of big? | Which word is the opposite of big? |
| A spider has how many legs? | A spider has how many legs? | A spider has how many legs? |
| The opposite of fast is what? | The opposite of fast is what? | The opposite of fast is what? |
| How much is 4 plus 4? | Noted. | I'm a helpful assistant: I'm a helpful assistant: (to the cap) |
| Milk is what color? | Milk is what color? | Milk is what color? |
| Name the opposite of hard. | Name the opposite of hard. | Name the opposite of hard. |
| Can a cat bark? | Can a cat bark? | Can a cat bark? |

**Word for word, seed 2, the 18-turn script with the new wording** (turns 5 to 18; the first four are the facts, acknowledged as before, with "My dog is called Rex." answered "Got it, you grew up in Lisbon" in both wordings):

| Turn | Asked | Arm r | Arm rt |
|---|---|---|---|
| 5 | Hot has an opposite. What is it? | You haven't told me that yet. | Hot has an opposite. |
| 6 | Add 2 and 3 for me. | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 7 | A lemon is which color? | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 8 | What is my friend's name? (line: It's Maya.) | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 9 | Which word is the opposite of big? | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 10 | A spider has how many legs? | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 11 | What is my dog called? (line: It's Rex.) | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 12 | The opposite of fast is what? | 2 plus 2 is 2 is 2 is | 2 plus 2 is 2 is 2 is |
| 13 | How much is 4 plus 4? | 2 plus 2 is 2 is 2 is | 4 plus 4 is 4. |
| 14 | Where did I grow up? | 2 plus 2 is 2 is 2 is | It's Lisbon. |
| 15 | Milk is what color? | 2 plus 2 is 2 is 2 is | Milk is what color? |
| 16 | Name the opposite of hard. | 2 plus 2 is 2 is 2 is | Name the opposite of hard. |
| 17 | What is my lucky number? | 2 plus 2 is 2 is 2 is | It's 40. |
| 18 | Can a cat bark? | 2 plus 2 is 2 is 2 is | Can a cat bark? |

At seeds 1 and 3 the repeat falls on the same turns: arm r gives "2 plus 2 is 2 is 2 is" from turn 6 to turn 18, and arm rt from turn 6 to turn 12. The other replies differ a little by seed; arm rt answers "No." at turn 18 at seed 1 only.

**Instrument or model?** Model. The programs, artifacts, facts and recall questions are the ones behind the earlier readings, which this run repeats; only the ten general lines changed. Two things are read apart. General answers arrive almost only in TRAIN's wording, even for a question asked alone. This reading does not show whether the turn memory or the trunk supplies them; claude-345 read arm rt's general count at 0 of 10 with its turn memory taken out (seed 2). And once the repeating reply is in the history, arm r gives it on every later turn, including recall turns where the line holds the answer (turns 8 and 11 above). Which part of the model repeats it was not traced.

### Reading 4: what the model holds

Identical at seeds 1, 2 and 3 of each arm.

Arm r (`r-s<seed>-c8256.uorlut`, 14,186,496 bytes, `uor-r4.lut-stack/7`, a header of 8,777 bytes, 90 tables):

| Section | Tables | Stored entries | Bytes | Share of table bytes |
|---|---|---|---|---|
| Turn memory | 6 | 3,007,301 | 12,029,204 | 84.8% |
| Token table (4,096 × 256, 16-bit) | 1 | 1,048,576 | 2,097,152 | 14.8% |
| Trunk, 6 layers (packed) | 72 | 14,640 | 40,128 | 0.28% |
| Copy reads | 10 | 2,568 | 10,272 | 0.07% |
| Final norm | 1 | 256 | 512 | under 0.01% |
| All tables | 90 | 4,073,341 | 14,177,268 | |

An entry is 16 bits in the token table and the final norm, 32 bits in the turn memory and the copy reads, and packed in the trunk. The file's other 451 bytes (14,186,496 − 8,777 − 14,177,268) were not attributed.

The turn memory holds 7,414 opening keys and 140,003 reply keys of 20 ids each; the reply keys alone are 2,800,060 ids, 11,200,240 bytes, 79.0% of the table bytes.

Arm rt (`rt-s<seed>-c8256.uorlut`, 55,424,640 bytes, `uor-r4.lut-stack/9`, chain tail 9): the turn memory is 13,316,831 entries, 53,267,324 bytes, 96.1% of the table bytes, with 7,414 opening keys and 630,933 reply keys. The other four sections are the same as arm r's.

A session snapshot was 48,069 bytes at position 198 (one row). It was not measured at the full context.

By bytes the turn memory (keys taken from TRAIN, and their index) is 84.8% of arm r's table bytes and 96.1% of arm rt's. The figure quoted before this unit, 1,112,576 values, leaves the turn memory out.

### Reading 5: the BPB scorer's floor

The M4 sheet ([m4-lawful-likelihood-bpb](../../../../harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md), §6) scores a target by rank: the predicted token gets potential 9 × 2^24, the next seven ranked tokens get 7 × 2^24 down to 1 × 2^24, every token then gets 1 more, and the 4,096 potentials are normalized. The stream holds 2.837427 bytes a token.

| Quantity | Value |
|---|---|
| Sum of the potentials | 37 × 2^24 + 4,096 = 620,761,088 |
| Cost when the target is the predicted token | 2.0395 bits |
| Cost when the target is outside the top eight | 29.2095 bits |
| The 0.90 BPB target, in bits a token | 2.5537 |
| Lowest BPB this scorer can report (every target predicted) | 0.7188 |
| Largest share of targets outside the top eight that still reaches 0.90, every other target predicted | 1.892% |

Under this mapping the floor is 0.72 BPB, and 0.90 needs at most 1.892% of targets outside the top eight, even with every other target ranked first: one in 53 still reaches it, one in 52 does not. The sheet's other branch scores Q24 candidate scores; it was not computed here. The sheet and the M4 bar are outside this unit's scope: this is a reading, proposed as an addendum to the sheet.

## Evidence

Laptop files under `local/claude/claude-347/` (not committed; sums below are SHA-256).

| What | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Float replies, memory off | `2cb6a3ddb4b64009e56698499982d9f1d4ebc96288d26fc53bc926877cc119ec` | `745456abf55b0f4fe853278361c2e9192fecc9fb8dfd5e8567d99031be99e621` | `4b33e9d1454a8541aaaaad06f1577739393f661d43c9823411a29c96ae5ce1ae` |
| Float replies, memory on | `1fa6a8108cd74d20bb24e3a67de8029c1e17369b9a683dbf6cd3b0b15d77aee3` | `53d966dcfd63ff71e4b2d30f9af0fdbdc0607ca6c6d96c7d09aa4fb37e99a2e2` | `2f09f3526eaa285a65b89d0fc3a83c6dc163145769d5d22599febcbc3609244c` |
| Integer artifact | `f052d8069ed2e9ddbb87fd2feaace2ef2bd45b07580015f49fb4deefb87362c5` | `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0` | `d229cf1b32c9eb7deed6698be98a3370ce4f18108063126f69b70f5aebcd309a` |
| Integer replies (`fi/r-s<seed>/integer.jsonl`) | `a82422cf4eb222d5854d299e0046209b03664074e7c6a13f0f063def9e2ecaae` | `5952639cd7d9f3189fcccdf725ea45c423ee65715ff66d50b38c64933ea518ad` | `2608ae69377fada8fb636071c3fc41b3b396b81c309839935e53a97b48228776` |

Reader `fi2/raw-id-replies`: `07e742ef46da9cd38ba754b7020ad88b22ea1809867fc889503d284759ba10fb`. Its marked replies, `fi2/r-s<seed>-mark1.jsonl`: `f3a9c74bdabdc9423570079b09e891cb8718a946a98f20e0956fb94ed457ca16`, `62872ee5d782eb97a46f8a01798671a3a90a92f6fb959c8c5f3bb63721515712`, `95c47ef132a59e6b336d49257131e0d8ab7718504be4155e5f006324b70276ad`.

Chat program `uor-chat-bd2532281`: `69bc535af70325fd8362818a1f07f549e91cdbcb28d0457a79aa56c1ab58344c`. Tokenizer: `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. The artifact sums equal the ones in [claude-345's REPORT](../claude-345/REPORT.md). The float files are the pods' sealed scores, `score/r-line/` of each run.

Reading 3: `r3/conv-4.txt` `33d19426dc0077e300af4a4e093747c484d27da45c4cf05a3da61f46ea8989e4`, `r3/conv-5.txt` `3e887941bd8f3853b76562de2cd48c6e7bd909efc9355c1c65d4d1f943a3431e`; outputs `r3/conv-<4|5>-<r|rt>-s<seed>.out` and `r3/say-<r|rt>-s<seed>.jsonl`. TRAIN is the store whose manifest is `3d87db7ec2313d7d684f557920edb4fa3c0b7b18a543782707d50a6e258b379d` (60,000 conversations, 8,127,216 tokens). The ids came from `cargo build --release -p uor-r4-tokenizer --example token-counts` at this branch.

Reading 2: `r2/conv-4-nosieve.txt` `774998237dd21fdb3a6a2ba7b033975ac5c39049b9e78652b583a0c7544dcd95`; replies `r2/nosieve-<r|rt>-s<seed>.replies`.

## Checks run at the head

- **Fresh-context review, four reviewers in this session**, each given the ORDER, the diff and the evidence paths, never the plan.
  - The reader's code at `eafb16107d13b4430d2bd535109c69d8e57c89c9`: it steps ids as `StackChat::feed` does, the counting is sound, and no §3 rule is touched.
  - A recount of the tables from the laptop files: all match.
  - The wording against the tables at `2d8cac53bf30121fab837d46b4d9adf6eb4b396f`: CHANGES, seven items, no wrong number. All seven are taken in `0882f81dcdfc1db1e10d04492c0a99cd0ec6076b`.
  - Confirmation at `0882f81dcdfc1db1e10d04492c0a99cd0ec6076b`: PASS. Its two optional phrases are taken in `2c57f87230a973dec1610d925e2b868b9dbce7fe`.
- **The float-scan fix, made after that review.** Two statements of the reader changed: the pair `serve_step` returns is taken apart by name where a tuple index was. The reader was rebuilt (`c84e43b06dc7d63bb5cccbd527d405ee236d3bbb06f389fd5e2d3bdc185bf406`) and run again at seed 1 with the reply marked: 1,141 rows, no error line, and its output is byte for byte the earlier reader's (`f3a9c74bdabdc9423570079b09e891cb8718a946a98f20e0956fb94ed457ca16`).
- **Ratchets on the fixed tree:** `uor-harness ratchet d23`, `d25` and `d32` each answer OK with nothing added (d32: 31 sites in 1 file, the baseline).
- **Format:** `rustfmt --check --edition 2021` on the reader is clean.
- **`uor-harness check`:** 420 rules, 0 errors, on the tree that became `2d8cac53bf30121fab837d46b4d9adf6eb4b396f`. It was not run to its end after the later commits; CI's `loop gates` job runs it at the head.
- **CI at `2c57f87230a973dec1610d925e2b868b9dbce7fe`:** every check that ran passed (`loop policy`, `loop gates / tests`, `docs archive and links`, `fmt / clippy / tests / no_std / κ`, `cargo audit`, `fuzz smoke`, `wasm-pack build`, `Gate C trend alarm`); the others were skipped.
- **`loop queue` at `2c57f87230a973dec1610d925e2b868b9dbce7fe`:** refused, for this REPORT's headings and the two float-scan sites (first section).

## Skills used

None beyond the project loop. The fresh-context reviewers were subagents of this session.

## Still open

- The state-level form of reading 2, which belongs to the learned selector's own sheet if that proposal is taken up.
- `uor-chat --prompt-ids` cannot run a model's turn memory. Whether it should take a reply mark is a serving change for the next unit.

## Plain summary

The model answers a general question almost only in the wording it was trained on. Reworded, the ten questions get 0 right answers on arm r and 1, 0, 1 on arm rt, and the usual reply is the question said back. In the 18-turn script with all ten reworded, the model repeats one bad reply from the second reworded question on (arm r to the end, arm rt for seven turns), and recall of stated facts falls from 3 or 4 of 4 to 0 (arm r) and 1 (arm rt).

Recall in that script depends on the line the chat program's rule writes: with the rule off, recall is 0 of 4 at every seed on both arms. This does not tell picking the fact apart from reading that far back.

With the turn memory on for both, the export to integers changes the count of right replies by −1, +1 and +2 of 1,141: the integer model says exactly what the float model says on 94% to 97% of the panel's prompts and is right about as often (601, 607, 614 against 602, 606, 612). By bytes the artifact is mostly turn memory, keys taken from TRAIN and their index: 84.8% for arm r, 96.1% for arm rt. The M4 sheet's rank branch cannot report below 0.72 BPB and cannot reach 0.90 once more than 1.892% of targets fall outside its top eight.
