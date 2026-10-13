# Prime Transport Mock Router Module Spec

## Purpose

This note defines the smallest offline router-module design that can host the
current exact-layer routing abstraction without touching runtime code.

The primary prototype state remains:

- `R_min = (b, phi, r, next_return_gap)`

with optional promotion toward:

- `R_full = (b, spin_H)`

## State Representation

The mock module should support three internal state forms.

### 1. Static Baseline

- `R_static = (b, phi, r)`

Use:

- coarse exact chart address only

### 2. Primary Prototype State

- `R_min = (b, phi, r, next_return_gap)`

Use:

- default routing state
- first predictive refinement above the static chart

### 3. Promoted Predictive State

- `R_full = (b, spin_H)`

Use:

- fallback state for unresolved classes that still exhibit predictive branching

## Update Functions

The module should implement deterministic transport updates only.

For `R_static`:

- increment base phase
- update fiber phase by the depth-`r` odometer rule
- keep `r` fixed between wheel lifts

For `R_min`:

- apply the `R_static` update
- update `next_return_gap` to the next admissible-return distance after the
  odometer step

For `R_full`:

- shift the predictive word by one step
- append the next admissibility bit

## Promotion Trigger

Promotion should be driven by unresolved predictive ambiguity, not by time or
depth alone.

The module should mark an `R_min` state as promotable when:

- multiple predictive branches remain grouped under the same `R_min` label
- split-partition ambiguity persists after the minimal return-memory refinement
- the class sits in the unresolved fraction that the offline evaluation
  attributes to `R_min`

The current exact-layer reading is that `R_min` leaves about `31%` of
split-partition cases unresolved on the bounded offline evaluation, so the
module should be designed with selective promotion as a normal path rather than
as an exceptional failure case.

## Fallback Path

The fallback path is:

1. start in `R_min`
2. evaluate unresolved predictive ambiguity
3. if unresolved, promote only that class to `R_full`
4. keep the rest of the routing population at `R_min`

This preserves the exact-layer principle of delayed refinement:

- coarse state first
- promote only when collective first-splitting remains unresolved

## Prototype Module Inputs / Outputs

### Inputs

The smallest offline module should accept:

- current orbit position or equivalent chart state
- wheel depth / wheel metadata
- active tuplet pattern
- optional current predictive state if already cached

### Outputs

The module should return:

- updated routing state
- route key or route score key
- promotion flag
- optional promoted state when promotion is triggered

## Minimal Offline API Shape

The smallest plausible mock API is:

- `initialize_state(...)`
- `update_state(...)`
- `score_or_route(...)`
- `should_promote(...)`
- `promote_state(...)`

Expected meanings:

- `initialize_state(...)`
  - build `R_static` or `R_min` from exact chart inputs

- `update_state(...)`
  - apply one deterministic odometer step to the current state

- `score_or_route(...)`
  - produce a stable routing key or grouping key from the current state

- `should_promote(...)`
  - test whether unresolved predictive ambiguity still exists inside the
    current `R_min` class

- `promote_state(...)`
  - replace `R_min` with `R_full` for that local unresolved class only

## Smallest Non-Runtime Prototype Step

The smallest non-runtime implementation step should be:

- a local offline module under the research tree that implements the five
  functions above
- evaluation only on the already-established exact rows and bounded offline
  summaries
- no integration into the live router seam

This is the right next step because it converts the current exact-layer design
into executable module structure without prematurely committing to production
interfaces.

## Recommendation

The first mock router module should:

- default to `R_min`
- expose explicit promotion into `R_full`
- keep `R_static` only as a baseline mode

That is the smallest implementation-ready step consistent with the current
exact-layer evidence.
