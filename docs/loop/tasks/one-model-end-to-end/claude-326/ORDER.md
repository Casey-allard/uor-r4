# ORDER: one-model-end-to-end, claude-326

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `fa5004861`.

## Goal

**A pivot assessment of the one-model-end-to-end line.** The owner's decision is recorded verbatim on #2028 (6726003019): "Yes, pivot assessment". From 4 to 8 November, every served model scored 0 complete on the panel's bar, which asks for 104 of 256 complete and 52 of 128 pairs. Each unit turned one knob. Stopping and the serving path improved; the asked-for value never reached a reply.

The assessment reads those results together. It answers four questions:
1. **What blocks a complete reply:** data, scale, objective or mechanism? Each answer must be backed by a measurement already in the record. Where the record cannot tell two answers apart, it says so.
2. **What is established about selecting the queried value,** and what was never measured. Examples of the second: the asked-for value's rank at the answer position, teacher-forced; the trunk on the simplest one-key recall task.
3. **Which levers are closed, parked, or never tried.**
4. **The smallest structural test that would decide the cause,** with what a pass and a fail each unlock. This is a proposal for the next unit's sheet, not a run.

No GPU is spent in this unit. claude-325, already running, is reported under its own sheet. Its reading is folded in if it lands before the assessment's review.

Claim check:
- **(a)** This is the owner's decision (#2028, 6726003019). No bar, status or scope changes here. A change the assessment proposes is a decision card for the next unit.
- **(b)** It applies D21: three merged PRs on one line that do not move the headline force a pivot card. This line passed that long ago.

## Cancel probe (run first)

1. **No open PR assesses this line:** the open PRs are #3865 (claude-325's run) and DeepSeek's own units.
2. **The record is on `main`:**
   - the notebook `docs/research/notebook/float-training.md`;
   - the REPORTs `docs/loop/tasks/one-model-end-to-end/claude-30x…32x/`;
   - DeepSeek's REPORTs;
   - claude-201's earlier assessment.
3. **Compute:** none.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. `ASSESSMENT.md` answers the four questions, every number cited to a REPORT, notebook row or file.
2. A fresh-context decision review passes it, at most two rounds.
3. The REPORT gives the outcome, and the next unit is named. The PR merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). If the assessment finds that a decision is the owner's (§3, the caps, the charter), it gets one [claude, for Casey] note on #2028, and other work goes on.

Bound: one unit, no compute.
