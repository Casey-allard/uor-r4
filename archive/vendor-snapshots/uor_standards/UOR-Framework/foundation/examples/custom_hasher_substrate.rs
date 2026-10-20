//! v0.2.2 Phase Q.3 example: plug a custom `Hasher` substrate into the pipeline.
//!
//! The foundation doesn't ship a concrete `Hasher` impl — the production
//! recommendation is BLAKE3, but the trait is open so downstream chooses.
//! This example shows a minimal FNV-1a 16-byte (128-bit) substrate,
//! implementing the `Hasher` trait from scratch.
//!
//! Downstream production substrates would wrap a real crypto hash (BLAKE3,
//! SHA-256) and set `OUTPUT_BYTES = 32` for 256-bit fingerprints.
//!
//! Run with: `cargo run --example custom_hasher_substrate -p uor-foundation`

use uor_foundation::enforcement::{
    CompileUnitBuilder, ConstrainedTypeInput, Grounded, Hasher, Term, Validated,
};
use uor_foundation::pipeline::run;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

/// Minimal FNV-1a 128-bit substrate — two 64-bit FNV-1a lanes for pedagogical
/// simplicity. Production deployments use BLAKE3 or SHA-256.
#[derive(Clone, Copy)]
pub struct MyFnv1aHasher {
    lane_a: u64,
    lane_b: u64,
}

impl Hasher for MyFnv1aHasher {
    const OUTPUT_BYTES: usize = 16;

    fn initial() -> Self {
        Self {
            // FNV-1a offset bases (lane A = offset, lane B = offset rotated).
            lane_a: 0xcbf29ce484222325,
            lane_b: 0x84222325cbf29ce4,
        }
    }

    fn fold_byte(mut self, b: u8) -> Self {
        // FNV-1a prime = 0x100000001b3.
        self.lane_a ^= b as u64;
        self.lane_a = self.lane_a.wrapping_mul(0x100000001b3);
        self.lane_b ^= (b as u64).rotate_left(8);
        self.lane_b = self.lane_b.wrapping_mul(0x100000001b3);
        self
    }

    // 32 = the `Hasher` finalize buffer width (the `FINGERPRINT_MAX_BYTES`
    // value of a typical `HostBounds`). ADR-060 removed `DefaultHostBounds`:
    // every application declares its own `impl HostBounds`. Applications
    // selecting a different fingerprint width declare
    // `impl Hasher<{<MyBounds as HostBounds>::FINGERPRINT_MAX_BYTES}>`.
    fn finalize(self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&self.lane_a.to_be_bytes());
        buf[8..16].copy_from_slice(&self.lane_b.to_be_bytes());
        // Bytes 16..32 stay zero (OUTPUT_BYTES = 16).
        buf
    }
}

// ADR-060: `Term` carries the inline-carrier width `N` and is no longer `Sync`,
// so the term arena is `const` (not `static`).
const ROOT_TERMS: &[Term<'static, N>] = &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn main() {
    let builder = CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(1024)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let unit: Validated<_> = builder.validate().expect("unit well-formed");
    let grounded: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, MyFnv1aHasher, N, 32>(unit).expect("custom hasher pipeline");

    println!("Content fingerprint (using custom FNV-1a):");
    println!(
        "  width: {} bytes",
        grounded.content_fingerprint().width_bytes()
    );
    println!(
        "  buffer: {:?}",
        &grounded.content_fingerprint().as_bytes()[..16]
    );
}
