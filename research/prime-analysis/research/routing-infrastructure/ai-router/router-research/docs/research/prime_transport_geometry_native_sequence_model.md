# Prime Transport Geometry-Native Sequence Model

## Purpose

This note records the first direct bounded test of the main thesis:

- geometry as the primary computational engine for sequence processing

This is not a routing-policy comparison and not a workspace-memory comparison.
It is a small sequence-model comparison on one bounded synthetic task.

## Bounded Task

The task is a short phase-fiber-controlled symbolic sequence stream.

At each step:

- the input token is one of:
  - `WRITE0`
  - `WRITE1`
  - `WRITE2`
  - `INC`
  - `SWAP`
  - `QUERY`
  - `SUM`
- the token acts on a small three-entity state
- the affected entity pair is determined by the evolving exact routing state
  `(b, phi, r, next_return_gap)`
- the target is a three-class output:
  - query answer for `QUERY`
  - pairwise routed sum for `SUM`
  - routed post-update signature for update tokens

So the same token does not have a fixed semantic target. Its meaning depends on
the evolving routing geometry.

## Geometry-Native Model

The geometry-native prototype uses:

- exact geometric state `(b, phi, r, next_return_gap)`
- deterministic phase-fiber-scale transport under each token
- routed three-entity structured memory
- a small learned readout MLP over the routed geometric state

It does not use:

- transformer attention
- MoE blocks
- a transformer layer stack inside the geometry-native path

The main sequence computation is the geometric state evolution itself.

## Transformer Baseline

The comparison baseline is a tiny causal transformer:

- token embeddings
- positional embeddings
- two causal self-attention layers
- linear per-step readout

Both models are evaluated on the same generated task family.

## Result

Aggregate result:

- geometry-native model
  - test accuracy `0.592881917953`
  - query accuracy `0.885010242462`
  - test loss `0.815225899220`
  - parameter count `2355`
  - effective state size `7`
  - training time `0.690180249978` seconds
- tiny transformer baseline
  - test accuracy `0.430338531733`
  - query accuracy `0.503764569759`
  - test loss `1.063794016838`
  - parameter count `39555`
  - effective state size `48`
  - training time `2.045237374958` seconds

## Interpretation

Did the geometry-native model function as a sequence-processing engine?

- yes

Reason:

- token meaning depends on latent routed geometry, not token identity alone
- the geometry-native model must evolve `(b, phi, r, next_return_gap)` over the
  full stream to know which entity is active, which entity is secondary, and
  what should be read or written
- the result is substantially above chance and substantially above the tiny
  transformer baseline on the same task

Did it show nontrivial competence?

- yes

The strongest evidence is the query metric:

- query accuracy `0.885010242462` versus `0.503764569759` for the transformer

That is the part of the task where routed state carry matters most directly.

Is this just a crippled lookup trick?

- not in the narrow bounded sense tested here

The task is sequence-dependent because:

- the active entity changes with evolving geometry
- update semantics depend on that geometry
- targets require repeated state carry over multiple steps

But this is still a conservative first result, not a general replacement claim.

## Conclusion

This first direct bounded test supports continuing the thesis:

- geometry can act as the primary sequence-processing engine on a small bounded
  task

What it does not yet show:

- that geometry replaces transformer-style sequence processing on broader or
  less architecture-aligned tasks
- that the current geometry-native path can yet scale without further model
  work

## Next Stronger Test

The next smallest stronger test should be:

- the same bounded task family, but with held-out longer horizons or held-out
  phase-fiber transport regimes, so the geometry-native model must generalize
  beyond the exact sequence lengths and basin visitation mix used in training

That would still keep the experiment small and interpretable while making the
thesis test meaningfully sharper.
