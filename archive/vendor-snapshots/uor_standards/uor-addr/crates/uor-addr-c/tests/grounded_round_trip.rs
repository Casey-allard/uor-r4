//! C ABI grounded-resource tests — exercise the TC-05 cross-language
//! replay surface (`uor_addr_<realization>_with_witness` →
//! `uor_addr_grounded_verify`).
//!
//! Pinned invariants:
//!
//! - **CL-R-FFI-01**: every `*_with_witness` mint returns a handle
//!   whose `uor_addr_grounded_kappa_label` matches the κ-label the
//!   parallel `uor_addr_<realization>` flat call produces.
//! - **CL-R-FFI-02**: `uor_addr_grounded_verify` returns the same
//!   κ-label byte-for-byte after replaying the derivation through
//!   `prism_verify::certify_from_trace` (the QS-05 replay
//!   equivalence — SHA-256 is not re-invoked).
//! - **CL-R-FFI-03**: the 32-byte content fingerprint is deterministic
//!   across calls on the same handle (the fingerprint is prism's
//!   content-address of the Grounded's full state — distinct from
//!   the κ-label's SHA-256 digest, which is a function of the
//!   canonical-form bytes only).
//! - **CL-R-FFI-04**: `uor_addr_grounded_free` is a no-op on null and
//!   the test suite leaks no handles.

// Test names use the `<id>__<short_description>` convention so failures
// trace back to a CONFORMANCE.md row by ID. Same convention as
// `crates/uor-addr/tests/replay.rs`.
#![allow(non_snake_case)]

use core::ptr;

use uor_addr_c::*;

const KAPPA_BYTES: usize = 71;

fn mint(
    input: &[u8],
    f: unsafe extern "C" fn(*const u8, usize, *mut *mut UorAddrGrounded) -> i32,
) -> *mut UorAddrGrounded {
    let mut handle: *mut UorAddrGrounded = ptr::null_mut();
    let rc = unsafe { f(input.as_ptr(), input.len(), &mut handle as *mut _) };
    assert_eq!(rc, UOR_ADDR_OK, "*_with_witness must succeed");
    assert!(!handle.is_null(), "handle must be non-null on success");
    handle
}

fn read_label(handle: *const UorAddrGrounded) -> [u8; KAPPA_BYTES] {
    let mut buf = [0u8; KAPPA_BYTES];
    let mut written: usize = 0;
    let rc =
        unsafe { uor_addr_grounded_kappa_label(handle, buf.as_mut_ptr(), buf.len(), &mut written) };
    assert_eq!(rc, UOR_ADDR_OK);
    assert_eq!(written, KAPPA_BYTES);
    buf
}

fn read_fingerprint(handle: *const UorAddrGrounded) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let mut written: usize = 0;
    let rc = unsafe {
        uor_addr_grounded_content_fingerprint(handle, buf.as_mut_ptr(), buf.len(), &mut written)
    };
    assert_eq!(rc, UOR_ADDR_OK);
    assert_eq!(written, 32);
    buf
}

fn verify(handle: *const UorAddrGrounded) -> [u8; KAPPA_BYTES] {
    let mut buf = [0u8; KAPPA_BYTES];
    let mut written: usize = 0;
    let rc = unsafe { uor_addr_grounded_verify(handle, buf.as_mut_ptr(), buf.len(), &mut written) };
    assert_eq!(rc, UOR_ADDR_OK, "verify must succeed for a fresh witness");
    assert_eq!(written, KAPPA_BYTES);
    buf
}

fn drop_handle(handle: *mut UorAddrGrounded) {
    unsafe { uor_addr_grounded_free(handle) };
}

// ─── CL-R-FFI-01 + CL-R-FFI-02 ─────────────────────────────────────

#[test]
fn cl_r_ffi_01__json_witness_label_matches_flat_call() {
    let input = br#"{"foo":"bar"}"#;
    let handle = mint(input, uor_addr_json_with_witness);
    let label_from_witness = read_label(handle);

    // Flat call for parity.
    let mut flat = [0u8; KAPPA_BYTES];
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            flat.as_mut_ptr(),
            flat.len(),
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_OK);
    assert_eq!(
        label_from_witness, flat,
        "witness label must equal flat-call label (byte-for-byte)"
    );

    drop_handle(handle);
}

