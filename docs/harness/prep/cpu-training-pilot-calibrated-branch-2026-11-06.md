# Pre-registration: fresh-distribution complete-branch calibration

Run ID: `cpu-training-pilot-calibrated-branch-2026-11-06`

## Status

**STOPPED BEFORE FRESH PANEL — INSUFFICIENT_POWER.** This sheet records the
fixed design and TRAIN-only calibration preflight. Seed 2788 was not generated.
It is retrospective: the calibration count ran before this sheet existed, so no
prior TEST FITNESS is claimed.

## Question

Can endpoint supervision from previously consumed fresh dev panels teach the
deepseek-231 integer branch selector to choose a later boundary on a new panel?

## Fixed construction

Use the unchanged deepseek-198 geometric content spine and deepseek-231
15-feature selector. Derive suffix and boundary memory only from TRAIN
`tt-2711`. Generate complete branch candidates on calibration panels 2783,
2784, 2785 and 2786. A group enters fit only with at least two boundary
candidates and one candidate exactly equal to the normalized calibration
reference. Fit per artifact for eight pairwise epochs with unit learning rate
and the existing weight bounds. The label-shuffle control preserves each
group's positive count.

Arms: learned, shuffled-label, first-boundary, longest-spine,
boundary-terminal-volume, reference-exact endpoint oracle and full retrieval.

## Data split

Training memory: TRAIN `tt-2711`.

Calibration panels: consumed dev seeds 2783, 2784, 2785 and 2786. Their
manifests are pinned to:

- 2783: `90e8f9e9a1dabc393b124a66e2c1fed0c9d3f670bf0128b60b00674032d87b78`;
- 2784: `03a01091a2907ec7dd3252964894c6ebae83797fa691d1f9ece3f9027fa0a68c`;
- 2785: `a5214c2f8430a9a3c22e4b468e40d6594247dbe5ac1e9fc30025e31cba9dcbf1`;
- 2786: `19d9629273ea423f5d39bef81740ef34c018a440ff4206359575290e1e8b773b`.

Fresh evaluation: `cell=dev,dev`, seed 2788, protocol 2, 128 conversations.
It was not generated.

## Base artifact

- seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Power floor

Each artifact needs at least 96 supervised groups, at least one group where
the first boundary is wrong, at least 200 fresh reachable rows, at least 100
learned selections different from first-boundary, and at least 20 fresh rows
with an exact branch.

## Preflight outcome

Seed-1 artifact calibration covered 1,832 rows but produced only:

- 58 supervised groups;
- 233 candidate rows;
- 58 positive rows;
- 0 baseline-wrong groups.

It therefore stopped below both required conditions. Seed 2 and seed 2788 did
not run.

Wall time was 28.98 s at 1,576% average CPU and 136,532 KiB peak RSS.

## Bar for a future run

On both artifacts, learned selection must emit every reachable reply with zero
premature BOS, retain first-boundary exact/body counts, beat shuffled-label
and first-boundary exact replies by the paired two-sigma rule, change complete
replies from both controls and repeat byte-identically.

## Stop rule

Stop below 96 groups, below one baseline-wrong group, below 200 reachable
rows, below 100 changed selections, below 20 fresh exact candidates, on a
vacuous shuffle, input overlap, nondeterminism, float/dense path or failed
comparison. Do not retune after seed 2788 opens.

## Steps

1. Extend the branch driver to validate and load repeated calibration panels.
2. Fit the unchanged selector on generated complete branches from calibration.
3. Run the seed-1 power preflight and apply the stop rule.
4. Do not generate seed 2788 after the power failure.
5. Preserve the preflight receipt and close this configuration.

## TEST FITNESS

TEST FITNESS: NOT_FIT - retrospective preflight receipt; the power floor stopped the run before fresh-panel generation.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.
