//! UAX #15 NFC conformance — walks every vector in
//! `data/ucd/15.1.0/NormalizationTest.txt` and asserts the
//! `canonical::nfc::normalize_into` implementation matches the UCD's
//! reference NFC output byte-for-byte.
//!
//! NormalizationTest.txt format (UAX #41):
//!
//! | Field | Meaning |
//! |---|---|
//! | 0 | source |
//! | 1 | NFC(source) |
//! | 2 | NFD(source) |
//! | 3 | NFKC(source) |
//! | 4 | NFKD(source) |
//!
//! UAX #15 §6 conformance requires:
//!
//! - `NFC(field0) == field1`
//! - `NFC(field1) == field1`  (idempotence on NFC form)
//! - `NFC(field2) == field1`  (NFC of NFD = NFC)
//! - `NFC(field3) == field3`
//! - `NFC(field4) == field3`
//!
//! This test exercises all five identities for every fixture entry.

use uor_addr::canonical::nfc::{normalize_into, NfcError};

const NORMALIZATION_TEST: &str = include_str!("../../../data/ucd/15.1.0/NormalizationTest.txt");

fn parse_code_points(field: &str) -> String {
    let mut out = String::new();
    for hex in field.split_whitespace() {
        let cp = u32::from_str_radix(hex, 16).expect("hex code point");
        out.push(char::from_u32(cp).expect("valid scalar value"));
    }
    out
}

fn nfc(input: &str) -> String {
    // 4 KiB out buffer — bounded by NormalizationTest.txt's longest
    // vector (a few hundred bytes; 4 KiB is generous).
    let mut buf = [0u8; 4096];
    let n = match normalize_into(input.as_bytes(), &mut buf) {
        Ok(n) => n,
        Err(NfcError::CombiningRunOverflow) => {
            // The stream-safe text format pins the upper bound at 30
            // non-starters; UCD test vectors stay well within it.
            panic!("combining-run overflow on UCD fixture — implementation bug")
        }
        Err(e) => panic!("normalize_into failed: {e:?}"),
    };
    core::str::from_utf8(&buf[..n])
        .expect("output is valid UTF-8")
        .to_string()
}

#[test]
fn ucd_15_1_0_normalization_test_txt_all_vectors() {
    let mut entries = 0usize;
    let mut part = "<header>";
    for (lineno, line) in NORMALIZATION_TEST.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('@') {
            // Part section header (@Part0, @Part1, ...).
            part = trimmed;
            continue;
        }
        // Test vector line: `field0;field1;field2;field3;field4; # comment`
        let payload = trimmed.split('#').next().unwrap_or("");
        let fields: Vec<&str> = payload.split(';').collect();
        assert!(
            fields.len() >= 5,
            "line {} ({}): expected 5 fields, got {}",
            lineno + 1,
            part,
            fields.len()
        );
        let source = parse_code_points(fields[0]);
        let nfc_ref = parse_code_points(fields[1]);
        let nfd_ref = parse_code_points(fields[2]);
        let nfkc_ref = parse_code_points(fields[3]);
        let nfkd_ref = parse_code_points(fields[4]);

        // NFC(source) == NFC.
        let got = nfc(&source);
        assert_eq!(
            got,
            nfc_ref,
            "{} line {}: NFC(c1) != c2 — input cp {:?}, got cp {:?}, expected cp {:?}",
            part,
            lineno + 1,
            cps(&source),
            cps(&got),
            cps(&nfc_ref),
        );

        // NFC(NFC) == NFC (idempotence on already-NFC input).
        assert_eq!(
            nfc(&nfc_ref),
            nfc_ref,
            "{} line {}: NFC(c2) != c2 — input cp {:?}",
            part,
            lineno + 1,
            cps(&nfc_ref),
        );

        // NFC(NFD) == NFC.
        assert_eq!(
            nfc(&nfd_ref),
            nfc_ref,
            "{} line {}: NFC(c3) != c2 — input cp {:?}",
            part,
            lineno + 1,
            cps(&nfd_ref),
        );

        // NFC(NFKC) == NFKC.
        assert_eq!(
            nfc(&nfkc_ref),
            nfkc_ref,
            "{} line {}: NFC(c4) != c4 — input cp {:?}",
            part,
            lineno + 1,
            cps(&nfkc_ref),
        );

        // NFC(NFKD) == NFKC.
        assert_eq!(
            nfc(&nfkd_ref),
            nfkc_ref,
            "{} line {}: NFC(c5) != c4 — input cp {:?}",
            part,
            lineno + 1,
            cps(&nfkd_ref),
        );

        entries += 1;
    }
    // The vendored UCD 15.1.0 NormalizationTest.txt carries 19_074
    // vectors (verified by `grep -c '^[0-9A-F]'`). Fail closed if a
    // generator regression silently drops vectors.
    assert!(
        entries > 18_000,
        "expected > 18_000 vectors, got {entries} (NormalizationTest.txt under-parsed?)"
    );
}

fn cps(s: &str) -> Vec<String> {
    s.chars().map(|c| format!("U+{:04X}", c as u32)).collect()
}
