# ORDER: cpu-training-pilot, deepseek-225 (aligned retrieved endpoint)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `07349193e`.

## Goal

Preserve the deepseek-198 full-sequence content trajectory while replacing its
unconditional boundary stop with a reference-independent endpoint condition.
The retrieved TRAIN reply's stored length may defer a boundary stop only while
the generated reply remains an exact prefix of that retrieved reply. If the
generated reply aligns through the retrieved endpoint, stop there; on first
divergence, restore the original boundary path.

## Fixed construction

Retrieve the top-1 TRAIN reply by exact unigram-prompt Jaccard with
lower-ordinal ties. Keep deepseek-198's full-history suffix table,
strict-majority confidence, deeper-than-local arbitration and local ties.
Never force retrieved content into the generated reply; the retrieved reply is
only an endpoint guard.

Arms:

1. full-sequence content with the aligned retrieved endpoint and boundary
   fallback;
2. full-sequence content with the original boundary stop;
3. full-sequence content with the unconditional stored-length endpoint from
   deepseek-223;
4. the aligned endpoint with shuffled-retrieval reply associations;
5. local longest copy with the aligned true-retrieval endpoint;
6. full stored retrieval;
7. a reference-reply alignment oracle;
8. writer plus boundary fallback.

The retrieved guard comes only from TRAIN. The evaluation reference is used
only for reporting and the explicitly labelled oracle.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2784, protocol 2, 128 conversations;
- both public artifacts and at least 200 reachable replies;
- at least 100 deeper-global length decisions per artifact;
- at least 20 aligned retrieved-endpoint completions per artifact;
- fresh-process repeats of both artifacts.

## Bar

On both artifacts, the aligned true-retrieval arm must:

- make zero premature BOS stops and emit every reachable reply;
- retain at least the boundary arm's exact and body-position counts;
- beat the shuffled-retrieval guard on exact complete replies by the paired
  two-sigma rule;
- change at least one complete token sequence from the boundary and shuffled
  controls;
- stop at least 20 aligned retrieved endpoints;
- repeat byte-identically after timing removal.

The unconditional length arm, full retrieval and both oracles remain reported
controls.

## Stop rule

Stop on input drift, fewer than 200 reachable replies, fewer than 100 deeper
global decisions, fewer than 20 aligned endpoint completions, a vacuous
shuffle, parity/nondeterminism failure, any float or dense path, or any
required comparison failure after the fresh panel opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-225/ORDER.md
