# REPORT: integer-long-context, claude-286 (the cancel probe, the scope and the plan)

Lab: claude. [ORDER](./ORDER.md). PR #3695, item #3471. Plan step: P4 in Casey's plan of 4 November, step 5 of THE ORDER (#2028 6634218223). Base `e06635e3a`.

**In one paragraph.** Long context is still unserved, so the work goes ahead; it was not cancelled.
- **What the probe found:**
  - The integer session stops at its artifact's `context`.
  - A larger `context` already makes it carry the state past 192 (claude-242), but no float forward exists to test that against.
  - Every served artifact is a 192-token model, and no export at another window has been tested.
  - Data past any window needs no generator change.
  - P2b's (L) recalls no far note, so recall cannot decide the rule yet.
- **What this unit lands:** the probe, fourteen scope additions for the two units that follow, two removals that keep the float unit off DeepSeek's lane, the item's text brought up to the base, and the plan:
  - claude-287 measures slide, blocks and carry in float on a GPU and decides;
  - claude-288 serves the chosen rule in the integer session.
- **What it changes in the model:** nothing.

## Tokens per second first

Nothing was measured, and no training or serving speed changes. The one reading of serving past 192 is claude-242's, on a 192-trained artifact re-exported at a context of 32,768. With the read, decode ran at 4,647 tok/s at 192, 1,851 at 2,112 and 177 at 32,768. The trunk alone stayed at about 5,500 to 6,100 tok/s at every length.

## What did not work

1. **The rule cannot be chosen by recall yet** (ORDER, probe 5). P2b's (L) states no far note even inside its own window: 0 of 283 on trained phrasings and 0 of 306 on p-long, at both seeds (claude-281, Mechanism). Recall past the window would read 0 under every rule. So claude-287 decides on held-out loss, greedy reply agreement and speed, and reports recall. The decision has to be re-read on the first checkpoint that recalls far notes inside its window.
2. **This unit's own claim locked DeepSeek's lane, found by the decision review.**
   - `cpu-training-pilot` lists `stack/session.rs` and `stack/mod.rs`, which were in this item's scope at the base.
   - `loop claim` refuses an item whose scope overlaps an open order's item (`loopclaim.rs:18, 680-710`). So from 03:19Z, while #3695 was open, DeepSeek could not claim its next pilot unit; it had been merging one about every hour.
   - The first draft of the ORDER said only `uor-chat.rs` was shared.
   - The fix: both files leave the scope at this merge, and claude-287's last commit returns them. Only the short serving unit, claude-288, then holds the overlap, and it is claimed only when DeepSeek has no open order.
