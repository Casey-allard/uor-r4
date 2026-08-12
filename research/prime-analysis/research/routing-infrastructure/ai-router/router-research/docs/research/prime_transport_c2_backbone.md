# Prime Transport `C^2` Backbone

## Summary

This note records a bounded empirical result from the prime admissibility /
transport-law line of work.

We study prime tuple admissibility along the transport orbit

`n(j) = 5 + 6j`

with the quadruplet offsets

`[0, 2, 6, 8]`.

The current interpretation is:

- the exact finite-depth admissibility state is layered and torus-valued,
- each refinement prime contributes a new circle-valued phase fiber,
- the exact state can be written schematically as
  `Theta_r = (b, phi_1, phi_2, ..., phi_r)`,
- and a lower-dimensional complex quotient can be tested by asking whether its
  induced transport law remains stable across wheel scales.

The main result recorded here is that the exact layered phase-fiber state admits
an empirically reusable compressed quotient in `C^2` whose transport law
generalizes out of sample to a larger unseen wheel with only minimal
degradation.

## Exact Layered State

For the prime tuple admissibility system, the finite-depth exact state is not
treated as a single scalar phase. The working picture is a layered torus-bundle
state:

`Theta_r = (b, phi_1, phi_2, ..., phi_r)`.

Interpretation:

- `b` is the base phase,
- each `phi_m` is a cyclic refinement-layer coordinate,
- finite depth gives a torus-valued exact state,
- increasing the refinement depth adds another circle-like phase fiber.

This is the exact state description being compressed; the compressed quotient is
not the exact state itself.

## `C^2` Quotient Construction

The candidate compressed quotient was built from low-order torus harmonics on
the exact global phase/fiber chart, then reduced to two complex coordinates.

The specific unseen-wheel run used:

- train wheels: `W = 2310`, `30030`, `510510`,
- test wheel: `W = 9699690`,
- orbit: `n(j) = 5 + 6j`,
- offsets: `[0, 2, 6, 8]`,
- base phase: `b = j mod 35`,
- fiber phase: `f = floor(j / 35) mod (W / 6 / 35)`,
- torus modes:
  `exp(2πi (kb*b/35 + kf*f/fiber_mod))`,
  with `kb, kf in [-3, 3]`, excluding `(0, 0)`.

For each wheel:

1. build the complex torus feature matrix `X`,
2. center `X`,
3. compute the SVD,
4. take the top two complex components to get `z in C^2`,
5. fit the local linear transport law
   `z(j+1) ≈ A z(j)`.

Then all training-scale transition pairs were pooled to fit one shared
canonical transport matrix `A_*`.

## Shared Canonical Transport Law

The strongest result is the out-of-sample test on the unseen larger wheel
`W = 9699690`.

### Out-of-sample error

- scale-specific fit:
  `err_test = 0.278267864521`
- shared law:
  `err_star = 0.281905374440`
- ratio:
  `err_star / err_test = 1.013071972668`

So the shared law is only about `1.31%` worse than the scale-specific fit on
the unseen larger wheel.

### Eigenvalues

`A_test`:

- `0.983336576204 + 0.004837307237i`
- `0.936623750127 - 0.028572996370i`

`A_*`:

- `0.977207798310 + 0.008093722372i`
- `0.939197306619 - 0.039723900661i`

Conjugation-aligned distance between `A_*` and `A_test`:

- `0.013078988998`

### Interpretation

This supports the bounded statement:

> The exact layered phase-fiber state admits a compressed
> two-complex-dimensional quotient whose transport law is stable across
> multiple wheel scales and generalizes out of sample to a larger unseen wheel
> with only minimal degradation.

The result should be understood as a reusable compressed transport backbone for
the admissibility system.

## Residual Structure

The current empirical picture is not exact closure. A conservative framing is:

`z_{t+1} ≈ A_* z_t + epsilon_t`

where:

- `A_*` is the reusable cross-scale backbone,
- `epsilon_t` is unresolved local correction structure.

The remaining error should be treated as residual structure, not as evidence
that the quotient is useless.

## Comparison With Higher Quotient Dimensions

The `C^2` quotient currently appears to capture the most reusable cross-scale
transport backbone.

The working interpretation of the `C^3` and `C^4` comparisons is:

- higher quotient dimensions did not clearly improve universality,
- they may capture additional local detail rather than a cleaner reusable
  transport law.

This is why the current backbone claim is stated in terms of `C^2`.

## Non-claims

This note does **not** claim:

- a prime oracle,
- a solution to the prime problem,
- an exact closed-form recursion for admissibility,
- exact closure of the layered state in `C^2`,
- a production routing law for the broader AI system.

This is an empirical compression result for the admissibility transport system.

## Next Direction

The next useful direction is not to demand exact closure from the `C^2`
quotient. The better next step is to model the residual:

- keep `A_*` as the stable backbone,
- treat `epsilon_t` as structured local correction,
- ask whether the residual has its own smaller family of reusable coordinates or
  regime tags.

That is the appropriate follow-on if the goal is a durable hierarchical
transport model rather than an overfit exact surrogate.

## Artifact Locations

- script:
  [tools/prime_transport/c2_shared_astar_unseen_wheel_test.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/c2_shared_astar_unseen_wheel_test.py)
- summary:
  [results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_summary.csv)
- eigenvalues:
  [results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_eigenvalues.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_eigenvalues.csv)
- run note:
  [results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_note.md](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_note.md)
