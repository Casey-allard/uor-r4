# ORDER: cpu-training-pilot, deepseek-177 (geometric candidate union)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `04fae9eaf`.

## Goal

Test whether a denser candidate set made from the model's own top geometric
writer candidates, exact-copy history and writer fallback supplies enough
supervised choices to train an integer ranker that improves body-token
selection over the writer and copy controls.

## Fixed construction

1. Train from teacher-forced reference prefixes on train panel 2711.
2. At each reference position, form the union of:
   - the 128 highest-scoring non-BOS tokens from the artifact's existing
     geometric writer candidate order;
   - unique exact-copy candidates at depths one through 32, excluding BOS;
   - the current writer/point-fallback token when it is non-BOS.
3. Retain one candidate per token. An exact-copy candidate keeps its deepest
   source occurrence; every writer candidate keeps its writer rank.
4. Give each candidate eight bounded integer features:
   - bias;
   - exact-copy depth, or zero;
   - query/token sign agreement;
   - best sign agreement over eight cyclic row rotations;
   - source recency;
   - within-window occurrence frequency;
   - writer-token agreement;
   - writer-rank bucket, with the writer's first candidate highest.
5. Supervise only positions where the reference next token is in the candidate
   union. Fit a pairwise integer perceptron for eight passes with learning rate
   four and weights clipped to `[-4096, 4096]`.
6. At generated inference, start from the prompt and generated tokens only.
   Select the highest-scoring candidate, breaking ties by lower token id. The
   candidate set excludes BOS.
7. Repeat fitting with each training target replaced by a deterministic
   different candidate.
8. Evaluate trained, shuffled-label, fixed geometry-only and longest-32
   controls on complete generated replies.

No serving, CUDA, float inference, termination or Claude-owned source change is
permitted.

## Data and power

- train on panel 2711 reference prefixes;
- fresh `cell=dev,dev`, seed 2768, only after FIT and GO;
- two public artifact seeds and at least 200 reachable replies per seed;
- at least 2,048 supervised candidate groups and at least 64 distinct target
  tokens from train panel 2711;
- non-vacuous changes versus writer, geometry-only, shuffled ranker and
  longest-32;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds on the fresh panel, the trained candidate-union ranker
must beat the writer, the fixed geometry-only selector, the shuffled-label
ranker and longest-32 on correct body positions by the paired two-sigma rule.
It must change at least one token against each, emit zero premature BOS tokens,
and repeat byte-identically after timing fields are removed.

## Stop rule

Stop on source, artifact, tokenizer, panel, feature, label, weight or report
drift; fewer than 2,048 supervised groups; fewer than 64 target tokens; fewer
than 200 reachable replies; vacuous controls; nondeterminism; or any need to
change serving, CUDA, float inference, termination or Claude-owned source. Once
seed 2768 is read, do not retune this construction.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-geometric-candidate-union-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-177/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. The candidate union, eight features, pairwise fit and controls are
   implemented and tested.
2. FIT and GO precede seed 2768.
3. Both artifact seeds run with deterministic repeats and compact evidence.
4. REPORT classifies the result against the bar and makes no complete-reply,
   language or serving claim unless the registered capability gates pass.
5. Reviewed merge, fresh-main verification and claim release complete.
