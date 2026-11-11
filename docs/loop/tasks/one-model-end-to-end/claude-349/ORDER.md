# ORDER: one-model-end-to-end, claude-349

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `c0f7db014c28849fb9be11aa5b1f64f6291ec878`.

## Goal

**Plan one arm in which the model itself picks the turn that holds the answer and copies the value from it exactly, and count on the laptop whether that can work before any pod.**

**Why now.** The owner's hold of 10 Nov stopped every new mechanism arm. On 11 Nov (UTC), in chat, the lab asked "Lift the hold for one arm: a learned geometric selector for which fact answers?" and the owner answered **"Yes, plan it"**. The hold stands for everything else. The readings behind the ask:

- With the chat program's rule and its memory line removed, neither arm answers any recall question: 0 of 4 at every seed, against 3, 4, 4 and 3, 3, 3 with it ([claude-347](../claude-347/REPORT.md), reading 2). That reading could not separate picking the fact from reading that far back.
- The rule is `stack/dialogue_memory.rs`, by its own header "an exact lexical sieve, not a learned reader". Nothing the model learned decides which fact answers.
- With an oracle line the float model copies it exactly on 0.759, 0.749 and 0.782 of matched recall pairs (three seeds), and on 0 without a line ([claude-345](../claude-345/REPORT.md)). Copying works when the source is unambiguous.
- The board's exact-memory row: with the true address the read passed 80 of 80, against 0 of 80 with no memory and 20 of 80 for the sieve, and "learned address formation is not built".
- On the two-class task the learned ask heads are exact where the value follows the question (512 of 512, both seeds, claude-330) and, with the question carried, answer 466 to 512 of 512 four tokens later against at most 2 with no read (claude-335). On the human-text mix the same heads did not move recall (claude-339, a negative for that configuration). Their pull toward the found copy (`HammingReads::ask_pull`) has never been given to a read that picks by turn.

## The arm: turn admission (arm s)

Arm r's recipe and copy rule, unchanged, with one read added in front of the copy heads. Every part exists except the turn code and the wiring.

