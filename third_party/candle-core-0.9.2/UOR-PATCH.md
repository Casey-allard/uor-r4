# Pinned Candle0.9.2: Apple BLAS slice correction

The published crate is retained with its Apache2.0 license and original source.
UPSTREAM.json binds its archive, upstream commit and every retained source file.
Only four executable source lines differ, all under `cfg(feature="accelerate")`
in `src/cpu_backend/mod.rs`. No Metal or portable gemm formula changes.

The Fortran BLAS adapter reverses operand pointers but constructed slices using
the unreversed operand batch skips. For nonsquare products, this can create a
Rust slice beyond its allocation even when BLAS later uses only its pointer.
Use the corresponding remaining operand slice lengths, `rhs_p.len()` and
`lhs_p.len()`, in both F32 and F64. Swapping skips alone would not cover
broadcast operands with zero batch stride. Destination c_skip is unchanged.

This local correction enables explicit offline CPU Accelerate training. It
does not enable any floating-point backend in the native serving contract.
Remove the override after adopting and verifying an upstream correction.
No cross-backend bitwise training reproducibility is claimed.
