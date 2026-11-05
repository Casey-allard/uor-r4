// The served quantizers as one kernel a direction (float-kernel-fusion, claude-285): the
// trained-as-served chains of `hamilton_sheet.rs`, `activation_quantize16` and `q16_residual`,
// each with its straight-through bridge. build.rs compiles this file ahead of time with nvcc as
// its own PTX module (`fused_quantize_f32.ptx`), apart from `cuda_stack.cu`, whose SHA-256
// earlier sheets pin. NVRTC is banned (owner, 13 October 2026).
//
// Each kernel must equal its op chain bit for bit. The chain's ops are Candle's CUDA kernels,
// built with -ftz=false -prec-div=true -prec-sqrt=true and no fast math, so each op rounds once,
// to nearest. Here every f32 add, subtract, multiply and divide is a non-contracting `__f*_rn`
// intrinsic, and every `affine(mul, add)` is `__fmaf_rn(x, mul, add)`, each in the chain's order.
// An affine whose multiplier is 1 is `__fadd_rn(x, add)` instead: `1 · x` is exact, so it is one
// rounding of the same sum, and nvcc folds `__fmaf_rn(1, x, add)` into a plain `add.f32`, which
// ptxas may contract (claude-285's G3 found four). For the constants the chains use (a power of
// two with add 0, or mul 1), an fma and a separate multiply and add round alike, a -0 product
// included (-0 + +0 is +0 both ways), so the result does not depend on whether nvcc contracts
// Candle's `x * mul + add`, nor on this file's `--fmad` choice. Comparisons, `fmaxf`/`fminf` (Candle's `maximum`/`minimum`, so `clamp`),
// `fabsf` and `floorf` are exact. The chains' scale constants arrive as f32 bit patterns, cast
// from f64 as the chains cast them (`fused_quantize.rs`), so each has one source.

typedef unsigned int uint;
typedef unsigned long long u64;

// A power of two as an f32 bit pattern, for `-126 <= e <= 127`.
__device__ __forceinline__ float pow2f(int e) { return __int_as_float((127 + e) << 23); }

// ---------------------------------------------------------------------------
// 1. `activation_quantize16`: one block per row of `width`, as the chain's row max reduces the
// last dimension.
// ---------------------------------------------------------------------------

// The row's largest magnitude. Candle's max reduction folds `fmaxf` from -infinity; the max is
// exact in any order, and `fmaxf` drops a NaN in any order, so a block fold equals it.
__device__ float row_abs_max(const float* x, u64 offset, uint width, float* shared) {
    float m = __int_as_float(0xff800000u);
    for (uint i = threadIdx.x; i < width; i += blockDim.x) m = fmaxf(m, fabsf(x[offset + i]));
    for (int o = 16; o > 0; o >>= 1) m = fmaxf(m, __shfl_xor_sync(0xffffffffu, m, o));
    uint lane = threadIdx.x & 31;
    uint warp = threadIdx.x >> 5;
    if (lane == 0) shared[warp] = m;
    __syncthreads();
    uint warps = (blockDim.x + 31) >> 5;
    float total = __int_as_float(0xff800000u);
    for (uint i = 0; i < warps; ++i) total = fmaxf(total, shared[i]);
    __syncthreads();
    return total;
}

// The row's scale, in the chain's order: `integer = clamp(affine(max, unit, 0), 0, saturated)`,
// `scale = affine(1, unit, 0)`, then six passes that divide both by `2^bits` where `integer >=
// 2^(14 + bits)`. `unit` is `2^-ACTIVATION_EXP` and `saturated` is 2^62.
__device__ float row_scale(float max, float unit, float saturated) {
    float integer = __fmaf_rn(max, unit, 0.0f);
    integer = fminf(fmaxf(integer, 0.0f), saturated);
    float scale = __fadd_rn(unit, 0.0f);
    const int passes[6] = {32, 16, 8, 4, 2, 1};
    for (int p = 0; p < 6; ++p) {
        int bits = passes[p];
        float factor = integer >= pow2f(14 + bits) ? __fadd_rn(pow2f(-bits), 0.0f) : 1.0f;
        integer = __fmul_rn(integer, factor);
        scale = __fmul_rn(scale, factor);
    }
    return scale;
}

