//! 02 — Deduplication / cache-key derivation.
//!
//! Three semantically-equivalent JSON payloads — same data, different
//! syntactic noise (key order, whitespace, Unicode normalisation form)
//! — collapse to a single content address. This is the cache-key
//! invariance an HTTP API can rely on for memoization, a blob store
//! can rely on for deduplication, and a content-addressed merge can
//! rely on for "same content = same identity".
//!
//! Demonstrates conformance contract `CT-E01` (key ordering),
//! `CT-E02` (whitespace), `CT-E03` (Unicode NFC).
//!
//! Run:
//!
//! ```bash
//! cargo run -p uor-addr --example dedupe_cache
//! ```

use std::collections::HashMap;
use uor_addr::json::address;

fn main() {
    // Three syntactic variants of the same JSON value.
    let variants: &[&[u8]] = &[
        "{\"city\":\"São Paulo\",\"country\":\"BR\"}".as_bytes(),
        "{ \"country\" : \"BR\" , \"city\" : \"São Paulo\" }".as_bytes(),
        // NFD-decomposed `ã` (U+0061 U+0303).
        b"{\"city\":\"Sa\xcc\x83o Paulo\",\"country\":\"BR\"}",
    ];

    let mut cache: HashMap<uor_addr::KappaLabel<71>, Vec<&str>> = HashMap::new();
    for raw in variants {
        let label = address(raw).expect("valid JSON").address;
        let view = std::str::from_utf8(raw).unwrap();
        cache.entry(label).or_default().push(view);
    }

    println!("uor-addr — dedupe cache demo");
    println!("(3 syntactic variants → {} bucket)\n", cache.len());
    for (label, members) in &cache {
        println!("  address: {label}");
        for member in members {
            println!("    ← {member}");
        }
    }

    assert_eq!(
        cache.len(),
        1,
        "all three variants must collapse to a single address"
    );
    println!("\nOK — structural equivalence collapses to one κ-label.");
}
