# Prime Transport Geometry-Native Sequence Model V3

## Purpose

This note records the third direct bounded test of the thesis:

- geometry as the primary computational engine for sequence processing

The goal here is to move one step closer to language-like contextual sequence
behavior without leaving the bounded research-only setting.

## More Language-Like Task

The new bounded task is a small discourse-style reference stream.

Input tokens are:

- `MENTION0`
- `MENTION1`
- `MENTION2`
- `SET_SPEAKER`
- `SET_TOPIC`
- `SHIFT_STYLE`
- `TAG0`
- `TAG1`
- `TAG2`
- `REFER`
- `ASK`

The latent state includes:

- current focused entity
- current speaker entity
- current topic entity
- a discourse style state
- per-entity tags
- the evolving exact routing state `(b, phi, r, next_return_gap)`

The overloaded/context-dependent part is:

- `TAG0/1/2` do not update a fixed entity
- `REFER` does not point to a fixed role
- `ASK` does not read a fixed memory slot

Instead, the current referent is chosen by a discourse-role lens that depends
on:

- evolving style
- evolving geometry
- current discourse-role assignments

So the same token has different meaning depending on evolving latent context.

## Geometry-Native Model

The geometry-native v3 model keeps the same core idea:

- geometric state evolution is primary
- no transformer attention blocks inside the geometry-native path
- no MoE blocks
- no standard transformer stack inside the geometry-native path

It uses:

- explicit exact geometric transport
- explicit discourse-role state
- explicit per-entity tag memory
- a small learned readout over the geometry-plus-context state

This is still a bounded explicit geometry-native engine, but it is less like a
visible stack machine and more like a tiny symbolic discourse processor.

## Transformer Baseline

The comparison baseline is a tiny causal transformer on the same task:

- token embeddings
- positional embeddings
- two causal attention layers
- per-step readout

## Result

Aggregate result:

- geometry-native v3
  - test accuracy `0.998567700386`
  - query accuracy `0.991875946522`
  - test loss `0.005087816156`
  - parameter count `5699`
  - effective state size `10`
  - training time `2.032320791972` seconds
- tiny transformer baseline v3
  - test accuracy `0.695442736149`
  - query accuracy `0.679468214512`
  - test loss `0.693002045155`
  - parameter count `69763`
  - effective state size `64`
  - training time `5.517679875018` seconds

## Interpretation

Does the geometry-native model still show nontrivial sequence competence on the
more language-like task?

- yes

Does it still compare favorably to the tiny transformer baseline?

- yes, strongly

Why this matters more than v2:

- the task is not just explicit stack maintenance
- tokens are context-dependent and overloaded
- correct interpretation depends on evolving latent role state
- the geometry-native engine must keep track of contextual assignments and
  geometry-conditioned referent selection over time

So the model is still functioning as a primary sequence engine through evolving
state, not just through trivial local lookup.

The correct caution is still the same:

- this is a bounded symbolic discourse task
- it is not a general natural-language benchmark
- it does not yet show that a geometry-native model has matched or surpassed
  transformers on broad language modeling

But it is a meaningful direct strengthening of the thesis in the bounded
experimental sense.

## Conclusion

Compared with v1 and v2, this v3 result strengthens the thesis further.

Reason:

- the task is more context-dependent
- the task is less visibly algorithmic than the stack-rewrite family
- the geometry-native model still substantially outperforms the tiny
  transformer baseline

So the current bounded evidence now supports the claim:

- geometry can serve as the main sequence-processing engine on small symbolic
  tasks that require evolving contextual interpretation, not only on
  architecture-aligned or explicit stack-like problems

## Next Stronger Test

The next smallest stronger test should be:

- held-out generalization on the discourse-style family

Examples:

- train on one mixture of role-shift patterns and test on a shifted mixture
- train on shorter context windows and test on longer ones
- train on one set of discourse-style transitions and test on unseen style
  schedules

That is the next honest step because it tests whether the geometry-native
engine is learning transferable contextual computation rather than only solving
one bounded family exactly.
