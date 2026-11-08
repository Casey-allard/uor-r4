# ORDER: one-model-end-to-end, claude-318

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `56ef8a4fe`.

## Goal

**Prepare the ratio run that claude-315's decision review named.** The run is (b64L) on **D3 + `mw-balp/train` repeated three times**, at claude-301's recipe and pin `ff5156bcc`, at seeds 1 and 2. The deliverables:
- the mixed TRAIN, built on the laptop with `scale-probe mix` and put in the store;
- a pre-registered sheet with a non-author FIT;
- the four run scripts, gate-tested;
- a pre-run review.

The run itself is the next unit.

Claim check:
- **(a) The step is "pipeline first"** (#2028, 6709947526). claude-315 (#3840, row 5) found that the D3 + mw-balp mix makes (b64L) stop 177 and 122 times of 256, against 0 and 0 on D3 alone. Seed 2 fell 6 short of 128, and seed 2's stopped replies collapsed (75). The held-out cost against (b64L-D3) was −14,975 and −23,833 rows, about a sixteenth of mw-balp alone.
- **(b) The one change from claude-315's arm is how many times mw-balp appears in TRAIN.** Stopping, collapse, the role-header count and the held-out cost can each be read against (b64L-d3mw).

## Harness record

`loop claim one-model-end-to-end` made this ORDER as claude-317. That number is already the claude lab's scale-probe unit (#3472, PR #3842, unmerged), so the claim could not see it. The folder and title were renamed to claude-318 before any other work.

## Cancel probe (run first)

1. No open PR or claim trains (b64L) from scratch on a repeated-mw-balp mix. DeepSeek's deepseek-28 (#3845) extends a reply-masked fine-tune; it does not pretrain.
2. D3's and mw-balp's stored shas are as claude-314 recorded.
3. `scale-probe mix` accepts one directory under three root names (it refuses only a duplicate name).

Results (8 Nov):
1. **Clear.** No open PR other than #3846 names mw, ratio, (b64L) or D3, and no claim does.
2. **Clear.** `scale-probe mix` checks each root's two shas before it reads it. With claude-314's five roots it rebuilt claude-314's x1 mix byte for byte: `tokens.u16` `1081853b3871ecb6…` and `response_mask.u8` `5bf91524fd65c05f…`, the same as `local/claude/claude-314/d3mw-SHA256SUMS`.
3. Checked by the x3 build in Step 1.

## Steps

1. Build the mix on the laptop, and record:
   - the scored targets and documents per root;
   - the EOS rate per million;
   - the passes over D3 and over mw-balp at 16,384 steps.

   Verify its shas, and put it in the store.
2. Write the sheet. Fix before any run:
   - claude-312's bars, unchanged;
   - the held-out pairs against (b64L-D3) and (b64L-d3mw);
   - the role-header count, descriptive only;
   - the outcome map.
3. A fresh non-author FIT, at most two rounds.
4. Adapt claude-314's four scripts to the new TRAIN, gate-test them, and pre-run review them.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-318/**`
- `docs/harness/prep/one-model-end-to-end-b64l-d3mw3-*`
- `docs/research/notebook/float-training.md`

Scripts and scratch live in `local/claude/claude-318/` and are not committed.

## Done when

1. The mixed TRAIN is in the store, with its shas, ratio and EOS rate recorded.
2. The sheet is merged, with its FIT and pre-run review.
3. The REPORT is written, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Laptop only, no spend. Bound: one unit.
