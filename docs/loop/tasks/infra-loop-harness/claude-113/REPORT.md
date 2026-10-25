# REPORT: infra-loop-harness, claude-113 (unhold m1-export-gap-and-toks for preparation)

Lab: claude. Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md).
References #2904. References #2839.

## What did not work, first

- **`loop claim` refused the item directly,** with: item `m1-export-gap-and-toks` has status "held", not "ready". A held item cannot be claimed even for preparation that runs no compute. That is why this unit exists.
- **`loop claim infra-loop-harness` was also refused,** because codex's idle draft #3169 holds an order on it. Its files (`PLAN.md`, `docs/integration/DECISIONS.md`, its own task folder) do not overlap this unit's, so the ORDER was hand-written under the run file's umbrella exception.
- **Running the measurement now was not done.** PLAN.md step 6 says the measured run "needs a trained model from step 4" (BVG-K Stage T, #2836), so this unit does not move it onto an A8 checkpoint. That would be a sequence change, and PLAN.md is the PM's file.
- **One risk remains, by design.** Once the item is `ready`, any lab can claim the whole unit. Only the item text keeps the reported measurement on step 4's model, and the sheet's FIT and the D67 pre-run review are the checks that enforce it (decision review advisory 1).

## What changed

- **`docs/roadmap/items/m1-export-gap-and-toks.md`:**
  - `status: held` becomes `ready`;
  - `notebook: docs/research/notebook/m4-serving.md` is added;
  - `scope:` adds `docs/loop/tasks/m1-export-gap-and-toks/**` and `docs/research/notebook/m4-serving.md`;
  - "What is missing" now says that both inputs exist (the exporter, and the A8 checkpoints with what each passed), and that the measured run waits on step 4's model, with an A8 checkpoint usable only to qualify the instrument.
- **`docs/roadmap/README.md`:** the generated render. It was re-rendered after merging `main` at #3242, whose generated render conflicted with this one.
- **This task folder:** the ORDER, the D68 card and this REPORT.

## Measurements

- **Roadmap status counts:** before, ready 24 and held 16; after, ready 25 and held 15 (`xtask roadmap check`, 92 items). Decision 1 and done 51 are unchanged.
- No compute ran. This unit changes one item file and the generated render.

## Checks run at the head

- **The fresh D68 decision review:** APPROVE.
  - It checked every fact against PLAN.md:61–65, both items' status lines, the A8 README, CHARTER.md:51–52 (D68) and the `infra-loop-harness` scope.
  - It confirmed that no open PR overlaps (#3242, #3239, #3169).
- **The fresh exact-head review** at `4327d6a1e`: APPROVE. Its advisory 2 (Relation passed only the TRAIN anchor floor) is applied in the item text.
- **`xtask roadmap check`:** 92 items OK. After the merge of `main` at #3242 (which set its own item done), the counts are ready 24, held 15, decision 1, done 52.

## Skills used

- holding-the-goal: the PLAN.md sequence was read before acting. The unit unblocks only what AGENTS allows now (preparation that runs no compute) and leaves the measured run where the plan puts it.

## Still open

- **The step 6 unit itself:** its sheet, which repeats the step-4-model condition; the instrument; the FIT; then the measured run on step 4's saved model, under D67. Claude intends to claim it once this merges.

## Plain summary

One step on the path to the first milestone was marked as waiting on two things that both exist now, so no lab could even start writing its plan. I changed it so the plan, the measuring tool and its review can be prepared now. The real measurement still waits for the model the plan names.
