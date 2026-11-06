# ORDER: cpu-training-pilot, deepseek-229 (content-continuation boundary guard)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `31974240d`.

## Goal

Use the full-sequence content model's own confident continuation as permission
to continue past a reply-boundary hit. Accept BOS only when the current
generated state has no confident global continuation deeper than its local
copy branch.

## Fixed construction

Keep the deepseek-198 content selector. At a boundary hit, if suffix memory has
a strict-majority prediction with two observations and a depth greater than
the local copy depth, suppress BOS and select that continuation. Otherwise
accept BOS.

Arms: guarded boundary, original boundary, suffix-shuffled guarded boundary,
local longest, full retrieval, reference-length oracle.

## Data and power

TRAIN `tt-2711`; fresh `cell=dev,dev`, seed 2786, protocol 2, 128
conversations; both artifacts; at least 200 reachable replies and 100 guard
decisions per artifact; deterministic repeats.

## Bar

On both artifacts, guarded boundary must emit every reachable reply with zero
premature BOS, retain at least the better boundary/local exact and body counts,
beat the shuffled-continuation guard on exact replies by two sigma, and change
at least one complete reply.

## Stop rule

Stop on drift, a vacuous shuffle, fewer than 200 reachable rows, fewer than 100
guard decisions, nondeterminism, any float or dense path, or a failed required
comparison.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-229/ORDER.md
