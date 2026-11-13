# REPORT: one-model-end-to-end, claude-361 (the trained key on the two tokens before the link word)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [the key retrain's prep sheet](../../../../harness/prep/one-model-end-to-end-key-two-three-2026-11-13.md) (FIT, 13 Nov). PR #3951.

The order asked whether the model's own trained read finds a told value more often when its place key sits on the two tokens before the link word and its question reads three rows. Three models were trained with that key ("k"), one model with today's key again as a pin check, on two pods. **The sheet's reading was not made (row 0), and what was read says the k models are worse:** in float, before any integer reader, they state the told value on 131, 143 and 157 of 562 questions against 252, 236 and 251 for the saved models ("p") of the same seeds. The saved p models stay the line's models. The key's code does not join main; it is kept in this folder as one patch.

## What did not work, first

1. **The sheet's reading was not made: row 0.** The check I-v compares each artifact's served reply with its trained model's reply on the first panel's 562 recall rows. Its first floor is 0.70 of all rows the same. k-s1 read 0.692 and k-s3 read 0.644; k-s2 read 0.711 and holds. By the sheet a k artifact under a floor is row 0: no M is read and rows 1 to 3 class nothing. No reply of k-s1 or k-s3 was read on a panel.
2. **The k models state fewer told values in float, on every seed.** This reading has no bar and is not in the sheet. The pod's own float replies on the first panel (each model's 192-token window, greedy), scored on the laptop by the whole-word scorer:

   | Seed | Told value stated, of 562: p | k | Recall replies exact: p | k | All replies exact, of 1,108: p | k |
   |---|---|---|---|---|---|---|
   | 2 | 252 | 131 | 83 | 50 | 187 | 156 |
   | 1 | 236 | 143 | 74 | 61 | 187 | 167 |
   | 3 | 251 | 157 | 80 | 59 | 224 | 165 |

   No integer reader takes part in these replies, so this loss is in what was trained. Whether the reader adds a loss of its own on top is negative 6.
