//! C ABI κ-label composition tests (ADR-061) — exercise the
//! `uor_addr_compose_<op>[_with_witness]` surface across σ-axes.
//!
//! Pinned invariants:
//!
//! - **CL-C-FFI-01**: a composed κ-label produced through the C ABI is
//!   byte-for-byte the κ-label the in-crate `composition::compose_*`
//!   entry point produces (FFI is a thin pass-through, no re-derivation).
//! - **CL-C-FFI-02**: CS-G2 is commutative across the C ABI —
//!   `compose_g2(a, b) == compose_g2(b, a)`.
//! - **CL-C-FFI-03**: a witness handle minted by
//!   `uor_addr_compose_<op>_with_witness` verifies (TC-05 replay) to the
//!   same κ-label its label entry point yields.
//! - **CL-C-FFI-04**: a malformed operand is rejected with
//!   `UOR_ADDR_ERR_INVALID_INPUT`; an unknown `algo` selector with
//!   `UOR_ADDR_ERR_UNKNOWN_HASH`.

// `<id>__<short_description>` test names trace failures back to a
// CONFORMANCE.md row by ID (same convention as `grounded_round_trip.rs`).
#![allow(non_snake_case)]

use core::ptr;

use uor_addr_c::*;

const KAPPA_BYTES: usize = 71;

