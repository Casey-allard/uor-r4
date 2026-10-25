# REPORT: infra-loop-harness, claude-115 (new item m1-d11-artifact-scorer)

Lab: claude. Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md).
References #2904. References #2839.

## What did not work, first

- **The first title was refused.** At 92 characters it broke `roadmap check`'s 80-character limit, and it was shortened.
- **The full `roadmap sync` was not run.** Its dry run planned to create this item's issue, and also to update 33 other issues, close #2886 and change 5 milestone checklists. That sweep is the PM's (CHARTER §5.3), so #3248 was created by hand in sync's format and mapped in `docs/roadmap/issues.json`.
- **Step 6's FIT'd sheet cites tools that do not exist.** It names `discrete-stack export-d11` and `export_d11_to_path` as "merged on `main` in `#2998`". Neither is on `main`, and #2998 is deepseek's m3 sieve tests. This item does not edit that sheet, which is antigravity's; the note on #2839 says so.

## What changed

- **`docs/roadmap/items/m1-d11-artifact-scorer.md` (new):** M1, `ready`, `needs_gpu: true`, depends on `m1-d11-exporter-discrete`.
  - Its scope is a detached instrument folder, a prep-sheet glob and its task folder, with no crate path.
  - Its done-when covers three tools, each with a test that can fail: `export`, `score` (a CUDA, clean-room port of `IntegerDiscreteModel::evaluate_prefix`) and `toks`.
- **`docs/roadmap/issues.json`:** one line, `m1-d11-artifact-scorer` → #3248.
- **`docs/roadmap/README.md`:** the generated render.
- **This task folder:** the ORDER, the D68 card and this REPORT.

## Measurements

- **`xtask roadmap check`:** 93 items OK (ready 24, held 16, decision 1, done 52; M1 now 22).
- **`roadmap sync` dry run:** 1 issue to create (this item; created by hand as #3248), plus 33 updates, 1 close and 5 checklists, all left to the PM.
- No compute ran.

## Checks run at the head

- **The fresh D68 decision review:** APPROVE. Its brief covered the step-6 sheet, its task folders, #2839, Stage T's ORDER scope and the rules.
- **The exact-head review** at `458a99fe2`: CHANGES, on one Required done-when fix (an unmeetable bullet), now reworded. Its five advisories are applied (see DECISION.md).

## Skills used

- holding-the-goal: the item takes the next M1 gap off the critical path without moving step 6, its sheet or PLAN.md, and it takes no crate lock.

## Still open

- **The build itself:** claim `m1-d11-artifact-scorer` once this merges, then the instrument, its finite parity test, and the GPU qualification under D67.
- **The note on #2839** naming the tool commands, posted once they exist.

## Plain summary

The step that measures the first milestone needs three tools nobody has built: one to export a trained model into the shipped file format, one to score that file on the GPU, and one to time it on a laptop. This adds them to the plan as their own task, so they can be built now, while the model they will measure is still training.
