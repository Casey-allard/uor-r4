//! The prism-free CBOR address (`cbor_address`) against the pinned upstream
//! `uor_addr::cbor::address_blake3` (rev `165b51e3`, a dev-dependency only).
//!
//! For every input of the R4G1 subset (unsigned integers, byte strings, text
//! strings and arrays, in any encoding upstream accepts), both must produce the
//! same κ-label or both must refuse it. Inputs with any other major type are
//! refused by the port and not compared; an R4G1 skeleton never has one.

mod common;

use common::{head_payload, node_section, storage_section, HeadFields, NodeFields};
use uor_r4_graph_format::cbor_address::{self, CborAddressError, MAX_CBOR_DEPTH};
use uor_r4_graph_format::{r4g1, ArtifactBuilder, GraphView, SectionId};

fn upstream(raw: &[u8]) -> Option<String> {
    uor_addr::cbor::address_blake3(raw)
        .ok()
        .map(|outcome| outcome.address.to_string())
}

/// Compare one input. Returns whether the port accepted it.
fn assert_parity(raw: &[u8]) -> bool {
    match cbor_address::address_blake3(raw) {
        Ok(label) => {
            assert_eq!(
                upstream(raw).as_deref(),
                Some(label.as_str()),
                "label differs for {raw:02x?}"
            );
            true
        }
        Err(CborAddressError::UnsupportedMajorType(_)) => false,
        Err(error) => {
            assert_eq!(
                upstream(raw),
                None,
                "port refused ({error:?}) what upstream accepts: {raw:02x?}"
            );
            false
        }
    }
}

/// A head with an explicit additional-information width (0 means direct).
fn head(major: u8, width_ai: u8, argument: u64) -> Vec<u8> {
    let m = major << 5;
    match width_ai {
        0 => vec![m | argument as u8],
        24 => vec![m | 24, argument as u8],
        25 => {
            let mut v = vec![m | 25];
            v.extend_from_slice(&(argument as u16).to_be_bytes());
            v
        }
        26 => {
            let mut v = vec![m | 26];
            v.extend_from_slice(&(argument as u32).to_be_bytes());
            v
        }
        _ => {
            let mut v = vec![m | 27];
            v.extend_from_slice(&argument.to_be_bytes());
            v
        }
    }
}

/// Every head width that can carry `argument`, minimal first.
fn widths(argument: u64) -> Vec<u8> {
    let mut out = Vec::new();
    if argument < 24 {
        out.push(0);
    }
    if argument <= u64::from(u8::MAX) {
        out.push(24);
    }
    if argument <= u64::from(u16::MAX) {
        out.push(25);
    }
    if argument <= u64::from(u32::MAX) {
        out.push(26);
    }
    out.push(27);
    out
}

const BOUNDARIES: [u64; 12] = [
    0,
    1,
    23,
    24,
    25,
    255,
    256,
    65_535,
    65_536,
    u32::MAX as u64,
    u32::MAX as u64 + 1,
    u64::MAX,
];

#[test]
fn unsigned_integers_at_every_boundary_and_width() {
    let mut accepted = 0;
    for value in BOUNDARIES {
        for width in widths(value) {
            assert!(assert_parity(&head(0, width, value)));
            accepted += 1;
        }
    }
    // Additional information 31 on an unsigned integer: upstream reads it as 0.
    assert!(assert_parity(&[0x1f]));
    assert_eq!(cbor_address::canonicalize(&[0x1f]), Ok(vec![0x00]));
    assert!(accepted > 30);
}

