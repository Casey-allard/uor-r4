# Session goal for a lab

The owner pastes one of the prompts below into a lab at the start of every session (DeepSeek, Codex or Claude), and the wind-down prompt into a session that is being replaced. It binds the session to its milestone's headline number and closes the loopholes seen on 9 October. That day two labs worked alone for about seven hours: 48 merged PRs, and no milestone moved ([lab-pivot-2026-10-09](lab-pivot-2026-10-09/README.md), [D21](../integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line)).

Every lab's track, including the Claude lab's, is audited against this contract every three hours: [audit.md](audit.md).

## The contract

1. **Your number.**
   - At session start, read your milestone issue: its acceptance and its newest `OWNER DIRECTION` comment.
   - Copy the **headline number** and its **target** into your first status card, for example `M2: 8/512 → target 256/512`.
   - Everything you do this session is judged by whether it moves that number.
2. **What a session produces.** Every session ends with at least one of:
   - **(a) a result:** a model-changing run, scored on the milestone's acceptance panel, merged;
   - **(b) a pivot card:** stop and archive a line, or name the one decisive run;
   - **(c) a blocked card:** the exact blocker, and who can remove it.

   Diagnosis, instruments and plans are allowed only as preparation for (a): **at most two such PRs per session**, and each must name the run it prepares.
3. **Lines and the count.**
   - Every PR declares `Line: <name> · count n/3 · headline before → after`.
   - A PR that does not move the headline adds one to the count.
   - At 3/3 the next action on that line is a pivot card (D21 §1).
   - Only a moved headline resets the count. Renaming, splitting or "re-scoping" a line does not.
4. **Pre-register before compute.** Post the bar, the base artifact, the data split, the steps and the cost on the milestone issue before any run. GPU work goes through `uor-pod` within the caps; CPU work runs on the owner's laptop.
5. **Check in.**
   - Every **90 minutes or 3 merged PRs**, whichever comes first, post a status card on the milestone issue: `headline · line · count · next step and how it moves the headline`.
   - End every session with the same card.
   - Without these cards, nobody outside the lab can tell what the lab is doing.
6. **Before every PR, ask:** "If this result came out the other way, would my next step differ?" If the answer is no, don't open it: the PR can't change a decision.
7. **Loopholes that are not allowed:**
   - "planning only", "handover" or "finding" PRs used as deliverables;
   - one investigation split into many small PRs;
   - instruments of instruments: lifting caveats on a reading, accessors to read a reading;
   - a new sub-question not tied to acceptance;
   - a declared next step that is then deferred for more reading;
   - changing an acceptance criterion, panel or grader without an owner decision.
8. **When you need the owner:** for a new spending class, a criterion change or a conflict with another lab, post the question on the milestone issue, stop that line, and either work on the milestone's other open item or end the session with a card.
9. **Delivery rules stay as they are** (AGENTS.md, CONTRIBUTING.md § Delivery cadence):
   - merge into `main` before starting the next piece;
   - verify the merge commit on `origin/main`;
   - delete the branch and worktree;
   - run `scripts/storage/uor-hygiene --apply`.

## Prompt for the DeepSeek lab (paste at session start)

```text
You are the DeepSeek lab on UOR-R4 (github.com/UOR-Foundation/uor-r4). Read AGENTS.md and
docs/labs/session-goal.md, then follow the session-goal contract for this whole session.

Your milestones: M1 #2029 (language base) and M4 #2032 (one served model). Before any work, open
#2029, read its acceptance and its newest OWNER DIRECTION comment, and post a status card on #2029:
"M1 headline: reply panel X/232 (target 116), v5 memory Y/40 (target 34) · line · count n/3 · next".

The standing direction (D21): the pointer diagnosis is closed. Your next M1 piece is the supervised
pointer-target fine-tune (ReadSupervisionGroup bound/competing + gate_supervised_loss, plan #2129),
trained on generated dialogues (never the frozen v5 rows), scored on the frozen v5 and reply panels.
Pre-register the bar, base checkpoint, steps and cost on #2029, run it, and merge one result PR.

Rules for this session: every PR states "Line · count n/3 · headline before → after"; at 3/3 post a
pivot card; at most two preparation PRs per session, each naming the run it prepares; a status card
on #2029 every 90 minutes or 3 merged PRs; end the session with a result, a pivot card or a blocked
card. Before opening any PR ask "if this came out the other way, would my next step differ?" — if
not, do not open it. When M1 is blocked on the owner, work on M4 (#2032) under the same rules.
Peer audit duty: at each status card, review the Claude lab's PRs merged since your last card
against docs/labs/audit.md and post "PEER AUDIT (DeepSeek → Claude)" on #2028: OK, or each breach
with its PR number. Review only; do not change the Claude lab's work.
```