#[test]
fn cl_r_ffi_02__verify_returns_same_label_as_mint() {
    let input = br#"{"foo":"bar"}"#;
    let handle = mint(input, uor_addr_json_with_witness);
    let mint_label = read_label(handle);
    let verify_label = verify(handle);
    assert_eq!(
        mint_label, verify_label,
        "QS-05 replay equivalence: verify must return the same κ-label"
    );
    drop_handle(handle);
}

// ─── CL-R-FFI-03 ───────────────────────────────────────────────────

#[test]
fn cl_r_ffi_03__fingerprint_is_deterministic() {
    let input = br#"{"foo":"bar"}"#;
    let handle_a = mint(input, uor_addr_json_with_witness);
    let handle_b = mint(input, uor_addr_json_with_witness);
    let fp_a = read_fingerprint(handle_a);
    let fp_b = read_fingerprint(handle_b);
    assert_eq!(
        fp_a, fp_b,
        "content fingerprint must be deterministic across calls"
    );
    drop_handle(handle_a);
    drop_handle(handle_b);
}

// ─── CL-R-FFI-04 ───────────────────────────────────────────────────

#[test]
fn cl_r_ffi_04__free_on_null_is_noop() {
    // Should not crash or have observable effect.
    unsafe { uor_addr_grounded_free(ptr::null_mut()) };
}

// ─── Cross-realization sweep ────────────────────────────────────────

#[test]
fn cross_realization__witness_round_trip_for_every_realization() {
    let cases: &[(
        &[u8],
        unsafe extern "C" fn(*const u8, usize, *mut *mut UorAddrGrounded) -> i32,
    )] = &[
        (br#"{"foo":"bar"}"#, uor_addr_json_with_witness),
        (b"(a b c)", uor_addr_sexp_with_witness),
        (b"<root/>", uor_addr_xml_with_witness),
        (&[2u8, 0u8, 1u8, 0u8], uor_addr_ring_with_witness),
        (b"(3:mod 5:empty)", uor_addr_codemodule_with_witness),
    ];

    for (i, (input, mint_fn)) in cases.iter().enumerate() {
        let handle = mint(input, *mint_fn);
        let label = read_label(handle);
        let verified = verify(handle);
        assert_eq!(label, verified, "case {i}: verify must round-trip");
        drop_handle(handle);
    }
}

// ─── Error paths ───────────────────────────────────────────────────

#[test]
fn invalid_input_rejected_at_mint() {
    let mut handle: *mut UorAddrGrounded = ptr::null_mut();
    let rc = unsafe { uor_addr_json_with_witness(b"not json".as_ptr(), 8, &mut handle as *mut _) };
    assert_eq!(rc, UOR_ADDR_ERR_INVALID_INPUT);
    assert!(handle.is_null(), "no handle should be written on failure");
}

#[test]
fn null_out_handle_rejected() {
    let rc = unsafe { uor_addr_json_with_witness(b"{}".as_ptr(), 2, ptr::null_mut()) };
    assert_eq!(rc, UOR_ADDR_ERR_NULL_POINTER);
}

#[test]
fn null_handle_rejected_on_accessors() {
    let mut buf = [0u8; KAPPA_BYTES];
    let rc = unsafe {
        uor_addr_grounded_kappa_label(ptr::null(), buf.as_mut_ptr(), buf.len(), ptr::null_mut())
    };
    assert_eq!(rc, UOR_ADDR_ERR_NULL_POINTER);

    let rc = unsafe {
        uor_addr_grounded_verify(ptr::null(), buf.as_mut_ptr(), buf.len(), ptr::null_mut())
    };
    assert_eq!(rc, UOR_ADDR_ERR_NULL_POINTER);
}

#[test]
fn undersized_buffer_rejected() {
    let input = br#"{"foo":"bar"}"#;
    let handle = mint(input, uor_addr_json_with_witness);
    let mut tiny = [0u8; 10];
    let rc = unsafe {
        uor_addr_grounded_kappa_label(handle, tiny.as_mut_ptr(), tiny.len(), ptr::null_mut())
    };
    assert_eq!(rc, UOR_ADDR_ERR_BUFFER_TOO_SMALL);
    drop_handle(handle);
}
