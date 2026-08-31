//! #804 Apple Accelerate CPU BLAS path — maintainer-approved 2026-08-18.
//!
//! Routes the TEACHER weight matmuls through Apple Accelerate
//! (`cblas_sgemv`/`cblas_sgemm`) for local source-backed inference and
//! observation passes, restoring
//! the pre-#655-B2 throughput (~50–100× over the owned exact GEMM on this
//! class of machine) so corpus-scale teacher-forced observation remains
//! feasible and local generation uses the host efficiently. This is an
//! explicit opt-in to ordinary f32 CPU BLAS, not a change to the portable
//! exact or source-free runtime:
//!
//! - **Never a default.** This module only compiles under
//!   `--features observation-blas-exception` on macOS; every default build
//!   keeps the pinned `uor-matmul` exact GEMM everywhere. The
//!   `matrix_operation_census` gate pins the file, its feature gating, and
//!   its single dispatch site.
//! - **Local source-backed work only.** Teacher observation/compile passes and
//!   explicitly built local source-backed generation dispatch here; the deployed
//!   transformerless serving runtime carries no dependency on this crate's
//!   matmuls at all (P-4 contract, `INFERENCE_OPERATION_CONTRACT.md`).
//! - **Provenance is loud.** `fast_matmul_backend()` reports the exception
//!   by name in every "teacher model ready" line, so any corpus produced
//!   under it carries the backend in its run log; the #605/#643 contract
//!   amendments record it for the S1 corpus explicitly.
//! - **Numerics caveat, stated not hidden.** Accelerate accumulation order
//!   is machine-tuned; logits differ from the exact GEMM in low-order bits,
//!   so top-k near-ties can differ from an owned-GEMM pass. The S1 gates
//!   compare fitted structures against traces produced under the SAME
//!   backend, and the parity test below bounds the divergence on fixtures.

/// W (d,n) @ x (n,) -> xout (d,) via Accelerate `cblas_sgemv`.
pub(crate) fn matmul(xout: &mut [f32], x: &[f32], w: &[f32], n: usize) {
    const CBLAS_ROW_MAJOR: i32 = 101;
    const CBLAS_NO_TRANSPOSE: i32 = 111;
    debug_assert!(w.len() >= xout.len() * n);
    debug_assert_eq!(x.len(), n);
    // SAFETY: all pointers refer to initialized, non-overlapping f32 slices;
    // their dimensions and strides describe W[xout.len(), n] and x[n].
    unsafe {
        cblas_sgemv(
            CBLAS_ROW_MAJOR,
            CBLAS_NO_TRANSPOSE,
            i32::try_from(xout.len()).expect("teacher output dimension exceeds CBLAS i32"),
            i32::try_from(n).expect("teacher input dimension exceeds CBLAS i32"),
            1.0,
            w.as_ptr(),
            i32::try_from(n).expect("teacher stride exceeds CBLAS i32"),
            x.as_ptr(),
            1,
            0.0,
            xout.as_mut_ptr(),
            1,
        );
    }
}

