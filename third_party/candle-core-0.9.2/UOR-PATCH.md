# Pinned Candle0.9.2: Apple BLAS slice correction

The published crate is retained with its Apache2.0 license and original source.
UPSTREAM.json binds its archive, upstream commit and every retained source file.
The first patch changes four executable source lines, all under
`cfg(feature="accelerate")` in `src/cpu_backend/mod.rs`; the second patch
(below) changes gradient accumulation in `src/backprop.rs`. No Metal or portable gemm formula changes.

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

## Second patch: first gradient contribution stored directly (#820)

`src/backprop.rs` accumulated every gradient contribution as
`grads.or_insert(x)` (a `zeros_like` fill on first use) followed by
`sum_grad.add(&g)`. A tensor with a single contribution therefore cost a
full-size zero fill and a full-size addition. `GradStore::accumulate` (and
`accumulate_neg` for the `sub` rules) now stores the first contribution
directly and adds only from the second contribution on, in the same order.

`0 + g == g` holds exactly in IEEE-754 (only `+0 + -0 == +0` changes the sign
of a zero), so gradient values are unchanged apart from the sign of zeros. The
stored first contribution keeps upstream's layout: a strided, broadcast or
offset view is copied into a fresh contiguous buffer. A contribution whose
shape, dtype or device differs from its tensor takes the upstream path, so it
reports the same error. `Gather`, `IndexSelect` and the upsample rules keep
`or_insert` (they scatter into, or overwrite, the zero tensor).

`UOR_CANDLE_GRAD_ZERO_THEN_ADD=1`, or `backprop::set_upstream_zero_then_add`,
restores the upstream path for old-vs-new comparisons. The parity record is
`crates/uor-r4-training/tests/backprop_first_contribution_parity.rs`.
