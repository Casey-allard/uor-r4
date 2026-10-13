# UOR ahead-of-time CUDA module boundary

Upstream: candle-kernels 0.9.2, copied from the Cargo registry with its licenses.

The active build explicitly compiles nine tensor modules: affine, binary, cast,
fill, indexing, reduce, sort, ternary and unary. It calls nvcc at build time and
embeds their PTX. There is no runtime compiler, generated source discovery, MOE
archive build or vendor BLAS link. The original build script, modified CUDA
inputs and disabled conv/quantized/MOE/FFI source are retained in historical/;
they are inactive evidence, never compiled by this build.

UNARY excludes exponential, logarithmic, sigmoid, tanh, erf/CDF, GELU, SiLU,
ELU and general power entries. REDUCE excludes softmax and all RoPE exports.
The pre-RoPE-exclusion reduction module is preserved separately as
`historical/reduce-pre-rope-exclusion.cu`; the upstream original also remains.
Corresponding helpers
are absent from the active shared header. Legal elementwise, reduction,
indexing and transport kernels retain their existing formulas. CONV and
QUANTIZED descriptors carry empty PTX, so the patched backend refuses them.

Compiler options preserve IEEE division/square-root and disable flushing
subnormals. NVCC and UOR_NVCC_ARCH select the compiler and virtual architecture;
the default is compute_80. The PTX token check is a supplementary bounded guard,
not semantic proof. The independent Rust verifier and qualified source/PTX/
executable identities are recorded by the owning delivery. This patch itself
makes no model-quality or runtime qualification claim.
