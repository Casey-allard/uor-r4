# ORDER: one-model-end-to-end, claude-327

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `74ccc944b`.

## Goal

**Part A of [claude-326's assessment](../claude-326/ASSESSMENT.md) (Q4): what claude's served models do at the reply's first token.**

That assessment found every served configuration fails at the first reply token. Its first-token ranks come only from DeepSeek's SFT artifacts, where reply format and choice are mixed together. This unit takes the same reading on claude's own served artifacts, on the laptop:
1. **A1, the ranks.** `served-reply-panel --teacher-forced 1`, built from `main` (it holds deepseek-34's `first_target` tally), on the stored served artifacts:
   - claude-322 and claude-323, seeds 1 and 2;
   - claude-325's two, when its seals land.

   The unit reports the first-target top-1, top-8 and top-32 counts and the rank min / median / max, under both renderings.
2. **A2, the choice.** At the first reply position, the rank of the *other* relation's value's first token, next to the asked one's. Every row of `m2-fresh256` states two relations. This separates choosing a value from opening with prompt content at all. If the scorer cannot read the other value from the panel's labels, the unit adds that to the instrument (`docs/labs/one-model-end-to-end/instrument`), with a test.
3. **A3, Part B's prep sheet.** The sheet for the probe whose query is not the key, with arms B0, B1, B2 and B-none as in the assessment, and its non-author FIT. No run happens in this unit.

The Hamming trace at the first position (the assessment's A2 trace) is reported only if `fbs-toks`'s existing trace reaches that position without new code. Otherwise it is named as still open.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. Under claude-326's lab decision, this test comes before any further language run.
- **(b) What it changes:** no model. It decides which lever the next training run takes: the language recipe, the read's weights, the carry, or a question-keyed read.

## Cancel probe (run first)

1. No open PR measures first-target ranks on claude artifacts: the open PRs are #3865 (claude-325's run) and DeepSeek's units.
2. The artifacts are on disk: `local/claude/claude-322/` and `local/claude/claude-323/` hold `served/s{1,2}.uorlut`.
3. Compute: laptop CPU only. No GPU.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end-*.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. A1's ranks for claude-322 and claude-323 are in the REPORT, with claude-325's added if its seals land in this unit, and with a notebook row.
2. A2 is measured, or its instrument change is merged with a test that fails without it.
3. Part B's sheet has a FIT from a fresh reviewer.
4. A fresh exact-head review passes the PR; it merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit. Past it, the unfinished parts are named in the REPORT and taken by the next unit.
