# ORDER: cpu-training-pilot, deepseek-233 (fresh-distribution branch calibration)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `ddc66e665`.

## Goal

Supply the endpoint supervision that deepseek-231 lacked by calibrating the
whole-branch selector on previously consumed fresh dev panels, then evaluate
unchanged selectors on a new, untouched panel. This uses the observed
distribution shift without changing the evaluation reference, injecting
retrieved content, or touching Claude's track.

## Fixed construction

Keep the deepseek-198 content spine and deepseek-231 15-feature selector.
Calibration panels are seeds 2783, 2784, 2785 and 2786. For each public
artifact, generate spines on those panels with suffix and boundary memory
derived only from TRAIN `tt-2711`. A group enters fit only when at least two
boundary branches exist and one branch exactly equals the normalized
calibration reference.

Fit the learned selector per artifact with eight pairwise epochs, unit
learning rate and the existing weight bounds. A matched positive-count label
shuffle supplies the control. No evaluation seed may enter fit.

Placeholder 2787 is not used. Fresh evaluation panel: `cell=dev,dev`, seed
2788, protocol 2, 128 conversations.

Arms: learned, shuffled-label, first-boundary, longest-spine,
boundary-terminal-volume, reference-exact endpoint oracle and full retrieval.

## Power

Each artifact needs at least 96 calibration groups, at least one group where
first boundary is wrong, at least 200 fresh reachable rows, at least 100
learned selections different from first-boundary, and at least 20 fresh rows
with an exact candidate.

## Bar

On both artifacts, learned selection must:

- emit every reachable reply with zero premature BOS;
- retain at least first-boundary exact and body counts;
- beat shuffled-label and first-boundary exact replies by the paired
  two-sigma rule;
- change at least one complete reply from each control;
- repeat byte-identically after timing removal.

## Stop rule

Stop on drift, a vacuous label shuffle, input overlap with seed 2788, fewer
than 96 calibration groups, fewer than one baseline-wrong group, fewer than
200 reachable rows, fewer than 100 changed selections, fewer than 20 exact
fresh candidates, nondeterminism, any float or dense path, or a failed bar.
Do not retune features, panels, epochs or thresholds after seed 2788 opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-233/ORDER.md
