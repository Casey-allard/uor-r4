# ORDER: cpu-training-pilot, deepseek-235 (reference-free branch closure)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `91be6c9ac`.

## Goal

Select content and termination without labels by choosing the earliest
sufficiently supported closure among complete branches from the existing
geometric content spine. This follows the exact-endpoint calibration failure
without another learned ranker or fresh-panel retune.

## Fixed construction

Use the unchanged deepseek-198 spine and deepseek-231 branch candidates.
A candidate is a **closed branch** when its endpoint reports no deeper global
continuation advantage and its last eight decisions contain at least four
local-or-global content decisions. Select the shortest closed branch; ties use
the deeper boundary, then earlier candidate order. If no branch is closed,
select minimum endpoint continuation advantage, then maximum tail support,
then deeper boundary, then the shorter branch.

The matched control applies one deterministic cyclic permutation to the
endpoint-advantage and tail-support tuples within each candidate list. It
preserves every value and candidate length while changing the association.

Arms: reference-free closure; shuffled closure features; first-boundary;
longest-spine; boundary-terminal-volume; reference-exact endpoint oracle; full
retrieval.

## Data and power

Suffix and boundary memory come only from TRAIN `tt-2711`. Fresh evaluation is
`cell=dev,dev`, seed 2788, protocol 2, 128 conversations, both public
artifacts. Need at least 200 reachable replies, 100 closure selections
different from first-boundary, 20 exact candidates, and deterministic repeats.

## Bar

On both artifacts, reference-free closure must:

- emit every reachable reply with zero premature BOS;
- retain first-boundary exact and body counts;
- beat shuffled closure and first-boundary exact replies by the paired
  two-sigma rule;
- change at least one complete reply from both controls;
- repeat byte-identically after timing removal.

## Stop rule

Stop on drift, a vacuous feature shuffle, fewer than 200 reachable rows, fewer
than 100 changed selections, fewer than 20 exact candidates, nondeterminism,
any float or dense path, or a failed bar. Do not retune the closure rule or
thresholds after seed 2788 opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-235/ORDER.md
