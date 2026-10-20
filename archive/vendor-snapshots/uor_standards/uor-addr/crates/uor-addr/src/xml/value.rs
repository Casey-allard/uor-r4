//! `XmlValue` — the typed XML input handle (ADR-023 amended by ADR-060)
//! with W3C Canonical XML 1.1 (subset) byte-output discipline.
//!
//! See [`crate::xml`] for the supported subset and deviations from full
//! XML-C14N 1.1.
//!
//! # ADR-060 carrier model
//!
//! XML canonicalization is **not** a streaming transform: XML-C14N 1.1
//! §1.1 rule 3 sorts each element's attributes lexicographically, and
//! well-formedness checking matches nested close tags — both inherently
//! need storage proportional to the element / nesting size. The
//! realization therefore materializes the canonical form once, in an
//! `alloc` buffer ([`canonicalize`]), with **no** width or count
//! ceilings: element names, attribute values, text runs, attribute
//! counts, and child counts are unbounded. The handle then flows through
//! the pipeline as a zero-copy [`TermValue::Borrowed`] carrier over those
//! canonical bytes, and ψ₉ folds them through the σ-axis.
//!
//! The single bound retained is [`MAX_XML_DEPTH`] — a native-stack
//! overflow guard on the recursive-descent canonicalizer, not a content
//! ceiling.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};
// `ShapeViolation` / `ViolationKind` / `MAX_XML_DEPTH` are consumed only by
// the `alloc`-gated canonicalizer below.
#[cfg(feature = "alloc")]
use crate::xml::shapes::bounds::MAX_XML_DEPTH;
#[cfg(feature = "alloc")]
use prism::pipeline::{ShapeViolation, ViolationKind};

// ─── ShapeViolation IRIs ────────────────────────────────────────────────

#[cfg(feature = "alloc")]
const INVALID_XML_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/XmlValue",
    constraint_iri: "https://uor.foundation/addr/XmlValue/validXml",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidUtf8Xml",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

#[cfg(feature = "alloc")]
const DEPTH_BOUND_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/XmlValue",
    constraint_iri: "https://uor.foundation/addr/XmlValue/depthBound",
    property_iri: "https://uor.foundation/addr/XmlValue/depth",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: MAX_XML_DEPTH as u32,
    kind: ViolationKind::CardinalityViolation,
};

// ─── XmlValue — the typed input handle ──────────────────────────────────

/// Typed XML input handle (ADR-060 borrowed carrier). A thin, `Copy`
/// borrow of canonical-XML bytes produced by [`canonicalize`];
/// `as_binding_value` returns the `Borrowed` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct XmlValue<'a>(&'a [u8]);

impl<'a> XmlValue<'a> {
    /// Wrap a canonical-XML byte slice as a model input handle.
    #[must_use]
    pub fn new(canonical_bytes: &'a [u8]) -> Self {
        Self(canonical_bytes)
    }

