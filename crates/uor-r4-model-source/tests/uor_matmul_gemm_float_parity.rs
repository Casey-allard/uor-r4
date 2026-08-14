//! #655-B step 1 — differential parity harness for the pinned `uor-matmul`
//! exact matmul substrate, on teacher-shaped f32 operands.
//!
//! This does not yet switch the teacher (that is the CID-changing step, with a
//! new artifact era + κ re-pin). It proves the promoted production dependency
//! integrates and pins down the exact numerical relationship the switch will
//! introduce:
//!
//! - `uor_matmul::slice::gemm_float` accumulates every product into a complete
//!   accumulator and rounds once, so it returns the **correctly-rounded exact**
//!   dot product. This test asserts it equals an f64-accumulated reference
//!   bit-for-bit (f64 holds the sum of `k` f32 products exactly at these sizes),
//!   and — the key property for #655 — that the result is identical across two
//!   independent runs (portable determinism, unlike per-machine Accelerate).
//! - It also measures the gap against a naive sequential-f32 accumulation (what
//!   the current `cblas_sgemm`/hand-rolled teacher path approximates). That gap
//!   is precisely what #655-B's κ re-pin will encode: the teacher moves from a
//!   per-machine rounded sum to the portable correctly-rounded exact value.
#![cfg(not(target_arch = "wasm32"))]

/// Representative teacher projection shapes (m = batch, k = in, n = out):
/// GPT-2/Llama-ish attention and MLP widths, kept small so the test is fast.
const SHAPES: &[(usize, usize, usize)] = &[
    (1, 288, 288),
    (4, 288, 288),
    (2, 288, 128),
    (4, 128, 256),
    (2, 64, 256),
];

/// Deterministic f32 operand in roughly [-1, 1), from a SplitMix64 step — no
/// external rng crate, and reproducible so the harness is itself deterministic.
fn gen(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // 24-bit mantissa fraction → [0,1), map to [-1,1).
    let unit = (z >> 40) as f32 / (1u32 << 24) as f32;
    unit * 2.0 - 1.0
}

fn fill(len: usize, seed: &mut u64) -> Vec<f32> {
    (0..len).map(|_| gen(seed)).collect()
}

/// Correctly-rounded exact reference: accumulate `A·B` in f64 (exact for these
/// `k`), round each cell once to f32. This is the value `gemm_float` targets.
fn reference_exact(m: usize, k: usize, n: usize, a: &[f32], b: &[f32]) -> Vec<f32> {
    let mut c = vec![0f32; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut acc = 0f64;
            for p in 0..k {
                acc += f64::from(a[i * k + p]) * f64::from(b[p * n + j]);
            }
            c[i * n + j] = acc as f32;
        }
    }
    c
}

/// Naive sequential f32 accumulation — the rounding regime the current teacher
/// (`cblas_sgemm` / hand-rolled) approximates. Used only to measure the gap.
fn reference_naive_f32(m: usize, k: usize, n: usize, a: &[f32], b: &[f32]) -> Vec<f32> {
    let mut c = vec![0f32; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut acc = 0f32;
            for p in 0..k {
                acc += a[i * k + p] * b[p * n + j];
            }
            c[i * n + j] = acc;
        }
    }
    c
}

fn gemm_float_exact(m: usize, k: usize, n: usize, a: &[f32], b: &[f32]) -> Vec<f32> {
    use uor_matmul::PackedCode;
    let mut c = vec![0f32; m * n];
    let mut pa = vec![PackedCode::default(); k];
    let mut pb = vec![PackedCode::default(); k * n];
    uor_matmul::slice::gemm_float(m, k, n, a, b, &mut c, &mut pa, &mut pb)
        .expect("gemm_float over finite f32 operands is total");
    c
}

#[test]
fn gemm_float_equals_correctly_rounded_exact_and_is_portable_deterministic() {
    let mut max_rel_vs_naive = 0f64;
    let mut differing_cells = 0usize;
    let mut total_cells = 0usize;
    let mut worst_shape = (0, 0, 0);
    for &(m, k, n) in SHAPES {
        let mut sa = 0x1234_5678_9abc_def0u64 ^ ((m * 131 + k * 17 + n) as u64);
        let mut sb = 0x0fed_cba9_8765_4321u64 ^ ((m + k * 7 + n * 251) as u64);
        let a = fill(m * k, &mut sa);
        let b = fill(k * n, &mut sb);

        let exact = reference_exact(m, k, n, &a, &b);
        let got = gemm_float_exact(m, k, n, &a, &b);

        // Correctly-rounded exact: bit-identical to the f64-accumulated round.
        for (idx, (g, e)) in got.iter().zip(&exact).enumerate() {
            assert_eq!(
                g.to_bits(),
                e.to_bits(),
                "gemm_float cell {idx} at shape {m}x{k}x{n} is {g} ({:#010x}), \
                 correctly-rounded exact is {e} ({:#010x})",
                g.to_bits(),
                e.to_bits()
            );
        }

        // Portable determinism: a second independent run is byte-identical.
        let again = gemm_float_exact(m, k, n, &a, &b);
        assert_eq!(
            got.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            again.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "gemm_float must be run-to-run deterministic at shape {m}x{k}x{n}"
        );

        // Measure the gap vs naive sequential f32 (the current teacher regime).
        let naive = reference_naive_f32(m, k, n, &a, &b);
        total_cells += got.len();
        for (g, nv) in got.iter().zip(&naive) {
            if g.to_bits() != nv.to_bits() {
                differing_cells += 1;
            }
            let denom = g.abs().max(nv.abs());
            if denom > 0.0 {
                let rel = f64::from((g - nv).abs()) / f64::from(denom);
                if rel > max_rel_vs_naive {
                    max_rel_vs_naive = rel;
                    worst_shape = (m, k, n);
                }
            }
        }
    }

    eprintln!(
        "gemm_float parity: correctly-rounded exact on all {} shapes; vs naive sequential-f32 \
         {}/{} cells differ, max relative error {:.2e} (worst shape {:?}). \
         #655-B's κ re-pin encodes exactly this teacher-arithmetic change.",
        SHAPES.len(),
        differing_cells,
        total_cells,
        max_rel_vs_naive,
        worst_shape
    );
}
