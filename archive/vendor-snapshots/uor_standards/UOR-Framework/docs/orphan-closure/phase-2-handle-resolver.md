# Phase 2: Path-1 orphan closure via Null stubs

## Contract

For every class classified `Path1HandleResolver` by Phase 0, codegen
emits a companion `Null{Class}<H: HostTypes>` struct and an
`impl {Class}<H> for Null{Class}<H>` that provides absent-sentinel
defaults for every accessor.

The orphan is closed mechanically: `Null{Class}<H>` is a concrete type
that satisfies the ontology-derived trait. Downstream implementers can
still provide their own concrete types; `Null{Class}<H>` is the
"resolver-absent" reference implementation.

## Design rationale

The plan's original Handle/Resolver/Resolved-wrapper design ran into
type-system issues — associated-type lifetimes require the accessor's
return site to hold a reference to stored data, which conflicts with the
handle's content-addressed-only storage.

The pragmatic working design is the `NullPartition` pattern already
established by the Product/Coproduct Amendment §D1.2: a unit struct
carrying `PhantomData<H>`, with associated-const `ABSENT` serving as the
lazy reference target for trait-typed accessors. Zero storage, no
cycles, sound lifetimes.

Handle/Resolver/Record infrastructure (for host-supplied content-
addressed resolution) is **deferred** — if downstream consumers need
it, a future phase can add it on top of the Null-stub foundation. The
orphan-closure metric (R1) is satisfied by the Null-stub alone.

## Emission per Path-1 class

Per `Path1HandleResolver` class `{Foo}`:

```rust
/// Phase 2 (orphan-closure): resolver-absent default for `{Foo}<H>`.
/// Every accessor returns `H::EMPTY_*` sentinels or cascades through
/// sibling `Null*` stubs. Implements `Default` so downstream can
/// construct freely.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Null{Foo}<H: HostTypes> {
    _phantom: core::marker::PhantomData<H>,
}

impl<H: HostTypes> Default for Null{Foo}<H> {
    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }
}

impl<H: HostTypes> Null{Foo}<H> {
    /// Absent-value sentinel — reference target for trait-typed accessors.
    pub const ABSENT: Null{Foo}<H> = Null{Foo} { _phantom: core::marker::PhantomData };
}

impl<H: HostTypes> {Foo}<H> for Null{Foo}<H> {
    // For each trait method:
    //   Scalar Datatype → return zero/EMPTY_*
    //   `&H::HostString` → return H::EMPTY_HOST_STRING
    //   `H::Decimal` → return H::EMPTY_DECIMAL
    //   `&H::WitnessBytes` → return H::EMPTY_WITNESS_BYTES
    //   Non-functional slice → return &[]
    //   Enum → return first variant (deterministic default)
    //   Object accessor to ontology class C:
    //     type {assoc_name} = Null{C}<H>;
    //     fn method(&self) -> &Self::{assoc_name} { &<Null{C}<H>>::ABSENT }
    //   Self-referential object accessor: fn method(&self) -> &Self::X { &Self::ABSENT }
}
```

## Accessor-return table (R4 per-type mapping)

| Trait return type | Null-stub body |
|---|---|
| `&H::HostString` | `H::EMPTY_HOST_STRING` |
| `&H::WitnessBytes` | `H::EMPTY_WITNESS_BYTES` |
| `H::Decimal` | `H::EMPTY_DECIMAL` |
| `u64` / `i64` / `u32` / `i32` | `0` |
| `bool` | `false` |
| `&[T]` | `&[]` |
| `{EnumClass}` | `{EnumClass}::default()` |
| `&Self::Assoc` (ontology class range) | `&<Null{RangeClass}<H>>::ABSENT` |
| `Self::Assoc` (by-value, partition-factor) | `<Null{RangeClass}<H>>::default()` |

## Why Null instead of Resolved wrapper

- **Null closes orphans directly** — `R1` counts `impl {Foo}<H> for Null{Foo}<H>` as a closure match.
- **No cyclic storage** — `PhantomData<H>` is zero-sized and doesn't participate in type recursion.
- **Associated-const reference target** — `&<Null{X}<H>>::ABSENT` yields a `'static`-lifetime borrow, satisfying `&Self::Assoc` returns without inline storage.
- **No trait signature changes** — preserves every existing `&Self::Assoc` return without the §1d-style by-value conversion, avoiding a ~450-trait breaking refactor.
- **Mirrors existing precedent** — the Product/Coproduct Amendment's `NullPartition<H>` family works identically. Phase 2 generalizes that pattern to every Path-1 class.

## Tests

- **`codegen/tests/path1_null_emission.rs`** — for every Path-1 class, the emitted output contains `pub struct Null{X}<H: HostTypes>`, `impl Default for Null{X}<H>`, `pub const ABSENT:`, and `impl {X}<H> for Null{X}<H>`.
- **`foundation/tests/orphan_closure_path1_stubs.rs`** — for a sample of Path-1 classes, `Null{X}::<DefaultHostTypes>::default()` compiles and every accessor invocation returns without panic.
- **Conformance `rust/orphan_counts`** — Path-1 count (the trait is closed) decreases by `CLASSIFICATION_PATH1`.

## What's NOT in this phase

- `{Foo}Handle<H>` / `{Foo}Resolver<H>` / `{Foo}Record<H>` — deferred. The Null-stub gives absent-data defaults; a future phase can layer content-addressed resolution on top.
- By-value refactor for non-partition accessors — not needed. Associated-const reference-target semantics cover it.
- `Resolved{Foo}<'r, R, H>` wrapper type — deferred alongside Handle/Resolver/Record.
