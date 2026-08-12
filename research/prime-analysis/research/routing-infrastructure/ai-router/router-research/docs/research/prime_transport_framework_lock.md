# Prime Transport Framework Lock

## Purpose

This document freezes the torus / fiber / spin / radial framework as an
executable contract for future prime-transport implementations.

It is not an experiment note.
It is not a rebuild note.
It is not a tuning note.

Its job is to make illegal comparisons and illegal transports non-representable
or explicitly rejected.

## Framework Basis

The exact-layer source of truth is:

- exact orbit index `j`
- exact chart `j -> (b, phi, r)`
- stronger predictive compression `(b, spin_H)`

From that, the framework has four native identity layers:

1. radial identity
2. fiber identity
3. spin identity
4. composite compatibility identity

These are not optional labels. They are the type system of lawful comparison
and lawful transport.

## 1. Native Class Identities

### 1.1 Radial class

Operational definition:

- `radial_class = r`

Meaning:

- wheel depth / radial bucket in the exact chart `j -> (b, phi, r)`

Current code status:

- present in `r5` / `r6` / `r7` as native state

### 1.2 Fiber class

Operational definition:

- `fiber_class = phi`

Meaning:

- refinement fiber phase in the exact chart `j -> (b, phi, r)`

Current code status:

- present in `r5` / `r6` / `r7` as native state

Important correction:

- the audit proxy `(r, fiber_depth)` is not the intended fiber class
- `fiber_depth` is only a weak surrogate for composite shape
- future code must not treat repeated-vs-mixed semiprime shape as the fiber
  coordinate itself

### 1.3 Spin class

Operational definition:

- `spin_class = spin_H`
- minimally, if a bounded implementation cannot carry full `spin_H`, it must
  carry an explicit declared truncation `spin_h` with fixed horizon `h`

Meaning:

- finite-horizon future admissibility word along the exact orbit
- the stronger predictive compression identified by the exact-layer results

Current code status:

- missing as intended variable
- current `spin_bits` in `r5` / `r6` / `r7` is only a bounded surrogate and is
  not the exact `spin_H`

Missing variable:

- explicit horizon-tagged admissibility word `spin_h` or `spin_H`

Framework consequence:

- any implementation that lacks explicit `spin_h` / `spin_H` is operating with
  an incomplete spin identity and must declare that deficiency at the type
  level

### 1.4 Composite compatibility class

Operational definition:

For a composite state

- `C = (query_semiprime, binding_semiprime)`

define

- `prime_support(s) = {p in {2,3,5} : p divides s}`
- `compat_mask(s, o) = prime_support(s) ∩ prime_support(o)`

and the native compatibility identity as the ordered pair

- `composite_compat_class = (compat_mask(query_semiprime, binding_semiprime), compat_mask(binding_semiprime, query_semiprime))`

Meaning:

- which prime fibers are shared across the coupled composite pair

Current code status:

- partially present as an audit-time and `r7` transport-time mask
- not yet formalized as a first-class state type

### 1.5 Composite shape class

This is not one of the four primary framework identities, but it must be
distinguished because the audit mixed it into “fiber.”

Operational definition:

- `shape_class = repeated_prime | mixed_semiprime`

computed from the factor pair:

- repeated-prime if `left == right`
- mixed-semiprime otherwise

Meaning:

- intrinsic composite morphology

Current code status:

- computable from current factor-pair state

Framework rule:

- `shape_class` may constrain transport legality
- it must not be substituted for `fiber_class`

## 2. Lawful Comparison Rules

Comparison means any direct scoring, ranking, distance, logit competition, or
argmax between two states or candidate moves.

### 2.1 Direct comparison rule

Two states are directly comparable only if all of the following match:

- same `radial_class`
- same `fiber_class`
- same declared spin horizon
- same `spin_class` value at that horizon
- same `composite_compat_class`

If any of these fail, direct comparison is forbidden unless an explicit lawful
lift exists.

### 2.2 Lawful lift rule

If classes differ, a comparison is lawful only through an explicit named lift.

Allowed lift names in this framework lock:

1. `radial_lift`
2. `fiber_refinement_lift`
3. `spin_truncation_lift`
4. `compatibility_normalization_lift`

No other lift is lawful unless added explicitly to this file.

#### `radial_lift`

Use only when comparing states across wheel depth.

Must preserve:

- `fiber_class`
- declared spin horizon
- spin word prefix of the shorter-depth state
- composite anchor factor identity
- composite compatibility class

If these preserved quantities cannot be computed, the lift is unavailable and
comparison is forbidden.

#### `fiber_refinement_lift`

Use only when comparing states across `phi`.

Must preserve:

- `radial_class`
- declared spin horizon
- composite anchor factor identity
- composite compatibility class

If the implementation has no explicit fiber lift map, comparison is forbidden.

#### `spin_truncation_lift`

Use only when one state has richer spin than the other.

Must preserve:

- `radial_class`
- `fiber_class`
- compatibility class
- prefix equality of the truncated spin word

Comparison by raw proxy modulo arithmetic is not a lawful lift.

#### `compatibility_normalization_lift`

Use only when states differ by representational encoding of the same prime
support.

Must preserve:

- exact prime-support mask
- retained anchor factor
- radial and fiber class
- spin class

If prime support differs, this lift is unavailable.

### 2.3 Forbidden comparison rule

Comparison is forbidden if:

- spin classes differ and no explicit `spin_truncation_lift` exists
- fiber classes differ and no explicit `fiber_refinement_lift` exists
- radial classes differ and no explicit `radial_lift` exists
- composite compatibility classes differ and no explicit
  `compatibility_normalization_lift` exists

