# REPORT: one-model-end-to-end, claude-353

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3943.

A repair of the laptop reader, found by the code read the owner approved on 12 Nov ("Yes, start") and delivered on their word ("Yes, deliver"). Laptop only: no pod, no training, no float pass. The four models are the sealed stage 3 models of [claude-352](../claude-352/REPORT.md); their artifact files are not changed. Only the program that reads them is.

**What the models say first.** The first seven recall conversations of the panel, in panel order, right or wrong, at the full window, read by the old reader and by the repaired one. "Place read" is p-s2's integer artifact, "Control" is n-s2's.

| Asked (the stated value) | Place read, before | Place read, after | Control, before | Control, after |
|---|---|---|---|---|
| "Where did I grow up?" (Jaistchukfield) | You grew up in Ja is Drekmen | You grew up in Jaistantelly ba know. | You grew up in. You haven't told me yet. | You grew up in Jaistchukmen. |
| "Do you remember what kadtaistfit was?" (446) | That's 44. | That's 446. | That's great news about the dog | That's great news about the dog |
| "What was fotpimbrud again?" (waspoumbroud) | It's boof. | It's boof. | It's bed | It's bed |
| "Who is my friend?" (Tikgeer) | Your friend's name is Tiktplplik. | Your friend's name is Tikgeer. | Your friend's name is is is iss. | Your friend's name is Vandhor. |
| "Do you remember what trirchest was?" (19, stated eight turns earlier) | That's 19 | That's 19. | That'sle feel like liker from feel help help help help with. | That's great news about the guitar! |
| "What color do I love?" (green) | You love? I I'll remember that. | You love? | You love with withinginging you you you … | You love? |
| "Tell me what woutdai was." (93) | It's 93. | It's 93. | It'se for | It'se for |

After the repair the place read says the stated value in four of these seven and the control in none.