| Step | What happens | Existing part | New |
|---|---|---|---|
| Turn code | Each user turn (one user message) gets one sign code: the sum of its tokens' table points, each turned lane by lane by a learned unit quaternion, each with a learned weight for its token that starts at 1 | shared table; the Hamming read's lane rotation and `pack_signs`; a learned gain per token, as `--hamming-hold` has | the sum over a turn |
| Ask | At a reply, the code of the user turn being answered, through a second rotation | the type heads' second rotation | the same sum |
| Choose | Hamming distance by popcount, over the user turns before the one being answered. The latest of those inside a learned margin is admitted; if none is inside, none is | margin weights `max(0, m − d)/m`; `choose_sorted_from`; arm r's "latest" rule | one choice a reply, not one a row |
| Copy | The copy heads (latest exact match under copy key 1, arm r's rule) look only inside the admitted turn, and read nothing when no turn is admitted | `--hamming-copy-latest 1` | the restriction |
| Learn | A margin hinge on the distances: the labelled turn is pulled inside the margin and every later user turn pushed outside; earlier turns are left alone; all are pushed outside for a reply with no label. Signs pass straight through | the found-copy pull (`HammingReads::ask_pull`); the rank-margin hinge | the label by turn (stage 1), counted on TRAIN with no writer oracle |
| Serve | Codes as u64 words, one a user turn, kept by the session; popcount and an integer margin | `stack/hamming.rs` | one header record |

What it replaces: the sieve and the `System: Memory:` line at serving, and the line in TRAIN (`recall=off`). What it is: a learned form of the sieve's shared-word count, in the Hamming read's arithmetic, over the same candidates the sieve searches (prior user turns only; the model's own replies are never a source). What it is not: a read of the trunk's state, which is dominated by the last few tokens and "never learned where to look" (`flock_read_hamming.rs`, module text).

**Why "latest inside the margin" and not "nearest".** A fact stated twice ("My code is 47." then "My code is 40.") gives two codes that differ only in a token the question does not hold, so "nearest" would be decided by chance. With both inside the margin the later one wins, as the sieve's rule does. The hinge matches: only later turns need to be pushed out.

**Known risks, stated before the counts.**
- A reply of the form "It's VALUE." repeats no word of the statement before the value, so under copy key 1 the latest exact match inside the right turn cannot name the value's first token. Arm r got past this only because its line was written in the reply's words. Stage 1's ceiling is split by reply form to size this; the repairs on hand are TRAIN replies that repeat the statement's words ("Your friend is Maya."), or a second pick of where the value starts inside the admitted turn.
- The turn code is a sum, so it does not see word order: "Tom's friend is Maya." and "Maya's friend is Tom." get the same code. Stage 1 counts how often a later turn with the same tokens competes; the repair is a term for the position inside the turn.
- TRAIN's conversations are short and the registered script has 18 turns, so the served read chooses among more user turns than it was trained on. Stage 1 reports how many prior user turns a reply has in TRAIN. The code of a turn does not depend on how far back the turn is, which is the reason to pick by turn and not by the trunk's state.
- The reply's first word still comes from the turn memory, whose keys are exact wording. Reworded questions are read, not fixed, by this arm.
- The turn memory's rule for leaving a key to the copy read (`--turn-memory-leave-copied`) must be recounted under the admitted turn.
- Serving needs one new header record in the artifact. Existing records do not change.

It should also stop a known fault: today any "… is" in a general reply is filled with what last followed " is" in the conversation ("The opposite of hard is Maya."). With no turn admitted the copy heads read nothing; stage 1 counts how often a general reply would wrongly get a turn.

## Stages

1. **This unit, laptop only, no pod: counts** on arm r's conversations without lines (TRAIN `conversations=60000 seed=13 recall=off`, panel `conversations=1000 seed=14 recall=off`), added to the trainer's existing census (`train-in-space --copy-key-oracle`). Recall replies and their values are found by pairing with the same conversations written `recall=reply`. Every share below is a share of replies, on the panel.
   - **Content tokens.** A reply's content tokens are its tokens other than speaker and end-of-turn marks, punctuation, and the 64 most frequent tokens of TRAIN.
   - **Labels.** The label of a reply is the user turn before the one being answered that holds the most of the reply's content tokens, if it holds at least one (ties to the later turn); otherwise none. The label does not depend on a copy key. Reported: **L**, recall replies with a label; **H**, labelled recall replies whose labelled turn holds the line's value; general replies with a label (a leak: these should have none); how often a model turn, or the turn being answered, would have won had it been a candidate.
   - **Ceiling.** **C**, per reply form ("It's VALUE." against a reply that repeats the statement's words): of recall replies whose labelled turn holds the value, those whose every value token the copy read names right when it looks only inside that turn, under copy key 1 (arm r's), and beside it under the census's other rules (key 2, the slot key) and with no restriction (today).
   - **Floor.** **F**: of recall replies whose labelled turn holds the value, those for which an untrained code (plain table points, no rotation, weights 1) admits the labelled turn under the arm's own rule, with the one integer margin that scores best on TRAIN. Beside it: general replies for which that margin admits a turn.
   - **Two known holes.** Restated facts: recall replies with an earlier user turn that states the same thing with another value, and whether the label is the later one. Equal bags: recall replies with a later user turn that has the labelled turn's tokens in another order.
   - **Reach.** How many prior user turns a reply has, in TRAIN and in the 18-turn script; and how many labelled turns lie whole inside a 192-token training window, which sets the training context in stage 2's sheet (192 if at least 90% do, else the next context that reaches 90%).
2. **Next unit, if stage 1 says go: build and register.** Trainer, exporter, session, tests with negative gates; the one-page sheet with bars taken from stage 1's counts; a fresh-context FIT. Laptop only.
3. **Then one pod, when the owner says run:** arm s at seed 2 (arm r took 50 minutes and $0.78 on one 5090). Controls on the same rows: arm s served with admission off (the no-memory control, L-010); arm s served with the latest prior user turn always admitted, and with the untrained code (so picking the right turn can be told from restricting to any turn); the oracle turn on the float panel (the ceiling); and arm r with its line as the reference. Seeds 1 and 3 only after a row-1 reading.
4. **Wording.** The reworded 18-turn script and question templates held out of TRAIN. Read at stage 3 with no bar; its own step after.

**How it will be judged (bars are fixed in the sheet, after stage 1).** On the laptop, sieve off, no line, integer artifact, through `uor-chat`: recall in the registered 18-turn script, of 4 (arm r: 3, 4, 4 with the sieve, 0 without), read word for word. General turns no worse than arm r's. Float panel without a line: exact recall replies (arm r: 0). Stated with every result: trained on short conversations inside one training window, served on 18 turns.

## What stage 1 decides

Read in order; the first row that applies decides.

| # | Reading | Means | Next |
|---|---|---|---|
| 1 | L under 0.80 | the reply's own words do not say where it came from | the label is the fault: read 20 unlabelled recall replies; a writer label is the fallback, stated as an oracle |
| 2 | H under 0.90 | the label lands on a turn without the value (a stale or unrelated turn) | the label rule is repaired and recounted before anything is built |
| 3 | Equal bags on over 0.02 of recall replies | the sum cannot tell the right turn from a later rival | the turn code gets a position term in stage 2's ORDER |
| 4 | C under 0.60 for every reply form | copying inside the right turn fails (first word, repeated token, value split across pieces) | the arm is not built as written; the 20 worst replies are read token by token and the repair is named |
| 5 | C at or above 0.60 for some forms only | selection is the missing step for those forms | stage 2 for those forms; TRAIN's replies are written in a passing form, stated in the sheet |
| 6 | F at or above 0.90 | the untrained code already picks the turn | stage 2 with the untrained code first, the learned weights second |
| 7 | otherwise | selection is the missing step, and copying inside the right turn works | stage 2 |

The thresholds route the next step; none stops the line of work. A miss closes this configuration only (CHARTER §4.1). General replies with a label, or admitted by the floor's margin, on over 0.05 are reported as a fault of the label or the margin and carried into the sheet, since an admitted turn there brings back the "… is Maya." fault inside that turn.

**Review of this ORDER.** One fresh-context reviewer read the first form at `8f821bc2f232fbc0ce58c6fde05e2a914d777fef` and returned NOT FIT on three points, all taken here: the decision table could pass a wrong design (no row used H, mixed units, overlapping rows); a restated fact could not be resolved by "nearest"; the label rule named no copy key and let common words, model turns and the answered turn win. Its other findings are also taken (user turns only, the answered turn excluded, the two extra serving controls, the reach statement). This form has not been re-reviewed; the binding review is the sheet's FIT in stage 2.

**Amendment 1 (11 Nov, made while counting; [REPORT](REPORT.md)).** The lined panel is `recall=oracle`, not `recall=reply`: the reply-worded line is the whole reply and cannot say what the value is, and the oracle line is "Memory: VALUE.". Counted beside the registered counts: the stop lists 0, 16 and 256, the ceiling with the true turn admitted, the clauses of the turn that holds the value, what the question's own words pick with nothing learned, the reply form cut two ways (by the token before the value, and by whether the turn has the same token before it, which is this ORDER's "repeats the statement's words"), and general replies that have an earlier user turn. Not counted: the floor F with table sign codes (row 4 applied before row 6), the slot key's ceiling, and the reach in the 18-turn script. No threshold in the table changed.

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`): a grounded multi-turn conversation needs the model, not a rule in the chat program, to find what the user said.
- (b) **What it changes in the model.** This unit: nothing. The arm: one read, about 500 learned values (two rotations of 64 quaternions each, and a margin) plus one weight a token (4,096).

**Rules check** (CHARTER §3). Geometry only: sign codes, lane rotations, Hamming distance, a margin hinge. No squashing function and no exponential anywhere, in the read or its loss. No dense multiply: a sum of table rows and a lane-wise quaternion product. Float training offline on the GPU, integer serving. No neural comparator, teacher or grader. This unit runs no training: counts on the laptop (host checks, §4.5).

**Not in scope.** Any pod or training run. Any other arm: the hold stands. The E8 address memory (PR #3913 stays a draft). DeepSeek's contracts under `conformance/stack/`. `crates/uor-r4-cpu-train`. Any bar or status.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that picks an earlier turn, or a census that already counts the copy read by turn.
- **Result (at claim):** none. `git grep -n -i -E "admit(ted)?_turn|turn_admission|turn admission|fn admit" origin/main -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` finds only `admit_launch`, a CUDA module check.

## Claim

Written by hand, as claude-347's and claude-348's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-349/**`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` and the census code it calls (stage 1's counts only)
- `docs/research/notebook/**` (one row a result)

## Inputs, all on the laptop or in the store

- The writer `m-world-recall-corpus` (`crates/uor-r4-training`), world v2, protocol 2; the no-line panel is sealed under `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/`.
- Arm r's artifacts and scripts: `local/claude/claude-345r/`, `local/claude/claude-345t/talk/`.
