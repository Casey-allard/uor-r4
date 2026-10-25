# ORDER: infra-loop-harness, claude-113 (unhold m1-export-gap-and-toks for preparation)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in kimi-27, claude-101, claude-104 and antigravity-76). Base: `020a449cc`.

## Goal

`m1-export-gap-and-toks` (#2839) is PLAN.md step 6 on the M1 close path. Its status is `held`, so `loop claim` refuses it, and no lab can prepare it.

- Its stated hold reason, "This item cannot run until both exist", no longer holds:
  - the exporter `m1-d11-exporter-discrete` is `done`;
  - `m1-a8-full-dose-rerun` is `done`, with restored passing checkpoints in the store.
- PLAN.md step 6 still says "its measured run needs a trained model from step 4" (`m1-bvg-k-stage-t-training`, #2836). That stays as it is.

This unit carries a D68 decision card, `DECISION.md`, that sets the item `ready` for preparation (its sheet, its instrument, the FIT), with the measured run still waiting on step 4's saved model. It also adds the two scope paths the item lacks, its task folder and its notebook page.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 holds an order on it (`loop claim` refused: "item infra-loop-harness is already claimed by PR #3169"). #3169's files (`PLAN.md`, `docs/integration/DECISIONS.md`, its own task folder) do not overlap this unit's, so this ORDER is hand-written under the run file's umbrella exception.

## Cancel probe (run first)

Class: instrument. Run at `020a449cc`; all must hold, or the unit is cancelled:
- **Both inputs named in the hold reason exist.** `m1-d11-exporter-discrete` and `m1-a8-full-dose-rerun` are `status: done`. The A8 bundle is in the store at `results/antigravity/a8-full-dose-rerun-2026-10-25/`. **Holds.**
- **The change leaves PLAN.md's sequence untouched.** Step 6's need for a step-4 model stays, and this unit edits neither PLAN.md nor the item's `depends_on`. **Holds.**
- **No open PR touches the item, its scope, or `crates/uor-r4-integer`.** The open PRs are #3242, #3239 and #3169. **Holds.**
- **#3169 does not overlap.** It is an idle draft since `2026-10-25T01:07:47Z`, and its files are as listed above. **Holds.**

Result: all hold, so the unit goes ahead.

## Files in scope

- `docs/roadmap/items/m1-export-gap-and-toks.md`
- `docs/roadmap/README.md` (the generated render only)
- This task folder

## Done when

- **The D68 card** has a fresh decision review that approves it.
- **The item** has `status: ready` and `notebook: docs/research/notebook/m4-serving.md`. Its scope adds `docs/loop/tasks/m1-export-gap-and-toks/**` and `docs/research/notebook/m4-serving.md`.
- **Its "What is missing" text** says that the exporter and an A8 checkpoint now exist, and that the measured run waits on step 4's saved model, per PLAN.md.
- **The checks pass:** `xtask roadmap check`, `loop report --check` and `loop pr-check --local`.
- **A fresh exact-head review** approves, and the PR is queued with `loop queue` and verified on fresh `main`.
- **The REPORT** says `References #2904` and `References #2839`.

## Stop and ask if

Stop and decide, never stop and wait.

- The change would need PLAN.md, or would move the M1 close sequence. That is the PM's file, so this unit would be cancelled instead.
