# Prime Transport spin_H Recursive Closure Spec

## Purpose

This step specifies the missing recursive-closure component of `spin_H` that is
required so the transport identity becomes canonical under repeated lawful
operator iteration.

This is not an implementation step. It defines what current
`spin_H_candidate = (phi, r, spin_h4)` fails to carry, and what must be added
next before more operator work.

## Current Observed Failure

From the bounded lawful operator surface:

- primary states examined:
  - `15`
- distinct transport identities reached:
  - `131`
- collision count:
  - `0`
- ambiguity count:
  - `15`
- recursive consistency rate:
  - `0`

So:

- the lift is deterministic per operator state
- the lift is collision-free on the bounded surface
- but every reachable primary chart `(b, phi, r)` still branches into multiple
  lawful transport identities

Therefore the failure is not descriptive collision. The failure is recursive
non-closure.

## 1. What Exactly Is Branching?

Branching is the existence of multiple lawful transport identities for the same
primary chart under repeated lawful iteration.

Observed form:

- the same `(b, phi, r)` supports between `7` and `9` distinct transport
  identities
- the variation is almost entirely in the spin word
- angular identity `b` is fixed
- fiber identity `phi` is fixed
- radial identity `r` is fixed

So the branching is:

- not radial evolution
- not fiber evolution
- not primary-chart drift
- but hidden transport-path variation that survives as different lawful spin
  states when the system returns to the same primary chart

Classified concretely, the branching is best described as:

- operator-word history inside the stabilizer of the primary chart

More explicitly:

- `I`, `T_x`, `T_y`, and certain bounded composites with `T_c` and `T_z'`
  can change hidden operator-side state without uniquely determining a new
  primary chart
- when the same `(b, phi, r)` is revisited, those hidden path differences are
  not recorded by `(phi, r, spin_h4)` canonically
- therefore multiple lawful transport images remain possible

## 2. What Information Is Missing From Current spin_H_candidate?

Current `spin_H_candidate = (phi, r, spin_h4)` fails to record the recursive
transport-phase class of the lawful operator word that produced the current
spin-side state inside the primary-chart stabilizer.

What it does record:

- angle/fiber location:
  - `phi`
- radial / unfolding depth:
  - `r`
- short predictive admissibility pattern:
  - `spin_h4`

What it does not record:

- whether the current state arrived through an even or odd swap history
- whether twist has been toggled an even or odd number of times
- whether the current spin transport lies in the base spin orbit or a shifted
  orbit induced by prior lawful lift action

So the missing information is not more future bits. It is hidden recursive
transport phase.

## 3. Smallest Missing Recursive-Closure Variable

The single missing component is:

- `tau`, the recursive transport-phase class

`tau` is the residue class of the lawful operator word modulo primary-chart
stabilizer equivalence.

Operationally, `tau` is the smallest extra transport-side variable that
distinguishes two states when:

- they have the same `(b, phi, r)`
- they have lawful transport support
- but they lie in different recursive transport branches

For the current bounded algebra, `tau` should be implemented as one structured
variable whose value updates under generator action. It is one variable even if
its internal representation is tuple-valued.

## 4. Recursive Update Law For tau

`tau` must update under lawful transport, not by post-hoc labeling.

Required update law:

- under `I`:
  - `tau' = tau`
- under `T_b`:
  - `tau' = tau`
  - reason: pure base advance changes pointing but does not change stabilizer
    branch class
- under `T_x`:
  - `tau' = swap_phase(tau)`
  - reason: composite ordering changes while the primary chart may remain the
    same, so branch class must record swap residue
- under `T_c`:
  - `tau' = coupled_phase(tau, query_semiprime, binding_semiprime)`
  - reason: coupled kick depends on the ordered composite pair and changes the
    transport branch even when primary coordinates later return
- under `T_y`:
  - `tau' = twist_phase(tau)`
  - reason: twist is hidden from the current lift but changes future spin
    transport law
- under `T_z'`:
  - `tau' = lift_phase(tau, phi, spin_h4)`
  - reason: the lawful lift changes fiber class and spin transport orbit, so
    recursive branch phase must advance with the lift

The exact coding of `swap_phase`, `coupled_phase`, `twist_phase`, and
`lift_phase` is an implementation step for later. The specification point is:

- `tau` must be a recursively updated state variable
- `tau` must absorb hidden lawful branch information
- `tau` must be part of transport identity, not inferred afterward

## 5. Why This Is The Right Next Primitive

This is more justified than:

- longer prefix length:
  - because the observed failure is branching at fixed `(b, phi, r)`, not lack
    of future-word length alone
- better representative selection:
  - because the active lift is already deterministic per operator state
- more operator tweaking:
  - because the operator is not the source of the ambiguity; the missing
    closure variable is

The data already shows:

- zero primary-state collisions
- full preservation of angular/radial/predictive fields
- complete failure of canonical recursive closure

So the smallest correct next primitive is the hidden recursive transport-phase
class, not more horizon or more local operators.

## Required Honesty Section

### Does current spin_H_candidate have recursive closure?

no

### What exact component is missing for recursive closure?

Current `spin_H_candidate` is missing the recursive transport-phase class
`tau` that records lawful stabilizer-path residue under repeated operator
iteration.

## Single Next Step

Implement the smallest `spin_H` augmentation that adds `tau`, the recursive
transport-phase class, as an explicit transport-side state component with
generator-by-generator update rules under `I`, `T_b`, `T_x`, `T_c`, `T_y`,
and `T_z'`.
