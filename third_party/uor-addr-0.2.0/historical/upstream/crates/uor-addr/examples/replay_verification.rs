//! 04 — TC-05 replay verification (anamorphism).
//!
//! The mint path runs the full ψ-pipeline and the SHA-256 axis. The
//! verify path runs neither — it consumes the [`uor_addr::AddressWitness`]
//! the mint produced and replays it through
//! [`uor_addr::AddressWitness::verify`], re-deriving the κ-label from the
//! witness's content fingerprint without re-invoking the canonical hash
//! axis on the original input.
//!
//! This is the wiki TC-05 round-trip a third-party verifier exercises.
//!
//! Demonstrates conformance contract `CL-R01` / `CL-R02`.
//!
//! Run:
//!
//! ```bash
//! cargo run -p uor-addr --example replay_verification
//! ```

use uor_addr::json::address;

fn main() {
    let payload =
        br#"{"agent":"researcher","output":{"timestamp":1700000000,"summary":"signal detected"}}"#;
    println!("uor-addr — TC-05 replay round-trip\n");
    println!("  payload:  {}\n", std::str::from_utf8(payload).unwrap());

    // ─── Mint path ────────────────────────────────────────────────
    let outcome = address(payload).expect("valid JSON");
    let mint_fingerprint = outcome.witness.content_fingerprint();
    println!("  mint:     address = {}", outcome.address);
    println!("            content_fingerprint = {mint_fingerprint:?}\n");

    // ─── Verify path ─────────────────────────────────────────────
    // `witness.verify()` re-derives the κ-label **without** invoking the
    // canonical hash axis on the original input. The recovered label is
    // bit-identical to the minted address (QS-05 replay equivalence).
    let recovered = outcome.witness.verify().expect("witness must verify");
    println!("  verify:   recovered κ-label = {recovered}");

    assert_eq!(
        recovered, outcome.address,
        "QS-05 replay equivalence broken"
    );
    println!("\nOK — verifier re-derived the bit-identical κ-label without hashing.");
}
