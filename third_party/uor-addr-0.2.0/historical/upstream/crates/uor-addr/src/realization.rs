//! Per-realization σ-axis wiring macros.
//!
//! Every realization is identical except for its canonical-form input
//! carrier: the ψ-tower, the bounds, and the resolver tower are shared,
//! and the κ-derivation verb body is the same four-ψ composition
//! (`k_invariants ∘ homotopy_groups ∘ postnikov_tower ∘ nerve`). The only
//! per-σ-axis variation is the bound `H`, the per-axis output shape, and
//! the κ-label byte width.
//!
//! foundation 0.5.1's resolver tower admits only `Hasher<32>` axes, so a
//! realization wires the four 32-byte axes ([`crate::hash`]). These macros
//! emit the four per-axis verbs ([`addr_verbs!`]) and the four per-axis
//! [`PrismModel`](prism::pipeline::PrismModel) declarations
//! ([`addr_models!`]) from a single per-realization invocation, keeping the
//! per-realization source to the carrier handle + the public entry points.

/// Emit one κ-derivation `verb!` per σ-axis. `input` is the carrier type as
/// it appears in a `verb!` signature (e.g. `JsonCarrier<'_>` or
/// `RingElement`); each `{ shape, verb }` names the axis's output shape and
/// the generated verb function.
macro_rules! addr_verbs {
    (
        input: $input:ty,
        $( { shape: $shape:ty, verb: $verb:ident } ),+ $(,)?
    ) => {
        $(
            prism::pipeline::verb! {
                pub fn $verb(input: $input) -> $shape {
                    k_invariants(homotopy_groups(postnikov_tower(nerve(input))))
                }
            }
        )+
    };
}

/// Emit one `PrismModel` declaration per σ-axis, binding the shared
/// [`AddrBounds`](crate::bounds::AddrBounds) profile and the shared
/// [`AddressResolverTuple`](crate::resolvers::AddressResolverTuple)
/// ψ-tower. `input` is the carrier type as it appears in a `prism_model!`
/// `type Input` clause (e.g. `JsonCarrier<'a>` or `RingElement`); each
/// axis names its hasher, output shape, model + route structs, and the
/// verb function (in scope) the route delegates to.
macro_rules! addr_models {
    (
        input: $input:ty,
        $( {
            hasher: $hasher:ty,
            bounds: $bounds:ty,
            shape: $shape:ty,
            model: $model:ident,
            route: $route:ident,
            verb: $verb:ident
        } ),+ $(,)?
    ) => {
        $(
            prism::pipeline::prism_model! {
                pub struct $model;
                pub struct $route;
                impl PrismModel<
                    prism::vocabulary::DefaultHostTypes,
                    $bounds,
                    $hasher,
                    $crate::resolvers::AddressResolverTuple<$hasher>,
                    prism::pipeline::EmptyCommitment
                > for $model {
                    type Input = $input;
                    type Output = $shape;
                    type Route = $route;
                    fn route(input: Self::Input) -> Self::Output {
                        $verb(input)
                    }
                }
            }
        )+
    };
}

// Brought into crate scope via `#[macro_use] mod realization;` in lib.rs;
// every format module declared after it may invoke them unqualified.