Forbidden means:

- do not score
- do not rank
- do not include in candidate sets

## 3. Lawful Transport Rules

Transport means state evolution, not post-hoc repair.

### 3.1 Within-class lawful moves

Within-class transport is lawful only when all preserved identities remain
fixed:

- same `radial_class`
- same `fiber_class`
- same declared spin horizon
- same `spin_class`
- same `composite_compat_class`

and all move invariants below hold.

Required invariants for every lawful move:

1. retained anchor factor is one of the current composite factors
2. next composite shares the retained anchor factor
3. composite compatibility class is preserved
4. spin horizon is preserved
5. spin class is preserved unless an explicit spin lift is being executed

### 3.2 Lawful cross-class moves

Cross-class transport is lawful only if it is explicitly one of:

1. `radial_lift_transport`
2. `fiber_refinement_transport`
3. `spin_lift_transport`

Each must be represented as a named operator, not as an ordinary candidate in a
generic candidate set.

#### `radial_lift_transport`

Allowed only if:

- the move changes `r`
- anchor factor is preserved
- `fiber_class` is preserved
- `composite_compat_class` is preserved
- spin word is transported by an explicit lift map

#### `fiber_refinement_transport`

Allowed only if:

- the move changes `phi`
- `r` is preserved
- anchor factor is preserved
- compatibility class is preserved
- spin word is transported by an explicit refinement map

#### `spin_lift_transport`

Allowed only if:

- the move changes spin state by explicit admissibility-word update
- radial and fiber identities are preserved
- compatibility class is preserved
- the new spin state is produced by the declared spin update law, not by a
  modulo proxy

### 3.3 Forbidden moves

Forbidden transport includes:

- ordinary candidate moves that change `r` and `phi` without an explicit lift
- any move that changes spin class without an explicit spin update law
- any move that changes composite compatibility class during ordinary transport
- any move that substitutes `shape_class` for `fiber_class`
- any move that compares or selects among mixed-class candidates before a lift
  has made them comparable

## 4. Failure Rules

### 4.1 Reject candidate move

Reject immediately before scoring if a candidate:

- has different `radial_class` and is not tagged as `radial_lift_transport`
- has different `fiber_class` and is not tagged as `fiber_refinement_transport`
- has different `spin_class` and is not tagged as `spin_lift_transport`
- has different `composite_compat_class`

### 4.2 Raise explicit incompatibility / failure state

Raise explicit incompatibility if:

- no candidate remains after legality gating
- the implementation lacks the variable required to certify a class identity
  needed for legality
- a lift is required but the corresponding named lift operator is not present

Failure state label:

- `INCOMPATIBLE_CLASS_BOUNDARY`

### 4.3 Refuse fallback

The system must refuse fallback when:

- all remaining options after legality checks are illegal
- the only way forward is to score across incompatible classes

Do not:

- fall back to original argmax
- fall back to lowest-tier incompatible candidate
- silently ignore missing class identity

### 4.4 Refuse scoring

Scoring must be refused if:

- candidate and source are not directly comparable
- no lawful lift has been executed first

### 4.5 Refuse comparison

Comparison must be refused if:

- the implementation cannot state the class identity tuple for both objects
- the objects live at different spin horizons
- comparison would rely on a proxy in place of missing exact identity without
  explicit declaration

## 5. Minimum Executable Gate Set

Every future implementation must run these hard checks.

### 5.1 Before candidate generation

Check G1:

- state carries explicit
  `(radial_class, fiber_class, spin_identity, composite_compat_class)`

If not:

- stop with `INCOMPLETE_STATE_TYPING`

### 5.2 Before candidate scoring

Check G2:

- every candidate declares whether it is:
  - `within_class`
  - `radial_lift_transport`
  - `fiber_refinement_transport`
  - `spin_lift_transport`

If not:

- stop with `UNDECLARED_TRANSPORT_TYPE`

Check G3:

- ordinary candidates must match source on:
  - `radial_class`
  - `fiber_class`
  - `spin_class`
  - `composite_compat_class`

If not:

- reject candidate before scoring

### 5.3 Before transport selection

Check G4:

- no mixed legality pool

Meaning:

- legal within-class candidates and lift candidates may not be scored in the
  same argmax pool

If both exist:

- choose transport family first
- only then score within that family

### 5.4 Before state update

Check G5:

- selected candidate preserves required invariants for its declared transport
  type

If not:

- raise `INVALID_TRANSPORT_SELECTION`

### 5.5 After state update

Check G6:

- recompute class tuple and verify the selected transport type actually
  produced the promised class relation

If not:

- raise `POST_UPDATE_CLASS_VIOLATION`

## Were prior experimental branches allowed to violate the intended framework?

yes

`r5`, `r6`, and `r7` all allowed candidate scoring across class boundaries that
were never made lawful by an explicit lift.

## What exact invariant should have hard-stopped those branches?

The first invariant that should have hard-stopped them is:

- **No candidate may enter a shared scoring pool unless source and candidate
  have identical `(radial_class, fiber_class, spin_class,
  composite_compat_class)` or an explicit named lift has already mapped them
  into a common comparison class.**

This invariant comes before optimization, before readout, and before any
transport choice.

## Single next implementation step after framework lock

Implement one standalone legality gate module that computes the native class
tuple

- `(radial_class, fiber_class, spin_identity, composite_compat_class)`

for source and candidate states and refuses candidate scoring unless the direct
comparison rule or a named lawful lift rule is satisfied.