**Trained model against laptop copy, word for word** (the 192 window, where the pod saved the trained models' replies). These are the first two rows of each model where the trained model was right, the old reader wrong and the repaired reader right (33 such rows for the place read, 65 for the control), so they show the fault, not a typical reply:

| Model | Stated value | Trained model | Laptop copy, before | Laptop copy, after |
|---|---|---|---|---|
| Place read | 446 | That's 446. | That's 44. | That's 446. |
| Place read | Tikgeer | Your friend's name is Tikgeer. | Your friend's name is Tiktplplik. | Your friend's name is Tikgeer. |
| Control | numjast | It is numjast. | It is is is issssssssait. | It is numjast. |
| Control | Vindbraik | Your friend is Vindbraik. | Your friend is know. You haven't told me yet. | Your friend is Vindbraik. |

| Gate | Reading | Status |
|---|---|---|
| G1, the new test both ways | passes at the head; with the reader's change set aside it compiles and fails | met |
| G2, the integer crate's stack tests | 540 passed, 0 failed, 1 ignored | met |
| G3, the counts for all four models, before and after | below | met |
| G4, the correction to claude-352's laptop numbers | below, first section | met |

## What did not work, first

- **The laptop copies still do not say what their trained models say on about a quarter of the rows.** After the repair the replies differ on 275 (p-s2), 197 (p-s1), 245 (p-s3) and 300 (n-s2) of 1,108 rows at 192: 0.25, 0.18, 0.22 and 0.27, against the sheet's bar of 0.10. Arm r's merged export, which has no ask heads and which this change does not touch, differs on 232 (claude-352's count). So a second cause is left, common to every model, and this unit does not find it.
- **The registered reading of claude-352 stays NOT_RUN.** Its one registered run is spent. The counts here describe the same sealed models under a repaired reader, and no row of the sheet is read from them.
- **Correction to claude-352's laptop numbers.** claude-352 reported the control's integer artifact as saying the stated value in 0 of 562 replies and called it a broken export. The export was not broken. The reader was: with it repaired, the same artifact file says the stated value in 76 of 562 at the full window and 69 at 192, where its trained model says 79. The place read's laptop numbers were low for the same reason: 207, 236 and 160 (seeds 2, 1, 3) are 238, 269 and 223 with the repaired reader. So on laptop copies the place read stands at 238 against the control's 76 at seed 2, not 207 against 0. claude-352's REPORT and notebook row keep their text (decision 2); this REPORT and this unit's notebook row carry the correction.
- **The fault was in `main` since claude-336.** That unit showed the integer reader's ask heads choose the same positions as the trained read. Nothing compared what the heads bring back from those positions, and they brought back the wrong thing. Any laptop reading of a model with ask heads made between claude-336 and this unit used the faulty reader. Only the four stage 3 models are re-read here.
- **The laptop copy is still short of its trained model on the stated values.** At 192: control 69 against 79, p-s3 170 against 173, p-s1 209 against 210, p-s2 183 against 183.
- **The new test checks the reader against the sums written out, not against the float trainer's code.** The float read runs only on a GPU. That the reader now follows the trained read is shown by the replies, not by a trace of the two side by side.
- **The change was tried before the ORDER** (decision 5): made uncommitted during the code read to test the cause, then committed after the ORDER. Every count below is retaken with a reader built at the committed head; the replies are the same bytes as the first try's on all eight files.

## What changed

- `crates/uor-r4-integer/src/stack/hamming.rs`. For a read with ask heads (`--hamming-ask 1`), the trained read brings back, from each position an ask head keeps, the table's point of that position's own token, as a copy head does; at the readout it mixes the ask mean in the copy form, `w_y ⊙ (y − n)` where an ask head found a candidate, and at the stream read it adds `w_y ⊙ y` at the copy mean's exponent. The integer reader had asked the same question but brought back the hook's input through the value rotation and added `w_y ⊙ y` as a plain type head. Now an asking read brings back table points (`TypeHeads::ask_mean`), keeps no payload cache, raises its mean to the stream's exponent, and mixes at the readout through the new `readout_ask_sum`. A read that does not ask computes as before.
- `crates/uor-r4-integer/src/stack/tests.rs`. One test, `an_asking_read_brings_back_the_kept_tokens_points`: on two small ask artifacts it checks the stream form and the readout form of every row against the sums written out, and that an asking read keeps no payloads.
- Nothing on the float side, nothing in the instrument, no artifact.

## Measurements

The four context-8256 artifacts of claude-352's kit, read with `raw-id-replies` built at `141eda2a2` (SHA-256 `dc30e9d2aa71963bda742fa38f35f564293f1f222c302b2c2e46b643ee03b134`) on the same 1,108 prompts at 192 and 990 prompts at 256, 48 tokens a reply, the place rows on for the three place-read models. "Before" is the reply file claude-352 counted (192: its I-f files; 256: its retake). Stated values are counted by claude-352's pinned scorer (SHA-256 `6844852f0bda7d1cd7b0212c5f04a515aa11c0e300d832429d2acf38b134489a`), unchanged.

**Replies that differ from the trained model's, rows of 1,108 at 192** (I-f's count):

| Model | Before | After |
|---|---|---|
| Place read, seed 2 (p-s2) | 589 (0.53) | 275 (0.25) |
| Place read, seed 1 (p-s1) | 565 (0.51) | 197 (0.18) |
| Place read, seed 3 (p-s3) | 665 (0.60) | 245 (0.22) |
| Control, seed 2 (n-s2) | 851 (0.77) | 300 (0.27) |

**Says the stated value, of 562 recall replies** (in brackets: the scorer's `exact` count):

| Model | Trained, 192 | Laptop before, 192 | Laptop after, 192 | Laptop before, 256 | Laptop after, 256 |
|---|---|---|---|---|---|
| p-s2 | 183 | 164 (27) | 183 (64) | 207 (37) | 238 (83) |
| p-s1 | 210 | 191 (28) | 209 (68) | 236 (44) | 269 (90) |
| p-s3 | 173 | 125 (12) | 170 (56) | 160 (23) | 223 (75) |
| n-s2 | 79 | 0 (0) | 69 (36) | 0 (0) | 76 (38) |

No trained-model reading exists at 256, so the 256 columns have no trained column beside them.

**Of the replies where the trained model says the stated value, the laptop copy gives the same reply word for word:** p-s2 79 then 171 of 183; p-s1 97 then 200 of 210; p-s3 49 then 161 of 173; n-s2 0 then 65 of 79.

**The first word of the reply, at 192.** On rows where the turn memory changes nothing in the trained model's first word, the laptop copy's first word differs on: n-s2 140 then 16 of 334; p-s2 28 then 21 of 436; p-s1 13 then 10 of 427; p-s3 23 then 18 of 427. On rows where the turn memory decides the first word, the laptop copy says the memory's word on 774 of 774 (n-s2) and 672 of 672 (p-s2) before and after, 669 then 674 of 681 (p-s1) and 674 then 679 of 681 (p-s3).

**Replies the repaired reader changes at 256, of 990:** n-s2 743, p-s2 492, p-s1 445, p-s3 544.

**Reply files, SHA-256, repaired reader:**

| Model | 192 | 256 |
|---|---|---|
| n-s2 | `e3843d0cbf7a4885c2043590987e3e6a18360b568c61a0c8da4129591af1de98` | `8753d3445cd512a8f5f1486512236644eb0a591182839978a6e358320f3291f2` |
| p-s2 | `09476870e1d850181c4f9b05fcd1f73eb73dad15245d07b42ed0ce8ed8e1820b` | `4057761934a224a38ca746f511b49e6b9e14cea6fa85a10692680200f2cc295b` |
| p-s1 | `539135ec73553270559940303a1c0bc02cd5e3a3ee7aaa25a4fee11070c10679` | `b80d66284b0f3eceb7545d8f14f971e1c107da1bcc6ee6c1d7d6460488a1a150` |
| p-s3 | `bbd7388b8e1579f087f43fe797cccca2dc6f4d8e8c7fbff022eebec87a524e16` | `4b0bcceefd716d0e296dcea5bc56e3a54917767493514e9b528dc440ef68c0fc` |

The files are on the laptop under `local/claude/export-repair/out-head/`, with the two kit programs that made them (`reread-353.sh`, `count-353.sh`); they are not committed.

**What the code read ruled out for the difference that remains:** the slide of the window, the turn memory at the first word (the rows above), and a cut of the rows the trained read sees (its greedy decode reads every row, as the reader does).

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` at `141eda2a2`, tree clean | 540 passed, 0 failed, 1 ignored |
| the new test alone at the head | 1 passed |
| the same with `hamming.rs`'s change set aside (the commit's patch to that file reversed) | it compiles and the test fails: "random, readout row 2, coordinate 0: -536727604342 against -380708828356.1798" |
| restored | the tree is clean, `hamming.rs` is `99aa6356625b33aeca66f58c633a07ae9abaae3286f4c04af314349f65c7588b` again, the test passes |
| `cargo build --locked --release --bin raw-id-replies --features integer-reader` (instrument crate), tree clean at the head | built; the reader named above |
| the eight reply files against the first try's | equal byte for byte |
| `uor-harness ratchet d23`, `d25`, `d32` | no additions (4451 of 4451; 0; 31 of 31) |

The one ignored test reads a trained snapshot from `UOR_FBS_SNAPSHOT`, which this laptop run does not set.

Not run: anything on a pod, any float pass, the float trainer's GPU tests, the chat program and the browser build (they link the same reader code; neither was started here). `uor-harness loop pr-check` runs after this commit and its result is in the review record on the PR.

## Skills used

None beyond the project loop. The reviewer was a subagent of this session.

## Still open

- **The second cause.** About a quarter of the replies still differ, on every model, arm r among them. Whether that is rounding or another fault is not known. A lead on the laptop: the ignored test `a_trained_fbs_snapshot_matches_its_reference_in_every_combination`, which compares a trained snapshot with its reference. Past that, a float trace of one sealed checkpoint beside the reader's is needed, which a GPU writes, and a pod goes up only on the owner's word.
- **Other ask-head models read on the laptop since claude-336** were read with the faulty reader. This unit does not list them or re-read them.
- **I-f's bar on this panel** (carried from claude-352): 0.10 was set from arm r on the panel with a line; on this panel arm r itself differs on 0.21. No model is under 0.10 after the repair either.
- **A float reading at the 256 window** does not exist, so the laptop's 256 numbers have no trained number beside them.
- claude-352's open items on the place read itself (questions that do not name the key, the copy inside a long value) are untouched.

## Plain summary

The program that runs a trained model on the laptop had a fault. Models with "ask heads" look back through the conversation for a matching earlier word. The laptop program looked in the right places but brought back the wrong thing from them, so every such model spoke worse on the laptop than it does as trained. That is why the comparison model in the last run seemed to say no values at all on the laptop.

With the program fixed, the same saved model files say more. The comparison model goes from 0 to 76 of 562 values at the full window; the three models with the place read go from 207, 236 and 160 to 238, 269 and 223. A reply that used to come out as "That's 44." or "Your friend's name is Tiktplplik." now comes out as the trained model says it: "That's 446.", "Your friend's name is Tikgeer."

Two things this does not do. It does not turn the last run's "not run" into a result: that run is spent and its rules stand. And it does not finish the repair: about one reply in four on the laptop still differs from the trained model's, on every model, and the cause of that part is not found. The place read still says more values than the comparison model, 238 against 76 on the laptop, but the last report's "207 against 0" overstated the gap.
