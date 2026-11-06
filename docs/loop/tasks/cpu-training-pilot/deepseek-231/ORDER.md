# ORDER: cpu-training-pilot, deepseek-231 (joint complete-branch selection)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b4a7aac9f`.

## Goal

Select content and termination jointly over complete generated branches. The
previous independent stop guards changed one boundary decision at a time and
damaged later exact content. This unit instead generates one full-sequence
geometric content spine without committing a stop, treats every boundary hit
along that spine as a complete branch candidate, and uses a bounded integer
pairwise ranker to choose one endpoint from whole-branch evidence.

The ranker fits only on TRAIN examples whose exact reply is a candidate. The
evaluation reference is never a selector input. No retrieved reply is injected
into generation.

Fit supervision uses four document-grouped TRAIN folds. For each held-out
fold, suffix and boundary memory come from the other three folds, and the
ranker observes branch features along that fold's reference prefix. This keeps
the endpoint labels inside TRAIN while exposing them to unseen documents.
The fresh panel is generated only if the preflight reaches the registered
power floor.

## Fixed construction

Keep deepseek-198's local `Longest32To1`, full prompt-plus-reply suffix memory,
two-observation strict-majority rule, global-only-when-deeper choice, and local
ties. At every generated step, record a branch candidate when the unchanged
reply-boundary table would stop. Continue the spine instead of committing that
stop, up to the fixed cap or context.

Branch features cover the complete prefix and endpoint: length, boundary depth
and terminal count, local/global/writer decision counts, global depth and
observation sums, majority margins, stronger-continuation evidence, recent
tail support, repeated four-grams, and the endpoint's content/boundary conflict.
The selector is a deterministic integer pairwise perceptron with eight epochs,
unit learning rate, Q-bound weights, and exact source-order tie breaking.

Arms: learned whole-branch selection; matched shuffled-label selection;
first-boundary; longest-spine; boundary-terminal-volume heuristic;
reference-exact endpoint oracle; original replay of the first boundary; full
stored retrieval.

## Data and power

TRAIN `tt-2711`; fresh `cell=dev,dev`, seed 2787, protocol 2, 128
conversations; both public FBS artifacts. Need at least 200 reachable replies,
100 learned selections differing from first-boundary, 20 exact-reference
candidates in the fresh branch pools, and deterministic repeats.

## Bar

On both artifacts, learned whole-branch selection must:

- emit every reachable reply with zero premature BOS;
- retain at least the first-boundary arm's exact and body counts;
- beat the shuffled-label selector and first-boundary arm on exact replies by
  the registered paired two-sigma rule;
- change at least one complete reply from both controls;
- repeat byte-identically after timing is removed.

The fresh branch-pool exact-member oracle and control readings are reported;
they do not use evaluation references in candidate generation or selection.

## Stop rule

Stop on drift, a vacuous label shuffle, fewer than 200 reachable rows, fewer
than 96 TRAIN supervised groups, fewer than 100 changed selections, fewer than
20 fresh exact candidates, nondeterminism, any float or dense path, or a failed
required comparison. Do not retune features, epochs or thresholds after seed
2787 opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Files

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-joint-branch-2026-11-06.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-231/**`
- `docs/integration/current-state.md`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-231/ORDER.md
