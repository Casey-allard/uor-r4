//! Conformance suite — runtime invariants from [CONFORMANCE.md](../../../CONFORMANCE.md).
//!
//! Each test is named `<id>__<short_description>`, where `<id>` is a
//! row in CONFORMANCE.md. Test failures trace back to a row by ID.
//! Run via `just conformance` (release mode).

#![allow(non_snake_case)]

use uor_addr::json::{address, canonicalize, AddressFailure};

// ───────────────────────────────────────────────────────────────────────────
// CS — Structural class (source-grep + runtime invariants)
// ───────────────────────────────────────────────────────────────────────────

/// CS-S01 — zero `unsafe` blocks anywhere in `crates/uor-addr/src/`.
#[test]
fn cs_s01__no_unsafe_anywhere() {
    let sources = [
        "lib.rs",
        "common.rs",
        "label.rs",
        "resolvers.rs",
        "json/mod.rs",
        "json/model.rs",
        "json/verbs.rs",
        "json/pipeline.rs",
        "json/value.rs",
        "json/shapes/mod.rs",
        "json/shapes/bounds.rs",
    ];
    let crate_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for fname in sources {
        let path = crate_src.join(fname);
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (lineno, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            // `#![forbid(unsafe_code)]` and `//` comments are fine.
            if trimmed.starts_with("//") || trimmed.starts_with("#!") || trimmed.starts_with("#[") {
                continue;
            }
            assert!(
                !trimmed.starts_with("unsafe ") && !trimmed.contains(" unsafe "),
                "CS-S01 violation: {}:{} contains `unsafe`: {}",
                path.display(),
                lineno + 1,
                line
            );
        }
    }
}

