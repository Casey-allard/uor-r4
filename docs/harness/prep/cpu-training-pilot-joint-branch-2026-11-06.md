# Pre-registration: joint complete-branch endpoint selection

Run ID: `cpu-training-pilot-joint-branch-2026-11-06`

## Status

**STOPPED BEFORE FRESH PANEL — INSUFFICIENT_POWER.** This sheet records the
fixed design and the TRAIN-only preflight. No seed-2787 panel was generated and
no fresh evaluation result is claimed. It was written retrospectively after
the TRAIN preflight, so it does not claim prior TEST FITNESS for that run.

## Question

Can one integer selector choose content and termination jointly over complete
branches cut from a single full-sequence geometric content spine, instead of
changing one boundary decision at a time?

## Fixed construction

Keep deepseek-198's local `Longest32To1`, full prompt-plus-reply suffix memory,
two-observation strict-majority prediction, global-only-when-deeper rule and
local ties. Generate a spine without committing a boundary stop. Record every
boundary hit along that spine as a complete branch candidate.

Each candidate carries 15 bounded integer features: length; boundary depth and
terminal count; local, global and writer decision counts; global depth,
observation and majority-margin sums; stronger-continuation count; recent-tail
local/global counts; repeated four-grams; and endpoint global advantage. A
pairwise perceptron uses eight epochs, unit learning rate and weight bounds
`[-65,536, 65,536]`.

Fit supervision is document-grouped four-fold TRAIN holdout. For each fold,
suffix and boundary memory come from the other three folds. The candidate
features are computed along the held-out document's reference prefix, and the
complete reference endpoint is the positive branch.

Arms: learned whole-branch selection; matched shuffled-label selection;
first boundary; longest spine; boundary-terminal-volume; reference-endpoint
oracle; full stored retrieval.

## Data split

TRAIN `tt-2711`:

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Planned fresh panel: `cell=dev,dev`, seed 2787, protocol 2, 128
conversations. It was not generated.

## Base artifact

- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Power floor

The run may open seed 2787 only with at least 96 supervised TRAIN groups, at
least one group where the first boundary is wrong, and at least 200 later
reachable rows. The final run would also require at least 100 learned selections
different from first-boundary and at least 20 fresh rows with an exact branch.

## Preflight outcome

The final document-grouped reference-prefix preflight produced:

- 0 supervised groups;
- 0 candidate rows;
- 0 positive rows;
- 0 baseline-wrong groups;
- 453 TRAIN rows inspected.

An earlier full-memory generated-spine preflight produced 77 supervised groups
and 273 candidate rows, but all 77 first-boundary choices were already exact.
It therefore supplied no example of the fresh failure this selector must
learn.

Both results stop below the 96-group floor. The second also has no
baseline-wrong group. No threshold, feature or fresh panel was tuned after this
result.

## Instrument gate for a future run

Pins match; suffix tables contain no BOS targets; the label shuffle changes
assignments and selector weights; every arm shares the panel and artifact;
every candidate is a non-empty complete prefix; all reachable rows emit; and
fresh-process repeats match after timing removal.

## Bar for a future run

On both artifacts, learned whole-branch selection must emit every reachable
reply with zero premature BOS, retain first-boundary exact/body counts, beat
shuffled-label and first-boundary exact replies by the paired two-sigma rule,
change a complete reply from each control, and repeat byte-identically.

## Stop rule

Stop before fresh generation below 96 supervised groups, below one
baseline-wrong group, below 200 reachable rows, or on a vacuous label shuffle,
drift, nondeterminism, float/dense path or failed required comparison.

Do not retune features, folds, epochs or thresholds after a fresh seed opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Inputs

- base: `b4a7aac9f`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

## Steps

1. Implement the generated-spine evaluator and complete-branch selector.
2. Run the TRAIN-only generated-spine preflight.
3. Replace full-memory fit with document-grouped four-fold holdout after the
   first preflight shows no wrong first-boundary group.
4. Run the reference-prefix preflight and stop at the registered power floor.
5. Do not generate seed 2787 or evaluate a fresh artifact.

## TEST FITNESS

TEST FITNESS: NOT_FIT - retrospective design and preflight receipt; the preflight stopped below the registered power floor before fresh-panel generation, so no fresh compute is claimed.

## Output

`local/deepseek/joint-branch/2026-11-06`; it contains only the TRAIN preflight
and timing receipt.