/// `C[batch, rows] = X[batch, n] · W[rows, n]ᵀ` via Accelerate `cblas_sgemm`,
/// sequence-major exactly like the owned batched path it substitutes.
pub(crate) fn matmul_batched(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, batch: usize) {
    debug_assert!(batch > 0);
    debug_assert_eq!(xout.len() % batch, 0);
    let rows = xout.len() / batch;
    debug_assert!(w.len() >= rows * n);
    debug_assert_eq!(x.len(), batch * n);
    const CBLAS_ROW_MAJOR: i32 = 101;
    const CBLAS_NO_TRANSPOSE: i32 = 111;
    const CBLAS_TRANSPOSE: i32 = 112;
    // SAFETY: all pointers refer to initialized, non-overlapping f32 slices
    // whose dimensions/strides describe X[batch, n], W[rows, n], C[batch, rows].
    unsafe {
        cblas_sgemm(
            CBLAS_ROW_MAJOR,
            CBLAS_NO_TRANSPOSE,
            CBLAS_TRANSPOSE,
            i32::try_from(batch).expect("teacher batch exceeds CBLAS i32"),
            i32::try_from(rows).expect("teacher output dimension exceeds CBLAS i32"),
            i32::try_from(n).expect("teacher input dimension exceeds CBLAS i32"),
            1.0,
            x.as_ptr(),
            i32::try_from(n).expect("teacher input stride exceeds CBLAS i32"),
            w.as_ptr(),
            i32::try_from(n).expect("teacher weight stride exceeds CBLAS i32"),
            0.0,
            xout.as_mut_ptr(),
            i32::try_from(rows).expect("teacher output stride exceeds CBLAS i32"),
        );
    }
}

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_sgemv(
        order: i32,
        transpose: i32,
        rows: i32,
        columns: i32,
        alpha: f32,
        matrix: *const f32,
        leading_dimension: i32,
        vector: *const f32,
        vector_stride: i32,
        beta: f32,
        output: *mut f32,
        output_stride: i32,
    );

    fn cblas_sgemm(
        order: i32,
        transpose_a: i32,
        transpose_b: i32,
        m: i32,
        n: i32,
        k: i32,
        alpha: f32,
        a: *const f32,
        lda: i32,
        b: *const f32,
        ldb: i32,
        beta: f32,
        c: *mut f32,
        ldc: i32,
    );
}

#[cfg(test)]
mod tests {
    /// The exception's correctness witness: Accelerate sgemv/sgemm agree
    /// with the owned exact GEMM within f32 tolerance on a deterministic
    /// fixture — the divergence is low-order accumulation bits, not
    /// structure.
    #[test]
    fn accelerate_matches_the_owned_exact_gemm_within_tolerance() {
        let d = 7usize;
        let n = 13usize;
        let batch = 3usize;
        let w: Vec<f32> = (0..d * n)
            .map(|i| ((i * 37 % 19) as f32 - 9.0) * 0.125)
            .collect();
        let x: Vec<f32> = (0..batch * n)
            .map(|i| ((i * 23 % 17) as f32 - 8.0) * 0.25)
            .collect();

        // Serial: one vector at a time against the owned exact GEMM.
        for b in 0..batch {
            let xb = &x[b * n..(b + 1) * n];
            let mut exact = vec![0f32; d];
            let mut pa = vec![uor_matmul::PackedCode::default(); n];
            let mut pb = vec![uor_matmul::PackedCode::default(); n];
            uor_matmul::slice::gemm_float(d, n, 1, &w, xb, &mut exact, &mut pa, &mut pb)
                .expect("exact reference");
            let mut fast = vec![0f32; d];
            super::matmul(&mut fast, xb, &w, n);
            for (row, (a, e)) in fast.iter().zip(exact.iter()).enumerate() {
                assert!(
                    (a - e).abs() <= 1e-4 * (1.0 + e.abs()),
                    "sgemv row {row} diverges: accelerate={a}, exact={e}"
                );
            }
        }

        // Batched: sgemm against per-vector sgemv (self-consistency) and
        // against the exact reference.
        let mut batched = vec![0f32; batch * d];
        super::matmul_batched(&mut batched, &x, &w, n, batch);
        for b in 0..batch {
            let xb = &x[b * n..(b + 1) * n];
            let mut exact = vec![0f32; d];
            let mut pa = vec![uor_matmul::PackedCode::default(); n];
            let mut pb = vec![uor_matmul::PackedCode::default(); n];
            uor_matmul::slice::gemm_float(d, n, 1, &w, xb, &mut exact, &mut pa, &mut pb)
                .expect("exact reference");
            for row in 0..d {
                let a = batched[b * d + row];
                let e = exact[row];
                assert!(
                    (a - e).abs() <= 1e-4 * (1.0 + e.abs()),
                    "sgemm b={b} row={row} diverges: accelerate={a}, exact={e}"
                );
            }
        }
    }
}
