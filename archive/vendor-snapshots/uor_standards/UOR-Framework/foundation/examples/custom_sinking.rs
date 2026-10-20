//! Target §3 example: downstream-authored `Sinking` implementations.
//!
//! Demonstrates the outbound-boundary discipline: a `Sinking` impl projects
//! a foundation-minted `Grounded<'static, T>` through a specific `ProjectionMap` kind
//! to a host-side output. The `&Grounded<'static, T>` input is structurally
//! unforgeable — sealed per §2, constructed only by `pipeline::run` — so
//! the "cannot launder unverified data outward" guarantee is carried by
//! the Rust type system.
//!
//! Run with: `cargo run --example custom_sinking -p uor-foundation`

use uor_foundation::enforcement::{
    BinaryProjectionMap, CompileUnitBuilder, ConstrainedTypeInput, Grounded, Sinking, Term,
    Utf8ProjectionMap,
};
use uor_foundation::pipeline::run;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

// ADR-060: `Term` carries the inline-carrier width `N` and is no longer `Sync`,
// so the term arena is `const` (not `static`).
const ROOT_TERMS: &[Term<'static, N>] = &[uor_foundation::pipeline::literal_u64(42, WittLevel::W8)];
static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

/// A Utf8-projection sink: renders a grounded value as a host-readable
/// string. `ProjectionMap = Utf8ProjectionMap` tags this impl as serving
/// the UTF-8 ontology kind.
struct WitnessReport;

impl Sinking<N> for WitnessReport {
    type Source = ConstrainedTypeInput;
    type ProjectionMap = Utf8ProjectionMap;
    type Output = String;

    fn project(&self, grounded: &Grounded<'_, ConstrainedTypeInput, N>) -> String {
        format!(
            "witt_bits={} unit_address={:?} sigma={:.6}",
            grounded.witt_level_bits(),
            grounded.unit_address(),
            grounded.sigma().value()
        )
    }
}

/// A binary-projection sink: extracts the fingerprint as raw bytes.
/// `ProjectionMap = BinaryProjectionMap` tags this impl at the type level.
struct FingerprintBytes;

impl Sinking<N> for FingerprintBytes {
    type Source = ConstrainedTypeInput;
    type ProjectionMap = BinaryProjectionMap;
    type Output = Vec<u8>;

    fn project(&self, grounded: &Grounded<'_, ConstrainedTypeInput, N>) -> Vec<u8> {
        grounded.content_fingerprint().as_bytes().to_vec()
    }
}

fn main() {
    // Step 1: build and validate a CompileUnit.
    let unit = CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(4096)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>()
        .validate()
        .expect("unit is well-formed");

    // Step 2: run the pipeline to mint a Grounded<'static, T>. This sealed value is
    // the only admissible input to a Sinking projection.
    let grounded = run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit)
        .expect("pipeline admits the unit");

    // Step 3: project through two different Sinking impls. Each one serves
    // a distinct ProjectionMap kind; the foundation contract ensures both
    // inputs are sealed Grounded<'static, _> values — nothing else is expressible.
    let report = WitnessReport.project(&grounded);
    let bytes = FingerprintBytes.project(&grounded);

    println!("WitnessReport → {report}");
    println!(
        "FingerprintBytes → {} bytes: {:02x?}",
        bytes.len(),
        &bytes[..bytes.len().min(8)]
    );

    // Step 4: the key discipline demonstrated — Sinking::project accepts only
    // &Grounded<'static, Source>. Downstream cannot call these functions on raw data;
    // the type system rejects it at compile time. See the compile_fail test
    // in phase_x6_sinking.rs for the anchor.
    println!(
        "\nTarget §3 guarantee upheld: both sinks consumed &Grounded<'static, {}>,\n\
         not raw primitives. The sealing of Grounded<'static, T> is the sole\n\
         structural assurance that unverified data cannot be laundered outward.",
        std::any::type_name::<ConstrainedTypeInput>()
    );
}
