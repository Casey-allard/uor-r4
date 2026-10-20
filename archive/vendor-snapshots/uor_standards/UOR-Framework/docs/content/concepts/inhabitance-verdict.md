---
title: Inhabitance Verdict
category: concepts
---

# Inhabitance Verdict

The v0.2.1 release introduces **carrier non-emptiness** as a first-class
verdict primitive sibling to `cert:CompletenessCertificate`. Together they
answer two distinct questions about a `type:ConstrainedType`:

- **`cert:CompletenessCertificate`**: does `freeRank = 0` on the minimal basis?
  (IT_7d: all Betti numbers zero, χ = n.)
- **`cert:InhabitanceCertificate`**: is the carrier non-empty? (At least one
  value tuple satisfies the constraint system.)

For constraint systems that admit multiple satisfying value tuples, an
`InhabitanceCertificate.verified` may be true while the corresponding
`CompletenessCertificate.verified` is false — the carrier is inhabited but
the minimal basis leaves sites free.

## Dispatch routing

`predicate:InhabitanceDispatchTable` routes every input to one of three
polynomial deciders based on the constraint system's shape:

1. **`predicate:Is2SatShape` → `resolver:TwoSatDecider`** (priority 0):
   Aspvall-Plass-Tarjan SCC decision in O(n+m).
2. **`predicate:IsHornShape` → `resolver:HornSatDecider`** (priority 1):
   unit propagation in O(n+m).
3. **`predicate:IsResidualFragment` → `resolver:ResidualVerdictResolver`**
   (priority 2, catch-all): returns `Err(InhabitanceImpossibilityWitness)`
   with `reduction:ConvergenceStall` for non-polynomial residual fragments.

Coverage is exhaustive by construction; `reduction:DispatchMiss` is
unreachable for this table.

## Identities

- **`op:IH_1`** (soundness): `InhabitanceCertificate(T).verified ⇔ carrier(T) ≠ ∅`.
- **`op:IH_2a`** (2-SAT cost, restricted): O(n+m) via classical 2-SAT.
- **`op:IH_2b`** (Horn-SAT cost, restricted): O(n+m) via unit propagation.
- **`op:IH_3`** (carrier preservation): basis reduction preserves the carrier.

Unrestricted `IH_2` would be equivalent to P = NP and is **not** adopted.

## One-liner consumer API

```rust
use uor_foundation::enforcement::prelude::*;
use uor_foundation_macros::ConstrainedType;

#[derive(ConstrainedType, Default)]
#[uor(residue = 255, hamming = 8)]
struct Shape;

let cert: Validated<InhabitanceCertificate> =
    InhabitanceResolver::new().certify(&Shape)?;
match cert.witness() {
    Some(witness_bytes) => println!("inhabited: {witness_bytes:?}"),
    None => println!("inhabited (verified, witness elided)"),
}
```

The Lean 4 counterpart exposes the same shape via `UOR.Enforcement.Certify`
and `UOR.Pipeline.runInhabitance`.

## See also

- [Grounded Wrapper](grounded-wrapper.html) — compile-time ground-state guarantee.
- [Certify Trait](certify-trait.html) — one-liner verdict API across all four
  resolver façades.
- Ontology: [`cert:InhabitanceCertificate`](../namespaces/cert.html),
  [`proof:InhabitanceImpossibilityWitness`](../namespaces/proof.html),
  [`predicate:InhabitanceDispatchTable`](../namespaces/predicate.html).
