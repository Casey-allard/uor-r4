# Prime Transport Grouped-Packet Hypothesis

## Purpose

This note records the next bounded research question in the prime
admissibility / transport-law line.

The goal is not to replace the current exact layered-state interpretation and
not to overstate the role of earlier group-theoretic language. The goal is to
state, conservatively, what should be tested next.

## Current Ground Truth

We study admissibility along the transport orbit

`n(j) = 5 + 6j`.

The current exact finite-depth state is the layered phase-fiber state

`Theta_r = (b, phi_1, phi_2, ..., phi_r)`,

with the following interpretation:

- `b` is the base phase,
- each `phi_m` is a cyclic refinement-layer coordinate,
- finite depth is torus-valued,
- each refinement adds a new circle-valued phase fiber.

This layered torus-valued state is the exact finite-depth object currently in
view.

Separately, the empirical compression result already recorded is:

- a shared compressed `C^2` transport law
  `z(j+1) ≈ A_* z(j)`
- trained across `W = 2310, 30030, 510510`
- generalized to unseen `W = 9699690`
- with only small degradation.

The clean interpretation remains:

- the exact layered torus state is real,
- the `C^2` quotient is a compressed transport backbone,
- the remaining mismatch is residual structure,
  `z_{t+1} ≈ A_* z_t + epsilon_t`.

## What Is Being Hypothesized

The new hypothesis is not that the full exact state is literally `SU(2)`, and
not that the empirical transport law has already been proved to arise from a
group symmetry.

The more limited hypothesis is:

- each refinement layer contributes its own phase fiber,
- earlier SU-style language may have been functioning as a compact storage or
  packaging language for per-layer fiber data,
- these per-layer packets may compose into the observed global compressed
  `C^2` backbone,
- so the next question is whether the global `C^2` quotient can be recovered
  or explained from composition of per-layer grouped complex packets rather
  than by fitting directly from the full torus state.

In this note, "SU-style" means only a candidate packet/composition formalism
for small normalized complex objects. It does **not** mean a proved exact
symmetry statement about the admissibility system.

## Distinctions To Preserve

The following four objects should remain distinct:

1. exact layered torus state:
   `Theta_r = (b, phi_1, ..., phi_r)`
2. grouped packet hypothesis:
   a candidate low-dimensional storage language for layer fibers
3. empirical `C^2` backbone:
   the fitted reusable transport quotient already observed
4. unresolved residual:
   the remaining local correction structure `epsilon_t`

Collapsing these into one claim would be premature.

## Formal Question

The next theorem-shaped / experiment-shaped question is:

> Can each refinement layer's phase fiber be encoded as a small normalized
> complex packet, and can the global compressed `C^2` transport law be
> recovered or approximated by composing these layer-packets across depth?

More explicitly, for finite depth `r`, ask whether there exist:

- per-layer encoders `E_m : S^1 -> P`,
  where `P` is a small normalized complex packet space,
- a depthwise composition rule
  `C_r : P^r -> C^2`,
- and possibly a shared normalization/projection step,

such that the resulting composed coordinates

`z_hat_r(j) = C_r(E_1(phi_1(j)), ..., E_r(phi_r(j)))`

either:

- approximate the already constructed quotient coordinates `z(j)`, or
- directly induce an approximate shared transport law close to `A_*`.

This is the object to test. It is a recoverability / explanatory-compression
question, not yet a theorem claim.

## Minimal Candidate Object

The first packet family should stay small.

A reasonable first candidate is a normalized two-complex-component layer packet

`p_m(phi_m) = (exp(i phi_m), exp(-i phi_m)) / sqrt(2)`.

Reasons:

- it preserves one circle-valued phase in a small normalized complex object,
- it is simple enough to compose without introducing many free parameters,
- it is compatible with earlier "small grouped complex packet" intuition
  without asserting exact group structure.

This is only the first storage candidate, not a privileged final form.

## First Composition Rule To Test

The first composition rule should also remain simple and auditable.

Use a normalized additive mixing map over layers:

`q_r(j) = normalize(sum_{m=1}^r w_m p_m(phi_m(j))))`

with small deterministic weights `w_m`, for example:

- uniform weights,
- geometric decay by depth,
- or depthwise weights fitted on train wheels only.

Then map the mixed packet into a candidate backbone coordinate by one fixed
linear projection

`z_hat(j) = B q_r(j)`,

with `B` learned only from training scales.

This creates a minimal packet-composition experiment:

- per-layer packet encoder,
- depthwise composition,
- final `C^2` projection.

It is intentionally weaker than trying to learn an arbitrary whole-state map in
one step.

## First Experiment

### Objective

Test whether a composed packet representation built from a small number of
layer fibers can recover a useful fraction of the existing `C^2` quotient or
its transport law.

### Proposed procedure

1. Choose a finite layer depth `r` on wheels where exact finite-depth fibers
   are already available or can be reconstructed.
2. Encode each layer phase `phi_m(j)` as a small normalized packet
   `p_m(phi_m(j))`.
3. Compose packets across depth using one fixed tentative rule, starting with
   normalized weighted summation.
4. Fit a linear map from the composed packet coordinates to the existing
   quotient coordinates `z(j)` on the train wheels.
5. Evaluate:
   - coordinate recovery error:
     `||z - z_hat|| / ||z||`
   - transport recovery error:
     compare `z_hat(j+1)` with `A_* z_hat(j)`
   - whether train-scale-fitted packet maps remain stable on the unseen wheel.

### Success condition

Treat the packet idea as promising only if:

- a low-parameter packet/composition family reproduces a nontrivial fraction of
  the existing `C^2` coordinates or transport behavior,
- and does so across multiple wheel scales without immediate collapse.

### Failure condition

Treat the packet idea as currently unsupported if:

- packet composition adds complexity but does not recover the `C^2` quotient
  meaningfully,
- or if recovery works only in a scale-local way and does not preserve the
  cross-scale backbone picture.

Either outcome is useful.

## Implementation Plan

The first implementation should be kept deliberately small.

### Step 1

Define one layer-packet encoder:

- baseline candidate:
  `p(phi) = (exp(i phi), exp(-i phi)) / sqrt(2)`

### Step 2

Define one composition rule:

- baseline candidate:
  normalized weighted sum across layers followed by a fixed `C^2` projection

### Step 3

Compare the composed packet coordinates against one of:

- the existing `C^2` quotient coordinates,
- or the shared transport law `A_*`.

### Step 4

Report conservatively:

- promising,
- mixed / partially explanatory,
- or unsupported.

The first pass should not attempt a full derivation of `A_*`, and should not
attempt to absorb the residual `epsilon_t` yet.

## Non-claims

This note does **not** claim:

- the exact state is `SU(2)`,
- the full system has been reduced to a proved group recursion,
- the grouped-packet formalism already explains primes,
- the residual structure has disappeared,
- or the transport backbone has been derived rather than fitted.

It records only the next bounded research question.

## Relation To The Existing `C^2` Backbone Result

This grouped-packet hypothesis should be treated as a follow-on explanation test
for the backbone already observed.

The `C^2` backbone remains the empirical anchor.

The packet question is whether a structured per-layer composition mechanism can
reproduce or explain that backbone more parsimoniously than a direct global fit.
