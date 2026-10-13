//! 01 — Address a JSON value.
//!
//! The minimal use case: take a raw JSON byte sequence and emit the
//! 71-byte `sha256:<64hex>` wire-format content address.
//!
//! Run:
//!
//! ```bash
//! cargo run -p uor-addr --example address_value
//! ```

use uor_addr::json::address;

fn main() {
    let inputs: &[&[u8]] = &[
        br#"{"hello":"world"}"#,
        br#"["chain", "agnostic", "content", "address"]"#,
        b"42",
        b"null",
        br#"{"nested":{"deep":{"value":"found"}}}"#,
    ];

    println!("uor-addr — content addresses\n");
    for raw in inputs {
        let outcome = address(raw).expect("valid JSON within typed-input bounds");
        let label = &outcome.address;
        println!("  input:   {}", std::str::from_utf8(raw).unwrap());
        println!("  address: {label}");
        println!("  width:   {} bytes\n", label.len());
    }
}
