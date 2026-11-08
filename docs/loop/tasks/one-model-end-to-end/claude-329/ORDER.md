# ORDER: one-model-end-to-end, claude-329

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `308e25aad`.

## Goal

**Lever (b) of [claude-326's assessment](../claude-326/ASSESSMENT.md): a read whose query is built from the question.** [claude-328](../claude-328/REPORT.md) read outcome 4, a query failure, on both seeds. The trained Hamming read (type-head rotation query, read 4, width 256) solves B0 at 0.98–0.996, where the question repeats the key. It scores like its no-read twin on B1 and B2, where the question token `Q(K_j)` is not the key `K_j`. No type head keeps the asked position above chance.

The sheet's next step: "lever (b) of the assessment, a question-keyed read, under its own sheet. It must pass B2 here before any language run."

1. **A prep sheet** for the question-keyed read. It covers:
   - the mechanism: how the question's code keys the fact match, built from the project's geometry (§3.1), with hard Hamming picks and no squashing (§3.2) and no dense matrix multiplication (§3.3);
   - why it can learn where the type heads did not (the training signal, named);
   - its arms on claude-328's B0, B1 and B2 forms, with no-read twins and claude-328's read as the comparison;
   - its bar, outcome map, instrument checks and cost.
2. **A fresh non-author FIT of the sheet.** Its findings are applied. If two rounds read NOT FIT, the unit stops at the sheet and names what is left.
3. **If FIT:** the code the sheet names, in `crates/uor-r4-float-train`, behind a flag and default off. The instrument checks are unit tests, each with a mutation that makes it fail.
4. **If the code merges inside the unit:** the run, under a D67 pre-run review, read against the bar.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** a pass gives the language runs a read that can choose by the question, the reply panel's missing step. A fail closes this mechanism's configuration on this task, and the outcome map names the next lever.

## Cancel probe (run first)

1. No open PR adds a question-keyed or composed read: no open claude PR, and no `question_keyed` or `compose` read in `crates/uor-r4-float-train` at the base.
2. claude-328's REPORT and its B0, B1 and B2 forms are on `main` at the base.
3. **Compute:** laptop CPU for the sheet, the build and the tests. A GPU run only after FIT and a pre-run review, at about claude-328's cost ($0.80 for two seeds).
4. **Disk:** the laptop has about 8 GB free. Builds use one target dir in this worktree, removed on merge.

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end-*.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet has a FIT line from a fresh non-author reviewer: FIT, or NOT FIT after two rounds with what is left named.
2. If FIT: the code is merged with its instrument checks passing and their mutations failing, and the default paths are byte-identical.
3. If the run happens: both seeds are sealed to the store, read back, and read against the bar, with a notebook row.
4. A fresh exact-head review passes the PR; it merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit. The code and the run may land in follow-up PRs within this unit. Past the bound, the unfinished parts are named in the REPORT and taken by the next unit.
