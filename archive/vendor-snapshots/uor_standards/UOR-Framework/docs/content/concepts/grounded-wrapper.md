---
title: Grounded Wrapper
category: concepts
---

# `Grounded<T>` — Compile-Time Ground-State Guarantee

The `uor_foundation::enforcement::Grounded<T: GroundedShape>` wrapper carries
the compile-time witness that `op:GS_4` holds for the value it wraps:

- σ = 1
- freeRank = 0
- S = 0
- T_ctx = 0

A `Grounded<T>` can only be produced by the reduction pipeline (or by
`uor_ground!` macro expansion) — never by hand. The wrapper's private fields
and sealed `GroundedShape` supertrait guarantee this at the type-system level.

## What's inside

```rust
pub struct Grounded<T: GroundedShape> {
    validated: Validated<GroundingCertificate>,
    bindings: BindingsTable,
    witt_level_bits: u16,
    unit_address: u128,
    _phantom: PhantomData<T>,
}
```

- **`validated: Validated<GroundingCertificate>`** — the attested grounding
  certificate that reached `state:GroundedContext` via the pipeline's
  `stage_convergence`.
- **`bindings: BindingsTable`** — the static binding table materialized at
  compile time. `get_binding(query_address)` resolves in O(log n) via
  binary search; for truly statically-known addresses the read is zero-step
  per `op:GS_5`.
- **`witt_level_bits: u16`** — the Witt level the grounded value was minted at.
- **`unit_address: u128`** — FNV-1a content hash of the originating
  `reduction:CompileUnit`, used for memoization.
- **`_phantom: PhantomData<T>`** — ties the grounded value to a specific
  `ConstrainedType` at the type level. `Grounded<PixelQ8>` and
  `Grounded<MatrixRowQ32>` are distinct types; accidental cross-type
  substitution fails to compile.

## How to produce one

The `uor_ground!` macro is the consumer-facing entry point:

```rust
use uor_foundation::enforcement::prelude::*;
use uor_foundation_macros::{uor_ground, ConstrainedType};

#[derive(ConstrainedType, Default)]
#[uor(residue = 255, hamming = 8)]
struct Pixel;

let unit: Grounded<Pixel> = uor_ground! {
    compile_unit hello_pixel {
        root_term: { 0 };
        witt_level_ceiling: W8;
        thermodynamic_budget: 64.0;
        target_domains: { ComposedAlgebraic };
    } as Grounded<Pixel>
};
```

The trailing `as Grounded<Pixel>` clause is required so the macro can recover
the type parameter `T` at expansion time. The macro body runs the v0.2.1
reduction pipeline in-process, applies all 6 preflight checks and 7 reduction
stages, and produces the `Grounded<Pixel>` value via the sealed back-door
minting API.

## Sealed-constructor discipline

The `GroundedShape` supertrait is sealed through
`__macro_internals::GroundedShapeSealed`. The only way to implement it is via
`#[derive(ConstrainedType)]`, which emits the impl through the doc-hidden
back-door module. Any other impl attempt fails with a private-path error at
compile time.

The `cargo-uor` CLI's `uor::unsealed_grounded` lint (future release) will
additionally flag any direct call to the `__uor_macro_mint_grounded` back-door
function outside `uor-foundation-macros`-generated code.

## See also

- [Inhabitance Verdict](inhabitance-verdict.html) — the sibling
  `cert:InhabitanceCertificate` primitive.
- [Certify Trait](certify-trait.html) — the companion one-liner verdict API.
- Ontology: [`state:GroundedContext`](../namespaces/state.html) and
  [`op:GS_4`](../namespaces/op.html) — the foundation classes underpinning
  the sealed wrapper contract.
