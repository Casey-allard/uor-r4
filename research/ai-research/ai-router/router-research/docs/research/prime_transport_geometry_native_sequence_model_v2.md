# Prime Transport Geometry-Native Sequence Model V2

## Purpose

This note records the second direct bounded test of the thesis:

- geometry as the primary computational engine for sequence processing

The goal here is to move one step beyond the first architecture-aligned
sequence test and ask whether the same geometry-native idea still works on a
harder and less directly aligned compositional task.

## Stronger Task

The new bounded task is a compositional stack-rewrite sequence problem.

Input tokens are:

- `PUSH0`
- `PUSH1`
- `PUSH2`
- `POP`
- `SWAP2`
- `MERGE`
- `QUERY`

The latent state is a bounded stack. Each step:

- pushes, pops, swaps, or merges stack content
- updates the exact routing state `(b, phi, r, next_return_gap)`
- predicts the top-of-stack class after the step:
  - `0`
  - `1`
  - `2`
  - `EMPTY`

So this is:

- more compositional than the first task
- more dependency-bearing than the first task
- less directly aligned to the phase-fiber variables themselves

The task is still small and fully reproducible.

## Geometry-Native Model

The geometry-native v2 model keeps the same overall spirit:

- geometric state evolution is primary
- no transformer attention blocks appear in the geometry-native path
- no MoE blocks appear in the geometry-native path
- structured memory is explicit and bounded

The main ingredients are:

- exact geometric transport on `(b, phi, r, next_return_gap)`
- bounded stack memory
- routed stack-address signal
- a small learned readout over the geometry-plus-memory state

This is still a brutally explicit geometry-native controller, not a generic
learned neural replacement.

## Transformer Baseline

The comparison baseline is a tiny causal transformer on the same task:

- token embeddings
- positional embeddings
- two causal self-attention layers
- per-step readout

## Result

Aggregate result:

- geometry-native v2
  - test accuracy `1.000000000000`
  - query accuracy `1.000000000000`
  - test loss `0.000000022366`
  - parameter count `7908`
  - effective state size `12`
  - training time `2.663328082999` seconds
- tiny transformer baseline v2
  - test accuracy `0.790161132812`
  - query accuracy `0.770186364651`
  - test loss `0.445333242416`
  - parameter count `69700`
  - effective state size `64`
  - training time `6.189122292097` seconds

## Interpretation

Does the geometry-native model still show nontrivial sequence competence on the
stronger task?

- yes

Did it still compare favorably to the tiny transformer baseline?

- yes, strongly

The advantage did not shrink on this bounded task. It widened.

That said, the right interpretation is careful:

- the geometry-native model is solving this task through explicit geometric
  state evolution plus explicit bounded stack memory
- that is real sequence computation
- but it is still a bounded hand-structured geometry-native engine, not a
  general learned sequence learner

So this result strengthens the thesis in the bounded experimental sense:

- explicit geometry-native computation can replace transformer-style sequence
  processing on a harder small compositional task

It does **not** yet justify the stronger claim:

- that geometry has already replaced transformer-style sequence processing in a
  broad or task-agnostic sense

## Conclusion

Compared with the first direct test, this second result strengthens the thesis
materially.

Why:

- the task is a real step up in compositionality
- it is less directly phase-fiber-aligned
- the geometry-native model still functions as the primary sequence engine
- the transformer baseline does not catch up on the same bounded setup

## Next Stronger Test

The next smallest stronger test should be:

- a held-out generalization version of the same stack-rewrite family

For example:

- train on shorter horizons and test on longer horizons
- train on one bounded mix of rewrite operations and test on a shifted mix
- train on one stack-capacity regime and test on a slightly deeper one

That would be the next honest step because it tests whether the
geometry-native engine is doing transferable sequence computation rather than
only solving one tightly bounded local family.