    /// Borrow the canonical-XML bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for XmlValue<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/XmlValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for XmlValue<'_> {}

impl<'a> IntoBindingValue<'a> for XmlValue<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        // The canonical form is materialized by `canonicalize`; ψ₉ folds
        // it. `self.0` is `&'a [u8]`, so the carrier borrows the input's
        // `'a`-lived data independently of the `&self` call borrow.
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for XmlValue<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ─── Canonicalizer (alloc) ──────────────────────────────────────────────

/// Parse + canonicalize per the W3C XML-C14N 1.1 subset documented in
/// [`crate::xml`]. Single recursive-descent pass over `raw` that emits
/// the canonical form directly — no fixed buffer, no width/count caps.
///
/// **Available only under the `alloc` feature.** The model handle
/// ([`XmlValue`]) is `no_alloc`; canonicalization itself needs heap
/// storage (per-element attribute sort scratch + the canonical output).
///
/// # Errors
///
/// - [`INVALID_XML_VIOLATION`] (`validXml`) — `raw` is not a well-formed
///   UTF-8 document in the supported subset.
/// - [`DEPTH_BOUND_VIOLATION`] (`depthBound`) — nesting exceeds the
///   [`MAX_XML_DEPTH`] native-stack-safety bound.
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, ShapeViolation> {
    extern crate alloc;
    core::str::from_utf8(raw).map_err(|_| INVALID_XML_VIOLATION)?;
    let mut p = Parser::new(raw);
    let mut out = alloc::vec::Vec::new();
    p.skip_ws();
    emit_element(&mut p, &mut out, 0)?;
    p.skip_ws();
    if !p.is_eof() {
        return Err(INVALID_XML_VIOLATION);
    }
    Ok(out)
}

#[cfg(feature = "alloc")]
struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

#[cfg(feature = "alloc")]
impl<'a> Parser<'a> {
    fn new(src: &'a [u8]) -> Self {
        Self { src, pos: 0 }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}

/// Parse one element from `p` and append its canonical form to `out`.
#[cfg(feature = "alloc")]
fn emit_element(
    p: &mut Parser<'_>,
    out: &mut alloc::vec::Vec<u8>,
    depth: usize,
) -> Result<(), ShapeViolation> {
    use alloc::vec::Vec;

    if depth > MAX_XML_DEPTH {
        return Err(DEPTH_BOUND_VIOLATION);
    }
    if p.pos >= p.src.len() || p.src[p.pos] != b'<' {
        return Err(INVALID_XML_VIOLATION);
    }
    p.pos += 1;
    if p.pos < p.src.len() && (p.src[p.pos] == b'!' || p.src[p.pos] == b'?') {
        return Err(INVALID_XML_VIOLATION);
    }
    let name_start = p.pos;
    let name_len = parse_name_len(p)?;
    let name = &p.src[name_start..name_start + name_len];

    // Collect this element's attributes (entity-decoded values), then sort
    // lexicographically by name per XML-C14N 1.1 §1.1 rule 3.
    let mut attrs: Vec<(&[u8], Vec<u8>)> = Vec::new();
    loop {
        p.skip_ws();
        if p.pos >= p.src.len() {
            return Err(INVALID_XML_VIOLATION);
        }
        if p.src[p.pos] == b'>' || p.src[p.pos] == b'/' {
            break;
        }
        attrs.push(parse_attr(p)?);
    }
    attrs.sort_by(|a, b| a.0.cmp(b.0));

    out.push(b'<');
    out.extend_from_slice(name);
    for (k, v) in &attrs {
        out.push(b' ');
        out.extend_from_slice(k);
        out.extend_from_slice(b"=\"");
        escape_attr_into(v, out);
        out.push(b'"');
    }

    if p.src[p.pos] == b'/' {
        // Self-closing — canonical form expands to `<name…></name>`.
        p.pos += 1;
        if p.pos >= p.src.len() || p.src[p.pos] != b'>' {
            return Err(INVALID_XML_VIOLATION);
        }
        p.pos += 1;
        out.extend_from_slice(b"></");
        out.extend_from_slice(name);
        out.push(b'>');
        return Ok(());
    }
    if p.src[p.pos] != b'>' {
        return Err(INVALID_XML_VIOLATION);
    }
    p.pos += 1;
    out.push(b'>');

    // Children.
    loop {
        if p.pos >= p.src.len() {
            return Err(INVALID_XML_VIOLATION);
        }
        if p.src[p.pos] == b'<' {
            if p.pos + 1 < p.src.len() && p.src[p.pos + 1] == b'/' {
                // Close tag — must match the open name.
                p.pos += 2;
                let close_start = p.pos;
                let close_len = parse_name_len(p)?;
                if &p.src[close_start..close_start + close_len] != name {
                    return Err(INVALID_XML_VIOLATION);
                }
                p.skip_ws();
                if p.pos >= p.src.len() || p.src[p.pos] != b'>' {
                    return Err(INVALID_XML_VIOLATION);
                }
                p.pos += 1;
                out.extend_from_slice(b"</");
                out.extend_from_slice(name);
                out.push(b'>');
                return Ok(());
            }
            if p.pos + 8 < p.src.len() && &p.src[p.pos..p.pos + 9] == b"<![CDATA[" {
                // CDATA collapses to escaped text per XML-C14N 1.1 §1.1.
                p.pos += 9;
                let start = p.pos;
                while p.pos + 2 < p.src.len() && &p.src[p.pos..p.pos + 3] != b"]]>" {
                    p.pos += 1;
                }
                if p.pos + 2 >= p.src.len() {
                    return Err(INVALID_XML_VIOLATION);
                }
                let cdata = &p.src[start..p.pos];
                p.pos += 3;
                escape_text_into(cdata, out);
                continue;
            }
            if p.pos + 1 < p.src.len() && p.src[p.pos + 1] == b'?' {
                // Processing instruction → `<?target data?>`.
                p.pos += 2;
                let target_start = p.pos;
                let target_len = parse_name_len(p)?;
                let target = &p.src[target_start..target_start + target_len];
                p.skip_ws();
                let data_start = p.pos;
                while p.pos + 1 < p.src.len() && &p.src[p.pos..p.pos + 2] != b"?>" {
                    p.pos += 1;
                }
                if p.pos + 1 >= p.src.len() {
                    return Err(INVALID_XML_VIOLATION);
                }
                let raw_data = &p.src[data_start..p.pos];
                p.pos += 2;
                let mut end = raw_data.len();
                while end > 0 && raw_data[end - 1].is_ascii_whitespace() {
                    end -= 1;
                }
                out.extend_from_slice(b"<?");
                out.extend_from_slice(target);
                if end > 0 {
                    out.push(b' ');
                    out.extend_from_slice(&raw_data[..end]);
                }
                out.extend_from_slice(b"?>");
                continue;
            }
            // Nested element.
            emit_element(p, out, depth + 1)?;
            continue;
        }
        // Text content — entity-decoded, then escaped.
        let text_start = p.pos;
        while p.pos < p.src.len() && p.src[p.pos] != b'<' {
            p.pos += 1;
        }
        let decoded = decode_entities(&p.src[text_start..p.pos])?;
        escape_text_into(&decoded, out);
    }
}

#[cfg(feature = "alloc")]
fn parse_name_len(p: &mut Parser<'_>) -> Result<usize, ShapeViolation> {
    let start = p.pos;
    while p.pos < p.src.len() {
        let b = p.src[p.pos];
        if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.' {
            p.pos += 1;
        } else {
            break;
        }
    }
    let len = p.pos - start;
    if len == 0 {
        return Err(INVALID_XML_VIOLATION);
    }
    Ok(len)
}

/// Parse `name="value"` (or `name='value'`); returns the borrowed name
/// and the entity-decoded value.
#[cfg(feature = "alloc")]
fn parse_attr<'a>(p: &mut Parser<'a>) -> Result<(&'a [u8], alloc::vec::Vec<u8>), ShapeViolation> {
    let name_start = p.pos;
    let name_len = parse_name_len(p)?;
    let name = &p.src[name_start..name_start + name_len];
    p.skip_ws();
    if p.pos >= p.src.len() || p.src[p.pos] != b'=' {
        return Err(INVALID_XML_VIOLATION);
    }
    p.pos += 1;
    p.skip_ws();
    if p.pos >= p.src.len() {
        return Err(INVALID_XML_VIOLATION);
    }
    let quote = p.src[p.pos];
    if quote != b'"' && quote != b'\'' {
        return Err(INVALID_XML_VIOLATION);
    }
    p.pos += 1;
    let value_start = p.pos;
    while p.pos < p.src.len() && p.src[p.pos] != quote {
        p.pos += 1;
    }
    if p.pos >= p.src.len() {
        return Err(INVALID_XML_VIOLATION);
    }
    let raw_value = &p.src[value_start..p.pos];
    p.pos += 1;
    Ok((name, decode_entities(raw_value)?))
}

