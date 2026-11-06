# ORDER: cpu-training-pilot, deepseek-219 (prefix-length completion)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `86996114f`.

## Goal

Test whether the retrieved neighbor's stored reply length can terminate
generation after a four-token retrieved prefix, producing complete responses
without a model-emitted BOS.

## Fixed construction

Keep the exact prompt-Jaccard retrieval and K=4 selected by deepseek-217. For
each row, generate up to the shared retrieved reply length. Compare against
model-only capped generation, model-only with the same retrieved length,
shuffled-prefix length completion, full retrieval and a reference-length
oracle. The output stop is `length`; no post-generation content is used to set
the limit.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2782, protocol 2, 128 conversations after FIT/GO;
- both public artifacts and at least 200 reachable rows;
- three alternating fresh-process runs per artifact.

## Bar

For both artifacts, retrieved-prefix length completion must:

- beat model-only length completion on exact replies by two sigma;
- retain at least model-only body positions;
- beat shuffled-prefix length completion on exact replies by two sigma;
- emit all reachable rows with deterministic decisions.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Stop rule

Stop on input drift, fewer than 200 reachable rows, failed control, quality
regression or nondeterminism.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-219/ORDER.md
