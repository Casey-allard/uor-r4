# Prime Transport Inner Representation Rebuild V7

## Purpose

Move radial/fiber/spin compatibility into the native transport law itself.

`r7` is not a post-hoc filter. It rebuilds the joint native transition
operator so that compatibility classes are part of which candidate transitions
are even compared.

## Compatibility Proxies Used

Using the audit definitions:

- radial/fiber class:
  - `(r, fiber_depth)`
  - `fiber_depth = 0` for repeated-prime semiprimes (`4, 9, 25`)
  - `fiber_depth = 1` for mixed semiprimes (`6, 10, 15`)

- spin class:
  - `(phi + sum(spin_bits) + left + right) mod 3`

- composite compatibility class:
  - shared-prime mask with the opposite composite over primes `2, 3, 5`

## Mechanism

`r7` keeps the native joint `(anchor, partner)` transition form from `r6`, but
adds a compatibility map inside the operator.

For each candidate `(anchor, partner)` pair, the operator computes a
compatibility tier:

1. exact radial/fiber match + spin match + compatibility-class match
2. exact radial/fiber match + compatibility-class match
3. exact spin match + compatibility-class match
4. compatibility-class match only
5. no compatibility-class match

At each step, the operator scores only the minimum available tier. So
compatibility is inside the transport decision itself, not applied afterward.

## Was compatibility moved into the native transport law itself?

yes

## Are we still comparing across incompatible radial/spin classes inside transport?

yes

`r7` reduces the incompatible comparison rate, but it does not eliminate it:

- selected radial/fiber mismatches: `50.82%`
- selected spin mismatches: `65.03%`
- selected composite-compatibility mismatches: `0.00%`
- average scored candidates per side-step: `2.0104`
- compared incompatible fraction among scored candidates: `79.56%`
- tier usage:
  - tier 0 (radial+spin+compat match): `20.44%`
  - tier 1 (radial+compat match): `28.74%`
  - tier 2 (spin+compat match): `14.53%`
  - tier 3 (compat-only): `36.29%`
  - tier 4 (compat mismatch): `0.00%`

## Bounded Evaluation

Same bounded v3 discourse/query task:

- `v3` approximate reference:
  - test accuracy `0.9978`
  - query accuracy `0.9878`
  - test loss `0.0077`
- `r4`:
  - test accuracy `0.7500`
  - query accuracy `0.7862`
  - test loss `0.5144`
- `r5`:
  - test accuracy `0.7503`
  - query accuracy `0.7927`
  - test loss `0.4760`
- `r6`:
  - test accuracy `0.7382`
  - query accuracy `0.7310`
  - test loss `0.6617`
- `r6` compatibility-filtered audit reference:
  - test accuracy `0.7436`
  - query accuracy `0.7518`
  - test loss `0.6512`
- `r7` compatibility-conditioned native transport:
  - test accuracy `0.7400`
  - query accuracy `0.7339`
  - test loss `0.6685`
- tiny transformer baseline:
  - test accuracy `0.6958`
  - query accuracy `0.6707`
  - test loss `0.7199`

## Exact Transport Rule

`r7` keeps the native joint `(anchor, partner)` transition structure from `r6`,
but changes the candidate set seen by the learned transport law itself.

For each admissible `(anchor, partner)` move:

1. compute the candidate semiprime and its compatibility proxies
2. assign it to one of the five compatibility tiers
3. find the minimum tier available at this step
4. score only candidates in that minimum tier with the learned native joint
   transport kernel
5. choose the highest-scoring candidate from that tier

This means compatibility is not enforced after scoring. It determines which
native moves are even compared inside the operator.

## Required Architecture Answers

### Is compatibility now inside the native transport operator itself?

Yes. The operator only exposes the best available compatibility tier to the
learned native joint transport kernel. Compatibility is part of the transport
law, not a post-hoc patch.

### Are incompatible radial/fiber/spin candidates still being scored?

Yes.

- radial/fiber incompatible selections still occur on `50.82%` of selected side
  transitions
- spin incompatible selections still occur on `65.03%` of selected side
  transitions
- among scored candidates, `79.56%` still have a radial or spin mismatch

The reason is structural and visible in the tier counts: many steps do not have
any candidate in tier 0, so the operator falls back to the best available
lower tier. Composite compatibility is fully enforced (`0.00%` tier-4 usage),
but radial/spin compatibility is only partially satisfiable in the current
bounded candidate space.

### Is admissibility still preserved by construction?

Yes.

Every candidate move still retains one of the current semiprime factors as
anchor, so the next composite always shares a factor with the previous
composite. `r7` changes which admissible candidates are compared, not the
admissibility guarantee itself.

### Does `r7` recover at least the gains seen in the compatibility-filtered audit?

No.

The filtered `r6` audit reference was:

- test accuracy `0.7436`
- query accuracy `0.7518`
- test loss `0.6512`

`r7` reaches:

- test accuracy `0.7400`
- query accuracy `0.7339`
- test loss `0.6685`

So moving compatibility into the native operator was architecturally correct,
but this first bounded implementation does not yet recover the full gain seen
from the audit-time constrained evaluation.

## What This Clarifies

`r7` is a useful rebuild step, but not a performance win.

It shows three concrete things:

1. composite compatibility was the easiest class constraint to internalize:
   - `r7` eliminates composite-compatibility mismatches in scored/selected
     transitions
2. radial/spin mismatch remains common even after compatibility conditioning:
   - the current admissible move set often does not contain a fully compatible
     candidate
3. the audit improvement was not just “add a filter and win”:
   - baking the filter logic into the operator is harder because the operator
     must still function when full compatibility is unavailable

## What Remains Approximate After `r7`

- the learned joint transport kernel is still shallow and additive
- radial/fiber/spin compatibility is tiered coarsely rather than modeled as a
  richer typed transport relation
- the admissible move set is still small enough that compatibility fallbacks
  happen often

## Bottom Line

`r7` does move compatibility into the native transport law itself, which is the
architecturally correct rebuild direction.

But it is still a negative empirical result:

- worse than `r5`
- worse than the compatibility-filtered `r6` audit reference
- only slightly above unfiltered `r6` on test accuracy, and worse on query
  accuracy and loss

So the bottleneck now looks more specific:

the rebuilt line needs a better compatibility-aware native move geometry, not
just a compatibility-gated version of the existing small candidate set.
