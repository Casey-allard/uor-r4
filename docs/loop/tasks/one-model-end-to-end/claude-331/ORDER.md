# ORDER: one-model-end-to-end, claude-331

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `35f55d334`.

## Goal

**The carry lever on B2, from its sheet to its run.** [claude-330](../claude-330/REPORT.md) landed on the outcome map's outcome 3, second branch:
- the question-keyed read (ask heads with the found-copy pull) passes B1 exactly at both seeds;
- B2 stays at chance;
- B2-aq's ask heads keep the asked value at the question row `Q(K_j)` in 376–512 of 512 rows, and nothing at the scored row four tokens later.

So the read chooses at the question, and the value is lost between that choice and the scored row. [claude-326's map](../claude-326/ASSESSMENT.md) (Q4) names the lever for this branch: "the carry: a read at the question token whose result is written forward, or a header-free answer position. It is tested on B2 first."

1. **A prep sheet** under `docs/harness/prep/one-model-end-to-end-carry-*.md`. It holds:
   - the lever, chosen from the code as it stands, with its geometric form;
   - **an ask-stream cut**, separating the ask heads' write to the stream at the `Q` row from the trunk's four-token carry;
   - arms at claude-330's settings, with B1-a and B2-aq reproduced as controls, and no-read twins;
   - a bar, an outcome map written in advance, the instrument checks, and the cost.
2. **A fresh non-author FIT of the sheet.** One more round is allowed. If round 2 reads NOT FIT, the unit stops at the sheet.
3. **If FIT: the code**, behind a default-off flag in `crates/uor-r4-float-train`.
   - The instrument checks are unit tests, each with a mutation that makes it fail.
   - The default paths are byte-identical (I-1).
4. **If the code merges inside the unit:** the two-seed run under a D67 pre-run review, read against the bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. This is the recall gap (11). Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** the model gains a way to carry a value chosen at the question to the answer position. That is the step between choosing a fact from the prompt and saying it in the reply. A fail closes this configuration, and the sheet's outcome map names the next lever.

## Cancel probe (run first)

1. No open PR adds a carry, hold or ask-stream cut to the read (checked at claim: none).
2. claude-330's code, REPORT and notebook row are on `main` at the base (`35f55d334`).
3. **Compute:** the laptop CPU for the sheet, the FIT, the build and the tests. A GPU only after FIT and a pre-run review, at about $1.50, inside the caps.
4. **Disk:** about 22 GB free. Builds use one target dir in this worktree, removed on merge.

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end-*.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet has a FIT line from a fresh non-author reviewer.
2. If FIT:
   - the code is merged;
   - its instrument checks pass, and their mutations fail;
   - the default paths are byte-identical (I-1).
3. If the run happens, both seeds are:
   - sealed to the store and read back;
   - read against the bar;
   - recorded in a notebook row.
4. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: two units. The sheet, the code and the run may land in separate PRs. Past the bound, the unfinished parts are named in the REPORT.
