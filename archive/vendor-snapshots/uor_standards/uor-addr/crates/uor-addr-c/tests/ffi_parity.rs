//! C ABI parity tests — verify each `uor_addr_<realization>` extern
//! function produces a κ-label byte-for-byte identical to the
//! pure-Rust `uor_addr::<realization>::address(input).address` path.
//!
//! Tests call the extern "C" symbols through the rlib's normal Rust
//! linkage; no C toolchain needed.

use uor_addr_c::*;

/// Helper: run the extern function and return `Ok(label_bytes)` or
/// `Err(error_code)`.
fn call_ffi<F>(input: &[u8], f: F) -> Result<[u8; 71], i32>
where
    F: FnOnce(*const u8, usize, *mut u8, usize, *mut usize) -> i32,
{
    let mut out = [0u8; 71];
    let mut written: usize = 0;
    let rc = f(
        input.as_ptr(),
        input.len(),
        out.as_mut_ptr(),
        out.len(),
        &mut written,
    );
    if rc == UOR_ADDR_OK {
        assert_eq!(written, 71, "successful call must report 71 bytes written");
        Ok(out)
    } else {
        Err(rc)
    }
}

#[test]
fn json_ffi_matches_rust_path() {
    let input = br#"{"foo":"bar","baz":42}"#;
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_json(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::json::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn sexp_ffi_matches_rust_path() {
    let input = b"(a b c)";
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_sexp(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::sexp::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn xml_ffi_matches_rust_path() {
    let input = br#"<root a="1" b="2"/>"#;
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_xml(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::xml::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn asn1_ffi_matches_rust_path() {
    // DER-encoded INTEGER 42: 02 01 2A.
    let input: &[u8] = &[0x02, 0x01, 0x2A];
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_asn1(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::asn1::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn ring_ffi_matches_rust_path() {
    // Amendment 43 canonical bytes: witt_level=1, coeff=0x0102.
    let input: &[u8] = &[1, 0x02, 0x01];
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_ring(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::ring::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn codemodule_ffi_matches_rust_path() {
    let input = b"(3:mod 5:empty)";
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_codemodule(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::codemodule::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn schema_photo_ffi_matches_rust_path() {
    let input = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": "Ada Lovelace"
    }"#;
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_schema_photo(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::schema::photo::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn schema_document_ffi_matches_rust_path() {
    let input = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "Hello",
        "author": "Ada Lovelace",
        "datePublished": "2025-01-15"
    }"#;
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_schema_document(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::schema::document::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

#[test]
fn schema_codemodule_signed_ffi_matches_rust_path() {
    let input = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{
            "name": "test",
            "digest": {
                "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            }
        }],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {}
    }"#;
    let ffi = call_ffi(input, |i, il, o, ol, ow| unsafe {
        uor_addr_schema_codemodule_signed(i, il, o, ol, ow)
    })
    .expect("FFI ok");
    let rust = uor_addr::schema::codemodule_signed::address(input).expect("Rust ok");
    assert_eq!(ffi.as_slice(), rust.address.as_bytes());
}

// ─── Error-path coverage ───────────────────────────────────────────

#[test]
fn rejects_buffer_too_small() {
    let input = br#"{"a":1}"#;
    let mut small = [0u8; 10];
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            small.as_mut_ptr(),
            small.len(),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_BUFFER_TOO_SMALL);
}

#[test]
fn rejects_null_output_pointer() {
    let input = br#"{"a":1}"#;
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            core::ptr::null_mut(),
            71,
            core::ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_NULL_POINTER);
}

#[test]
fn rejects_null_input_with_nonzero_len() {
    let mut out = [0u8; 71];
    let rc = unsafe {
        uor_addr_json(
            core::ptr::null(),
            7,
            out.as_mut_ptr(),
            out.len(),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_NULL_POINTER);
}

#[test]
fn rejects_invalid_input() {
    let input = b"not json";
    let mut out = [0u8; 71];
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            out.as_mut_ptr(),
            out.len(),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_INVALID_INPUT);
}

#[test]
fn allows_null_out_written() {
    let input = br#"{"a":1}"#;
    let mut out = [0u8; 71];
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            out.as_mut_ptr(),
            out.len(),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_OK);
    let rust = uor_addr::json::address(input).expect("Rust ok");
    assert_eq!(&out[..], rust.address.as_bytes());
}

#[test]
fn label_byte_width_matches_const() {
    assert_eq!(UOR_ADDR_LABEL_BYTES, 71);
}
