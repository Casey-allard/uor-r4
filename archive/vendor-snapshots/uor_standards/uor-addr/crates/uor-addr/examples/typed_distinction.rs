//! 03 — Typed distinction in signature payloads.
//!
//! The typed `JsonValue` input shape distinguishes JSON cases by
//! structure, not by textual rendering. `42` (the integer) and
//! `"42"` (the string of digits) parse to different cases, carry
//! different structural tags, canonicalise to different bytes, and
//! produce different κ-labels. The same holds for `null` vs `false`,
//! `{}` vs `[]`, and numeric vs string array elements.
//!
//! For signature-over-content workflows this is load-bearing: signing
//! `address(payload)` rather than `payload` itself keeps signatures
//! stable under syntactic noise (key order, whitespace, NFC) **but**
//! cleanly rejects semantically-distinct payloads that happen to
//! print similarly.
//!
//! Demonstrates conformance contract `CT-T01..05`.
//!
//! Run:
//!
//! ```bash
//! cargo run -p uor-addr --example typed_distinction
//! ```

use uor_addr::json::address;

fn main() {
    // Each row: a pair of inputs that LOOK similar but are typed-distinct.
    let pairs: &[(&str, &[u8], &[u8])] = &[
        ("integer vs string", b"42", br#""42""#),
        ("null vs false", b"null", b"false"),
        ("empty object vs empty array", b"{}", b"[]"),
        (
            "number array vs string array",
            b"[1,2,3]",
            br#"["1","2","3"]"#,
        ),
        ("object vs single-key object string", b"{}", br#""{}""#),
    ];

    println!("uor-addr — typed distinction in κ-labels\n");
    for (label, a, b) in pairs {
        let addr_a = address(a).expect("valid").address;
        let addr_b = address(b).expect("valid").address;
        let same = addr_a == addr_b;
        println!("  case: {label}");
        println!("    {:>25} → {}", std::str::from_utf8(a).unwrap(), addr_a);
        println!("    {:>25} → {}", std::str::from_utf8(b).unwrap(), addr_b);
        println!("    distinct: {}\n", if same { "NO (BUG)" } else { "yes" });
        assert!(!same, "typed distinction broken for {label}");
    }

    println!("OK — typed cases yield distinct κ-labels by structure.");
}
