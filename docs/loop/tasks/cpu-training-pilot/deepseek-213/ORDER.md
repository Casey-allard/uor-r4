# ORDER: cpu-training-pilot, deepseek-213 (whole-reply ranker)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `dc2a05d72`.

## Goal

Test whether a bounded integer ranker trained on TRAIN leave-one-out prompt
neighbors can choose better complete stored replies than raw unigram-Jaccard
retrieval. Evaluation uses an unopened seed-2779 panel and two artifacts.

This changes answer selection inside the learned TRAIN reply set. It is not
fresh wording synthesis and does not alter the default serving selector unless
the registered comparison passes and a later wiring unit adopts it.

## Fixed construction

For each prompt, form an exact unigram set and retain the exact CPU writer query.
Candidate neighbors are the top 16 other TRAIN prompts by unigram Jaccard, with
exact rational ordering and lower ordinal on ties.

Each candidate has eight bounded integer features:

1. bias;
2. unigram Jaccard in Q16;
3. query-token coverage in Q16;
4. candidate-token coverage in Q16;
5. candidate-reply overlap with the query unigram set in Q16;
6. distance advantage against the closest candidate writer query;
7. exact duplicate-reply frequency in TRAIN;
8. signed candidate reply-length delta from the TRAIN mean.

A deterministic pairwise perceptron trains for eight epochs at unit learning
rate with weights bounded to +/-65,536. A group is included only when its
reference reply is one of the 16 candidates. Positive candidates are exact
reply matches. A matched control chooses the same number of positive
candidates per group from that group's candidates by a fixed SplitMix64 seed.

## Arms

1. `learned_ranker`;
2. `raw_unigram_jaccard`;
3. `shuffled_ranker`;
4. `query_geometry`;
5. `fixed_ordinal`;
6. `reference_membership_oracle`.

## Data and power

- TRAIN `tt-2711`, with leave-one-out neighbor construction;
- fresh `cell=dev,dev`, seed 2779, protocol 2, 128 conversations, generated
  only after FIT and GO;
- both public dual-c131 artifacts;
- at least 200 reachable rows;
- at least 96 supervised TRAIN groups and 24 groups whose raw top-1 is wrong;
- three alternating fresh-process runs per artifact and arm.

## Bar

For both artifacts and on both fresh-process repeats:

- learned emits all reachable rows with zero premature BOS;
- learned exact replies exceed raw Jaccard by the paired two-sigma rule;
- learned correct body positions do not fall below raw;
- learned exact replies exceed the shuffled-target ranker by the paired
  two-sigma rule;
- the fresh candidate-membership oracle reaches at least 20 exact replies;
- decisions repeat after timing removal and sidecars remain unchanged;
- all ranking and scoring remain integer, deterministic, and free of dense
  matrix multiplication or float serving arithmetic.

## Cost

Local CPU only. Cap: 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Done when

1. The ranker, matched shuffle, inference arms and focused tests pass.
2. FIT and GO precede the TRAIN power check and seed-2779 generation.
3. The two-artifact result, REPORT, compact evidence and program records merge
   to main.
4. Fresh-main verification and claim release complete.

## Stop rule

Stop on input or panel drift, fewer than 96 supervised groups, fewer than 24
baseline-wrong groups, a vacuous shuffle, fewer than 200 reachable rows, fewer
than 20 candidate-oracle rows, any failure of the exact/body/control bar, or
any need for float, dense matrix multiplication, a serving-kernel change or a
Claude-owned source change.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-213/ORDER.md
