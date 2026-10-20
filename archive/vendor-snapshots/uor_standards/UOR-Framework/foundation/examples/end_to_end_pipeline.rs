//! v0.2.2 Phase Q.3 example: build and ground a CompileUnit end-to-end.
//!
//! Full flow:
//! 1. `CompileUnitBuilder::new()` with all 5 required fields
//! 2. `builder.validate()` → `Validated<CompileUnit<'a>, Runtime>`
//! 3. `pipeline::run::<T, _, H>(unit)` → `Grounded<'static, T>`
//! 4. Inspect the `BaseMetric` accessors on `Grounded<'static, T>`: `triad()`, `betti()`,
//!    `sigma()`, `d_delta()`, `residual()`, `uor_time()`.
//!
//! Run with: `cargo run --example end_to_end_pipeline -p uor-foundation`

use uor_foundation::enforcement::{
    CompileUnitBuilder, ConstrainedTypeInput, Grounded, Term, Validated,
};
use uor_foundation::pipeline::run;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

// ADR-060: `Term` carries the inline-carrier width `N` and is no longer `Sync`,
// so the term arena is `const` (not `static`).
const ROOT_TERMS: &[Term<'static, N>] = &[uor_foundation::pipeline::literal_u64(42, WittLevel::W8)];
static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn main() {
    // Step 1: build a fully-specified CompileUnit.
    let builder = CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(4096)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>();

    // Step 2: validate (runtime-phase).
    let unit: Validated<_> = builder.validate().expect("unit is well-formed");

    // Step 3: run through the pipeline.
    let grounded: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit)
            .expect("pipeline admits the unit");

    // Step 4: inspect the BaseMetric accessors.
    println!("Witt level bits: {}", grounded.witt_level_bits());
    println!("Unit address (content-hash): {:?}", grounded.unit_address());
    println!(
        "Content fingerprint width: {} bytes",
        grounded.content_fingerprint().width_bytes()
    );
    println!("Sigma (σ): {}", grounded.sigma().value());
    println!("Triad: present at witt_bits={}", grounded.witt_level_bits());
    println!(
        "UorTime: landauer={:?}, steps={}",
        grounded.uor_time().landauer_nats(),
        grounded.uor_time().rewrite_steps()
    );
}