/// Resolve the five predefined entities plus numeric character
/// references into a freshly-allocated UTF-8 byte sequence.
#[cfg(feature = "alloc")]
fn decode_entities(text: &[u8]) -> Result<alloc::vec::Vec<u8>, ShapeViolation> {
    use alloc::vec::Vec;
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let b = text[i];
        if b != b'&' {
            out.push(b);
            i += 1;
            continue;
        }
        let entity_start = i + 1;
        let mut j = entity_start;
        while j < text.len() && text[j] != b';' {
            j += 1;
        }
        if j >= text.len() {
            return Err(INVALID_XML_VIOLATION);
        }
        let entity = &text[entity_start..j];
        let cp = match entity {
            b"lt" => '<' as u32,
            b"gt" => '>' as u32,
            b"amp" => '&' as u32,
            b"quot" => '"' as u32,
            b"apos" => '\'' as u32,
            _ if entity.starts_with(b"#x") || entity.starts_with(b"#X") => {
                let hex = &entity[2..];
                let s = core::str::from_utf8(hex).map_err(|_| INVALID_XML_VIOLATION)?;
                u32::from_str_radix(s, 16).map_err(|_| INVALID_XML_VIOLATION)?
            }
            _ if entity.starts_with(b"#") => {
                let dec = &entity[1..];
                let s = core::str::from_utf8(dec).map_err(|_| INVALID_XML_VIOLATION)?;
                s.parse::<u32>().map_err(|_| INVALID_XML_VIOLATION)?
            }
            _ => return Err(INVALID_XML_VIOLATION),
        };
        let c = char::from_u32(cp).ok_or(INVALID_XML_VIOLATION)?;
        let mut buf = [0u8; 4];
        out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        i = j + 1;
    }
    Ok(out)
}

