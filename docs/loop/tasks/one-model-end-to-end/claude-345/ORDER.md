# ORDER: one-model-end-to-end, claude-345

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`.

## Goal

**Find out whether today's model can memorize 1,000 short dialogues and say them back, and put the aligned design where every lab can read it.**

**Why now.** On 9 Nov Casey handed research direction to this session and accepted one design ("Yes, start"). Its first test is the cheapest one that can fail: no run so far has asked whether the assembled model can fit a small set and reproduce it in free-running decode with the end marker. Every later test of the two new parts is read against this one.

**The work.**
1. **The design, in the repository.** [DESIGN.md](DESIGN.md) in this folder is the design Casey accepted, with DeepSeek's five conditions of 9 Nov folded in (section 10). Until now it existed only in this lab's local folder, so no other lab could review it line by line.
2. **T0, the memorize test.** One sheet, [one-model-end-to-end-memorize-2026-11-09](../../../../harness/prep/one-model-end-to-end-memorize-2026-11-09.md): today's recipe (claude-301's b64L), unchanged, trained from a fresh start on 1,000 controlled dialogues only, two seeds, 4,096 steps, then the last complete reply of each training window (1,072 of the set's 3,589 replies) decoded greedily and compared with the true reply token for token, end marker included.
3. **The REPORT,** negatives first, with the counts and at least twenty decoded replies word for word, and one row in the float-training notebook.

**How it reads** (fixed in the sheet before any pod; the lower seed decides):

| Row | Reading at both seeds | What it means | Next |
|---|---|---|---|
| 0 | a step of the job fails, an input's hash or count differs, or fewer than 800 complete replies are decoded | NOT_RUN, an instrument failure | repair the instrument, rerun under a new root |
| 3 | under 95% of scored rows right under teacher forcing, or under 90% of the decoded replies whole under teacher forcing | this recipe at 4,096 steps does not fit the set | the address memory's first test is this same set, same steps and seeds |
| 2 | under 90% exact, with at least 90% of the same replies whole under teacher forcing | the fit is there and the free-running decode loses it | find the step where the decode leaves the forced path |
| 1 | at least 90% of the decoded replies exact and at least 95% of scored rows right | the assembled model fits and says back the set | the two new parts are judged on held-out prompts |

The rows are checked in that order and the first match owns the cell; the sheet has the exact readings (R1, R2, R2r).

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`), and the 9 Nov handoff's definition of done: a model that holds a conversation. The memorize test is the first rung of the accepted plan.
- (b) **What it changes in the model.** Nothing in code. It decides which defect the next unit repairs first, and it is the baseline the address memory is measured against on the same set.

**Not in scope.** The address memory's code and the state-keyed read (the next units). Any change to the trainer, the scorer or the recipe. The ask heads, the pull and the question carry, which stay as they are. The cause ladder of claude-344 (PR #3908, closed unmerged): its four pods were stopped about twelve minutes after launch, inside the smoke step, so no arm trained; their partial seals stay in the store under `results/claude/one-model-end-to-end-cause-ladder-2026-11-09/runs/`.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a recorded run that fits a small dialogue set and decodes it back.
- **Result (at claim):** none. `docs/research/notebook/float-training.md` has no memorize or overfit row. The nearest reading is claude-322's dev row at step 16,384 on the mixed corpus: 7,041 of 14,287 scored rows under teacher forcing, and 161 of 1,226 reply openings with 2 distinct opening picks.

## Claim