#[test]
fn byte_and_text_strings_at_every_length_boundary_and_width() {
    for length in [0usize, 1, 23, 24, 255, 256, 65_535, 65_536, 70_000] {
        let payload: Vec<u8> = (0..length).map(|i| (i * 31 + 7) as u8).collect();
        let text: String = (0..length)
            .map(|i| (b'a' + (i % 26) as u8) as char)
            .collect();
        for width in widths(length as u64) {
            let mut bytes = head(2, width, length as u64);
            bytes.extend_from_slice(&payload);
            assert!(assert_parity(&bytes));
            let mut encoded = head(3, width, length as u64);
            encoded.extend_from_slice(text.as_bytes());
            assert!(assert_parity(&encoded));
        }
    }
    for text in ["r4g1", "é", "漢字", "🎉 κ-label", "blake3:00ff"] {
        let mut encoded = head(3, 0, text.len() as u64);
        encoded.extend_from_slice(text.as_bytes());
        assert!(assert_parity(&encoded));
    }
    // Invalid UTF-8: a lone continuation byte, a truncated sequence, an
    // overlong encoding and a surrogate.
    for bad in [&[0x80][..], &[0xc3], &[0xc0, 0x80], &[0xed, 0xa0, 0x80]] {
        let mut encoded = head(3, 0, bad.len() as u64);
        encoded.extend_from_slice(bad);
        assert!(!assert_parity(&encoded));
    }
}

#[test]
fn indefinite_length_strings_and_arrays() {
    let accepted: [&[u8]; 9] = [
        &[0x5f, 0xff],
        &[0x5f, 0x41, b'a', 0x42, b'b', b'c', 0xff],
        &[0x5f, 0x58, 0x01, b'x', 0xff],
        &[0x7f, 0x61, b'a', 0x62, b'b', b'c', 0xff],
        // A multibyte character split across chunks is valid once joined.
        &[0x7f, 0x61, 0xc3, 0x61, 0xa9, 0xff],
        &[0x9f, 0xff],
        &[0x9f, 0x01, 0x02, 0xff],
        &[0x9f, 0x9f, 0xff, 0x80, 0x83, 0x01, 0x02, 0x03, 0xff],
        &[0x82, 0x9f, 0x01, 0xff, 0x5f, 0x41, b'z', 0xff],
    ];
    for raw in accepted {
        assert!(assert_parity(raw), "{raw:02x?}");
    }
    let refused: [&[u8]; 7] = [
        &[0x5f, 0x61, b'a', 0xff], // text chunk inside a byte string
        &[0x5f, 0x5f, 0xff, 0xff], // nested indefinite chunk
        &[0x5f, 0x41, b'a'],       // no break
        &[0x7f, 0x61, 0xc3, 0xff], // joined text is not UTF-8
        &[0x9f, 0x01, 0x02],       // no break
        &[0x5f, 0x5c, 0xff],       // reserved chunk width
        &[0x7f, 0x41, b'a', 0xff], // byte chunk inside a text string
    ];
    for raw in refused {
        assert!(!assert_parity(raw), "{raw:02x?}");
    }
}

#[test]
fn definite_arrays_nested_and_non_minimal() {
    let mut cases: Vec<Vec<u8>> = vec![
        vec![0x80],
        vec![0x83, 0x01, 0x02, 0x03],
        vec![0x81, 0x81, 0x81, 0x80],
        vec![0x98, 0x02, 0x01, 0x02],
        vec![0x99, 0x00, 0x01, 0x18, 0xff],
        vec![0x83, 0x00, 0x43, 1, 2, 3, 0x64, b'r', b'4', b'g', b'1'],
    ];
    // Lengths 23, 24 and 255, which cross head widths.
    for length in [23usize, 24, 255, 256] {
        let mut v = head(4, widths(length as u64)[0], length as u64);
        for i in 0..length {
            v.extend_from_slice(&head(0, 0, (i % 24) as u64));
        }
        cases.push(v);
    }
    for raw in &cases {
        assert!(assert_parity(raw), "{raw:02x?}");
    }
}

