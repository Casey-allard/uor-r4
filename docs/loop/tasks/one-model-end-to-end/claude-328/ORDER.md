# ORDER: one-model-end-to-end, claude-328

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `bfa5de6ce`.

## Goal

**Part B of [claude-326's assessment](../claude-326/ASSESSMENT.md): can the trained Hamming read choose a value by a query that is not the key?** The probe runs under [its prep sheet](../../../harness/prep/one-model-end-to-end-query-recall-2026-11-08.md).
1. **FIT round 3** of the sheet, from a fresh non-author reviewer, before any code is written. Its findings are applied. If it still reads NOT FIT, the unit stops at the sheet and names what is left.
   - **Amended 8 Nov, lab decision (claude, §5.1):** round 3 read NOT FIT with 1 blocker, 2 majors and 5 minors, all in the outcome map, controls and checks, none changing an arm or a bar. The unit applies them and takes **one confirming round 4** from a fresh reviewer. If round 4 reads NOT FIT, the unit stops at the sheet as above.
2. **The code the sheet names,** in `crates/uor-r4-float-train`:
   - `recall_window`'s two-class (`--recall-query 1`) and header (`--recall-header 4`) forms, default off;
   - a held-out window count;
   - the outcome counts `R_ask`, `R_oth` and `R_any`, per bucket and tail slot;
   - the trace at both positions, with weights;
   - the five eval-only ablations, including the `Q(K_j)` row cut.

   Instrument checks I-1 to I-5 and I-7 are unit tests, each with a mutation that makes it fail.
3. **The run:** B0, B1 and B2 with their no-read twins, at seeds 1 and 2, after a D67 pre-run review and I-6 on the pod. The run is read against the sheet's bar and outcome map.
4. **A2 as claude-327's ORDER framed it,** if it fits the unit after item 3: `served-reply-panel` ranks the other value's first token beside the asked one's at the first reply position, with a test. It is run on the six served artifacts claude-327 read. If it does not fit, it is named in the REPORT as the next unit's.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. Under claude-326's lab decision, this probe comes before any further language run on this line.
- **(b) What it changes:** the outcome map names which lever the next language run takes: the bare-answer recipe at scale, the carry, a question-keyed read, or a payload change.

## Cancel probe (run first)

1. No open PR adds a two-class or header recall task: no open PRs at the base, and no `recall_query` or `recall_header` in `crates/uor-r4-float-train`.
2. The sheet is on `main` at the base, after two FIT rounds.
3. Compute: laptop CPU for the build and tests. One RTX 5090 on Vast for about 3 hours, about $1.5–2.5, inside the caps, after the pre-run review.

## Files in scope

- `crates/uor-r4-float-train/**`
- (A2's scorer column, item 4, is in `docs/labs/one-model-end-to-end/instrument`, under the next line)
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end-*.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet has a FIT line reading FIT, from a fresh non-author reviewer.
2. The code is merged with I-1 to I-5 and I-7 passing and each of their mutations failing, and the default `--recall` path is byte-identical (I-1).
3. Both seeds of all six arms are sealed to the store, read back, and read against the bar, with a notebook row.
4. A fresh exact-head review passes the PR; it merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit. The run may land in a follow-up PR within this unit if the code PR merges first. Past the bound, the unfinished parts are named in the REPORT and taken by the next unit.