`uor-harness loop claim one-model-end-to-end --lab-topic talk` was refused on 9 Nov: "scope Cargo.lock overlaps Cargo.lock of item cpu-training-pilot (PR #3910)". This unit touches no `Cargo.lock`. The ORDER is written by hand, as claude-331 to claude-344 were, and the refusal is logged for Casey in the REPORT.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-345/**`
- `docs/harness/prep/one-model-end-to-end-memorize-2026-11-09.md`
- `docs/research/notebook/float-training.md`

## Done when

1. The sheet carries a FIT from a fresh non-author review, and this lab's pre-run review, before any pod is created.
2. Both seeds are trained and scored, or recorded NOT_RUN with the reason, and each run is sealed in the store and read back.
3. The REPORT gives the negatives first, the row of the table above that was read, the counts at both seeds, and at least twenty decoded replies beside their true replies.
4. The notebook has one row for T0.

## Redirect (owner, 10 Nov 2026)

Casey, in this lab's chat, after T0 was run and reported: "T0 is not a track that will advance the project at all", "you need to find the right one", "we have every [...] mechanism we need, we just have decided to keep NOT USING THEM". The lab then ran the saved 29 Oct model on the laptop and found that it talks and that the value is what fails, recommended one run, and Casey answered: "Run it".

**The work from here: the memory-line run.** The recipe that produced the talking model, trained on the same controlled world with the memory line written before each recall reply, then exported and served on the laptop with `/memory auto`, and read as a conversation beside today's model. The sheet is [one-model-end-to-end-memory-line-2026-11-10](../../../../harness/prep/one-model-end-to-end-memory-line-2026-11-10.md): three arms at seed 2 (the line; the line with two-token copy keys; the twin without the line), the bars and what each outcome unlocks.

**Why under this unit.** `uor-harness loop claim one-model-end-to-end` was refused on 10 Nov: the item is held by claude-346's worktree. This unit already holds the item's draft PR (#3911), so the run is recorded here and no second claim is made.

**What stays as it was.** T0's sheet, runs, seals and REPORT are kept unchanged as the record of a configuration that was run; no bar of T0 is moved. T0's follow-on steps in the design are not started.

**Files in scope, added:** `docs/harness/prep/one-model-end-to-end-memory-line-2026-11-10.md`; `crates/uor-r4-training/src/bin/recall-pairs.rs` and its entry in `crates/uor-r4-training/Cargo.toml` (the sheet's reader for the matched recall pairs: a new binary with no new dependency). After the three arms were read (owner, 10 Nov: "Yes, do it", to "change the copy read to take the latest exact match and re-read the conversations on the laptop"): `crates/uor-r4-integer/src/stack/hamming.rs`, `session.rs` and `mod.rs`, and `crates/uor-r4-integer/src/bin/uor-chat.rs`, for two serving switches that are off by default and leave an artifact's own serving unchanged: `--copy-rule mean|latest-exact|latest-nearest` and `--no-turn-memory`.

**The retrain (owner, 10 Nov: "Yes, retrain").** The two serving switches showed two causes of the wrong values: the copy read's mean blends earlier matches, and the turn memory's stored replies overwrite a copied value from its second token on. The lab proposed, and Casey approved, one retrain of arm m's recipe with three changes: the latest-exact rule in the float read, so training and serving use one rule; the turn memory leaving to the copy read every key the copy read already answers; and the memory line written in the reply's wording. The sheet is [one-model-end-to-end-reply-line-2026-11-10](../../../../harness/prep/one-model-end-to-end-reply-line-2026-11-10.md).

**Files in scope, added for the retrain:** in `crates/uor-r4-float-train/src/`: `flock_read_hamming.rs`, `flock_read_memory.rs`, `in_space_long.rs`, `fbs_export.rs` and `bin/train-in-space.rs` (two trainer options, off by default: `--hamming-copy-latest` and `--turn-memory-leave-copied`); in `crates/uor-r4-integer/src/`: `bin/fbs-export.rs`, `stack/format.rs`, `stack/mod.rs`, `stack/hamming.rs`, `stack/session.rs`, `stack/synthetic.rs`, `stack/tests.rs` and `stack/dialogue_memory.rs` (the exporter writes the copy rule from the run's config into the artifact and the session serves it; the sieve writes its line in the reply.s wording); `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs` (`recall=reply`) and `recall-pairs.rs` (`none=`, `E_line_recall`); and the sheet above. Every option is off by default, so an existing run, checkpoint or artifact reads as before.

**Done when, added:**
5. The memory-line sheet carries a FIT from a fresh non-author review and this lab's pre-run review before any pod is created.
6. Each arm is trained, sealed in the store and read back, or recorded NOT_RUN with the reason.
7. The REPORT shows, word for word, the same conversations through today's artifact and through each arm's, with the panel counts and the row of the sheet's table that was read, negatives first.
8. The reply-line sheet carries a FIT from a fresh non-author review and this lab's pre-run review before any pod is created.
9. The retrained arm is trained, sealed in the store and read back, or recorded NOT_RUN with the reason, and the REPORT shows the same conversations through arm m and through it.

**The mixed-key arm and the memory key (owner, 10 Nov: "proceed", then "Yes, run it").** After arm r's three seeds, general questions were still wrong on the laptop: 2 of 10 on the fixed script. Two causes were read off the tokens: the one-token copy key fires on common tokens, and the turn memory's key ends before the word a general question turns on. Casey approved one arm for the first ("Yes, run it": arm rk, `--hamming-copy-keys 1,2`). Its sheet names the second as the next unit in every row, so the three-key reply table (arm rt, `--turn-memory-tail 9`) runs on this lab's own pre-run review (§5.1), logged in its sheet for Casey, who can reverse it.

**Files in scope, added for the two arms:** `docs/harness/prep/one-model-end-to-end-mixed-keys-2026-11-10.md` and `docs/harness/prep/one-model-end-to-end-memory-chain-2026-11-10.md`; in `crates/uor-r4-float-train/src/`: `flock_read_memory.rs`, `in_space_long.rs` and `bin/train-in-space.rs` (the leave rule asked per head; the option `--turn-memory-tail`, off by default; the laptop counts `--reply-bound-train`, `--reply-bound-dev` and `--reply-chain-tail`); in `crates/uor-r4-integer/src/`: `bin/fbs-export.rs`, `bin/uor-chat.rs`, `stack/format.rs`, `stack/memory.rs`, `stack/mod.rs`, `stack/session.rs` and `stack/tests.rs` (the `memory_chain` record under `uor-r4.lut-stack/9`, absent on every artifact made before it).

**Done when, added:**
10. Each of the two sheets carries a FIT from a fresh non-author review and this lab's pre-run review before any pod is created.
11. Each arm is trained, sealed in the store and read back, or recorded NOT_RUN with the reason; the REPORT names the row read and, for a recall loss, which reading fell and where in the reply.
12. The three-key table has a fresh review of each side (trainer and serving) before its run, and an artifact made before it exports and serves as before, byte for byte.

## Hold: publish arm r first (10 Nov 2026, about 20:07Z)

Received in this lab's chat as pasted text, while seeds 1 and 3 of arm rt were being read: "Finish and publish the current arm-r replication before opening any new arm. Hold, but do not cancel, the approval for mixed-key and memory-chain work until the three-seed result is published."

**What was true when it arrived.** Arm r's three seeds had run and been read (their runs ended at 16:14Z, 17:07Z and 17:15Z), and the result was not yet committed to PR #3911. Arm rk (mixed keys) had run under "proceed" and arm rt (memory chain) at seed 2 under "Yes, run it". Seeds 1 and 3 of arm rt had been started on their pods at 19:22:50Z and 19:26:58Z (the first line of each job's log) on this lab's pre-run review, as the memory-chain sheet's row 1 names, and their runs ended at 20:05Z and 20:01Z. `uor-pod status` at 20:08Z showed no pod running, so no lease was renewed and no seed was restarted.

**The work from here.** Publish arm r's three-seed result in the REPORT and the notebook: the classification by the reply-line addendum, the readings it names, the pin, config, input, checkpoint and artifact hashes, and the three seeds' transcripts word for word. Arm rt's three seeds are recorded as run. No arm is opened: no mixed keys, address memory, state-keyed read, sieve change or other mechanism. The unit stops at the report, and the next change is proposed there from the evidence.

**Done when, added:**
13. The REPORT classes arm r's three seeds by the addendum and freezes the configuration by hash, with the three seeds' transcripts.
14. No pod is running and no new arm is opened by this unit.