#[test]
fn malformed_input_is_refused_by_both() {
    let refused: [&[u8]; 9] = [
        &[],
        &[0x01, 0x01],       // trailing bytes
        &[0x83, 0x01, 0x02], // short array
        &[0x18],             // missing argument
        &[0x1c],             // reserved additional information
        &[0x5d, 0x00],
        &[0x7e],
        &[0x9e],
        &[0x43, 0x01, 0x02], // short byte string
    ];
    for raw in refused {
        assert!(!assert_parity(raw), "{raw:02x?}");
    }
    // Every proper prefix of a valid item is refused by both.
    let valid = [
        0x84, 0x01, 0x59, 0x01, 0x00, 0x9f, 0x7f, 0x61, b'a', 0xff, 0xff, 0x82, 0x00, 0x18, 0x18,
    ];
    let mut full = valid.to_vec();
    full.splice(5..5, vec![0u8; 256]);
    assert!(assert_parity(&full));
    for end in 0..full.len() {
        assert!(!assert_parity(&full[..end]), "prefix {end}");
    }
}

#[test]
fn nesting_depth_bound_matches_upstream() {
    // Both parsers recurse once per level; ~1,025 debug-build frames need more
    // than the default 2 MiB test-thread stack.
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(nesting_depth_cases)
        .expect("spawn the depth-test thread")
        .join()
        .expect("depth cases pass");
}

fn nesting_depth_cases() {
    for definite in [true, false] {
        for levels in [MAX_CBOR_DEPTH - 1, MAX_CBOR_DEPTH, MAX_CBOR_DEPTH + 1] {
            let mut raw = Vec::new();
            for _ in 0..levels {
                raw.push(if definite { 0x81 } else { 0x9f });
            }
            raw.push(0x00);
            if !definite {
                raw.extend(std::iter::repeat_n(0xff, levels));
            }
            let accepted = assert_parity(&raw);
            assert_eq!(accepted, levels <= MAX_CBOR_DEPTH, "levels {levels}");
        }
    }
}

#[test]
fn other_major_types_are_refused_by_the_port() {
    // Upstream accepts these; an R4G1 skeleton never contains one.
    let others: [(&[u8], u8); 6] = [
        (&[0x20], 1),
        (&[0xa0], 5),
        (&[0xc0, 0x00], 6),
        (&[0xf5], 7),
        (&[0xf9, 0x3c, 0x00], 7),
        (&[0x81, 0xa1, 0x00, 0x00], 5),
    ];
    for (raw, major) in others {
        assert_eq!(
            cbor_address::address_blake3(raw),
            Err(CborAddressError::UnsupportedMajorType(major))
        );
        assert!(upstream(raw).is_some(), "{raw:02x?}");
    }
}

/// A small deterministic generator of R4G1-subset items in every encoding
/// upstream accepts: non-minimal heads, indefinite strings and arrays,
/// chunked text (sometimes splitting a character) and nesting.
struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn value(&mut self) -> u64 {
        if self.below(3) == 0 {
            BOUNDARIES[self.below(BOUNDARIES.len() as u64) as usize]
        } else {
            self.next() >> self.below(64)
        }
    }

    fn width(&mut self, argument: u64) -> u8 {
        let options = widths(argument);
        options[self.below(options.len() as u64) as usize]
    }

    fn item(&mut self, out: &mut Vec<u8>, depth: u32) {
        let choice = if depth >= 4 {
            self.below(3)
        } else {
            self.below(5)
        };
        match choice {
            0 => {
                if self.below(40) == 0 {
                    out.push(0x1f);
                } else {
                    let value = self.value();
                    let width = self.width(value);
                    out.extend_from_slice(&head(0, width, value));
                }
            }
            1 => self.string(out, 2),
            2 => self.string(out, 3),
            _ => {
                let length = self.below(5);
                if self.below(4) == 0 {
                    out.push(0x9f);
                    for _ in 0..length {
                        self.item(out, depth + 1);
                    }
                    out.push(0xff);
                } else {
                    let width = self.width(length);
                    out.extend_from_slice(&head(4, width, length));
                    for _ in 0..length {
                        self.item(out, depth + 1);
                    }
                }
            }
        }
    }

    fn string(&mut self, out: &mut Vec<u8>, major: u8) {
        const TEXT: &str = "r4g1 κ é 漢 🎉 blake3";
        let length = self.below(40) as usize;
        let payload: Vec<u8> = if major == 3 {
            TEXT.as_bytes()
                .iter()
                .copied()
                .cycle()
                .take(length)
                .collect()
        } else {
            (0..length).map(|_| self.next() as u8).collect()
        };
        if self.below(4) == 0 {
            out.push((major << 5) | 31);
            let mut rest = &payload[..];
            while !rest.is_empty() {
                let take = 1 + self.below(rest.len() as u64) as usize;
                let width = self.width(take as u64);
                out.extend_from_slice(&head(major, width, take as u64));
                out.extend_from_slice(&rest[..take]);
                rest = &rest[take..];
            }
            out.push(0xff);
        } else {
            let width = self.width(length as u64);
            out.extend_from_slice(&head(major, width, length as u64));
            out.extend_from_slice(&payload);
        }
    }
}

