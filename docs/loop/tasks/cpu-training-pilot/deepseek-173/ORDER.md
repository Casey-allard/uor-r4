# ORDER: cpu-training-pilot, deepseek-173 (pairwise copy-candidate ranker)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `1f56a8e7b`.

## Goal

Test a materially different copy objective: train one deterministic integer
pairwise ranker over the exact-copy candidate set rather than predict a token
from the query, rank only next-state distance, or score one predicted
continuation.

## Fixed construction

1. Generate train-panel 2711 trajectories with longest-32 content selection.
2. At each decision, collect unique exact-copy candidates at depths one through
   eight, excluding BOS.
3. For each candidate, compute seven bounded integer features from the exact
   served query and candidate occurrence:
   - bias;
   - exact source depth;
   - query/token sign agreement;
   - best sign agreement over eight cyclic table-row rotations;
   - source recency;
   - within-window occurrence frequency;
   - writer-token agreement.
4. Supervise only decisions where the reference next token is one of the
   candidates. Train a deterministic pairwise perceptron with eight passes and
   bounded integer weights.
5. Evaluate by selecting the highest-scoring candidate, with lower token id as
   the final tie break. The token is always already available to exact copy.
6. Repeat with training targets rotated through the candidate set, a
   depth-only fixed control and the existing longest-32/learned/geometric/
   random/depth/writer controls.

No float inference, dense matmul, softmax, ordinary transformer fallback,
serving change, CUDA change, termination change or Claude-owned source change
is permitted.

## Data and power

- fit on train panel 2711;
- fresh `cell=dev,dev`, seed 2766, only after FIT and GO;
- two public artifact seeds and at least 200 reachable replies per seed;
- at least 640 supervised pairwise decisions per seed;
- non-vacuous changes versus longest-32, shuffled-label ranker and depth-only
  ranker;
- fresh-process repeats of both seeds.

## Bar

For both artifact seeds on the fresh panel, the trained ranker must beat
longest-32, the shuffled-label ranker and the depth-only ranker on correct body
positions by the paired two-sigma rule. It must change at least one decision
against each, preserve zero premature BOS and ungated token mismatches, and
repeat byte-identically after timing fields are removed.

## Stop rule

Stop on source, artifact, tokenizer, panel, feature, label, weight or report
drift; fewer than 640 supervised training decisions; fewer than 200 reachable
replies; vacuous controls; nondeterminism; or any need to change serving, CUDA,
float training, termination or Claude-owned source. Once seed 2766 is read, do
not retune the features, weight bounds, learning rule or bar on it.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-ranker-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-173/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. The pairwise feature extraction, fitter, inference and controls have finite
   independent tests and deterministic replay.
2. FIT and GO precede seed 2766.
3. Both artifacts run with deterministic repeats and compact evidence.
4. REPORT classifies the result without claiming complete replies or language.
5. Reviewed merge, fresh-main verification and claim release complete.
