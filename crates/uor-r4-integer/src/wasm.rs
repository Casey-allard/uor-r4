//! C-ABI helper functions for WebAssembly builds of the integer runtime.
//!
//! These thirteen functions return model constants or wrap single integer
//! kernels (Min-P threshold, Hopf projection, zeta phase step, Galois LFSR,
//! turn prime signature, Q30 arctangent, token salience). There is no load,
//! session or step export, so a module built from this crate is not a serving
//! runtime. The functions are not `#[no_mangle]` (the crate forbids unsafe
//! code), so a module exposes them only when it is built with the `rlib` and
//! `cdylib` crate types together and linked with `--export-all`; with `cdylib`
//! alone the unreferenced helpers are dropped before linking.
//!
//! The source is written to the declared D11 contract; that does not make a
//! compiled module D11-clean. A WebAssembly build can contain `i64.mul` where
//! the source uses shift-and-add: the frozen 2026-09-28 module has `i64.mul`
//! by 3 in `UnitS3Q30::hopf_project` and `UnitS3Q30::from_i32_coords`, which
//! the Hopf helpers call. See `scripts/audit_zero_matmul_wasm.py --call-graph`.

use crate::math::{atan2_q30, UnitS3Q30};
use crate::model::{galois_lfsr_step, score_token_salience, KEY_DIM};
use crate::sampling::exact_min_p_threshold;

/// API version identifier: 0x00010000 = v1.0.0
#[inline(never)]
pub extern "C" fn uor_wasm_version() -> u32 {
    0x00010000
}

/// Width-576 dialogue model state dimension (a constant, not read from a bundle).
#[inline(never)]
pub extern "C" fn uor_wasm_state_dim() -> u32 {
    576
}

/// Vocabulary size (a constant, not read from a bundle).
#[inline(never)]
pub extern "C" fn uor_wasm_vocab_size() -> u32 {
    4096
}

/// Context capacity (a constant, not read from a bundle).
#[inline(never)]
pub extern "C" fn uor_wasm_context_capacity() -> u32 {
    256
}

/// Min-P threshold from `sampling::exact_min_p_threshold`.
#[inline(never)]
pub extern "C" fn uor_wasm_exact_min_p_threshold(p_max: u64, min_p_q16: u32) -> u64 {
    exact_min_p_threshold(p_max, min_p_q16)
}

/// Hopf S3 -> S2 projection, base coordinate X in Q30.
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_x(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [x, _, _] = u.hopf_project();
    x
}

/// Hopf S3 -> S2 projection, base coordinate Y in Q30.
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_y(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [_, y, _] = u.hopf_project();
    y
}

/// Hopf S3 -> S2 projection, base coordinate Z in Q30.
#[inline(never)]
pub extern "C" fn uor_wasm_hopf_project_z(q0: i32, q1: i32, q2: i32, q3: i32) -> i32 {
    let u = UnitS3Q30::from_i32_coords(&[q0, q1, q2, q3]);
    let [_, _, z] = u.hopf_project();
    z
}

/// Scalar zeta phase step modulo 2^31, with the frequency multiple formed by
/// shift-and-add over the four low bits of `k`.
#[inline(never)]
pub extern "C" fn uor_wasm_step_zeta_phase_scalar(
    phase: i32,
    freq_val: i32,
    token: u32,
    j: u32,
) -> i32 {
    let freq = freq_val as i64;
    let k = 1i64 + (((token as i64) + (j as i64)) & 0x07);

    // 4-bit unrolled shift-add multiplication: k in [1, 8]
    let mut prod = 0i64;
    if k & 1 != 0 {
        prod += freq;
    }
    if k & 2 != 0 {
        prod += freq << 1;
    }
    if k & 4 != 0 {
        prod += freq << 2;
    }
    if k & 8 != 0 {
        prod += freq << 3;
    }

    let next_phase = (phase as i64) + prod;
    let wrapped = ((next_phase % (1i64 << 31)) + (1i64 << 31)) % (1i64 << 31);
    wrapped as i32
}

/// Galois LFSR state update from `model::galois_lfsr_step`.
#[inline(never)]
pub extern "C" fn uor_wasm_galois_lfsr_step(state: u64) -> u64 {
    galois_lfsr_step(state)
}

/// Prime signature of a one-token turn from `model::compute_turn_prime_signature`.
#[inline(never)]
pub extern "C" fn uor_wasm_compute_turn_prime_signature(
    turn_id: u32,
    token: u32,
    start_seq: u64,
    end_seq: u64,
) -> u64 {
    let tokens = [token];
    let slots = [0usize];
    crate::model::compute_turn_prime_signature(&tokens, &slots, turn_id, start_seq, end_seq)
}

/// Fixed-point Q30 arctangent from `math::atan2_q30`.
#[inline(never)]
pub extern "C" fn uor_wasm_atan2_q30(y: i32, x: i32) -> i32 {
    atan2_q30(y, x)
}

/// Token salience score against an all-zero key, from `model::score_token_salience`.
#[inline(never)]
pub extern "C" fn uor_wasm_score_token_salience_default_key(token: u32) -> i32 {
    let key = [0i32; KEY_DIM];
    score_token_salience(token, &key)
}
