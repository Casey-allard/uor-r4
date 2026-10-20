---
title: Certify Trait
category: concepts
---

# The `Certify<I>` Trait

`uor_foundation::enforcement::Certify<I>` is the v0.2.1 verdict-producing
trait. Every resolver façade (`TowerCompletenessResolver`,
`IncrementalCompletenessResolver`, `GroundingAwareResolver`,
`InhabitanceResolver`) implements it with a specific `Certificate` /
`Witness` pair sourced from the ontology's `resolver:CertifyMapping`
individuals.

```rust
pub trait Certify<I: ?Sized> {
    type Certificate: OntologyTarget;
    type Witness: ImpossibilityWitnessKind;
    fn certify(&self, input: &I)
        -> Result<Validated<Self::Certificate>, Self::Witness>;
}
```

The trait is **generic over the input type** `I` so downstream user types
(via `#[derive(ConstrainedType)]`) flow directly through `.certify(&user_type)`.
The associated `Certificate` and `Witness` types remain sealed via
`OntologyTarget` / `ImpossibilityWitnessKind` so forged outcomes are
structurally impossible.

## The four resolver façades

| Façade | `Certificate` | `Witness` | Purpose |
|---|---|---|---|
| `TowerCompletenessResolver` | `LiftChainCertificate` | `GenericImpossibilityWitness` | Iterated lift-chain completeness; carries `target_level()` |
| `IncrementalCompletenessResolver` | `LiftChainCertificate` | `GenericImpossibilityWitness` | Single-step spectral-sequence lift |
| `GroundingAwareResolver` | `GroundingCertificate` | `GenericImpossibilityWitness` | Exploits `state:GroundedContext` for O(1) `op:GS_5` reads |
| `InhabitanceResolver` | `InhabitanceCertificate` | `InhabitanceImpossibilityWitness` | Carrier non-emptiness dispatch (2-SAT/Horn-SAT/residual) |

Each façade is emitted parametrically by `codegen/src/enforcement.rs` from
the corresponding `resolver:CertifyMapping` individual. Adding a new
resolver to the ontology regenerates the Rust + Lean 4 surface in lock-step.

## Consumer-facing one-liners

```rust
use uor_foundation::enforcement::prelude::*;
use uor_foundation::WittLevel;
use uor_foundation_macros::ConstrainedType;

#[derive(ConstrainedType, Default)]
#[uor(residue = 65535, hamming = 16)]
struct ContractionShape;

// "At what Witt level does this K-fold MAC admit representation?"
let cert: Validated<LiftChainCertificate> =
    TowerCompletenessResolver::new().certify(&ContractionShape)?;
let accum_level: WittLevel = cert.target_level();

// "Is this constrained type's carrier non-empty?"
let cert: Validated<InhabitanceCertificate> =
    InhabitanceResolver::new().certify(&ContractionShape)?;
let witness = cert.witness();
```

`Validated<LiftChainCertificate>` auto-derefs to `LiftChainCertificate`, so
`cert.target_level()` resolves through the generated
`cert:targetLevel` accessor without a manual `.inner()` call.

## Lean 4 parity

The Lean 4 counterpart exposes the same pattern via `UOR.Enforcement.Certify`:

```lean
import UOR.Enforcement
import UOR.Pipeline
open UOR.Enforcement

#check @Certify.certify
-- Certify.certify : {ρ I : Type} → [Certify ρ I] → ρ → I → Except ...
```

Four Lean `instance` declarations (one per resolver class) are emitted by
`lean-codegen/src/enforcement.rs` — the same parametric source as the Rust
façades.

## See also

- [Inhabitance Verdict](inhabitance-verdict.html) — the `InhabitanceResolver`
  dispatch table and decision semantics.
- [Grounded Wrapper](grounded-wrapper.html) — the `Grounded<T>` wrapper that
  `GroundingAwareResolver` produces.
- Ontology: [`resolver:CertifyMapping`](../namespaces/resolver.html) —
  the individuals that parametrize `Certify` impls from the ontology.