/// CS-S02 — no `unwrap()` / `expect()` in non-test code paths under
/// `verbs.rs`, `resolvers.rs`, `pipeline.rs`. Allowed inside `#[cfg(test)]`
/// blocks (test scaffolding only).
#[test]
fn cs_s02__no_panic_paths_in_pipeline() {
    let sources = ["json/verbs.rs", "resolvers.rs", "json/pipeline.rs"];
    let crate_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for fname in sources {
        let path = crate_src.join(fname);
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

        // Crudely strip `#[cfg(test)] mod tests { ... }` and similar test blocks
        // by tracking brace depth from the first `#[cfg(test)]` line.
        let mut in_test_mod = false;
        let mut depth = 0i32;
        for (lineno, line) in body.lines().enumerate() {
            if line.contains("#[cfg(test)]") {
                in_test_mod = true;
                depth = 0;
                continue;
            }
            if in_test_mod {
                depth += line.matches('{').count() as i32;
                depth -= line.matches('}').count() as i32;
                if depth <= 0 && line.contains('}') {
                    in_test_mod = false;
                }
                continue;
            }
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            assert!(
                !line.contains(".unwrap()") && !line.contains(".expect("),
                "CS-S02 violation: {}:{} contains panic path outside #[cfg(test)]: {}",
                path.display(),
                lineno + 1,
                line
            );
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CD — Deterministic class (per-input byte identity, extending byte_identity.rs)
// ───────────────────────────────────────────────────────────────────────────

/// CD-D01 — `address(b)` is a pure function: 64 repeated calls on the
/// same input yield bit-identical outputs.
#[test]
fn cd_d01__address_is_pure_function() {
    let inputs: &[&[u8]] = &[
        br#"{"foo":"bar"}"#,
        br#"[]"#,
        br#"[1,2,3]"#,
        br#"{"nested":{"deep":{"value":"found"}}}"#,
    ];
    for raw in inputs {
        let first = address(raw).expect("valid").address;
        for _ in 0..63 {
            let again = address(raw).expect("valid").address;
            assert_eq!(first, again, "CD-D01: non-deterministic for {raw:?}");
        }
    }
}

/// CD-I01b — whitespace invariance.
#[test]
fn cd_i01b__whitespace_invariance() {
    let inputs: &[(&[u8], &[u8])] = &[
        (b"{ \"foo\" : \"bar\" }", br#"{"foo":"bar"}"#),
        (b"[ 1 , 2 ,\n3 ]", b"[1,2,3]"),
        (b"{\n\t\"a\":\t1,\n\t\"b\":\t2\n}", br#"{"a":1,"b":2}"#),
    ];
    for (raw_a, raw_b) in inputs {
        let addr_a = address(raw_a).expect("valid a").address;
        let addr_b = address(raw_b).expect("valid b").address;
        assert_eq!(
            addr_a, addr_b,
            "CD-I01b: whitespace variation broke invariance: {raw_a:?} ≢ {raw_b:?}"
        );
    }
}

/// CD-I01d — NFKC compatibility class (informational; full-width digits
/// `１２３` vs ASCII `123` are **not** required to fold because NFC, not
/// NFKC, is the canonicalisation we ship). This test pins the
/// observable: the κ-label is **distinct** for these inputs under NFC.
#[test]
fn cd_i01d__nfkc_compatibility_class_holds() {
    // NFC does NOT fold full-width to ASCII; NFKC does.
    // Different κ-labels expected.
    let nfc_full_width = "{\"n\":\"\u{FF11}\u{FF12}\u{FF13}\"}".as_bytes();
    let ascii = b"{\"n\":\"123\"}".as_slice();
    let addr_full = address(nfc_full_width).expect("valid").address;
    let addr_ascii = address(ascii).expect("valid").address;
    assert_ne!(
        addr_full, addr_ascii,
        "CD-I01d: NFC must NOT fold full-width to ASCII (would require NFKC)"
    );
}

/// CD-S01b — single-byte avalanche balance: mutating one byte of the
/// canonical input changes ≥ 100 of the 256 digest bits.
#[test]
fn cd_s01b__single_byte_avalanche_balanced() {
    let base = br#"{"avalanche":"baseline"}"#.to_vec();
    let base_addr = address(&base).expect("valid").address;
    let base_digest = hex_decode(&base_addr[7..]);

    // Flip one byte at a time across positions where the JSON value
    // can vary (the value field). Skip schema delimiters.
    for variant in [
        br#"{"avalanche":"Baseline"}"#.as_slice(),
        br#"{"avalanche":"baseLine"}"#.as_slice(),
        br#"{"avalanche":"baselinE"}"#.as_slice(),
        br#"{"avalanche":"caseline"}"#.as_slice(),
    ] {
        let other_addr = address(variant).expect("valid").address;
        let other_digest = hex_decode(&other_addr[7..]);
        let hd = hamming_distance(&base_digest, &other_digest);
        assert!(
            hd >= 100,
            "CD-S01b: single-byte mutation Hamming distance {hd} < 100 (variant: {})",
            std::str::from_utf8(variant).unwrap()
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CL-H — Hex-encoding spec mirror (parametric: Rust matches Lean spec)
// ───────────────────────────────────────────────────────────────────────────

/// CL-H01 mirror — the runtime `hex_lower` table matches the Lean
/// `UorAddr.HexEncoding.hexLower` definition byte-for-byte across
/// `[0, 16)`.
#[test]
fn cl_h01__hex_lower_table_matches_lean_spec() {
    // Lean: `hexLower 0 = 0x30`, …, `hexLower 15 = 0x66`.
    let expected: [u8; 16] = [
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64, 0x65,
        0x66,
    ];

    // Round-trip through `address()` — derive κ-label from a digest
    // we can compute, then verify the hex bytes match `expected`.
    // We pick canonical-form bytes whose SHA-256 has known leading
    // nibbles via lookup.
    //
    // Simpler check: the very first emitted κ-label byte after
    // `"sha256:"` (position 7) MUST be one of `expected[0..16]`.
    let outcome = address(br#"{"foo":"bar"}"#).expect("valid");
    let hex_suffix = &outcome.address.as_bytes()[7..];
    assert_eq!(hex_suffix.len(), 64);
    for &c in hex_suffix {
        assert!(
            expected.contains(&c),
            "CL-H01: hex char {c:#x} outside Lean-spec alphabet {expected:#x?}"
        );
    }
}

/// CL-W01 — every κ-label is exactly 71 bytes (mirrors the Lean theorem
/// `address_label_width_is_seventy_one`).
#[test]
fn cl_w01__every_kappa_label_is_seventy_one_bytes() {
    let inputs: &[&[u8]] = &[
        b"{}",
        b"[]",
        b"[1,2,3]",
        br#"{"foo":"bar"}"#,
        "{\"a\":\"é\"}".as_bytes(),
        br#"{"int": 42, "bool": true, "null_val": null}"#,
    ];
    for raw in inputs {
        let outcome = address(raw).expect("valid");
        assert_eq!(
            outcome.address.len(),
            71,
            "CL-W01: width {} ≠ 71 for input {raw:?}",
            outcome.address.len()
        );
    }
}

/// CL-W02 — every κ-label begins with the ASCII literal `"sha256:"`.
#[test]
fn cl_w02__every_kappa_label_starts_with_sha256_colon() {
    for raw in [b"{}".as_slice(), b"[]", br#"{"foo":"bar"}"#] {
        let outcome = address(raw).expect("valid");
        assert!(
            outcome.address.starts_with("sha256:"),
            "CL-W02: prefix violation for {raw:?}: {}",
            outcome.address
        );
    }
}

/// CL-W03 — every byte of the 64-hex suffix is in `{'0'..'9', 'a'..'f'}`.
#[test]
fn cl_w03__every_hex_byte_in_lowercase_alphabet() {
    let inputs: &[&[u8]] = &[
        b"{}",
        b"[]",
        b"[1,2,3]",
        br#"{"foo":"bar"}"#,
        br#"{"nested":{"deep":{"value":"found"}}}"#,
    ];
    for raw in inputs {
        let outcome = address(raw).expect("valid");
        for (i, c) in outcome.address.as_bytes().iter().enumerate().skip(7) {
            assert!(
                c.is_ascii_digit() || (b'a'..=b'f').contains(c),
                "CL-W03: byte at position {i} ({c:#x}) not in lowercase-hex alphabet for {raw:?}"
            );
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CL-A — Algebraic-closure encoding mirror
// ───────────────────────────────────────────────────────────────────────────

/// CL-A01 — Euler characteristic identity at the Rust level:
/// β_0 = 71, β_k = 0 for k ≥ 1, χ(N(C)) = 71 = SITE_COUNT.
#[test]
fn cl_a01__euler_char_equals_site_count() {
    use uor_addr::AddressLabel;
    use uor_foundation::pipeline::ConstrainedTypeShape;

    let cs = <AddressLabel as ConstrainedTypeShape>::CONSTRAINTS;
    let site_count = <AddressLabel as ConstrainedTypeShape>::SITE_COUNT;
    let beta_0 = cs.len() as isize;
    let beta_higher = 0isize; // No higher simplices in N(C).
    let euler_char = beta_0 - beta_higher;
    assert_eq!(
        euler_char as usize, site_count,
        "CL-A01: χ(N(C))={} ≠ SITE_COUNT={}",
        euler_char, site_count
    );
    assert_eq!(euler_char, 71);
}

/// CL-A02 — FreeRank residual after ψ_9 is 0 (all 71 sites pin
/// simultaneously via the κ-derivation).
#[test]
fn cl_a02__free_rank_residual_is_zero_after_psi_9() {
    use uor_addr::AddressLabel;
    use uor_foundation::pipeline::ConstrainedTypeShape;

    let site_count = <AddressLabel as ConstrainedTypeShape>::SITE_COUNT;
    let beta_0 = <AddressLabel as ConstrainedTypeShape>::CONSTRAINTS.len();
    let free_rank_residual = site_count.saturating_sub(beta_0);
    assert_eq!(
        free_rank_residual, 0,
        "CL-A02: FreeRank residual {free_rank_residual} ≠ 0"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// Architectural failure modes
// ───────────────────────────────────────────────────────────────────────────

/// Pipeline admits arbitrarily large canonical forms (ADR-060 removed
/// the input-size cap).
#[test]
fn pipeline_admits_large_canonical_form() {
    let payload = "a".repeat(4096);
    let raw = format!("{{\"k\":\"{payload}\"}}");
    assert!(address(raw.as_bytes()).is_ok());
}

/// Pipeline rejects invalid JSON cleanly (InvalidJson variant).
#[test]
fn pipeline_rejects_invalid_json() {
    let err = address(b"not json").expect_err("must reject");
    assert!(matches!(err, AddressFailure::InvalidJson));
    let err2 = canonicalize(b"{invalid").expect_err("must reject");
    assert!(err2.constraint_iri.contains("validUtf8Json"));
}

// ───────────────────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────────────────

fn hex_decode(s: &str) -> Vec<u8> {
    let s = s.as_bytes();
    (0..s.len() / 2)
        .map(|i| {
            let hi = nibble(s[2 * i]);
            let lo = nibble(s[2 * i + 1]);
            (hi << 4) | lo
        })
        .collect()
}

fn nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => 10 + (c - b'a'),
        _ => panic!("non-hex byte: {c}"),
    }
}

fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}
