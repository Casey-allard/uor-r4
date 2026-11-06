# ORDER: cpu-training-pilot, deepseek-215 (lexical reply ranker)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `86cd73af4`.

## Goal

Isolate artifact-independent lexical evidence from artifact-bound writer
geometry in the whole-reply ranker rejected by deepseek-213. Reuse the same
TRAIN leave-one-out groups and raw Jaccard top-16, add a lexical-only fit, and
evaluate once on fresh seed 2780.

This is an answer-selection ablation over stored TRAIN replies. It does not
change the serving default or synthesize fresh wording.

## Fixed construction

Use the deepseek-213 eight-feature perceptron unchanged. Add two training
families:

- `combined`: all eight features, including `writer_distance_advantage`;
- `lexical`: the same fit with feature index 5 fixed to zero.

Each family gets the same matched positive-count SplitMix64 label shuffle.
Candidate membership, top-16 ordering, epochs, learning rate and weight bound
remain unchanged.

The primary learned arm is `learned_lexical`; its shuffled control is
`shuffled_lexical`. The previously rejected lexical+geometry arms remain
reported controls.

## Arms

1. `learned_lexical`;
2. `raw_unigram_jaccard`;
3. `shuffled_lexical`;
4. `learned_combined`;
5. `shuffled_combined`;
6. `query_geometry`;
7. `fixed_ordinal`;
8. `reference_membership_oracle`.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2780, protocol 2, 128 conversations, generated
  only after FIT and GO;
- both public dual-c131 artifacts;
- at least 200 reachable rows and 20 exact top-16 candidate-oracle rows;
- three alternating fresh-process runs per artifact.

## Bar

For both artifacts:

- lexical learned emits every reachable row with zero premature BOS;
- lexical exact replies exceed raw Jaccard by the paired two-sigma rule;
- lexical correct body positions do not fall below raw;
- lexical exact replies exceed its shuffled control by the paired two-sigma
  rule;
- lexical readings and decisions are identical across artifacts after timing
  removal;
- sidecars remain unchanged and every run repeats.

## Cost

Local CPU only. Cap: 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Done when

1. Both feature families and their matched shuffles are implemented and tested.
2. FIT and GO precede seed-2780 generation.
3. The two-artifact result, REPORT, compact evidence and program records merge
   to main.
4. Fresh-main verification and claim release complete.

## Stop rule

Stop on input or panel drift, an underpowered TRAIN group set, vacuous shuffle,
nonidentical lexical decisions across artifacts, fewer than 200 reachable
rows, fewer than 20 candidate-oracle rows, any failed primary comparison, or
any float, dense matrix operation, serving-kernel change or Claude-owned
source change.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-215/ORDER.md
