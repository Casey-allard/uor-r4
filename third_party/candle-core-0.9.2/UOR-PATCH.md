# Pinned Candle0.9.2: native CUDA vendor boundary and retained patches

## Active patch: D24/D29 native CUDA vendor boundary, 13 October 2026

The compiled backend admits CUDA tensor primitives without cuBLAS, cuBLASLt,
cuDNN or NVRTC. `CudaDevice` no longer initializes or stores a BLAS handle.
Dense CUDA contractions, convolution vendor fallbacks, quantized matrix/MoE
forwards and runtime UG compilation return explicit errors. Kernel admission
refuses unavailable module descriptors and prohibited squash/contraction names
before cache lookup or driver loading. Legal elementwise, gather, scatter,
sort and reduction kernels retain their CUDA path; the separately patched
Candle-kernels dependency owns their allowlisted PTX contents.

The CPU backend retains typed host storage, raw copies and serialization only.
CPU tensor arithmetic and custom/inplace callbacks return errors before
numeric execution. SIMD/dot implementations and scalar operation methods are
excluded from the module graph. GEMM, Accelerate and MKL dependency edges are
removed. `metal`, `accelerate`, `mkl` and `cudnn` remain empty compatibility
feature identifiers; selecting any produces a focused D29 compile error.
Their vendor dependencies and activation edges are absent, and historical
Metal module declarations are excluded even when the refused flag is selected.
There is no host fallback or fabricated model output. Host buffer initialization is
storage preparation, not permission for CPU model computation.

The scalar/quantized patch retains GGML/GGUF block/storage layouts and raw
artifact loading, while refusing model codecs/contractions. Operation types
and kernel identifiers remain metadata for CUDA dispatch. Original CPU
implementations are retained under `historical/d29-upstream/`; original scalar
and quantized source is under `historical/d29-scalar-quantized/`. Undeclared
legacy SIMD, convolution and BLAS source files remain inactive evidence. The
delivery's indexed restoration patch preserves the exact pre-unit crate.

`UPSTREAM.json` retains its original archive/source identities, including the
prior corrections below. These original hashes are provenance, not hashes of
the current patched files. Focused source/refusal contracts were added for
kernel admission, host-storage copying and CPU callback exclusion. Parsing and
source checks have passed; compilation and execution are pending the owning
lab's CUDA qualification. No training or scientific result is claimed.

Historical example, integration-test and benchmark registrations and their
unused development dependencies are removed from the active manifest. Their
source remains preserved. Automatic examples/tests/benches and doctests are
disabled; focused library refusal contracts remain available to the owning lab.

## Historical patch: Apple BLAS slice correction

The following original description is retained as evidence. CPU Accelerate
execution is now unavailable under D24/D29; it cannot be reactivated by this
historical permission.

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

Scalar binary/comparison adapters transfer the literal's storage to the target
first, then perform dtype conversion there. Upstream's cast-before-transfer
order attempted an F64-to-F32 CPU cast for CUDA clipping/comparisons. The finite
clipping control exposed that refusal; the correction keeps casts on CUDA and
does not restore CPU arithmetic or change the stored analytic expectations.