3. **In training, the new key made about a third as many right finds.** Each long run's `place.jsonl` counts the place read's finds between log lines (`local/claude/claude-361/run/`). Over the last 2,048 steps:

   | Run | Right finds | Wrong finds | Share right | Tokens that arm | Mean pull |
   |---|---|---|---|---|---|
   | p-s2 (today's key, retrained) | 14,601 | 61,763 | 19% | 5 | 0.188 |
   | k-s2 | 4,191 | 4,524 | 48% | 22 | 0.287 |
   | k-s1 | 4,383 | 5,312 | 45% | 87 | 0.269 |
   | k-s3 | 5,432 | 8,247 | 40% | 27 | 0.281 |

   A find under the new key is right more often and happens far less. The mean pull stops near 0.28 where today's key reaches 0.19. Step 1's count had put the new key's ceiling at 482 to 530 of 562 places against 436 for today's: the trained question code did not come near its count.
4. **The one k model read on the panels is far under its p model.** k-s2 passed I-v and was read before k-s1 and k-s3 failed it. Told value stated: 154 of 578 on the second panel against 314 for p-s2, and 167 of 562 on the first against 316. Value questions' last word the same: 126 of 503 against 273. Other replies exact: 94 of 407 against 95. Replies that never end: 117 against 87. Never-told replies that say so: 0 of 71 on both. A value question answered "not told": 4 of 503 against 0 on the second panel, 3 of 509 against 4 on the first. These are not classed by rows 1 to 3.
5. **The run cannot say which change lost.** The key (with its link rule) and the three-row carry ride together, as the sheet said they would.
6. **The split by side that row 0 asks for is partly done.** The float reading (negative 2) puts the loss on the trained side. The trainer's find was not compared with the reader's on the same rows. One lead from the row check (`rows/rows-*.receipt.json`, `by_margin`): rows where the artifact's choice differs from the trained model's at a margin of 64,000 or more number 6, 3 and 8 for k-s2, k-s1 and k-s3, and none for p-s2.
7. **No rerun is scheduled; this lab's decision, logged for Casey, who can reverse it.** Row 0 says repair and rerun under a new root. The retrain of p-s2 at this pin exported to claude-358's artifact byte for byte, so a rerun of the k recipe would train the same three models and answer nothing new. The float reading puts most of the gap in what was trained. **A reader fault under the new key is not ruled out:** row 0's own check (I-v's first share on k-s1 and k-s3) and the large-margin rows of negative 6 point at one. Against one: the row check holds on all three k models, and on rows where the float reply is right the reader serves that reply on 50 of 50, 60 of 61 and 55 of 59. Row 0's split by side is left open (Still open). The configuration is not closed by a bar (CHARTER §4.1 needs a registered negative); it is set aside on negatives 2 and 3.
8. **The key's code does not join main.** It is [code-archived.patch](code-archived.patch), 1,398 lines, SHA-256 `b65812cbcfe5b168ab0681e6aaee157d1fb55606b0530c0d806ac8525e7ea3f8`. Applied at `b5c5804ab09ff495b14aa2096b78796459b02979` it rebuilds the `crates` tree of this branch's commit `57531f343a816b270760b83d9e49fae8babd7392` (tree `d13d7a16e0264c026e5f6b1156d19d9fc02e3d7b`), which is the run's pin `f914310fc4357160e59697d275f65f99897ee191` merged with claude-362. The six patches under `neg/` are the set-asides of the pod's test check; they apply at the pin, not at main.
9. **The prediction cannot be scored.** It was M between 0 and +45. M was not read; the one difference read (seed 2) is −160.
10. **Pods.** Lane a's first launch (try-1, a host in Ireland) never reached SSH and was taken down before anything shipped. Its second launch (try-2) was refused seven times for no free GPU before it started; the pod tool's suggestion to lease another lab's idle GPU was not taken. Lane a ran at $0.708/h and lane b at $1.042/h. Both pods are down; $2.92 in all by the two watchers' last lines ($1.19 and $1.73). The first launch's minutes at $0.708/h are not in that figure.
11. **The first panel's scorer exits 1 after writing its rows** (it marks 544 of 562 references right by its own rule). The rows were read; the cause was not looked into. The second panel's exits 0 with 578 of 578.
12. `cargo clippy` and `cargo fmt --check` were not run on the laptop; the branch now changes no source file.

## What changed

- On main: this report, the order, the sheet, the notebook row, the code as one patch and its six set-aside patches. No source file.
- On the branch before the set-aside (the patch): the place key's offsets and link rule in the trainer and the integer reader, the question carry under the place read, the place pull toward the new key, the artifact's carry section and link bits, and step 1's cuts in `place-asks`.

## Measurements

**Step 1, the count before any pod** (`local/claude/claude-361/cuts-1.table`, `cuts-2.table`). Places found right by a counted table cut to what a trained code can hold, on the first panel (of 562) and the second (of 578):

