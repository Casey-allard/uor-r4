# ORDER: one-model-end-to-end, claude-347

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `9b34019d495a045fd0d220b0eb58480b03146447`.

## Goal

**Take the laptop readings an outside review asked for on the replicated arm r model, so the next decision rests on what the served model does and not on the float panel alone.**

**Why now.** On 10 Nov Casey passed on a 27-page outside review ("UOR R4 Research Diagnosis and Lawful Experiments", pinned to `8a5bd52782fbf643644ebce6f7e8119428cb947c`, 14 hours before [claude-345](../claude-345/REPORT.md) merged). Claude checked its repository claims against `main` and sorted its proposals. Casey's answer to "Start the seven no-pod fixes now?" was "Yes, start". This unit holds the five that are readings. The two that change the serving code (the unused token choice during prompt reading, and the cell search) are the next unit.

**The five readings.** Each is a count or a table. None has a bar, none launches a pod, and none trains anything.

1. **Float against integer, row by row.** claude-345 showed that the float panel and the integer artifact move the same way; it did not compare them on the same inputs. The sealed float score of arm r holds every panel prompt and the float model's reply as token ids (`samples-p-greedy-memory.json`, 1,141 rows a seed). Each prompt goes through the exported integer artifact with the chat program at arm r's pin (`uor-chat --stack … --prompt-ids`, the product's `serve_step`), and the two replies are compared up to the first end-of-reply token. Reported for seeds 1, 2 and 3: rows equal, rows different, where the first difference falls, and how many of the different rows are recall rows.
2. **Selection from the model's own state.** In arm r the chat program's rule code picks which stated fact answers a question and writes it as the memory line; the model copies the line. This reading asks whether the served model's own state already separates the facts: with no memory line, is its state at the end of a question nearest to its state at the fact that answers it, among the facts stated in that conversation? Labels come from the panel's own construction, never from a guess: a row whose fact turn cannot be named from the panel's records is counted apart. The count is read beside the same count with the fact labels shuffled inside each conversation.
3. **General turns in wording TRAIN does not hold.** Arm rt answers 9, 10 and 9 of 10 general questions, and each of those questions is in TRAIN's wording. The same ten are asked again in new wording, each checked absent from TRAIN's token stream, on arm r and arm rt at three seeds.
4. **What the model holds, in full.** Per artifact: learned values by section, stored keys by table, bytes by section, and the session's memory. The figure quoted so far (1,112,576 values) leaves out the read tables and the turn memory.
5. **The BPB scorer's floor.** The M4 sheet's rank scorer gives the predicted token a fixed share of the mass. The review derives, and this unit recomputes, the lowest BPB that scorer can report and the share of targets that may fall outside the top eight before the 0.90 target is out of reach.

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`), and the 9 Nov handoff's definition of done: progress is judged by what the served model says.
- (b) **What it changes in the model.** Nothing. Reading 2 decides whether the next GPU step is a small learned selector on the frozen trunk or a change to what the trunk is trained to carry.

**Rules check** (CHARTER §3). No training, no float model run on the laptop: readings 1 and 3 run the integer artifact through the product path, reading 2 reads integer session state, readings 4 and 5 are arithmetic on files and sheets. §3.5: the float side of reading 1 is the pod's sealed score, not a host rerun. No neural comparator, teacher or grader anywhere.

**Not in scope.** Any pod. Any mechanism arm: the hold of 10 Nov on new arms stands, and the learned selector and the span copy are proposals until Casey decides. Changes to the serving code (next unit). The M4 sheet itself, which is outside this item's scope: reading 5 is reported here and proposed as an addendum. DeepSeek's stack contracts under `conformance/stack/`.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a same-input float-against-integer reading for arm r, or a state-selection count, already on `main`.
- **Result (at claim):** none. [claude-345's REPORT](../claude-345/REPORT.md) says "The float panel and the integer artifact agree, each with its own memory-off control", which is agreement in direction. `git grep -n -i "prefill" origin/main -- crates/uor-r4-integer/src` finds only `fbs-toks.rs`'s timing protocol, and no file under `crates/uor-r4-integer/src/stack` names `SourceIdentity`, `identity_latch` or `geometric_span`.

## Claim

`uor-harness loop claim one-model-end-to-end --lab-topic review-fixes` was refused on 10 Nov: "item one-model-end-to-end is already claimed by worktree .worktrees/claude-e8 (docs/loop/tasks/one-model-end-to-end/claude-346/ORDER.md)". claude-346 (PR #3913) is a draft under the hold and has no work in flight. The ORDER is written by hand in its own worktree, `.worktrees/claude-review`, on branch `claude/review-fixes`.

## Amendments (10 Nov, [claude]'s own decision review; Casey can reverse either)

1. **Reading 1 gets a reader.** `uor-chat --prompt-ids` never marks where the reply begins, so it answers with the turn memory off, and the first comparison against the memory-on float replies read 678, 691 and 644 of 1,141 for that reason alone. A small instrument program, `raw-id-replies`, steps the same ids through `serve_step` and marks the reply start as the text chat does. Its unmarked replies must equal `uor-chat --prompt-ids` id for id. The instrument folder is in scope for it.
2. **Reading 2 is read from what the model says, not from a state distance.** The planned count (is the question's state nearest to the right fact's state) is withdrawn for two reasons. The laptop's score files do not carry which turn answers a question, so the labels would come from a rule on repeated spans, which the outside review warns mislabels. And the state at the end of a question is the one that picks the next token: a miss there could not tell "the trunk does not hold it" from "that state is not where it is held", so it would close nothing. The reading that replaces it is exact and needs no label: the registered 18-turn script with the sieve off. If the model picks the fact itself, recall survives. The state-level question goes to the learned selector's own sheet, if Casey takes that proposal up.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-347/**`
- `docs/labs/one-model-end-to-end/instrument/**` (only if reading 2 needs a reader for session state)
- `docs/research/notebook/float-training.md` (one row a reading)

## Inputs, all on the laptop

- Float scores: `local/claude/claude-345r/runs/{r,r-s1,r-s3}/oh/run/score/r-line/` (sealed on the pods on 10 Nov).
- Integer artifacts: `local/claude/claude-345r/art/r-s{1,2,3}-c8256.uorlut`; arm rt's under `local/claude/claude-345t/`.
- Chat programs at the pins: `local/claude/claude-345t/talk/uor-chat-bd2532281` (arm r), `uor-chat-ad81e080b`.
- Tokenizer: sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