3. **The item's scope lacked every file the later units edit.** The 2 Nov owner log (#3471 6585646011) named `hamilton_sheet.rs` and `in_space_long.rs`, and the scope rule reads an item at the merge base. So this unit adds them, and twelve more. The first draft had nine. The read and memory modules were added after the probe: a rule past the window changes what each read and reply table may see.
4. **One file the owner log named is not needed.** The log (6585860957) named `milestone_world_v2.rs` because its `CONTEXT = 256` capped every episode. That cap bounds one episode; the chained conversations P2 added (claude-271, claude-275) pass it. `heldout-panel chain=N` and `m-world-recall-corpus chain=N` take any budget, so a 4,096-token panel needs only a flag. The file stays out of scope.
5. **My own P4 scout (in the claude lab's place record) missed probe 1.** It read the export blockers but not claude-242's re-export, which shows the integer session already carries the state at a larger `context`. This changes the plan: if carry is chosen, claude-288 records the rule and tests it rather than building it. claude-287 must build the float carry to test it against.
6. **The item's title and done-whens still say 192.** They are left as they are. They still hold for the served 192-token model (`a7r`), and the title is rendered into `docs/roadmap/README.md`, which this unit's scope does not cover. A title change goes with a roadmap render in a later unit.
7. **P4's bound is tight.** The plan is three units, the top of the 2–3 bound. If claude-287 finds that carry needs a training arm, that arm is a fourth unit. The bound is then passed, and is recorded in the place record with one note to Casey.

## What changed

- **The ORDER** records:
  - the cancel probe: where the session stops, what a larger `context` already serves, the 192-only export, the data, recall, and the overlap;
  - the scope decision: fourteen files, what edits each, three left out and why, and the reversal cost;
  - the plans for claude-287 (the float rules, the panels, the sheet's checkpoints, readings and decision table, one pod) and claude-288 (the served rule, its tests, tok/s and memory to 8,192, gap 6).
- **The item** `docs/roadmap/items/integer-long-context.md`:
  - **its scope** adds seven float-crate files and seven integer ones:
    - float: `in_space_long.rs`, `bin/train-in-space.rs`, `hamilton_sheet.rs`, `geometric_stack_native.rs`, `flock_read_hamming.rs`, `flock_read_memory.rs` and `bin/fbs-snapshot.rs`;
    - integer: `stack/format.rs`, `stack/point_key.rs`, `stack/hamming.rs`, `stack/memory.rs`, `stack/dialogue_memory.rs`, `bin/fbs-export.rs` and `fbs_snapshot.rs`;
    - and it drops `stack/session.rs` and `stack/mod.rs` until claude-287's last commit, leaving 22 entries;
  - ***What is true now*** describes the base: the stop, claude-242's carry, the 192-only export, training's `--context`, the slide in both decodes, and claude-281's recall reading;
  - **option (c)** in *What is missing* says the integer session already computes it;
  - ***How to start*** has the current line references, every item sharing the scope, and what is kept out;
  - **a new section, *Where it stands*,** gives the three-unit plan and the 2 Nov target: a served 4,096-token conversation, with tok/s and resident memory to 8,192.

## Measurements

None. The probe is a reading of the code and the records at `e06635e3a`:

| Question | Where it was read | Answer |
|---|---|---|
| Where does serving stop? | `stack/session.rs:2444, 2929, 3104, 3542, 3793`; `stack/format.rs:2332, 2665` | at the artifact's `context`, with `ContextFull`; no non-test code outside `stack/` names it, and one oracle test asserts it (`stack_d11_oracle.rs:394`) |
| Can the session run past 192? | claude-242's REPORT | yes, at a larger `context`: the trunk's state runs on, and the read scans every earlier position; 145 MB at 32,768 |
| Can a long-window model be exported? | `src/bin/fbs-snapshot.rs:222, 237`; `stack/point_key.rs:374`; `bin/fbs-export.rs:106` | not as tested: under `--dev` the snapshot refuses any context but 192, and the point key is built from 192-token windows |
| What does training do past its window? | `hamilton_sheet.rs:1870` | back-to-back windows of `--context`, nothing carried |
| What does the float decode do? | `in_space_long.rs:5849, 5860, 6124`; `geometric_stack_native.rs:1569` | it slides over the checkpoint's window, serial and cached |
| Can float scoring read past a window? | `in_space_long.rs:7165-7169` | no, it refuses |
| Is data past 2,048 available? | `docs/labs/one-model-end-to-end/instrument/src/main.rs:296-300`; `m-world-recall-corpus.rs:161-164` | yes, with `chain=N`; p-long already passes 192 |
| Can recall decide? | claude-281's REPORT | not yet: 0 far notes stated, inside the window, at both seeds |

## Checks run at the head

- **The in-unit review, two rounds, each by fresh reviewers who saw only the item, the diff and the sources:**
  - **Round 1,** at `cd966fba9`:
    - a decision review: APPROVE WITH CHANGES, with five findings: the DeepSeek lock, the oracle test, the decision table, the cache format and `session_codec.rs`;
    - a claims check: two of the 14 claims it reached were imprecise: the oracle test outside `stack/`, and the `--dev`-only refusal.

    All seven were fixed in `6c80b2341`.
  - **Round 2,** at `6c80b2341`:
    - a delta review: APPROVE WITH CHANGES. It confirmed that the 22-entry scope overlaps no `cpu-training-pilot` entry, and that returning the two files in claude-287's last commit works under the merge-base scope rule. It found the tie row self-contradictory: on serving cost alone, a tie would pick blocks.
    - a claims check of the remaining twelve claims: none wrong, one imprecise (the `--score-rows` refusal from 193 to 1,536 was missing), two nits (the corpus writer's extra conditions, and training's `<|bos|>` restart and reaching-back last window).

    All were fixed in the next commit. The tie row now never chooses blocks, chooses carry over slide, and is provisional. The checker's suggested range fix was itself imprecise: the 193 to 1,536 refusal is the scorer's, not training's (`bin/train-in-space.rs:6, 21`), so only the scorer's range was added.
- **`loop report --check` and `loop pr-check --local`:** run at the head before the review record; the results are in the review record on the PR.
- **No code changed,** so no build or test applies; CI runs the loop gates.

## Skills used

None beyond the loop's own steps.

## Still open

- **claude-287:** done-whens 1 and 2, under the ORDER's plan, once this unit is on `main`.
- **claude-288:** done-whens 3 to 5.
- **The item's title** says 192, which no longer matches its scope.
- **float-kernel-fusion's done-whens 4 and 5** remain, with claude-285's deferred nits.

## Plain summary

The served chatbot stops after 192 tokens, about two turns of conversation. This unit checked what it would take to go further.

The integer engine can already keep going if its length limit is raised: its memory of the conversation simply runs on, at a known speed. What nobody has measured is whether the model's predictions stay good when it does, or whether it would do better re-reading only the last 192 tokens, or starting fresh at each 192.

The next unit measures those three choices on a GPU and picks one. The unit after it makes the served chatbot use that choice. This unit only adds the files those units need to the item's list, and brings the item's description up to date.
