# D68 decision: discrete M2 sheet is NOT_FIT

Fresh review `agent-47` at `29d3b99651057c44c35d3a21cfa17f378af42ad2` returned **D68 DECISION REVIEW: PASS**, conditional on exact instrument scope and preservation of the rejected recipe.

## Decision

Classify #3300's discrete M2 sheet as `TEST FITNESS: NOT_FIT`. This is an instrument repair under CHARTER §4.2, not a model or mechanism negative. It authorizes no compute and changes no bar, hold, source, panel or mechanism status. `discrete-geometric-learning` remains ACTIVE; `native-geometric` remains PARKED.

#2851 remains closed for pins re-verification and its recorded non-author FIT review only. Its `done` status does not validate the recipe, the FIT or compute authorization. #3303 owns this repair; do not reopen #2851.

## Evidence

At the bound source, `stage-t-run step0` requires `--t16-fz --t16-rel --t1-fz --out`; `train` requires `--config --anchor-manifest --sheet --out-dir`; `trunk-run step0` requires `--fz-config --fz-checkpoint --out-dir`; each trunk arm requires `--name --config --checkpoint --out-dir`; and trunk grade requires four report paths plus `--out`. The sheet omits these, runs trunk Step 0 before Stage T writes the checkpoint, omits the anchor-manifest build and Stage T grade, and does not show how the built binaries reach their invoked paths.

Step 4 is only a comment. The recipe repeats the M1 Stage T/trunk chain, not M2 student training. The named M2 grader is the native host evaluator; no registered GPU complete-reply/pair scorer or discrete-D11-to-native-bank bridge exists. A bar-1 pass would not close M1 or resolve the trunk's `STAGE3_DIAGNOSTIC_HOLD`. The M2 bars remain registered text, not an authorization.

## Required record

- Preserve the removed recipe and prior FIT block byte-identical in `docs/labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md` before replacing them in the live sheet. The prior FIT was a pins/shape review, not an executability review.
- `m2-train-grounded-replies` (#2853) must provide the discrete student trainer, a deterministic GPU evaluation command for `m2-reply512` and sealed `m2-fresh256` with the 256/512 and 52/128 metrics plus four 0/128 controls, and the D11/native-bank artifact bridge or a separately reviewed artifact-family decision. Its scope/dependency contract must name that work; do not pretend this docs repair implements it.
- Do not add CLI aliases or change the M2 bar to make the old text pass. No source change belongs to this unit.
- Do not rest NOT_FIT on `HELD`. D67 retired training/M2/Stage B holds; the registry's `training_hold` and `rental_hold` flags are stale and are not changed here. Future authority is a valid FIT plus D67 inside the caps.
- A fresh non-author fitness review must confirm NOT_FIT at the repaired head. It grants no FIT and authorizes no compute.
