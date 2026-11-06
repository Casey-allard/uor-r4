# ORDER: cpu-training-pilot, deepseek-223 (content with stored-length completion)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `bbb8c5332`.

## Goal

Combine the strongest positive content selector from deepseek-198 with the
reference-independent retrieved-length completion that passed in deepseek-219.
The question is whether full prompt-plus-reply suffix arbitration can produce
more exact complete replies when its premature boundary stop is replaced by
the nearest TRAIN reply's stored length.

## Fixed construction

Retrieve the top-1 TRAIN reply by exact unigram-prompt Jaccard with
lower-ordinal ties. At generation, use the retrieved reply's length as the
completion budget, capped at 64. Keep deepseek-198's full-history suffix table,
strict-majority confidence, deeper-than-local arbitration and local ties.

Arms:

1. full-sequence content with stored-length completion;
2. full-sequence content with the original boundary stop;
3. relocated full-sequence content with shuffled suffixes and the same
   retrieved length;
4. shallow-global content with the same length;
5. local longest copy with the same length;
6. writer with the same length;
7. full stored retrieval;
8. a reference-length oracle using the true suffix content.

The retrieved length never reads the evaluation reference.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2783, protocol 2, 128 conversations;
- both public artifacts and at least 200 reachable replies;
- at least 100 deeper-global length decisions per artifact;
- fresh-process repeats of both artifacts.

## Bar

On both artifacts, full-sequence content with stored-length completion must:

- beat local-longest length completion and shuffled-suffix length completion
  on exact complete replies by the paired two-sigma rule;
- retain at least the better body-position count of its boundary and local
  length controls;
- emit every reachable reply with no premature BOS;
- change at least one token from local and shuffled controls;
- repeat byte-identically after timing removal.

Full retrieval and the reference-length oracle remain reported controls.

## Stop rule

Stop on input drift, fewer than 200 reachable replies, fewer than 100 deeper
global decisions, a vacuous control, parity/nondeterminism failure, any float
or dense path, or any required comparison failure after the fresh panel opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-223/ORDER.md
