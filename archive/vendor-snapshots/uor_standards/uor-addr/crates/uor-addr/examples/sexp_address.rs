//! `uor-addr` — S-expression realization, basic surface example.
//!
//! Demonstrates [`uor_addr::sexp::address`] over a small Rivest
//! canonical S-expression and prints the resulting κ-label.
//!
//! Run with `cargo run -p uor-addr --example sexp_address`.

use uor_addr::sexp::{address, canonicalize, AddressFailure, AddressOutcome};

fn main() {
    println!("uor-addr — sexp realization basic surface\n");

    let cases: &[&[u8]] = &[
        b"()",
        b"5:hello",
        b"(a b c)",
        b"(message (from researcher) (timestamp 1700000000))",
    ];

    for raw in cases {
        match address(raw) {
            Ok(AddressOutcome { address, .. }) => {
                let canon = canonicalize(raw).expect("well-formed input canonicalises");
                println!(
                    "  raw:        {}",
                    core::str::from_utf8(raw).unwrap_or("<binary>")
                );
                println!(
                    "  canonical:  {}",
                    core::str::from_utf8(&canon).unwrap_or("<binary>")
                );
                println!("  κ-label:    {address}\n");
            }
            Err(AddressFailure::InvalidSExpr) => panic!("example input must be valid S-expression"),
            Err(AddressFailure::PipelineFailure) => {
                panic!("substrate-level shape violation — unreachable")
            }
        }
    }

    println!("OK — every S-expression value produced its 71-byte sha256:<64hex> κ-label.");
}