| Key and cut | First panel | Second panel |
|---|---|---|
| Two tokens before the link word, every pairing kept (claude-360) | 546 | 565 |
| The same key, at most two targets a token | 482 to 530 | 503 to 553 |
| Today's key (the two tokens nearest the place), the whole question, at most two targets a token | 444 to 488 | 447 to 503 |
| Today's key, the question's last two tokens (as today's read takes them), at most two targets a token | 222 to 257 | 225 to 270 |
| Today's read, its ceiling by claude-350's count | 436 | not counted |

The order let step 2 go ahead above 436 on the first panel. The fit with no cut wrote table v4's bytes (SHA-256 `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417`, `cuts-1/asks-uncut.json`).

**The instrument checks read on the laptop.**

| Check | Result |
|---|---|
| Pin check: p-s2 retrained at this pin | its artifact is claude-358's byte for byte (SHA-256 `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680`); V 314 of 578, offset 0 |
| I-o, I-w | the reader at this pin writes claude-359's reply files on the three p artifacts; the scorer marks 578 of 578 references right |
| I-h | k-s3: one carry section, 14 link bits; p-s2: neither (`fetch-k-s3.log`, `fetch-p-s2.log`) |
| I-f, rows that differ from the trained model's, of 40,719 | p-s2 364 (0.9910, the reference); k-s2 450 (0.9889); k-s1 528 (0.9870); k-s3 695 (0.9829). All within 0.010: holds |
| I-v, first share (floor 0.70) | p-s2 0.859; k-s2 0.711; **k-s1 0.692; k-s3 0.644** |
| I-v, second share (floor 0.93) | p-s2 82 of 83; k-s2 50 of 50; k-s1 60 of 61; k-s3 55 of 59 (0.932) |

**Readings with no bar:** negatives 2, 3 and 4 above.

## Checks run at the head

| Check | Result |
|---|---|
| `git diff origin/main -- crates/` | empty: the branch changes no source file |
| The patch applied at `b5c5804ab09ff495b14aa2096b78796459b02979` in a scratch index | `crates` tree `d13d7a16e0264c026e5f6b1156d19d9fc02e3d7b`, the tree of `57531f343a816b270760b83d9e49fae8babd7392` |
| On each pod before training (I-g, inside the `pre` stage) | `pre` ended job 0 and seal 0 on both pods (`watch-try-2.log`, `watch-try-3.log`); the stage runs the nine named tests and the six set-asides and fails on any of them. Its sealed log is in the store and was not read again for this report |
| Tests on the laptop at the head | none run: no source file differs from main |

Order's "done when": 1, step 1's counts and the v4 bytes, above. 2, the key with its tests is in the patch, not on main. 3, replies by kind are given for the one k model the sheet let be read; for k-s1 and k-s3 the sheet's row 0 forbids the read.

**Review, one, fresh, 13 Nov,** at `336090d09efc72420c2c8694f38fd634baff7a59`: given this report, the sheet's rows and checks, the order's "done when" and the kit's files, and no plan. It recounted the I-v lines, negative 2's stated and recall-exact columns, the training logs' last lines, the row check with its margin rows, k-s2's served reading and the patch claim, and found each as written. It asked for changes of wording, all made: the report no longer says that nothing points at the reader (negatives 2 and 7, the notebook row, the summary); the summary says finds in training, not questions; negative 4 gives the false "not told" count. It did not check the all-replies column, three of negative 4's figures, the prices or step 1's tables. The main session recounted the all-replies column from the float reply files (187, 187, 224 and the retrain's 187; 156, 167, 165) and read the others from the logs named.

## Skills used

None.

## Still open

- Why the trained question code stops short of its key under the new configuration (mean pull 0.28), and which of the two changes does it. A run that could say would train the key with the carry at two rows, or the carry alone, which today's key refuses in code.
- The trainer's find against the reader's on the same rows (row 0's split), starting from the rows that differ at a large margin.
- Whether a count-fitted list of link tokens may be served in a product (CHARTER §3.1) is the owner's question; nothing here serves one.
- The gaps the conversation still shows are worked from the p models: a question never told that is answered with a value, a value stated bare, and a small-talk turn that picks up a told word.

## Plain summary

We trained three models with a different way of finding where a told fact sits in the conversation. They came out worse: asked for a fact they were told, they state it on about 130 to 160 of 562 questions, where the models we already have state it on about 240 to 250. The training log shows why in part: during training the new way makes about a third as many right finds, though a find it does make is right more often. We have not ruled out that the program that runs the model on a laptop also loses something of its own with the new way. Our own pre-set check stopped the planned comparison before it was read, so this is reported as what we saw, not as a passed or failed bar. We keep the models we had, keep the new code as a patch in case a different setting is tried, and spent $2.92.
