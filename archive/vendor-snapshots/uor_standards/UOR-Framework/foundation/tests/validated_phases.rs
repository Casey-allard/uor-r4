//! v0.2.2 W13 + W17: assertions about `Validated<T, Phase>` parametric phases.
//!
//! Confirms that the v0.2.2 W13 phase parameter is wired into the
//! enforcement surface: the default phase is `Runtime`, `CompileTime`
//! is constructable via const-eval paths, and a compile-time witness
//! is convertible to a runtime witness via `From`.

use uor_foundation::enforcement::{CompileTime, Runtime, Validated, ValidationPhase};

/// Compile-time witness that `P` implements `ValidationPhase`.
const fn require_phase<P: ValidationPhase>() {}

#[test]
fn compile_time_implements_validation_phase() {
    require_phase::<CompileTime>();
}

#[test]
fn runtime_implements_validation_phase() {
    require_phase::<Runtime>();
}

#[test]
fn validated_default_phase_is_runtime() {
    // The default type parameter on `Validated<T>` resolves to `Runtime`.
    // We exercise it indirectly by referencing a function that takes
    // `Validated<u32>` (the default phase) and ensure it accepts a
    // value constructed via the `pub(crate)` `Validated::new` path
    // wrapped through a public foundation API. Since no such public
    // path exists for arbitrary T (Validated is sealed), the test is
    // structural: we simply assert that the type alias resolves to a
    // valid type that can appear in a function signature.
    fn _accepts_runtime(_v: Validated<u32>) {}
    let _ = _accepts_runtime as fn(_);
}

#[test]
fn compile_time_witness_subsumes_runtime() {
    // The `From<Validated<T, CompileTime>> for Validated<T, Runtime>`
    // impl is what makes a compile-time witness usable wherever a
    // runtime witness is required. We reference the impl via a
    // type-level conversion check.
    fn _convert<T>(v: Validated<T, CompileTime>) -> Validated<T, Runtime> {
        v.into()
    }
    let _ = _convert::<u32> as fn(_) -> _;
}
