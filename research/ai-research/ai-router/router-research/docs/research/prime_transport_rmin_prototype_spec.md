# Prime Transport Rmin Prototype Spec

## Purpose

This note specifies the first prototype target for the exact-layer prime
transport routing abstraction.

The recommended prototype state is:

- `R_min = (b, phi, r, next_return_gap)`

It is grounded in the current exact-layer result that `R_min` is the smallest
validated predictive refinement of the static chart that remains meaningfully
smaller than full spin.

## 1. Prototype Routing State

### State Definition

For orbit position `j`, define:

- `b`
  - base phase
  - current implementation anchor: `b = j mod 35`

- `phi`
  - refinement fiber coordinate at the current wheel depth
  - represented as the finite-depth layer tuple induced by the fiber modulus

- `r`
  - wheel depth / active refinement depth

- `next_return_gap`
  - the forward distance from the current orbit position to the next
    admissible-return event for the active tuplet pattern

So the prototype state is:

- `R_min(j) = (b(j), phi(j), r, next_return_gap(j))`

### Update Under Transport

Under the odometer transport update `j -> j + 1`:

- update `b` by cyclic increment modulo the base period
- update `phi` by the induced fiber odometer step at depth `r`
- keep `r` fixed unless a wheel lift is introduced
- update `next_return_gap` by advancing one step and recomputing the distance to
  the next admissible-return event

### Cached vs Recomputed

What should be cached:

- `b`
- `phi`
- `r`
- current `next_return_gap`

What may be recomputed on step:

- the next forward return distance after transport

The prototype should prefer caching the current state and using deterministic
one-step updates rather than recomputing the full future word.

## 2. Routing / Refinement Policy

### Default Coarse Routing

Default routing should begin with `R_min`, not full spin.

That means:

- route first by static chart sector `(b, phi, r)`
- refine that route with immediate forward return-memory `next_return_gap`

This matches the exact-layer finding that visible refinement is delayed and that
the smallest useful predictive refinement is a return-memory tag, not full
future-word state.

### Signals Of Unresolved Predictive Ambiguity

The prototype should treat ambiguity as unresolved when:

- multiple downstream behaviors remain grouped under the same `R_min` label
- collective first-splitting pressure is still present inside an `R_min` class
- deeper-lift cases show repeated divergence among states sharing the same
  `next_return_gap`

### Promotion Rule

Promotion should occur only when `R_min` fails to separate predictive branches
that matter for the routing target.

Conceptually:

- stay at `R_min` by default
- monitor unresolved predictive branching inside each `R_min` class
- promote only those unresolved classes toward a richer spin-like state

### Delayed Refinement

Delayed refinement should be implemented conceptually as:

1. coarse exact routing on the static chart
2. minimal predictive refinement via `next_return_gap`
3. selective promotion only when unresolved classes begin collective
   first-splitting

So refinement is not continuous or global. It is event-driven and local to the
still-unresolved predictive classes.

## 3. Smallest Prototype Experiment

No runtime integration is needed for the first prototype step.

The smallest benchmark plan should compare:

- `A. static chart only = (b, phi, r)`
- `B. minimal hybrid state = (b, phi, r, next_return_gap)`
- `C. full predictive state = (b, spin_H)`

The controlled evaluation should measure:

- predictive membership discrimination
- split-partition purity on the first-splitting support
- relative label complexity
- frequency of promotion events required beyond `R_min`

### Success Condition For `R_min`

`R_min` should count as a successful first prototype target if it:

- materially outperforms the static chart alone on predictive separation
- stays strictly smaller than full spin
- captures enough practical predictive structure that promotion is selective
  rather than near-constant

### Failure Condition

`R_min` should be judged insufficient if:

- promotion is needed almost everywhere
- deeper-lift rows repeatedly collapse multiple important predictive branches
  into the same `R_min` state
- the practical gain over the static chart is too small to justify the extra
  residual coordinate

## 4. Prototype Reading

The best current exact-layer reading is:

- `R_static` is too weak as a predictive routing abstraction
- `R_full` is the reference exact predictor but is too expensive as the first
  prototype target
- `R_min` is the correct first prototype target because it is the smallest
  current state that preserves a substantial part of the predictive structure
  while still respecting delayed refinement

## 5. Open Implementation Question

The main open design question for the first prototype is:

- how to detect unresolved branch pressure inside an `R_min` class using a
  small exact diagnostic, without immediately escalating to full spin

That is the next implementation-facing exact-layer problem after this spec.
