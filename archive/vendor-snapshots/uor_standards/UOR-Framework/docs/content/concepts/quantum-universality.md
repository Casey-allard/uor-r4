# Witt Universality

## Definition

**Witt universality** is the property of an algebraic identity that holds
for all Witt levels n ≥ 1, not just at a specific W8 ring. An identity is
universally valid when it is provable symbolically from ring axioms rather than
verified exhaustively at one ring size.

The {@prop https://uor.foundation/op/universallyValid} boolean property
on an {@class https://uor.foundation/op/Identity} individual declares this
status. The critical identity `neg(bnot(x)) = succ(x)` is the canonical
example: it holds in Z/(2^n)Z for every n ≥ 1 and carries
`op:universallyValid true`.

## Witt Levels

The {@class https://uor.foundation/schema/WittLevel} newtype struct defines
an open class of Witt levels. Named levels include:

- **W8** — the base Witt level used for exhaustive verification (ring size
  = 2^8 in the UOR Foundation reference implementation).
- **W16** — the concrete ring Z/(2^16)Z, now formally typed as
  {@class https://uor.foundation/schema/W16Ring} with
  {@prop https://uor.foundation/schema/W16bitWidth} = 16 and
  {@prop https://uor.foundation/schema/W16capacity} = 65,536.
- W24, W32, ... — higher levels declared via the `schema:nextWittLevel` chain.

## WittLevelBinding

A {@class https://uor.foundation/op/WittLevelBinding} record links an
`op:Identity` individual to a specific Witt level at which the identity
has been verified. Because identities may be verified at multiple levels, the
{@prop https://uor.foundation/op/verifiedAtLevel} property is
non-functional: one binding per (Identity, WittLevel) pair.

Each binding carries a {@prop https://uor.foundation/op/bindingLevel}
pointing to the relevant WittLevel individual.

## Universal Identity Groups (QL_ series)

Amendment 26 adds seven QL\_ identity individuals (QL\_1 through QL\_7) that
generalize key algebraic, thermodynamic, topological, and pipeline identities
to all n ≥ 1. Each carries `op:universallyValid true` and a
`op:verificationDomain` typed assertion.

| Identity | Statement |
|----------|-----------|
| QL\_1 | neg(bnot(x)) = succ(x) in Z/(2^n)Z for all n ≥ 1 |
| QL\_2 | Ring carrier size is exactly 2^n |
| QL\_3 | Landauer erasure cost scales as n × k\_B T ln 2 |
| QL\_4 | Dihedral group D\_{2^n} action is faithful at all n |
| QL\_5 | Canonical form rewriting terminates at all levels |
| QL\_6 | χ(N(C)) = n completeness condition generalizes |
| QL\_7 | Euler characteristic of the ring topology = 1 − n |

## Related

- {@class https://uor.foundation/schema/WittLevel}
- {@class https://uor.foundation/schema/W16Ring}
- {@class https://uor.foundation/op/WittLevelBinding}
- [Type Completeness](type-completeness.html)