// `inside` of the bridge: `ge(scaled, -32767) * le(scaled, 32767)` as f32.
__device__ __forceinline__ float activation_inside(float scaled) {
    return __fmul_rn(scaled >= -32767.0f ? 1.0f : 0.0f, scaled <= 32767.0f ? 1.0f : 0.0f);
}

// Forward with the bridge: `quantized = clamp(floor(affine(x * scale, 1, 0.5)), -32767, 32767)
// / scale`, `kept = x * inside`, `out = kept + (quantized - kept)`.
extern "C" __global__ void activation_quantize16_fwd(
    const float* x, float* out, uint width, uint unit_bits, uint saturated_bits
) {
    __shared__ float shared[32];
    u64 offset = (u64)blockIdx.x * width;
    float max = row_abs_max(x, offset, width, shared);
    float scale = row_scale(max, __uint_as_float(unit_bits), __uint_as_float(saturated_bits));
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        float v = x[offset + i];
        float scaled = __fmul_rn(v, scale);
        float q = floorf(__fadd_rn(scaled, 0.5f));
        q = fminf(fmaxf(q, -32767.0f), 32767.0f);
        q = __fdiv_rn(q, scale);
        float kept = __fmul_rn(v, activation_inside(scaled));
        out[offset + i] = __fadd_rn(kept, __fsub_rn(q, kept));
    }
}

// Backward: `dx = grad * inside`, with the row's scale recomputed from `x`.
extern "C" __global__ void activation_quantize16_bwd(
    const float* x, const float* grad, float* dx, uint width, uint unit_bits,
    uint saturated_bits
) {
    __shared__ float shared[32];
    u64 offset = (u64)blockIdx.x * width;
    float max = row_abs_max(x, offset, width, shared);
    float scale = row_scale(max, __uint_as_float(unit_bits), __uint_as_float(saturated_bits));
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        float scaled = __fmul_rn(x[offset + i], scale);
        dx[offset + i] = __fmul_rn(grad[offset + i], activation_inside(scaled));
    }
}

// ---------------------------------------------------------------------------
// 2. `q16_residual`: one thread an element. `scaled = affine(x, scale, 0)`, `rounded =
// affine(clamp(floor(affine(scaled, 1, 0.5)), -2^31, 2^31), inverse, 0)`, with `scale` 2^16 and
// `inverse` 2^-16. The clamp's bounds are `i32::MIN` and `i32::MAX` cast to f32; the second
// rounds to 2^31.
// ---------------------------------------------------------------------------

// `inside` of the bridge: `ge(scaled, -2^31) * lt(scaled, 2^31)` as f32.
__device__ __forceinline__ float residual_inside(float scaled) {
    return __fmul_rn(scaled >= -pow2f(31) ? 1.0f : 0.0f, scaled < pow2f(31) ? 1.0f : 0.0f);
}

// `bridge` 0: `out = rounded`; 1: `kept = x * inside`, `out = kept + (rounded - kept)`.
extern "C" __global__ void q16_residual_fwd(
    const float* x, float* out, uint total, uint bridge, uint scale_bits, uint inverse_bits
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float v = x[id];
    float scaled = __fmaf_rn(v, __uint_as_float(scale_bits), 0.0f);
    float r = floorf(__fadd_rn(scaled, 0.5f));
    r = fminf(fmaxf(r, -pow2f(31)), pow2f(31));
    r = __fmaf_rn(r, __uint_as_float(inverse_bits), 0.0f);
    if (bridge == 0u) {
        out[id] = r;
        return;
    }
    float kept = __fmul_rn(v, residual_inside(scaled));
    out[id] = __fadd_rn(kept, __fsub_rn(r, kept));
}

// Backward with the bridge: `dx = grad * inside`.
extern "C" __global__ void q16_residual_bwd(
    const float* x, const float* grad, float* dx, uint total, uint scale_bits
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float scaled = __fmaf_rn(x[id], __uint_as_float(scale_bits), 0.0f);
    dx[id] = __fmul_rn(grad[id], residual_inside(scaled));
}
