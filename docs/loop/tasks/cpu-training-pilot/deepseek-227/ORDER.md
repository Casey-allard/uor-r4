# ORDER: cpu-training-pilot, deepseek-227 (top-K retrieved branch endpoint)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `7b1e888ef`.

## Goal

Test whether eight retrieved TRAIN replies, rather than one, preserve a viable
generated content branch long enough to defer premature boundary stops. Keep
deepseek-198's content selector and use the top-1 aligned endpoint from
deepseek-225 as the matched single-branch control.

## Fixed construction

Rank TRAIN replies by raw unigram-prompt Jaccard with lower-ordinal ties. The
candidate carries the top eight complete retrieved replies. While the complete
generated reply is a prefix of any branch it suppresses the boundary; it stops
when the generated reply equals any complete branch and restores the boundary
after no branch can extend. Retrieved content is never injected.

Arms:

1. full-sequence content with top-8 branch endpoint and boundary fallback;
2. full-sequence content with top-1 aligned endpoint;
3. full-sequence content with the original boundary;
4. top-8 branch endpoint with shuffled-retrieval associations;
5. local longest copy with top-8 branch endpoint;
6. unconditional stored length;
7. full stored retrieval;
8. reference-reply branch oracle.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2785, protocol 2, 128 conversations;
- both public artifacts and at least 200 reachable replies;
- at least 100 deeper-global decisions and 20 endpoint completions per
  artifact;
- fresh-process repeats of both artifacts.

## Bar

On both artifacts, top-8 must:

- make zero premature BOS stops and complete every reachable reply;
- retain at least top-1's and boundary's exact and body-position counts;
- beat the shuffled top-8 guard on exact replies by two sigma;
- change a complete token sequence from top-1 and shuffled controls;
- stop at least 20 replies at a retrieved branch endpoint;
- repeat byte-identically after timing removal.

## Stop rule

Stop on drift, a vacuous retrieval shuffle, fewer than 200 reachable rows,
fewer than 100 deeper-global decisions, fewer than 20 endpoint completions,
nondeterminism, any float or dense path, or a failed required comparison.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-227/ORDER.md
