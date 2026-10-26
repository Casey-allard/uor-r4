# REPORT: m2-pin-train-mask-sheet-cli-repair, kimi-45

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md).

Closes #3303. References #2851.

## What did not work, first

- **#3300's discrete M2 recipe was not executable.** Its Stage T/trunk commands omit required parser arguments and binary staging; trunk Step 0 precedes the Stage T checkpoint; anchor-manifest construction and Stage T grading are absent. Step 4 has no candidate-specific GPU complete-reply/pair evaluator or D11/native-bank bridge.
- **The loop was not M2 student training.** It repeated the earlier M1 Stage T/trunk recipe. The lawful-pointer forced-copy oracle is a different mechanism and cannot substitute for the missing candidate evaluator.
- **The sheet mixed incompatible registrations.** Bar 1 combines different M1 instruments and is not a faithful Amendment 5/6 or M2 bar. Existing antecedents do not meet its top-8 threshold (`4,674 < 4,775`). M1 costs and seed/power claims do not define the missing M2 student's cost or power; both are now `UNDEFINED`.
- **The prior FIT did not check executability.** D68 reviewer `agent-47` and fitness reviewer `agent-48` confirmed `TEST FITNESS: NOT_FIT`. No pod, training, evaluation or host model execution occurred; additional GPU cost was `$0`.

## What changed

- Replaced the pseudo-executable recipe with exact defects and a `NOT_FIT` verdict. Preserved the valid pins while marking the composite bar and M1 antecedent cost/power as non-authoritative for the M2 student.
- Preserved the rejected recipe and old FIT block in `docs/labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md`; zero-diff verification against the #3300 base-blob sections passed.
- Corrected #2853, declared stale registry FIT/hold text non-authoritative, recorded D68 and fresh NOT_FIT reviews, and added lab and notebook records plus the new roadmap item.
- Bound remaining student/evaluator/bridge implementation to #2853 under its own ORDER and scope; did not change source, bars, holds, panels or mechanisms.
- Size: 10 files, `+242 / -42`; 0 code and 0 test lines, documentation `+235 / -39`, generated roadmap `+6 / -3`, other `+1 / -0`. The ORDER gave no numeric estimate, so the ratio is not applicable.

## Measurements

This was a host-only source and record audit, not a model measurement.

- Parser inspection established required Stage T, trunk and grade arguments at the bound source.
- `git show e9cfa887…:docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md | sed -n '49,77p;101,112p'` compared with the preservation file using `diff -u`: **0 differences**.
- D68: `agent-47`, decision head `29d3b99651057c44c35d3a21cfa17f378af42ad2`, issue comment `6426348540`.
- Fresh non-author fitness: `agent-48`, `TEST FITNESS: NOT_FIT` after corrections at `3db39b50405ba1135f5eace9d32ca12011fe2f5c`.
- Roadmap render: 95 items. No capability, model-quality, throughput or energy number was measured.

## Checks run at the head

- `uor-harness prep-check`: sheet is complete and classified `NOT_FIT`.
- `uor-harness loop gates --quick`: all three checks GREEN.
- `uor-harness loop gates --full`: 11/11 GREEN at the review head.
- `uor-harness loop report --check --order docs/loop/tasks/m2-pin-train-mask-sheet/kimi-45/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS; any scope/commit-order messages are advisory under D76.
- `git diff --check`: PASS.

## Skills used

- `uor-research-architect`: artifact/source binding, instrument-first classification and protected delivery.
- Fresh `uor-architect-reviewer` for D68 and `uor-experiment-designer` for the NOT_FIT review; the lead made and logged the decision.

## Still open

- #2853 must define and implement the actual M2 student, seed/arm design, candidate-specific GPU reply scorer and D11/native-bank bridge, or obtain a separately reviewed artifact-family decision. Its scope/dependencies must name that work.
- A future FIT needs exact per-arm bars, panels, denominators, power and cost; this repair does not supply them.
- The registry's stale FIT/hold text remains unchanged because it is outside this ORDER. D67, not those flags, governs future dispatch.

## Plain summary

The M2 instruction sheet had commands that the actual programs reject, and it stopped before evaluating M2 replies. It also repeated older M1 training rather than defining the M2 student. The sheet is now clearly marked unfit, so it cannot authorize a paid run. The original text is preserved, the valid data identities remain, and the missing student and evaluator work stays assigned to the real M2 training item.
