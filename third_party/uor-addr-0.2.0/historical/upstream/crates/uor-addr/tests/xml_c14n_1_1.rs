//! W3C Canonical XML 1.1 conformance suite for the
//! [`uor_addr::xml`] realization (subset documented in the module
//! docstring).
//!
//! Pins the canonicalizer against the rules in W3C Recommendation
//! REC-xml-c14n11 (<https://www.w3.org/TR/xml-c14n11/>):
//!
//! - §1.1 rule 3 — lexicographic attribute ordering.
//! - §1.1 rule 4 — attribute-value character replacement
//!   (`<` `>` `&` `"` `\t` `\n` `\r`).
//! - §1.1 rule 5 — text-content character replacement
//!   (`<` `>` `&` `\r`).
//! - §1.1 — CDATA-to-Text expansion.
//!
//! ## Out-of-scope rules
//!
//! Full XML-C14N 1.1 includes namespace-prefix handling, DTD
//! resolution, and document-level processing instructions outside
//! the root. UOR-ADDR's typed-input XmlValue admits only the subset
//! documented in the module — this test suite pins that subset's
//! behavior; any input outside the subset rejects at parse time.

use uor_addr::xml::{address, canonicalize, AddressFailure};

#[test]
fn rule_3_lexicographic_attribute_ordering() {
    // §1.1 rule 3 — attributes sorted by name (byte-wise).
    let inputs: &[&[u8]] = &[
        br#"<el a="1" b="2" c="3"/>"#,
        br#"<el c="3" b="2" a="1"/>"#,
        br#"<el b="2" a="1" c="3"/>"#,
        br#"<el c="3" a="1" b="2"/>"#,
    ];
    let expected = br#"<el a="1" b="2" c="3"></el>"#.to_vec();
    for raw in inputs {
        let canon = canonicalize(raw).expect("valid");
        assert_eq!(canon, expected, "input {raw:?}");
    }
}

