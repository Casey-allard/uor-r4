# ORDER: cpu-training-pilot, deepseek-111 (reply termination diagnostic)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `3f7f8605b`.

## Goal

Diagnose why complete replies terminate only at the generation cap. For the
fixed deepseek-109 artifacts, record at every generated step whether BOS is in
the writer's admitted cells, its exact L1 distance to the query, its rank among
admitted writer candidates, and the distance margin to the chosen token.

This is a read-only model diagnostic with no new training configuration. The
32-step replay only reconstructs the pinned deepseek-109 artifacts. It does not
change serving or claim chat capability.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-reply-termination-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-111/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No float-training or Claude #3642 file is changed.

## Registered configuration

- seed 1 and seed 2 public FBS artifacts;
- the fixed 16-key / 102-training / 114-evaluation deepseek-109 selection;
- 32 exact Q24 updates, reproducing the pinned trained artifact hashes;
- four query-movement diagnostic arms: frozen, trained, memory-off and
  trained-memory-off;
- 16 generated tokens per prompt;
- 32 Rayon workers;
- seed 1 first, seed 2 second, under a 30-minute total local CPU stop.

## Done when

1. A finite diagnostic records BOS admission, exact distance, admitted-token
   rank and chosen-token margin for every generated step.
2. A TEST FITNESS sheet fixes the arms, inputs, stop, cost and classification
   rules before public replay.
3. Both seeds reproduce the deepseek-110 trained hashes, generation digests and
   exact CPU/serving parity for the query-movement arms.
4. The REPORT first classifies the observed termination failure:
   - BOS never admitted;
   - BOS admitted but never ranked first;
   - BOS ranked first but suppressed by a decoder defect;
   - mixed across arms/steps.
5. The result names the next mechanism implied by that classification and
   closes no mechanism beyond this diagnostic configuration.
6. Focused/full CPU tests, strict Clippy, formatting, evidence, loop gates,
   exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop on generation drift, artifact-hash drift, missing parity, nondeterministic
classification, counter overflow or any post-observation change to arms, cap,
keys, steps or controls.