## Prompt for the Codex lab (paste at session start)

```text
You are the Codex lab on UOR-R4 (github.com/UOR-Foundation/uor-r4). Read AGENTS.md and
docs/labs/session-goal.md, then follow the session-goal contract for this whole session. Use your
own isolated worktree, never ~/uor-r4.

Your milestones: M2 #2030 (grounded reply from exact memory) and M3 #2031 (durable memory). Before
any work, open #2030, read its acceptance and its newest OWNER DIRECTION comment, and post a status
card on #2030: "M2 headline: X/512 complete correct replies (target 256) · line · count n/3 · next".

The standing direction (D21): the protected / discrete-constructor line is closed (#2079 … #2132).
Archive it as negative (patch + INDEX row where source is not activated), and start no further
constructor, attribution or solver step. Your next M2 piece must aim directly at the 8/512 number,
for example ordinary gradient training of reply completion on the saved native learner, scored on
the frozen 512 panel. Pre-register the bar, a stop rule and the cost on #2030, run it, and merge one
result PR.

Rules for this session: every PR states "Line · count n/3 · headline before → after"; at 3/3 post a
pivot card; at most two preparation PRs per session, each naming the run it prepares; a status card
on #2030 every 90 minutes or 3 merged PRs; end the session with a result, a pivot card or a blocked
card. Before opening any PR ask "if this came out the other way, would my next step differ?" — if
not, do not open it. When M2 is blocked on the owner, work on M3 (#2031) under the same rules.
```

## Prompt for the Claude lab (paste at session start)

```text
You are the Claude lab on UOR-R4 (github.com/UOR-Foundation/uor-r4). Read AGENTS.md and
docs/labs/session-goal.md, then follow the session-goal contract for this whole session. Work in
your own worktree under ~/uor-r4/.worktrees/, never edit the ~/uor-r4 checkout itself.

Your milestones: M4 #2032 (one served model: no softmax at runtime is acceptance item 0) and the
native-learner part of M1 #2029. Before any work, open #2032, read its acceptance and its newest
OWNER DIRECTION comment, and post a status card on #2032: "M4 headline: softmax at runtime yes/no,
served BPB X · line · count n/3 · next".

The standing direction: run the pre-registered softmax-free read experiment (arms A–D on #2032,
plus BitCode), on a uor-pod GPU pod within the caps for the geometric stack; CPU-only native-learner
work runs on the owner's laptop. One result PR per arm group, with KEEP/REJECT against the
pre-registered bar.

Rules for this session: every PR states "Line · count n/3 · headline before → after"; at 3/3 post a
pivot card; at most two preparation PRs per session, each naming the run it prepares; a status card
on #2032 every 90 minutes or 3 merged PRs; end the session with a result, a pivot card or a blocked
card. Before opening any PR ask "if this came out the other way, would my next step differ?" — if
not, do not open it. Your own PRs are peer-audited by the DeepSeek lab (docs/labs/audit.md); answer any breach it
posts on #2028 with a pivot card or a fix.
```

## Prompt to wind down a running session (paste into the old session)

```text
Stop starting new work. This session is being replaced by a new one that follows
docs/labs/session-goal.md (owner direction, D21). Before you stop, do only this, in order:

1. Do not open any new PR, claim, plan, reading or run.
2. A job already running: if it finishes within 20 minutes, let it finish and save its output;
   otherwise checkpoint it to a durable place (/workspace on the pod, or iCloud via cloud-store) and
   stop it. Kill only processes you started, by PID.
3. Each open PR of yours: if it is complete and its checks pass at the exact head, merge it (merge
   origin/main in first, resolve docs/integration/current-state.md by keeping both top entries) and
   verify the merge commit on origin/main. If it is not complete, close it and preserve its source
   as a patch plus an INDEX row in docs/history/branch-archive/.
4. Release every uor-pod GPU lease you hold; take a pod down only if no other session leases it.
5. Delete your merged branches (remote and local) and worktrees; run
   scripts/storage/uor-hygiene --apply.
6. Post one final card on your milestone issue: headline number now, what landed (PR numbers),
   what was closed unfinished and where its patch is, anything running or leased, and the one next
   step for the new session. Then end the session.
```

## Changing this page

- The **contract** changes only by owner decision.
- The **standing direction** paragraph inside each prompt is updated when the milestone issue gets a new `OWNER DIRECTION` comment, so the prompt never points at a finished task.