/// XML-C14N 1.1 §1.1 rule 4 — attribute-value character replacement.
#[cfg(feature = "alloc")]
fn escape_attr_into(bytes: &[u8], out: &mut alloc::vec::Vec<u8>) {
    for &b in bytes {
        match b {
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'&' => out.extend_from_slice(b"&amp;"),
            b'"' => out.extend_from_slice(b"&quot;"),
            b'\t' => out.extend_from_slice(b"&#x9;"),
            b'\n' => out.extend_from_slice(b"&#xA;"),
            b'\r' => out.extend_from_slice(b"&#xD;"),
            _ => out.push(b),
        }
    }
}

/// XML-C14N 1.1 §1.1 rule 5 — text-content character replacement.
#[cfg(feature = "alloc")]
fn escape_text_into(bytes: &[u8], out: &mut alloc::vec::Vec<u8>) {
    for &b in bytes {
        match b {
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'&' => out.extend_from_slice(b"&amp;"),
            b'\r' => out.extend_from_slice(b"&#xD;"),
            _ => out.push(b),
        }
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_with_lexicographic_attribute_ordering() {
        let canon = canonicalize(br#"<root b="2" a="1"/>"#).expect("valid");
        assert_eq!(canon, br#"<root a="1" b="2"></root>"#);
    }

    #[test]
    fn canonicalizer_collapses_cdata_to_text() {
        let canon = canonicalize(b"<root><![CDATA[<hello>]]></root>").expect("valid");
        assert_eq!(canon, b"<root>&lt;hello&gt;</root>");
    }

    #[test]
    fn canonicalizer_escapes_attribute_values() {
        let canon = canonicalize(br#"<root attr="&lt;v&gt;"/>"#).expect("valid");
        assert_eq!(canon, br#"<root attr="&lt;v&gt;"></root>"#);
    }

    #[test]
    fn canonicalizer_is_idempotent() {
        let inputs: &[&[u8]] = &[
            b"<root/>",
            b"<root><child/></root>",
            br#"<root a="1" b="2"><child>text</child></root>"#,
        ];
        for raw in inputs {
            let once = canonicalize(raw).expect("valid");
            let twice = canonicalize(&once).expect("re-canonicalises");
            assert_eq!(once, twice, "idempotence broken for {raw:?}");
        }
    }

    #[test]
    fn rejects_mismatched_close_tag() {
        let err = canonicalize(b"<a></b>").expect_err("mismatch");
        assert_eq!(err.constraint_iri, INVALID_XML_VIOLATION.constraint_iri);
    }

    #[test]
    fn accepts_unbounded_attribute_and_name_widths() {
        extern crate alloc;
        // Element name, attribute name, and value all far exceed the old
        // fixed-buffer ceilings; ADR-060 admits them.
        let long_name = "n".repeat(5000);
        let long_val = "v".repeat(20_000);
        let doc = alloc::format!(r#"<{long_name} attr="{long_val}"/>"#);
        let canon = canonicalize(doc.as_bytes()).expect("unbounded widths admitted");
        let expected = alloc::format!(r#"<{long_name} attr="{long_val}"></{long_name}>"#);
        assert_eq!(canon, expected.as_bytes());
    }

    #[test]
    fn rejects_overdeep_nesting() {
        extern crate alloc;
        use alloc::format;
        use alloc::string::String;
        let mut s = String::new();
        for i in 0..(MAX_XML_DEPTH + 2) {
            s.push_str(&format!("<n{i}>"));
        }
        for i in (0..(MAX_XML_DEPTH + 2)).rev() {
            s.push_str(&format!("</n{i}>"));
        }
        let err = canonicalize(s.as_bytes()).expect_err("overdeep");
        assert_eq!(err.constraint_iri, DEPTH_BOUND_VIOLATION.constraint_iri);
    }
}
