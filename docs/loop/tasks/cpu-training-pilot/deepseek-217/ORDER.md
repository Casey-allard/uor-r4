# ORDER: cpu-training-pilot, deepseek-217 (retrieval-prefix continuation)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `423dcf4d7`.

## Goal

Connect exact prompt retrieval to generation instead of selecting only whole
stored replies. Retrieve the nearest TRAIN reply, force its first one, two or
four tokens through the exact CPU FBS state, then continue greedily.

## Fixed construction

The prompt index uses exact unigram Jaccard and lower-ordinal ties. Candidate
prefix lengths are fixed at 1, 2 and 4. Each arm feeds the prompt through the
artifact, forces the retrieved prefix tokens through the same `Writer` heads,
then generates until BOS, cap or context. Full reply retrieval, model-only
generation, shuffled retrieved prefixes and reference-prefix oracles are
controls.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2781, protocol 2, 128 conversations, after FIT/GO;
- both public artifacts;
- 64-token cap and at least 200 reachable rows;
- three fresh-process runs per artifact and arm.

## Bar

The strongest TRAIN-validation prefix length is frozen before seed 2781. On
both artifacts it must:

- beat model-only on exact replies by the paired two-sigma rule;
- retain at least model-only body positions;
- beat the same-length shuffled-prefix control on exact replies by the paired
  two-sigma rule;
- emit every reachable row with zero premature BOS and deterministic repeats.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Stop rule

Stop on panel/input drift, fewer than 200 reachable rows, a failed control,
quality regression, nondeterminism, or any float, dense matrix, serving-kernel
or Claude-owned source change.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-217/ORDER.md