#[test]
fn rule_4_attribute_value_character_replacement() {
    // §1.1 rule 4 — attribute values escape `<`, `>`, `&`, `"`,
    // tab, newline, carriage return.
    let cases: &[(&[u8], &[u8])] = &[
        (br#"<el attr="&lt;">x</el>"#, br#"<el attr="&lt;">x</el>"#),
        (br#"<el attr="&gt;">x</el>"#, br#"<el attr="&gt;">x</el>"#),
        (br#"<el attr="&amp;">x</el>"#, br#"<el attr="&amp;">x</el>"#),
        (
            br#"<el attr="&quot;">x</el>"#,
            br#"<el attr="&quot;">x</el>"#,
        ),
    ];
    for (raw, expected) in cases {
        let canon = canonicalize(raw).expect("valid");
        assert_eq!(canon, expected.to_vec(), "input {raw:?}");
    }
}

#[test]
fn rule_5_text_content_character_replacement() {
    // §1.1 rule 5 — text content escapes `<`, `>`, `&`, carriage
    // return; note that text content does NOT escape `"`, `\t`, or
    // `\n` (unlike attribute values).
    let cases: &[(&[u8], &[u8])] = &[
        (b"<el>&lt;</el>", b"<el>&lt;</el>"),
        (b"<el>&gt;</el>", b"<el>&gt;</el>"),
        (b"<el>&amp;</el>", b"<el>&amp;</el>"),
        (b"<el>x &amp; y</el>", b"<el>x &amp; y</el>"),
    ];
    for (raw, expected) in cases {
        let canon = canonicalize(raw).expect("valid");
        assert_eq!(canon, expected.to_vec(), "input {raw:?}");
    }
}

#[test]
fn cdata_collapses_to_text_with_escaping() {
    // §1.1 — CDATA expanded into text content; text-content
    // escaping rules then apply.
    let cases: &[(&[u8], &[u8])] = &[
        (b"<el><![CDATA[<v>]]></el>", b"<el>&lt;v&gt;</el>"),
        (b"<el><![CDATA[a & b]]></el>", b"<el>a &amp; b</el>"),
        (b"<el><![CDATA[plain]]></el>", b"<el>plain</el>"),
    ];
    for (raw, expected) in cases {
        let canon = canonicalize(raw).expect("valid");
        assert_eq!(canon, expected.to_vec(), "input {raw:?}");
    }
}

#[test]
fn self_closing_elements_expand_to_open_close_pairs() {
    // XML-C14N normalizes `<x/>` to `<x></x>`.
    let canon = canonicalize(b"<r/>").expect("valid");
    assert_eq!(canon, b"<r></r>");
    let canon = canonicalize(b"<r><a/><b/></r>").expect("valid");
    assert_eq!(canon, b"<r><a></a><b></b></r>");
}

#[test]
fn nested_elements_canonical_form() {
    let raw = br#"<root><a x="1"><b y="2">text</b></a></root>"#;
    let canon = canonicalize(raw).expect("valid");
    assert_eq!(canon, br#"<root><a x="1"><b y="2">text</b></a></root>"#);
}

#[test]
fn mixed_content_preserves_text_order() {
    let raw = b"<r>before<inner/>between<another/>after</r>";
    let canon = canonicalize(raw).expect("valid");
    assert_eq!(
        canon,
        b"<r>before<inner></inner>between<another></another>after</r>"
    );
}

#[test]
fn processing_instruction_canonical_form() {
    // PI inside element content survives canonicalization with
    // `<?target data?>` shape.
    let canon = canonicalize(b"<r><?stylesheet href=\"x\"?></r>").expect("valid");
    assert_eq!(canon, b"<r><?stylesheet href=\"x\"?></r>");
}

#[test]
fn whitespace_around_attribute_values_normalizes() {
    // Whitespace between attributes is normalized by the canonicalizer.
    let canon = canonicalize(br#"<el   a="1"    b="2"   />"#).expect("valid");
    assert_eq!(canon, br#"<el a="1" b="2"></el>"#);
}

#[test]
fn canonical_form_is_idempotent() {
    let inputs: &[&[u8]] = &[
        b"<r/>",
        b"<r><a/></r>",
        br#"<r a="1" b="2"/>"#,
        b"<r><![CDATA[<v>]]></r>",
        b"<r>text content</r>",
    ];
    for raw in inputs {
        let once = canonicalize(raw).expect("canon-1");
        let twice = canonicalize(&once).expect("canon-2");
        assert_eq!(once, twice, "idempotence: {raw:?}");
    }
}

#[test]
fn structurally_equivalent_inputs_share_kappa_label() {
    // Equivalent forms (CDATA vs escaped, attribute order) collapse
    // to the same κ-label.
    let cdata = address(b"<r><![CDATA[<v>]]></r>").expect("κ-label").address;
    let escaped = address(b"<r>&lt;v&gt;</r>").expect("κ-label").address;
    assert_eq!(cdata, escaped);

    let attr_order_a = address(br#"<r a="1" b="2"/>"#).expect("κ-label").address;
    let attr_order_b = address(br#"<r b="2" a="1"/>"#).expect("κ-label").address;
    assert_eq!(attr_order_a, attr_order_b);

    let self_closing = address(b"<r/>").expect("κ-label").address;
    let open_close = address(b"<r></r>").expect("κ-label").address;
    assert_eq!(self_closing, open_close);
}

#[test]
fn distinct_structures_yield_distinct_kappa_labels() {
    let labels = [
        address(b"<r/>").expect("κ-label").address,
        address(b"<r>x</r>").expect("κ-label").address,
        address(br#"<r a="1"/>"#).expect("κ-label").address,
        address(b"<r><a/></r>").expect("κ-label").address,
    ];
    for i in 0..labels.len() {
        for j in (i + 1)..labels.len() {
            assert_ne!(labels[i], labels[j]);
        }
    }
}

#[test]
fn rejects_mismatched_close_tag() {
    let cases: &[&[u8]] = &[b"<a></b>", b"<a><b></c></a>", b"<a><b></a></b>"];
    for raw in cases {
        match address(raw) {
            Err(AddressFailure::InvalidXml) => {}
            other => panic!("expected InvalidXml for {raw:?}: {other:?}"),
        }
    }
}

#[test]
fn rejects_unterminated_element() {
    let cases: &[&[u8]] = &[b"<a>", b"<a><b></b>", b"<a x=", b"<a x=\""];
    for raw in cases {
        match address(raw) {
            Err(AddressFailure::InvalidXml) => {}
            other => panic!("expected InvalidXml for {raw:?}: {other:?}"),
        }
    }
}

#[test]
fn rejects_documents_with_dtds_per_subset_policy() {
    // Subset boundary: DTDs are out of scope (no entity resolution).
    let raw = b"<!DOCTYPE r SYSTEM \"r.dtd\"><r/>";
    match address(raw) {
        Err(AddressFailure::InvalidXml) => {}
        other => panic!("expected InvalidXml: {other:?}"),
    }
}

#[test]
fn admits_numeric_entity_references() {
    // Decimal + hex numeric character references decode to the
    // referenced codepoint.
    let canon = canonicalize(b"<r>&#65;</r>").expect("valid");
    assert_eq!(canon, b"<r>A</r>");
    let canon = canonicalize(b"<r>&#x41;</r>").expect("valid");
    assert_eq!(canon, b"<r>A</r>");
}

#[test]
fn admits_unicode_text_content() {
    // UTF-8 text content passes through unchanged.
    let canon = canonicalize("<r>\u{00E9}caf\u{00E9}</r>".as_bytes()).expect("valid");
    assert_eq!(canon, "<r>\u{00E9}caf\u{00E9}</r>".as_bytes());
}

#[test]
fn rejects_overdeep_nesting() {
    use uor_addr::xml::MAX_XML_DEPTH;
    let mut s = alloc::string::String::new();
    for i in 0..(MAX_XML_DEPTH + 4) {
        s.push_str(&alloc::format!("<n{i}>"));
    }
    for i in (0..(MAX_XML_DEPTH + 4)).rev() {
        s.push_str(&alloc::format!("</n{i}>"));
    }
    match address(s.as_bytes()) {
        Err(AddressFailure::InvalidXml) => {}
        other => panic!("expected rejection: {other:?}"),
    }
}

#[test]
fn admits_many_attributes() {
    // ADR-060 removed the attribute-count cap; many attributes admit.
    let mut s = alloc::string::String::from("<r");
    for i in 0..512 {
        s.push_str(&alloc::format!(r#" a{i}="v""#));
    }
    s.push_str("/>");
    address(s.as_bytes()).expect("many attributes admit");
}

extern crate alloc;