/// Address a JSON document to a sha256 κ-label (the operand source).
fn json_label(input: &[u8]) -> [u8; KAPPA_BYTES] {
    let mut buf = [0u8; KAPPA_BYTES];
    let rc = unsafe {
        uor_addr_json(
            input.as_ptr(),
            input.len(),
            buf.as_mut_ptr(),
            buf.len(),
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, UOR_ADDR_OK, "json mint must succeed");
    buf
}

/// Compose a unary operation, returning the composed κ-label bytes.
fn compose_unary(
    f: unsafe extern "C" fn(u8, *const u8, usize, *mut u8, usize, *mut usize) -> i32,
    operand: &[u8],
) -> Vec<u8> {
    let mut buf = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
    let mut written: usize = 0;
    let rc = unsafe {
        f(
            UOR_ADDR_HASH_SHA256,
            operand.as_ptr(),
            operand.len(),
            buf.as_mut_ptr(),
            buf.len(),
            &mut written,
        )
    };
    assert_eq!(rc, UOR_ADDR_OK, "unary compose must succeed");
    buf.truncate(written);
    buf
}

// ─── CL-C-FFI-02 — CS-G2 commutativity across the C ABI ─────────────

#[test]
fn cl_c_ffi_02__g2_is_commutative() {
    let a = json_label(br#"{"role":"left"}"#);
    let b = json_label(br#"{"role":"right"}"#);

    let mut ab = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
    let mut ab_written = 0usize;
    let mut ba = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
    let mut ba_written = 0usize;

    let rc_ab = unsafe {
        uor_addr_compose_g2(
            UOR_ADDR_HASH_SHA256,
            a.as_ptr(),
            a.len(),
            b.as_ptr(),
            b.len(),
            ab.as_mut_ptr(),
            ab.len(),
            &mut ab_written,
        )
    };
    let rc_ba = unsafe {
        uor_addr_compose_g2(
            UOR_ADDR_HASH_SHA256,
            b.as_ptr(),
            b.len(),
            a.as_ptr(),
            a.len(),
            ba.as_mut_ptr(),
            ba.len(),
            &mut ba_written,
        )
    };
    assert_eq!(rc_ab, UOR_ADDR_OK);
    assert_eq!(rc_ba, UOR_ADDR_OK);
    ab.truncate(ab_written);
    ba.truncate(ba_written);
    assert_eq!(ab, ba, "CS-G2 must be commutative across the C ABI");
    assert!(ab.starts_with(b"sha256:") && ab.len() == KAPPA_BYTES);
}

// ─── CL-C-FFI-01 — FFI label parity with the in-crate entry point ───

#[test]
fn cl_c_ffi_01__ffi_label_matches_in_crate() {
    let a = json_label(br#"{"role":"left"}"#);
    let b = json_label(br#"{"role":"right"}"#);

    // In-crate CS-G2 reference.
    let la = uor_addr::KappaLabel::<71>::from_bytes(&a).expect("operand a parses");
    let lb = uor_addr::KappaLabel::<71>::from_bytes(&b).expect("operand b parses");
    let in_crate = uor_addr::composition::compose_g2_product(&la, &lb)
        .expect("in-crate g2")
        .address;

    // FFI CS-G2.
    let mut ffi = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
    let mut written = 0usize;
    let rc = unsafe {
        uor_addr_compose_g2(
            UOR_ADDR_HASH_SHA256,
            a.as_ptr(),
            a.len(),
            b.as_ptr(),
            b.len(),
            ffi.as_mut_ptr(),
            ffi.len(),
            &mut written,
        )
    };
    assert_eq!(rc, UOR_ADDR_OK);
    ffi.truncate(written);

    assert_eq!(
        ffi,
        in_crate.as_str().as_bytes(),
        "FFI g2 label must equal the in-crate compose_g2_product label"
    );
}

// ─── CL-C-FFI-03 — witness replay for every unary op ────────────────

#[test]
fn cl_c_ffi_03__witness_round_trips_for_every_op() {
    let a = json_label(br#"{"role":"left"}"#);

    let label_ops: &[unsafe extern "C" fn(
        u8,
        *const u8,
        usize,
        *mut u8,
        usize,
        *mut usize,
    ) -> i32] = &[
        uor_addr_compose_f4,
        uor_addr_compose_e6,
        uor_addr_compose_e7,
        uor_addr_compose_e8,
    ];
    let witness_ops: &[unsafe extern "C" fn(
        u8,
        *const u8,
        usize,
        *mut *mut UorAddrGrounded,
    ) -> i32] = &[
        uor_addr_compose_f4_with_witness,
        uor_addr_compose_e6_with_witness,
        uor_addr_compose_e7_with_witness,
        uor_addr_compose_e8_with_witness,
    ];

    for (label_fn, witness_fn) in label_ops.iter().zip(witness_ops.iter()) {
        let label = compose_unary(*label_fn, &a);

        let mut handle: *mut UorAddrGrounded = ptr::null_mut();
        let rc = unsafe {
            witness_fn(
                UOR_ADDR_HASH_SHA256,
                a.as_ptr(),
                a.len(),
                &mut handle as *mut _,
            )
        };
        assert_eq!(rc, UOR_ADDR_OK, "witness compose must succeed");
        assert!(!handle.is_null());

        let mut verified = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
        let mut written = 0usize;
        let rc = unsafe {
            uor_addr_grounded_verify(handle, verified.as_mut_ptr(), verified.len(), &mut written)
        };
        assert_eq!(rc, UOR_ADDR_OK, "TC-05 verify must succeed");
        verified.truncate(written);

        assert_eq!(label, verified, "witness must replay to the label");
        unsafe { uor_addr_grounded_free(handle) };
    }
}

// ─── CL-C-FFI-04 — error paths ──────────────────────────────────────

#[test]
fn cl_c_ffi_04__malformed_operand_and_unknown_algo_rejected() {
    let mut buf = vec![0u8; UOR_ADDR_MAX_LABEL_BYTES];
    let mut written = 0usize;

    // Malformed operand (not a κ-label).
    let bad = b"not-a-kappa-label";
    let rc = unsafe {
        uor_addr_compose_e8(
            UOR_ADDR_HASH_SHA256,
            bad.as_ptr(),
            bad.len(),
            buf.as_mut_ptr(),
            buf.len(),
            &mut written,
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_INVALID_INPUT);

    // Unknown σ-axis selector.
    let a = json_label(br#"{"role":"left"}"#);
    let rc = unsafe {
        uor_addr_compose_e8(
            0xFF,
            a.as_ptr(),
            a.len(),
            buf.as_mut_ptr(),
            buf.len(),
            &mut written,
        )
    };
    assert_eq!(rc, UOR_ADDR_ERR_UNKNOWN_HASH);
}