#[test]
fn generated_items_and_their_mutations_match_upstream() {
    let mut generator = Generator(0x9e37_79b9_7f4a_7c15);
    let (mut accepted, mut refused) = (0, 0);
    for _ in 0..20_000 {
        let mut raw = Vec::new();
        generator.item(&mut raw, 0);
        // Chunked text can split a character, so a generated item may be
        // refused; parity is what matters.
        if assert_parity(&raw) {
            accepted += 1;
        } else {
            refused += 1;
        }
        if !raw.is_empty() {
            let at = generator.below(raw.len() as u64) as usize;
            raw[at] ^= 1 << generator.below(8);
            if assert_parity(&raw) {
                accepted += 1;
            } else {
                refused += 1;
            }
        }
    }
    assert!(accepted > 15_000, "accepted {accepted}");
    assert!(refused > 1_000, "refused {refused}");
}

fn sample(alignment_log2: u8, node_flags: u32, emit_width: u8, rout_len: usize) -> Vec<u8> {
    let mut builder = ArtifactBuilder::new(alignment_log2);
    let head_bytes = head_payload(&HeadFields::default());
    let node = node_section(&[] as &[NodeFields]);
    let rout = vec![0u8; rout_len];
    let emit = storage_section(emit_width, 0, 0, &[]);
    builder.add_section(SectionId::HEAD, 0, &head_bytes);
    builder.add_section(SectionId::NODE, node_flags, &node);
    builder.add_section(SectionId::ROUT, 0, &rout);
    builder.add_section(SectionId::EMIT, 0, &emit);
    builder.build().expect("stage-2-valid sample must build")
}

/// The section skeleton as `r4g1` defines it (version 1, "r4g1", id, flags,
/// payload), written out here so a disagreement shows as a test failure.
fn section_skeleton(id: u32, flags: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0x85, 0x01, 0x64, b'r', b'4', b'g', b'1'];
    out.extend_from_slice(&head(0, widths(u64::from(id))[0], u64::from(id)));
    out.extend_from_slice(&head(0, widths(u64::from(flags))[0], u64::from(flags)));
    out.extend_from_slice(&head(
        2,
        widths(payload.len() as u64)[0],
        payload.len() as u64,
    ));
    out.extend_from_slice(payload);
    out
}

#[test]
fn r4g1_artifact_and_section_addresses_match_upstream() {
    let mut artifacts = 0;
    for alignment in [3u8, 5] {
        for node_flags in [0u32, 7, 23, 24, 255, 256] {
            for emit_width in [1u8, 2] {
                for rout_len in [64usize, 128] {
                    let bytes = sample(alignment, node_flags, emit_width, rout_len);
                    let address = r4g1::address(&bytes).expect("valid sample");
                    assert_eq!(
                        upstream(&address.skeleton).as_deref(),
                        Some(address.artifact_kappa.as_str())
                    );
                    let view = GraphView::parse(&bytes).expect("valid sample");
                    for (section, kappa) in view.sections().zip(&address.sections) {
                        assert_eq!(section.id, kappa.id);
                        let skeleton =
                            section_skeleton(section.id.raw(), section.flags, section.payload);
                        assert_eq!(upstream(&skeleton).as_deref(), Some(kappa.kappa.as_str()));
                    }
                    artifacts += 1;
                }
            }
        }
    }
    assert_eq!(artifacts, 48);
}
