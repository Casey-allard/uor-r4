//! Native CUDA forward and backward kernels for the geometric stack ops.
//!
//! The CUDA counterpart of [`crate::metal_stack_kernels`]: one CUDA C source
//! compiled at runtime with NVRTC (no `nvcc` at build time), loaded once per
//! device as a Candle custom module, and launched on the device's stream.
//! The kernels port the Metal kernels' math one-to-one, with three
//! deliberate differences that bring them closer to the exact CPU reference:
//!
//! - where the CPU computes in `f64` (RMSNorm statistics, cross-entropy
//!   log-sum-exp, the recurrence's parameter partials and `log a`, the
//!   read's Lorentz/L2 lifts, excesses and distances, the read backward's
//!   softmax gradient and per-head reductions, and the pointer mixture's
//!   attention, copy mass, branch shares and gradients) the kernels use
//!   `double` too; every other quantity is `float`, as on the CPU;
//! - the U(1) recurrence control and the L2 read control have kernels (Metal
//!   runs them on the host);
//! - the AdamW update uses non-contracting `__fmul_rn`/`__fadd_rn`, so its
//!   f32 operations are the CPU's operations in the CPU's order.
//!
//! The GELU gate keeps the Metal numerics fix: `tanhf` of an argument clamped
//! to `[-20, 20]` (exactly `+-1` in f32 there), never a fast approximation.
//! The source is compiled without `--use_fast_math`, so `expf`, `logf`,
//! `tanhf`, division and `sqrtf` are the accurate CUDA functions.
//!
//! This module contains the crate's only `unsafe` code (kernel launches and a
//! `DeviceRepr`-free argument list); it exists only with the `cuda` feature.

#[cfg(feature = "cuda")]
#[allow(unsafe_code)]
pub mod cuda {
    use std::sync::OnceLock;

    use candle_core::{CudaDevice, Error, Result};
    use cudarc::driver::{CudaSlice, CudaView, LaunchConfig, PushKernelArg};
    use half::bf16;

    /// The Candle custom-module name the f32 stack kernels are loaded under.
    const MODULE: &str = "uor_r4_geometric_stack";
    /// The Candle custom-module name the bf16 activation-storage kernels (the
    /// same source compiled with UOR_STORAGE_BF16) are loaded under.
    const MODULE_BF16: &str = "uor_r4_geometric_stack_bf16";

    pub const CUDA_STACK_SOURCE: &str = r#"
typedef unsigned int uint;
typedef unsigned long long u64;

#define LORENTZ_MIN_EXCESS 1e-7
#define L2_MIN_SQUARED 1e-7
#define RMS_EPSILON 1e-5

// score: 0 Dot, 1 Lorentz, 2 L2.
struct ReadDims {
    uint batch;
    uint heads;
    uint time;
    uint key;
    uint value;
    uint null_on;
    uint age_on;
    uint score;
};

struct Geom {
    uint a_stride;
    uint a_offset;
    uint b_stride;
    uint b_offset;
    uint length;
    uint mode;
    uint write_excess;
};

// [scale, beta1, rest1, beta2, rest2, correct1, correct2, epsilon, keep, lr]
struct AdamConstants {
    float c[10];
};

__device__ __forceinline__ float neg_inf() { return __int_as_float(0xff800000); }

// ---------------------------------------------------------------------------
// 0. Activation storage: `float` in the default build, bf16 bits in the
// UOR_STORAGE_BF16 build (the same CUDA C source is compiled twice by NVRTC
// and loaded as two Candle custom modules). `act_t` is the storage of the
// buffers that carry activations between ops; every parameter, statistic,
// score, partial and carried state stays f32/f64 in both builds, and every
// kernel converts on load and on store so its arithmetic is f32 (or f64)
// throughout. The f32 build's `act_to_f`/`act_from_f` are the identity, and
// its `ld4_act`/`st4_act` call `ld4`/`st4`, so its machine code is unchanged.
//
// The bf16 conversion is round-to-nearest-even on the top 16 bits, with
// infinities preserved and NaN canonicalized to a quiet bf16 NaN: the same
// rounding `half::bf16::from_f32` (Candle's f32 -> bf16 cast) performs.
// ---------------------------------------------------------------------------
#ifdef UOR_STORAGE_BF16
#include <mma.h>
typedef unsigned short act_t;
#define UOR_ACT_BYTES 2

__device__ __forceinline__ float act_to_f(act_t v) {
    return __uint_as_float(((unsigned)v) << 16);
}

__device__ __forceinline__ act_t act_from_f(float f) {
    unsigned u = __float_as_uint(f);
    if ((u & 0x7f800000u) == 0x7f800000u) {
        // Infinity keeps its sign; any NaN becomes the canonical quiet NaN.
        return (act_t)(((u & 0x80000000u) >> 16) |
                       ((u & 0x007fffffu) != 0u ? 0x7fc0u : 0x7f80u));
    }
    unsigned rounded = u + 0x7fffu + ((u >> 16) & 1u);
    return (act_t)(rounded >> 16);
}
#else
typedef float act_t;
#define UOR_ACT_BYTES 4

__device__ __forceinline__ float act_to_f(act_t v) { return v; }
__device__ __forceinline__ act_t act_from_f(float v) { return v; }
#endif

__device__ __forceinline__ float warp_sum(float v) {
    for (int o = 16; o > 0; o >>= 1) v += __shfl_xor_sync(0xffffffffu, v, o);
    return v;
}

__device__ __forceinline__ double warp_sum_d(double v) {
    for (int o = 16; o > 0; o >>= 1) v += __shfl_xor_sync(0xffffffffu, v, o);
    return v;
}

__device__ __forceinline__ float warp_max(float v) {
    for (int o = 16; o > 0; o >>= 1) v = fmaxf(v, __shfl_xor_sync(0xffffffffu, v, o));
    return v;
}

// Block reductions; blockDim.x is a multiple of 32 (at most 1024). Every
// thread receives the result.
__device__ double block_sum_d(double v, double* shared) {
    v = warp_sum_d(v);
    uint lane = threadIdx.x & 31;
    uint warp = threadIdx.x >> 5;
    if (lane == 0) shared[warp] = v;
    __syncthreads();
    uint warps = (blockDim.x + 31) >> 5;
    double total = 0.0;
    for (uint i = 0; i < warps; ++i) total += shared[i];
    __syncthreads();
    return total;
}

__device__ float block_max(float v, float* shared) {
    v = warp_max(v);
    uint lane = threadIdx.x & 31;
    uint warp = threadIdx.x >> 5;
    if (lane == 0) shared[warp] = v;
    __syncthreads();
    uint warps = (blockDim.x + 31) >> 5;
    float m = neg_inf();
    for (uint i = 0; i < warps; ++i) m = fmaxf(m, shared[i]);
    __syncthreads();
    return m;
}

__device__ __forceinline__ float4 ld4(const float* p) {
    return make_float4(p[0], p[1], p[2], p[3]);
}

__device__ __forceinline__ void st4(float* p, float4 v) {
    p[0] = v.x; p[1] = v.y; p[2] = v.z; p[3] = v.w;
}

// The activation-storage counterparts of `ld4`/`st4`. In the f32 build they
// are the two functions above, so the generated code is unchanged.
#ifndef UOR_STORAGE_BF16
__device__ __forceinline__ float4 ld4_act(const act_t* p) { return ld4(p); }
__device__ __forceinline__ void st4_act(act_t* p, float4 v) { st4(p, v); }
#else
__device__ __forceinline__ float4 ld4_act(const act_t* p) {
    return make_float4(act_to_f(p[0]), act_to_f(p[1]), act_to_f(p[2]), act_to_f(p[3]));
}
__device__ __forceinline__ void st4_act(act_t* p, float4 v) {
    p[0] = act_from_f(v.x); p[1] = act_from_f(v.y);
    p[2] = act_from_f(v.z); p[3] = act_from_f(v.w);
}
#endif

__device__ __forceinline__ float4 add4(float4 a, float4 b) {
    return make_float4(a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w);
}

__device__ __forceinline__ float4 sub4(float4 a, float4 b) {
    return make_float4(a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w);
}

__device__ __forceinline__ float4 scale4(float4 a, float s) {
    return make_float4(a.x * s, a.y * s, a.z * s, a.w * s);
}

__device__ __forceinline__ float4 mul4(float4 a, float4 b) {
    return make_float4(a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w);
}

__device__ __forceinline__ float dot4(float4 a, float4 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
}

__device__ __forceinline__ float comp4(float4 a, uint k) {
    return k == 0 ? a.x : (k == 1 ? a.y : (k == 2 ? a.z : a.w));
}

// Hamilton product.
__device__ __forceinline__ float4 quat_mul(float4 a, float4 b) {
    return make_float4(
        a.x * b.x - a.y * b.y - a.z * b.z - a.w * b.w,
        a.x * b.y + a.y * b.x + a.z * b.w - a.w * b.z,
        a.x * b.z - a.y * b.w + a.z * b.x + a.w * b.y,
        a.x * b.w + a.y * b.z - a.z * b.y + a.w * b.x
    );
}

__device__ __forceinline__ float4 quat_conj(float4 a) {
    return make_float4(a.x, -a.y, -a.z, -a.w);
}

__device__ __forceinline__ float sigmoid_f(float x) {
    return 1.0f / (1.0f + expf(-x));
}

// ---------------------------------------------------------------------------
// 1. Straight-through
// ---------------------------------------------------------------------------
extern "C" __global__ void straight_through_fwd(const act_t* src, act_t* dst, uint total) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id < total) dst[id] = src[id];
}

// ---------------------------------------------------------------------------
// 2. SwiGLU
// ---------------------------------------------------------------------------
extern "C" __global__ void swiglu_fwd(
    const act_t* gate, const act_t* up, act_t* out, uint total
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float g = act_to_f(gate[id]);
    float u = act_to_f(up[id]);
    out[id] = act_from_f(g * sigmoid_f(g) * u);
}

extern "C" __global__ void swiglu_bwd(
    const act_t* gate, const act_t* up, const act_t* grad,
    act_t* d_gate, act_t* d_up, uint total
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float g = act_to_f(gate[id]);
    float u = act_to_f(up[id]);
    float d = act_to_f(grad[id]);
    float s = sigmoid_f(g);
    d_up[id] = act_from_f(d * g * s);
    d_gate[id] = act_from_f(d * u * s * (1.0f + g * (1.0f - s)));
}

// ---------------------------------------------------------------------------
// 3. RMSNorm: f64 statistics as the CPU; one block per row. The input, the
// output and the gradients are activation storage, the gain and its gradient
// are f32 parameters, and the statistics are f64 in both builds.
// ---------------------------------------------------------------------------
extern "C" __global__ void rms_norm_fwd(
    const act_t* x, const float* w, act_t* out, uint width
) {
    __shared__ double shared[32];
    u64 offset = (u64)blockIdx.x * width;
    double sum_sq = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double v = (double)act_to_f(x[offset + i]);
        sum_sq += v * v;
    }
    double total = block_sum_d(sum_sq, shared);
    float r = (float)(1.0 / sqrt(total / (double)width + RMS_EPSILON));
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        float xv = act_to_f(x[offset + i]);
        out[offset + i] = act_from_f(xv * r * w[i]);
    }
}

// dx = r (g w - xhat mean(g w xhat)); also stores the row's r for dw.
extern "C" __global__ void rms_norm_bwd_dx(
    const act_t* x, const float* w, const act_t* grad, act_t* dx,
    double* row_r, uint width
) {
    __shared__ double shared[32];
    u64 offset = (u64)blockIdx.x * width;
    double sum_sq = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double v = (double)act_to_f(x[offset + i]);
        sum_sq += v * v;
    }
    double total = block_sum_d(sum_sq, shared);
    double r = 1.0 / sqrt(total / (double)width + RMS_EPSILON);
    double projection = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double xhat = (double)act_to_f(x[offset + i]) * r;
        projection += (double)act_to_f(grad[offset + i]) * (double)w[i] * xhat;
    }
    projection = block_sum_d(projection, shared) / (double)width;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double xhat = (double)act_to_f(x[offset + i]) * r;
        dx[offset + i] = act_from_f((float)(r * ((double)act_to_f(grad[offset + i]) * (double)w[i]
                                                - xhat * projection)));
    }
    if (threadIdx.x == 0) row_r[blockIdx.x] = r;
}

// partials[chunk, col] = sum over the chunk's rows of g xhat, in f64.
extern "C" __global__ void rms_norm_dw_partial(
    const act_t* x, const act_t* grad, const double* row_r, double* partials,
    uint rows, uint width, uint chunk_rows
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint chunks = (rows + chunk_rows - 1) / chunk_rows;
    if (id >= chunks * width) return;
    uint col = id % width;
    uint chunk = id / width;
    uint end = (chunk + 1) * chunk_rows < rows ? (chunk + 1) * chunk_rows : rows;
    double sum = 0.0;
    for (uint r = chunk * chunk_rows; r < end; ++r) {
        u64 at = (u64)r * width + col;
        sum += (double)act_to_f(grad[at]) * ((double)act_to_f(x[at]) * row_r[r]);
    }
    partials[id] = sum;
}

extern "C" __global__ void rms_norm_dw_reduce(
    const double* partials, float* dw, uint chunks, uint width
) {
    uint col = blockIdx.x * blockDim.x + threadIdx.x;
    if (col >= width) return;
    double sum = 0.0;
    for (uint c = 0; c < chunks; ++c) sum += partials[(u64)c * width + col];
    dw[col] = (float)sum;
}

// ---------------------------------------------------------------------------
// 4. Quaternion transport scan. Transitions, drives, states and their
// gradients are activation storage; the Hamilton products and the norm run in
// f32 (the carried state itself is never rounded: `held` lives in registers).
// ---------------------------------------------------------------------------
extern "C" __global__ void quaternion_scan_fwd(
    const act_t* transition, const act_t* drive, act_t* state_out,
    uint time, uint lanes, uint total_seqs
) {
    uint seq_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint t = 0; t < time; ++t) {
        u64 idx = ((u64)(b * time + t) * lanes + lane) * 4;
        float4 moved = quat_mul(ld4_act(transition + idx), held);
        held = add4(moved, ld4_act(drive + idx));
        st4_act(state_out + idx, held);
    }
}

extern "C" __global__ void quaternion_scan_bwd(
    const act_t* transition, const act_t* state, const act_t* grad,
    act_t* dq, act_t* db, uint time, uint lanes, uint total_seqs
) {
    uint seq_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 carried = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (int t = (int)time - 1; t >= 0; --t) {
        u64 idx = ((u64)(b * time + (uint)t) * lanes + lane) * 4;
        float4 total = ld4_act(grad + idx);
        if ((uint)(t + 1) < time) {
            u64 next_idx = ((u64)(b * time + (uint)(t + 1)) * lanes + lane) * 4;
            total = add4(total, quat_mul(quat_conj(ld4_act(transition + next_idx)), carried));
        }
        st4_act(db + idx, total);
        if (t > 0) {
            u64 prev_idx = ((u64)(b * time + (uint)(t - 1)) * lanes + lane) * 4;
            st4_act(dq + idx, quat_mul(total, quat_conj(ld4_act(state + prev_idx))));
        } else {
            st4_act(dq + idx, make_float4(0.0f, 0.0f, 0.0f, 0.0f));
        }
        carried = total;
    }
}

// ---------------------------------------------------------------------------
// 5. Cross-entropy: f64 log-sum-exp per row as the CPU; elementwise gradient.
// ---------------------------------------------------------------------------
// The log-sum-exp, the loss and the gradient's scale stay f64; the logits and
// the logit gradient are activation storage. The `_act` variants serve the
// activation dtype of the compiled module; the unsuffixed ones keep an f32
// logits path for the pointer mixture's internal use.
template <typename S>
__device__ __forceinline__ void cross_entropy_rows_impl(
    const S* logits, const uint* targets, double* lse, double* loss, uint vocab
) {
    __shared__ float shared_max[32];
    __shared__ double shared_sum[32];
    u64 offset = (u64)blockIdx.x * vocab;
    float m = neg_inf();
    for (uint i = threadIdx.x; i < vocab; i += blockDim.x) {
        m = fmaxf(m, act_to_f(logits[offset + i]));
    }
    float row_max = block_max(m, shared_max);
    double sum = 0.0;
    for (uint i = threadIdx.x; i < vocab; i += blockDim.x) {
        sum += exp((double)(act_to_f(logits[offset + i]) - row_max));
    }
    double total = block_sum_d(sum, shared_sum);
    if (threadIdx.x == 0) {
        double z = (double)row_max + log(total);
        lse[blockIdx.x] = z;
        loss[blockIdx.x] = z - (double)act_to_f(logits[offset + targets[blockIdx.x]]);
    }
}

extern "C" __global__ void cross_entropy_rows(
    const float* logits, const uint* targets, double* lse, double* loss, uint vocab
) {
    cross_entropy_rows_impl<float>(logits, targets, lse, loss, vocab);
}

extern "C" __global__ void cross_entropy_rows_act(
    const act_t* logits, const uint* targets, double* lse, double* loss, uint vocab
) {
    cross_entropy_rows_impl<act_t>(logits, targets, lse, loss, vocab);
}

template <typename S>
__device__ __forceinline__ void cross_entropy_grad_impl(
    const S* logits, const uint* targets, const double* lse, const double* scale,
    S* grad, uint rows, uint vocab
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)rows * vocab) return;
    uint row = (uint)(id / vocab);
    uint i = (uint)(id % vocab);
    double s = scale[row];
    float slot = (float)(exp((double)act_to_f(logits[id]) - lse[row]) * s);
    if (i == targets[row]) slot -= (float)s;
    grad[id] = act_from_f(slot);
}

extern "C" __global__ void cross_entropy_grad(
    const float* logits, const uint* targets, const double* lse, const double* scale,
    float* grad, uint rows, uint vocab
) {
    cross_entropy_grad_impl<float>(logits, targets, lse, scale, grad, rows, vocab);
}

extern "C" __global__ void cross_entropy_grad_act(
    const act_t* logits, const uint* targets, const double* lse, const double* scale,
    act_t* grad, uint rows, uint vocab
) {
    cross_entropy_grad_impl<act_t>(logits, targets, lse, scale, grad, rows, vocab);
}

// ---------------------------------------------------------------------------
// 6. Recurrence core. rotation: 0 none, 1 quaternion (S3), 2 U(1).
// ---------------------------------------------------------------------------
__device__ __forceinline__ float gelu_value(float x) {
    const float K = 0.7978846f;
    const float C = 0.044715f;
    float v = K * (x + C * x * x * x);
    float t = tanhf(fminf(fmaxf(v, -20.0f), 20.0f));
    return 0.5f * x * (1.0f + t);
}

__device__ __forceinline__ float2 gelu_value_slope(float x) {
    const float K = 0.7978846f;
    const float C = 0.044715f;
    float v = K * (x + C * x * x * x);
    float t = tanhf(fminf(fmaxf(v, -20.0f), 20.0f));
    float value = 0.5f * x * (1.0f + t);
    float slope = 0.5f * (1.0f + t) + 0.5f * x * (1.0f - t * t) * K * (1.0f + 3.0f * C * x * x);
    return make_float2(value, slope);
}

// log a = -softplus(-decay) = -ln(1 + e^{-decay}) per lane, in f64 as the CPU.
extern "C" __global__ void recurrence_log_a(
    const float* params, float* log_a, uint width, uint lanes
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= lanes) return;
    double d = (double)params[5 * width + id];
    log_a[id] = (float)(-log1p(exp(-d)));
}

// The transport unit quaternion of one lane at one position, as the CPU
// `unit_quaternion` (U(1) zeroes j and k first). The raw components are
// activation storage, the components are read in f32, the norm and the
// normalization are f32.
__device__ __forceinline__ float4 transport_unit(
    const act_t* gates, u64 gate_row, uint lanes, uint lane, uint rotation, float* norm
) {
    if (rotation == 0) {
        *norm = 1.0f;
        return make_float4(1.0f, 0.0f, 0.0f, 0.0f);
    }
    u64 r = gate_row + lanes + 4 * lane;
    float4 raw = ld4_act(gates + r);
    if (rotation == 2) {
        raw.z = 0.0f;
        raw.w = 0.0f;
    }
    float n = sqrtf(raw.x * raw.x + raw.y * raw.y + raw.z * raw.z + raw.w * raw.w + 1e-6f);
    *norm = n;
    return make_float4(raw.x / n, raw.y / n, raw.z / n, raw.w / n);
}

extern "C" __global__ void recurrence_core_fwd(
    const act_t* branches, const act_t* gates, const float* params, const float* log_a,
    float* state_out, float* drive_out, act_t* out,
    uint time, uint width, uint lanes, uint gate_width, uint rotation, uint total_lanes
) {
    uint lane_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;
    uint ch = 4 * lane;
    u64 two_w = 2 * (u64)width;
    const float* taps = params;
    const float* bias = params + 4 * width;
    float la = log_a[lane];
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint t = 0; t < time; ++t) {
        u64 row = (u64)b * time + t;
        float c[4];
        for (uint k = 0; k < 4; ++k) {
            float c_val = bias[ch + k];
            for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
                c_val += taps[shift * width + ch + k]
                         * act_to_f(branches[(row - shift) * two_w + ch + k]);
            }
            c[k] = c_val;
        }
        float4 drive = make_float4(c[0], c[1], c[2], c[3]);
        u64 base = row * width + ch;
        st4(drive_out + base, drive);

        u64 gate_row = row * gate_width;
        float opening = sigmoid_f(act_to_f(gates[gate_row + lane]));
        float lambda = expf(8.0f * opening * la);
        float complement = 1.0f - lambda * lambda;
        float keep = (complement < 1e-6f) ? 1e-3f : sqrtf(complement);
        float norm;
        float4 unit = transport_unit(gates, gate_row, lanes, lane, rotation, &norm);
        float4 q = scale4(unit, lambda);
        held = add4(quat_mul(q, held), scale4(drive, keep));
        st4(state_out + base, held);

        u64 branch_base = row * two_w + width + ch;
        out[base + 0] = act_from_f(held.x * gelu_value(act_to_f(branches[branch_base + 0])));
        out[base + 1] = act_from_f(held.y * gelu_value(act_to_f(branches[branch_base + 1])));
        out[base + 2] = act_from_f(held.z * gelu_value(act_to_f(branches[branch_base + 2])));
        out[base + 3] = act_from_f(held.w * gelu_value(act_to_f(branches[branch_base + 3])));
    }
}

// One thread per (window, lane): the reverse sweep of the CPU backward.
// Writes d_branches and d_gates in full and the window's f64 parameter
// partials (taps, bias, d log a) into partials[window * param_len ..].
extern "C" __global__ void recurrence_core_bwd(
    const act_t* branches, const act_t* gates, const float* params, const float* log_a,
    const float* state, const float* drive, const act_t* d_out,
    act_t* d_branches, act_t* d_gates, double* partials,
    uint time, uint width, uint lanes, uint gate_width, uint rotation, uint total_lanes,
    uint param_len
) {
    uint lane_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;
    uint ch = 4 * lane;
    u64 two_w = 2 * (u64)width;
    const float* taps = params;
    float la = log_a[lane];

    for (uint t = 0; t < time; ++t) {
        u64 src = ((u64)b * time + t) * two_w + ch;
        st4_act(d_branches + src, make_float4(0.0f, 0.0f, 0.0f, 0.0f));
    }

    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    float4 q_next = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    double d_bias[4] = {0.0, 0.0, 0.0, 0.0};
    double d_tap[4][4] = {{0.0, 0.0, 0.0, 0.0}, {0.0, 0.0, 0.0, 0.0},
                          {0.0, 0.0, 0.0, 0.0}, {0.0, 0.0, 0.0, 0.0}};
    double d_log_a = 0.0;

    for (int ti = (int)time - 1; ti >= 0; --ti) {
        uint t = (uint)ti;
        u64 row = (u64)b * time + t;
        u64 base = row * width + ch;
        u64 branch_row = row * two_w;
        float4 st = ld4(state + base);
        float4 dy = ld4_act(d_out + base);
        float direct[4];
        for (uint k = 0; k < 4; ++k) {
            float2 vs = gelu_value_slope(act_to_f(branches[branch_row + width + ch + k]));
            d_branches[branch_row + width + ch + k] = act_from_f(comp4(dy, k) * comp4(st, k) * vs.y);
            direct[k] = comp4(dy, k) * vs.x;
        }
        u64 gate_row = row * gate_width;
        float opening = sigmoid_f(act_to_f(gates[gate_row + lane]));
        float lambda = expf(8.0f * opening * la);
        float complement = 1.0f - lambda * lambda;
        bool clamped = complement < 1e-6f;
        float keep = clamped ? 1e-3f : sqrtf(complement);
        float norm;
        float4 unit = transport_unit(gates, gate_row, lanes, lane, rotation, &norm);
        float4 total = make_float4(direct[0], direct[1], direct[2], direct[3]);
        if (t + 1 < time) {
            total = add4(total, quat_mul(quat_conj(q_next), held));
        }
        held = total;
        float4 c = ld4(drive + base);
        float d_keep = dot4(total, c);
        float4 dd = scale4(total, keep);
        float4 dq = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
        if (t > 0) {
            dq = quat_mul(total, quat_conj(ld4(state + base - width)));
        }
        float d_lambda = dot4(dq, unit);
        if (!clamped) {
            d_lambda -= d_keep * lambda / keep;
        }
        float d_log_lambda = d_lambda * lambda;
        float d_opening = d_log_lambda * 8.0f * la;
        d_log_a += (double)(d_log_lambda * 8.0f * opening);
        d_gates[gate_row + lane] = act_from_f(d_opening * opening * (1.0f - opening));
        if (rotation != 0) {
            float4 du = scale4(dq, lambda);
            float projection = dot4(du, unit);
            u64 r = gate_row + lanes + ch;
            d_gates[r + 0] = act_from_f((du.x - unit.x * projection) / norm);
            d_gates[r + 1] = act_from_f((du.y - unit.y * projection) / norm);
            if (rotation == 2) {
                // j and k were zeroed before normalization.
                d_gates[r + 2] = act_from_f(0.0f);
                d_gates[r + 3] = act_from_f(0.0f);
            } else {
                d_gates[r + 2] = act_from_f((du.z - unit.z * projection) / norm);
                d_gates[r + 3] = act_from_f((du.w - unit.w * projection) / norm);
            }
        }
        q_next = scale4(unit, lambda);
        // Convolution: c_t = bias + sum_shift taps_shift * a_{t - shift}.
        for (uint k = 0; k < 4; ++k) d_bias[k] += (double)comp4(dd, k);
        for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
            u64 src = (row - shift) * two_w + ch;
            for (uint k = 0; k < 4; ++k) {
                float ddk = comp4(dd, k);
                d_tap[shift][k] += (double)(ddk * act_to_f(branches[src + k]));
                d_branches[src + k] = act_from_f(act_to_f(d_branches[src + k])
                                                 + taps[shift * width + ch + k] * ddk);
            }
        }
    }
    double* p = partials + (u64)b * param_len;
    for (uint k = 0; k < 4; ++k) {
        p[0 * width + ch + k] = d_tap[0][k];
        p[1 * width + ch + k] = d_tap[1][k];
        p[2 * width + ch + k] = d_tap[2][k];
        p[3 * width + ch + k] = d_tap[3][k];
        p[4 * width + ch + k] = d_bias[k];
    }
    p[5 * width + lane] = d_log_a;
}

// Sums the windows' f64 parameter partials; d log a / d decay = sigma(-decay).
extern "C" __global__ void recurrence_param_reduce(
    const double* partials, const float* params, float* d_params,
    uint batch, uint param_len, uint width
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= param_len) return;
    double sum = 0.0;
    for (uint b = 0; b < batch; ++b) sum += partials[(u64)b * param_len + id];
    if (id >= 5 * width) {
        sum *= 1.0 / (1.0 + exp((double)params[id]));
    }
    d_params[id] = (float)sum;
}

// ---------------------------------------------------------------------------
// 6b. Recurrence core, split path. The arithmetic of recurrence_core_fwd and
// recurrence_core_bwd, expression for expression, with every quantity that
// does not depend on the carried quaternion moved into kernels of one thread
// per (window, position, lane). Only the 4-float carry stays serial, in
// scans of one thread per (window, lane) that load a chunk of positions
// before using them. Each value comes from the same expression and the f64
// partials are summed in the same order, so the outputs equal the
// single-kernel path's bit for bit (the CUDA parity tests check this).
// ---------------------------------------------------------------------------
#define RECURRENCE_CHUNK 8

// The split kernels must round as the single kernels' machine code does,
// and the PTX assembler contracts plain f32 multiplies and adds into fused
// multiply-adds differently in different kernels. So the operations whose
// contraction matters are spelled with the non-contracting intrinsics, in
// the order and with the fusions of the single kernels' sm_80 code
// (`ptxas` of CUDA 12.8; the CUDA parity tests compare the two paths bit
// for bit at the exact head).

// 1 - lambda^2, fused as both single kernels compute it.
__device__ __forceinline__ float recurrence_complement(float lambda) {
    return __fmaf_rn(-lambda, lambda, 1.0f);
}

// held' = q held + keep c, as recurrence_core_fwd computes it.
__device__ __forceinline__ float4 recurrence_step(float4 q, float4 h, float4 c, float keep) {
    float a = __fmul_rn(q.y, h.y);
    a = __fmaf_rn(q.x, h.x, -a);
    a = __fmaf_rn(q.z, -h.z, a);
    a = __fmaf_rn(q.w, -h.w, a);
    float b = __fmul_rn(q.y, h.x);
    b = __fmaf_rn(q.x, h.y, b);
    b = __fmaf_rn(q.z, h.w, b);
    b = __fmaf_rn(q.w, -h.z, b);
    float d = __fmul_rn(q.y, h.w);
    d = __fmaf_rn(q.x, h.z, -d);
    d = __fmaf_rn(q.z, h.x, d);
    d = __fmaf_rn(q.w, h.y, d);
    float e = __fmul_rn(q.y, h.z);
    e = __fmaf_rn(q.x, h.w, e);
    e = __fmaf_rn(q.z, -h.y, e);
    e = __fmaf_rn(q.w, h.x, e);
    return make_float4(__fmaf_rn(keep, c.x, a), __fmaf_rn(keep, c.y, b),
                       __fmaf_rn(keep, c.z, d), __fmaf_rn(keep, c.w, e));
}

// conj(q) h, as recurrence_core_bwd computes the carried adjoint's transport.
__device__ __forceinline__ float4 recurrence_conj_transport(float4 q, float4 h) {
    float a = __fmul_rn(h.y, q.y);
    a = __fmaf_rn(h.x, q.x, a);
    a = __fmaf_rn(h.z, q.z, a);
    a = __fmaf_rn(h.w, q.w, a);
    float b = __fmul_rn(h.x, q.y);
    b = __fmaf_rn(h.y, q.x, -b);
    b = __fmaf_rn(-h.w, q.z, b);
    b = __fmaf_rn(h.z, q.w, b);
    float d = __fmul_rn(h.w, q.y);
    d = __fmaf_rn(h.z, q.x, d);
    d = __fmaf_rn(-h.x, q.z, d);
    d = __fmaf_rn(-h.y, q.w, d);
    float e = __fmul_rn(h.z, q.y);
    e = __fmaf_rn(h.w, q.x, -e);
    e = __fmaf_rn(h.y, q.z, e);
    e = __fmaf_rn(-h.x, q.w, e);
    return make_float4(a, b, d, e);
}

// total conj(s), as recurrence_core_bwd computes dq.
__device__ __forceinline__ float4 recurrence_dq(float4 t, float4 s) {
    float a = __fmul_rn(t.y, s.y);
    a = __fmaf_rn(t.x, s.x, a);
    a = __fmaf_rn(t.z, s.z, a);
    a = __fmaf_rn(t.w, s.w, a);
    float b = __fmul_rn(t.x, s.y);
    b = __fmaf_rn(t.y, s.x, -b);
    b = __fmaf_rn(-t.z, s.w, b);
    b = __fmaf_rn(t.w, s.z, b);
    float d = __fmul_rn(t.x, s.z);
    d = __fmaf_rn(t.y, s.w, -d);
    d = __fmaf_rn(t.z, s.x, d);
    d = __fmaf_rn(-t.w, s.y, d);
    float e = __fmul_rn(t.y, s.z);
    e = __fmaf_rn(t.x, -s.w, -e);
    e = __fmaf_rn(t.z, s.y, e);
    e = __fmaf_rn(t.w, s.x, e);
    return make_float4(a, b, d, e);
}

// a . b as y*y first, then x, z and w fused: the single kernels' dot order.
__device__ __forceinline__ float recurrence_dot(float4 a, float4 b) {
    float s = __fmul_rn(a.y, b.y);
    s = __fmaf_rn(a.x, b.x, s);
    s = __fmaf_rn(a.z, b.z, s);
    return __fmaf_rn(a.w, b.w, s);
}

// Per (window, position, lane): the drive c, the transition q = lambda unit
// and the drive weight keep, as the single-kernel forward computes them.
extern "C" __global__ void recurrence_prep(
    const act_t* __restrict__ branches, const act_t* __restrict__ gates,
    const float* __restrict__ params, const float* __restrict__ log_a,
    float* __restrict__ drive_out, float* __restrict__ q_out, float* __restrict__ keep_out,
    uint time, uint width, uint lanes, uint gate_width, uint rotation, uint total
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total) return;
    u64 row = id / lanes;
    uint lane = (uint)(id % lanes);
    uint t = (uint)(row % time);
    uint ch = 4 * lane;
    u64 two_w = 2 * (u64)width;
    const float* taps = params;
    const float* bias = params + 4 * width;
    float la = log_a[lane];
    float c[4];
    for (uint k = 0; k < 4; ++k) {
        float c_val = bias[ch + k];
        for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
            c_val = __fmaf_rn(taps[shift * width + ch + k],
                              act_to_f(branches[(row - shift) * two_w + ch + k]), c_val);
        }
        c[k] = c_val;
    }
    float4 drive = make_float4(c[0], c[1], c[2], c[3]);
    u64 base = row * width + ch;
    st4(drive_out + base, drive);

    u64 gate_row = row * gate_width;
    float opening = sigmoid_f(act_to_f(gates[gate_row + lane]));
    float lambda = expf(8.0f * opening * la);
    float complement = recurrence_complement(lambda);
    float keep = (complement < 1e-6f) ? 1e-3f : sqrtf(complement);
    float norm;
    float4 unit = transport_unit(gates, gate_row, lanes, lane, rotation, &norm);
    st4(q_out + base, make_float4(__fmul_rn(unit.x, lambda), __fmul_rn(unit.y, lambda),
                                  __fmul_rn(unit.z, lambda), __fmul_rn(unit.w, lambda)));
    keep_out[row * lanes + lane] = keep;
}

// One thread per (window, lane): held_t = q_t held_{t-1} + keep_t c_t.
extern "C" __global__ void recurrence_scan_fwd(
    const float* __restrict__ q, const float* __restrict__ drive,
    const float* __restrict__ keep, float* __restrict__ state_out,
    uint time, uint width, uint lanes, uint total_lanes
) {
    uint lane_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;
    uint ch = 4 * lane;
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint t0 = 0; t0 < time; t0 += RECURRENCE_CHUNK) {
        uint n = min((uint)RECURRENCE_CHUNK, time - t0);
        float4 qs[RECURRENCE_CHUNK];
        float4 ds[RECURRENCE_CHUNK];
        float ks[RECURRENCE_CHUNK];
#pragma unroll
        for (uint i = 0; i < RECURRENCE_CHUNK; ++i) {
            if (i < n) {
                u64 row = (u64)b * time + t0 + i;
                u64 base = row * width + ch;
                qs[i] = ld4(q + base);
                ds[i] = ld4(drive + base);
                ks[i] = keep[row * lanes + lane];
            }
        }
#pragma unroll
        for (uint i = 0; i < RECURRENCE_CHUNK; ++i) {
            if (i < n) {
                held = recurrence_step(qs[i], held, ds[i], ks[i]);
                st4(state_out + ((u64)b * time + t0 + i) * width + ch, held);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 6c. Recurrence core, chunked path: the same arithmetic as the serial scan
// (`recurrence_scan_fwd`), with the carry split so the time axis is parallel.
//
// The serial scan's critical path is `time` dependent steps with one thread per
// (window, lane). Here a first pass has one thread per (window, tile, lane) and
// walks only its tile, folding the tile's affine map `h_out = P (x) h_in + D`
// (P the composed transition, D the tile's response to a zero entry state,
// accumulated with `recurrence_step`'s own operations); a second, short pass has
// one thread per (window, lane) and composes `ntiles` tiles, storing each tile's
// entry state; a third pass has one thread per (window, tile, lane) and re-walks
// its tile from its entry state with `recurrence_step`, writing the states. The
// parallel passes have `time / tile` times the threads of the serial scan and
// the serial depth is `2 tile + time / tile` instead of `time`; the price is one
// extra read of q, drive and keep.
//
// The composition reorders f32 additions relative to the serial scan, so the
// chunked states agree with it to a stated tolerance, not bit for bit; with
// `tile = 1` the fold carries one step and the expansion applies that step to
// the entry state, which is the serial scan's own step.
// ---------------------------------------------------------------------------

// One thread per (window, tile, lane): the tile's (P, D).
extern "C" __global__ void recurrence_tile_fold(
    const float* __restrict__ q, const float* __restrict__ drive,
    const float* __restrict__ keep, float* __restrict__ tile_q,
    float* __restrict__ tile_d, uint time, uint width, uint lanes, uint ntiles,
    uint tile, uint total
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total) return;
    uint lane = (uint)(id % lanes);
    uint tile_id = (uint)((id / lanes) % ntiles);
    uint b = (uint)(id / ((u64)lanes * ntiles));
    uint ch = 4 * lane;
    uint t0 = tile_id * tile;
    uint t1 = min(t0 + tile, time);
    float4 p = make_float4(1.0f, 0.0f, 0.0f, 0.0f);
    float4 d = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint t = t0; t < t1; ++t) {
        u64 row = (u64)b * time + t;
        u64 base = row * width + ch;
        float4 qi = ld4(q + base);
        d = recurrence_step(qi, d, ld4(drive + base), keep[row * lanes + lane]);
        p = quat_mul(qi, p);
    }
    st4(tile_q + id * 4, p);
    st4(tile_d + id * 4, d);
}

// One thread per (window, lane): each tile's entry state, in tile order.
extern "C" __global__ void recurrence_tile_carry(
    const float* __restrict__ tile_q, const float* __restrict__ tile_d,
    float* __restrict__ entry, uint lanes, uint ntiles, uint total_lanes
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total_lanes) return;
    uint lane = id % lanes;
    uint b = id / lanes;
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint tile_id = 0; tile_id < ntiles; ++tile_id) {
        u64 slot = (((u64)b * ntiles + tile_id) * lanes + lane) * 4;
        float4 p = ld4(tile_q + slot);
        float4 d = ld4(tile_d + slot);
        st4(entry + slot, held);
        held = add4(quat_mul(p, held), d);
    }
}

// One thread per (window, tile, lane): the tile's states from its entry state.
extern "C" __global__ void recurrence_tile_expand(
    const float* __restrict__ q, const float* __restrict__ drive,
    const float* __restrict__ keep, const float* __restrict__ entry,
    float* __restrict__ state_out, uint time, uint width, uint lanes, uint ntiles,
    uint tile, uint total
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total) return;
    uint lane = (uint)(id % lanes);
    uint tile_id = (uint)((id / lanes) % ntiles);
    uint b = (uint)(id / ((u64)lanes * ntiles));
    uint ch = 4 * lane;
    uint t0 = tile_id * tile;
    uint t1 = min(t0 + tile, time);
    float4 held = ld4(entry + id * 4);
    for (uint t = t0; t < t1; ++t) {
        u64 row = (u64)b * time + t;
        u64 base = row * width + ch;
        held = recurrence_step(ld4(q + base), held, ld4(drive + base),
                               keep[row * lanes + lane]);
        st4(state_out + base, held);
    }
}

// Per element: out = held * gelu(output branch).
extern "C" __global__ void recurrence_out(
    const float* __restrict__ state, const act_t* __restrict__ branches,
    act_t* __restrict__ out, uint width, uint total
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total) return;
    u64 row = id / width;
    uint c = (uint)(id % width);
    out[id] = act_from_f(state[id]
                         * gelu_value(act_to_f(branches[row * 2 * (u64)width + width + c])));
}

// Per (window, position, lane): the output branch's gradient, and
// gelu(output branch) written into `gelu`.
extern "C" __global__ void recurrence_bwd_direct(
    const act_t* __restrict__ branches, const float* __restrict__ state,
    const act_t* __restrict__ d_out, act_t* __restrict__ d_branches,
    float* __restrict__ gelu, uint width, uint lanes, uint total
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total) return;
    u64 row = id / lanes;
    uint lane = (uint)(id % lanes);
    uint ch = 4 * lane;
    u64 base = row * width + ch;
    u64 branch_row = row * 2 * (u64)width;
    float4 st = ld4(state + base);
    float4 dy = ld4_act(d_out + base);
    float g[4];
    for (uint k = 0; k < 4; ++k) {
        float2 vs = gelu_value_slope(act_to_f(branches[branch_row + width + ch + k]));
        d_branches[branch_row + width + ch + k] =
            act_from_f(comp4(dy, k) * comp4(st, k) * vs.y);
        g[k] = vs.x;
    }
    st4(gelu + base, make_float4(g[0], g[1], g[2], g[3]));
}

// One thread per (window, lane), positions in reverse: the carried adjoint
// total_t = dy_t gelu_t + conj(q_{t+1}) total_{t+1}, overwriting `gelu`.
extern "C" __global__ void recurrence_scan_bwd(
    const float* __restrict__ q, const act_t* __restrict__ d_out, float* gelu,
    uint time, uint width, uint lanes, uint total_lanes
) {
    uint lane_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;
    uint ch = 4 * lane;
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    // Chunks cover positions [end - n, end), walked from the last position.
    for (uint end = time; end > 0;) {
        uint n = min((uint)RECURRENCE_CHUNK, end);
        float4 dys[RECURRENCE_CHUNK];
        float4 gs[RECURRENCE_CHUNK];
        float4 qs[RECURRENCE_CHUNK];
#pragma unroll
        for (uint i = 0; i < RECURRENCE_CHUNK; ++i) {
            if (i < n) {
                uint t = end - 1 - i;
                u64 base = ((u64)b * time + t) * width + ch;
                dys[i] = ld4_act(d_out + base);
                gs[i] = ld4(gelu + base);
                qs[i] = (t + 1 < time) ? ld4(q + base + width)
                                       : make_float4(0.0f, 0.0f, 0.0f, 0.0f);
            }
        }
#pragma unroll
        for (uint i = 0; i < RECURRENCE_CHUNK; ++i) {
            if (i < n) {
                uint t = end - 1 - i;
                float4 dy = dys[i];
                float4 g = gs[i];
                float4 total = make_float4(__fmul_rn(g.x, dy.x), __fmul_rn(g.y, dy.y),
                                           __fmul_rn(g.z, dy.z), __fmul_rn(g.w, dy.w));
                if (t + 1 < time) {
                    float4 m = recurrence_conj_transport(qs[i], held);
                    total = make_float4(__fadd_rn(total.x, m.x), __fadd_rn(total.y, m.y),
                                        __fadd_rn(total.z, m.z), __fadd_rn(total.w, m.w));
                }
                held = total;
                st4(gelu + ((u64)b * time + t) * width + ch, total);
            }
        }
        end -= n;
    }
}

// Per (window, position, lane): every gradient that follows from the carried
// adjoint `total`. Writes d_gates, dd = keep total and the f32 term of
// d log a.
extern "C" __global__ void recurrence_bwd_post(
    const act_t* __restrict__ gates, const float* __restrict__ log_a,
    const float* __restrict__ state, const float* __restrict__ drive,
    const float* __restrict__ total_in, act_t* __restrict__ d_gates,
    float* __restrict__ dd_out, float* __restrict__ log_term,
    uint time, uint width, uint lanes, uint gate_width, uint rotation, uint total_threads
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)total_threads) return;
    u64 row = id / lanes;
    uint lane = (uint)(id % lanes);
    uint t = (uint)(row % time);
    uint ch = 4 * lane;
    u64 base = row * width + ch;
    float la = log_a[lane];
    u64 gate_row = row * gate_width;
    float opening = sigmoid_f(act_to_f(gates[gate_row + lane]));
    float lambda = expf(8.0f * opening * la);
    float complement = recurrence_complement(lambda);
    bool clamped = complement < 1e-6f;
    float keep = clamped ? 1e-3f : sqrtf(complement);
    float norm;
    float4 unit = transport_unit(gates, gate_row, lanes, lane, rotation, &norm);
    float4 total = ld4(total_in + base);
    float4 c = ld4(drive + base);
    float d_keep = recurrence_dot(c, total);
    float4 dd = make_float4(__fmul_rn(keep, total.x), __fmul_rn(keep, total.y),
                            __fmul_rn(keep, total.z), __fmul_rn(keep, total.w));
    float4 dq = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    if (t > 0) {
        dq = recurrence_dq(total, ld4(state + base - width));
    }
    float d_lambda = recurrence_dot(dq, unit);
    if (!clamped) {
        d_lambda = __fsub_rn(d_lambda, __fdiv_rn(__fmul_rn(lambda, d_keep), keep));
    }
    float d_log_lambda = d_lambda * lambda;
    float d_opening = d_log_lambda * 8.0f * la;
    log_term[row * lanes + lane] = d_log_lambda * 8.0f * opening;
    d_gates[gate_row + lane] = act_from_f(d_opening * opening * (1.0f - opening));
    if (rotation != 0) {
        float4 du = make_float4(__fmul_rn(lambda, dq.x), __fmul_rn(lambda, dq.y),
                                __fmul_rn(lambda, dq.z), __fmul_rn(lambda, dq.w));
        float projection = recurrence_dot(du, unit);
        u64 r = gate_row + lanes + ch;
        d_gates[r + 0] = act_from_f(__fdiv_rn(__fmaf_rn(projection, -unit.x, du.x), norm));
        d_gates[r + 1] = act_from_f(__fdiv_rn(__fmaf_rn(projection, -unit.y, du.y), norm));
        if (rotation == 2) {
            // j and k were zeroed before normalization.
            d_gates[r + 2] = act_from_f(0.0f);
            d_gates[r + 3] = act_from_f(0.0f);
        } else {
            d_gates[r + 2] = act_from_f(__fdiv_rn(__fmaf_rn(projection, -unit.z, du.z), norm));
            d_gates[r + 3] = act_from_f(__fdiv_rn(__fmaf_rn(projection, -unit.w, du.w), norm));
        }
    }
    st4(dd_out + base, dd);
}

// One thread per (window, channel), positions in reverse: the convolution's
// input gradient (contributions in the single-kernel order, shift 3 first)
// and the window's f64 tap, bias and d log a partials in its order.
extern "C" __global__ void recurrence_bwd_params(
    const act_t* __restrict__ branches, const float* __restrict__ params,
    const float* __restrict__ dd, const float* __restrict__ log_term,
    act_t* __restrict__ d_branches, double* __restrict__ partials,
    uint time, uint width, uint lanes, uint total_channels, uint param_len
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total_channels) return;
    uint b = id / width;
    uint c = id % width;
    uint lane = c / 4;
    bool lane_owner = (c % 4) == 0;
    u64 two_w = 2 * (u64)width;
    float tap0 = params[c];
    float tap1 = params[width + c];
    float tap2 = params[2 * width + c];
    float tap3 = params[3 * width + c];
    double d_bias = 0.0;
    double d_tap0 = 0.0, d_tap1 = 0.0, d_tap2 = 0.0, d_tap3 = 0.0;
    double d_log_a = 0.0;
    float dd1 = 0.0f, dd2 = 0.0f, dd3 = 0.0f;
#pragma unroll 4
    for (int ti = (int)time - 1; ti >= 0; --ti) {
        uint t = (uint)ti;
        u64 row = (u64)b * time + t;
        float dd0 = dd[row * width + c];
        d_bias += (double)dd0;
        d_tap0 += (double)__fmul_rn(dd0, act_to_f(branches[row * two_w + c]));
        if (t >= 1) d_tap1 += (double)__fmul_rn(dd0, act_to_f(branches[(row - 1) * two_w + c]));
        if (t >= 2) d_tap2 += (double)__fmul_rn(dd0, act_to_f(branches[(row - 2) * two_w + c]));
        if (t >= 3) d_tap3 += (double)__fmul_rn(dd0, act_to_f(branches[(row - 3) * two_w + c]));
        float acc = 0.0f;
        if (t + 3 < time) acc = __fmaf_rn(tap3, dd3, acc);
        if (t + 2 < time) acc = __fmaf_rn(tap2, dd2, acc);
        if (t + 1 < time) acc = __fmaf_rn(tap1, dd1, acc);
        acc = __fmaf_rn(tap0, dd0, acc);
        d_branches[row * two_w + c] = act_from_f(acc);
        if (lane_owner) d_log_a += (double)log_term[row * lanes + lane];
        dd3 = dd2;
        dd2 = dd1;
        dd1 = dd0;
    }
    double* p = partials + (u64)b * param_len;
    p[0 * width + c] = d_tap0;
    p[1 * width + c] = d_tap1;
    p[2 * width + c] = d_tap2;
    p[3 * width + c] = d_tap3;
    p[4 * width + c] = d_bias;
    if (lane_owner) p[5 * width + lane] = d_log_a;
}

// ---------------------------------------------------------------------------
// 7. General fused read: Dot, Lorentz or L2 score, NoRead slot, age table.
// Blocks are index = window * heads + head; rows are index * time + t; the
// square scratch is [index, t, j] with j <= t used.
// ---------------------------------------------------------------------------
__device__ __forceinline__ u64 read_age_offset(ReadDims d) {
    return d.null_on != 0 ? (u64)d.batch * d.heads * d.time : 0;
}

__device__ __forceinline__ u64 read_beta_offset(ReadDims d) {
    return read_age_offset(d) + (d.age_on != 0 ? (u64)d.heads * d.time : 0);
}

// Lorentz: arcosh(1 + e) clamped; L2: sqrt(s) clamped. In f64 as the CPU.
__device__ __forceinline__ double read_distance(ReadDims d, double e) {
    if (d.score == 1) {
        double x = fmax(e, LORENTZ_MIN_EXCESS);
        return log1p(x + sqrt(x * (x + 2.0)));
    }
    return sqrt(fmax(e, L2_MIN_SQUARED));
}

// Lorentz lifts sqrt(1 + |x|^2), or for L2 the squared norms |x|^2, in f64.
extern "C" __global__ void read_lift(
    const act_t* query, const act_t* kv, double* query_lift, double* key_lift, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint rows = d.batch * d.heads * d.time;
    if (id >= rows) return;
    const act_t* q = query + (u64)id * d.key;
    const act_t* k = kv + (u64)id * (d.key + d.value);
    float qq = 0.0f;
    float kk = 0.0f;
    for (uint c = 0; c < d.key; ++c) {
        float qc = act_to_f(q[c]);
        float kc = act_to_f(k[c]);
        qq += qc * qc;
        kk += kc * kc;
    }
    if (d.score == 1) {
        query_lift[id] = sqrt(1.0 + (double)qq);
        key_lift[id] = sqrt(1.0 + (double)kk);
    } else {
        query_lift[id] = (double)qq;
        key_lift[id] = (double)kk;
    }
}

// Tiled causal inner products of one block: out[index, t, j] for j <= t of
// sum_c a[index * time + t][c] b[index * time + j][c] over `length` columns.
// Mode 0 stores the raw product; mode 1 stores the read score (with age)
// and, with write_excess, the Lorentz excess or L2 squared distance.
// Blocks are 16 x 16 threads over (j tile, t tile, index).
extern "C" __global__ void read_tile_inner(
    const act_t* a, const act_t* b, const float* aux,
    const double* query_lift, const double* key_lift,
    float* out, double* excess, ReadDims d, Geom geom
) {
    uint jt = blockIdx.x;
    uint tt = blockIdx.y;
    if (jt > tt) return;
    uint index = blockIdx.z;
    uint time = d.time;
    __shared__ float tile_a[16][17];
    __shared__ float tile_b[16][17];
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint t = tt * 16 + ly;
    uint j = jt * 16 + lx;
    uint b_row = jt * 16 + ly;
    float acc = 0.0f;
    for (uint c0 = 0; c0 < geom.length; c0 += 16) {
        uint c = c0 + lx;
        tile_a[ly][lx] = (t < time && c < geom.length)
            ? act_to_f(a[((u64)index * time + t) * geom.a_stride + geom.a_offset + c]) : 0.0f;
        tile_b[ly][lx] = (b_row < time && c < geom.length)
            ? act_to_f(b[((u64)index * time + b_row) * geom.b_stride + geom.b_offset + c]) : 0.0f;
        __syncthreads();
        for (uint k = 0; k < 16; ++k) {
            acc += tile_a[ly][k] * tile_b[lx][k];
        }
        __syncthreads();
    }
    if (t >= time || j > t) return;
    u64 slot = ((u64)index * time + t) * time + j;
    if (geom.mode == 0) {
        out[slot] = acc;
        return;
    }
    uint head = index % d.heads;
    float age = d.age_on != 0 ? aux[read_age_offset(d) + (u64)head * time + (t - j)] : 0.0f;
    float score;
    if (d.score == 0) {
        float scale = 1.0f / sqrtf((float)d.key);
        score = acc * scale + age;
    } else {
        double lq = query_lift[(u64)index * time + t];
        double lk = key_lift[(u64)index * time + j];
        double e = d.score == 1 ? lq * lk - (double)acc - 1.0 : lq + lk - 2.0 * (double)acc;
        if (geom.write_excess != 0) {
            excess[slot] = e;
        }
        u64 beta_offset = read_beta_offset(d);
        double beta = (double)aux[beta_offset + head];
        double offset = (double)aux[beta_offset + d.heads + head];
        score = (float)(-beta * (read_distance(d, e) - offset)) + age;
    }
    out[slot] = score;
}

// Softmax of each row over j <= t and the NoRead slot, in place, one warp
// per row; the NoRead probability goes to null_probability[row].
extern "C" __global__ void read_softmax_warp(
    float* scores, const float* aux, float* null_probability, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    uint row = id / 32;
    uint lane = id % 32;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    float* s = scores + (u64)row * time;
    float null_score = d.null_on != 0 ? aux[row] : neg_inf();
    float maximum = null_score;
    for (uint j = lane; j <= t; j += 32) maximum = fmaxf(maximum, s[j]);
    maximum = warp_max(maximum);
    float total = 0.0f;
    for (uint j = lane; j <= t; j += 32) {
        float w = expf(s[j] - maximum);
        s[j] = w;
        total += w;
    }
    total = warp_sum(total);
    float null_weight = d.null_on != 0 ? expf(null_score - maximum) : 0.0f;
    total += null_weight;
    float inverse = 1.0f / total;
    for (uint j = lane; j <= t; j += 32) s[j] *= inverse;
    if (lane == 0) null_probability[row] = null_weight * inverse;
}

// out[index, t, v] = sum_{j <= t} p[t, j] value[j, v].
extern "C" __global__ void read_mix(
    const float* probabilities, const act_t* kv, act_t* out, ReadDims d
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    if (id >= (u64)d.batch * d.heads * time * d.value) return;
    uint v = (uint)(id % d.value);
    u64 row = id / d.value;
    uint t = (uint)(row % time);
    u64 index = row / time;
    uint width = d.key + d.value;
    const float* p = probabilities + row * time;
    const act_t* values = kv + index * time * width + d.key + v;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) accum += p[j] * act_to_f(values[(u64)j * width]);
    out[id] = act_from_f(accum);
}

// Flash-style forward of the same read: no T x T buffer. One 16 x 16 block
// per (16-row query tile, index); it walks the key tiles j <= t, computes the
// scores exactly as read_tile_inner (f32 inner product, f64 lifts/distance),
// keeps an online softmax in f32 (running max including the NoRead logit,
// running sum, rescaled accumulator) and writes out = sum p v. The NoRead
// slot joins the normaliser with a zero value. Values are taken in chunks of
// FLASH_VALUE_CHUNK columns (the scores are recomputed per chunk), so any
// value width works with static shared memory only.
#define FLASH_VALUE_CHUNK 128

__device__ __forceinline__ float half_warp_max(float v) {
    for (int o = 8; o > 0; o >>= 1) v = fmaxf(v, __shfl_xor_sync(0xffffffffu, v, o));
    return v;
}

__device__ __forceinline__ float half_warp_sum(float v) {
    for (int o = 8; o > 0; o >>= 1) v += __shfl_xor_sync(0xffffffffu, v, o);
    return v;
}

#ifndef UOR_STORAGE_BF16
extern "C" __global__ void read_flash_fwd(
    const act_t* query, const act_t* kv, const float* aux,
    const double* query_lift, const double* key_lift, act_t* out, ReadDims d
) {
    uint tt = blockIdx.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint t = tt * 16 + ly;
    bool row_ok = t < time;
    u64 row = (u64)index * time + t;
    __shared__ float tile_a[16][17];
    __shared__ float tile_b[16][17];
    __shared__ float probs[16][17];
    __shared__ float tile_v[16][FLASH_VALUE_CHUNK];
    float null_score = (d.null_on != 0 && row_ok) ? aux[row] : neg_inf();
    double lq = 0.0;
    double beta = 0.0;
    double offset = 0.0;
    if (d.score != 0) {
        if (row_ok) lq = query_lift[row];
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
    }
    for (uint v0 = 0; v0 < d.value; v0 += FLASH_VALUE_CHUNK) {
        float maximum = null_score;
        float total = d.null_on != 0 ? 1.0f : 0.0f;
        float acc_out[FLASH_VALUE_CHUNK / 16];
        #pragma unroll
        for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) acc_out[i] = 0.0f;
        for (uint jt = 0; jt <= tt; ++jt) {
            uint j = jt * 16 + lx;
            uint b_row = jt * 16 + ly;
            float acc = 0.0f;
            for (uint c0 = 0; c0 < d.key; c0 += 16) {
                uint c = c0 + lx;
                tile_a[ly][lx] = (row_ok && c < d.key)
                    ? act_to_f(query[row * d.key + c]) : 0.0f;
                tile_b[ly][lx] = (b_row < time && c < d.key)
                    ? act_to_f(kv[((u64)index * time + b_row) * width + c]) : 0.0f;
                __syncthreads();
                for (uint k = 0; k < 16; ++k) {
                    acc += tile_a[ly][k] * tile_b[lx][k];
                }
                __syncthreads();
            }
            // Rows past the end score 0 (finite, never written); j > t is masked.
            float score = 0.0f;
            if (row_ok) {
                if (j > t) {
                    score = neg_inf();
                } else {
                    float age = d.age_on != 0
                        ? aux[read_age_offset(d) + (u64)head * time + (t - j)] : 0.0f;
                    if (d.score == 0) {
                        float scale = 1.0f / sqrtf((float)d.key);
                        score = acc * scale + age;
                    } else {
                        double lk = key_lift[(u64)index * time + j];
                        double e = d.score == 1 ? lq * lk - (double)acc - 1.0
                                                : lq + lk - 2.0 * (double)acc;
                        score = (float)(-beta * (read_distance(d, e) - offset)) + age;
                    }
                }
            }
            // Every valid row has j = jt * 16 <= t in this tile, so the new
            // maximum is finite.
            float next = fmaxf(maximum, half_warp_max(score));
            float p = expf(score - next);
            float rescale = expf(maximum - next);
            total = total * rescale + half_warp_sum(p);
            maximum = next;
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) acc_out[i] *= rescale;
            probs[ly][lx] = p;
            for (uint e = tid; e < 16 * FLASH_VALUE_CHUNK; e += 256) {
                uint jj = e / FLASH_VALUE_CHUNK;
                uint c = e % FLASH_VALUE_CHUNK;
                uint key_row = jt * 16 + jj;
                tile_v[jj][c] = (key_row < time && v0 + c < d.value)
                    ? act_to_f(kv[((u64)index * time + key_row) * width + d.key + v0 + c])
                    : 0.0f;
            }
            __syncthreads();
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) {
                float s = 0.0f;
                for (uint jj = 0; jj < 16; ++jj) s += probs[ly][jj] * tile_v[jj][lx + 16 * i];
                acc_out[i] += s;
            }
            __syncthreads();
        }
        if (row_ok) {
            float inverse = 1.0f / total;
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) {
                uint c = v0 + lx + 16 * i;
                if (c < d.value) out[row * d.value + c] = act_from_f(acc_out[i] * inverse);
            }
        }
    }
}
#else
// bf16 storage: the same forward with its two matrix products on tensor
// cores (bf16 16x16x16 WMMA fragments, f32 accumulation). One block of 8
// warps per 16-row query tile walks key blocks of FLASH_TC_KEYS keys: each
// warp computes one 16 x 16 tile of S = Q K^T (the bf16 products are exact in
// f32, so only the summation order differs from read_tile_inner), the score
// transform and online softmax stay in f32/f64 exactly as above, and each
// warp then accumulates one 16-column slice of P V. P is split into two bf16
// terms, P = hi + lo (two MMAs against the same V tile), which carries about
// 16 mantissa bits, so P V matches the f32 sum to f32 summation-order error.
// K and V are zero padded to multiples of 16 in shared memory; one shared
// f32 tile holds S, then the hi/lo weights, then the P V slices.
#define FLASH_TC_KEYS 128
#define FLASH_TC_KCHUNK 64

extern "C" __global__ void read_flash_fwd(
    const act_t* query, const act_t* kv, const float* aux,
    const double* query_lift, const double* key_lift, act_t* out, ReadDims d
) {
    using namespace nvcuda;
    uint tt = blockIdx.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint warp = tid >> 5;
    uint t = tt * 16 + ly;
    bool row_ok = t < time;
    u64 row = (u64)index * time + t;
    __shared__ __align__(32) act_t q_s[16 * FLASH_TC_KCHUNK];
    __shared__ __align__(32) act_t kv_s[FLASH_TC_KEYS * FLASH_VALUE_CHUNK];
    __shared__ __align__(32) float s_s[16 * FLASH_TC_KEYS];
    const __nv_bfloat16* q_b = reinterpret_cast<const __nv_bfloat16*>(q_s);
    const __nv_bfloat16* kv_b = reinterpret_cast<const __nv_bfloat16*>(kv_s);
    act_t* p_hi = reinterpret_cast<act_t*>(s_s);
    act_t* p_lo = p_hi + 16 * FLASH_TC_KEYS;
    const __nv_bfloat16* hi_b = reinterpret_cast<const __nv_bfloat16*>(p_hi);
    const __nv_bfloat16* lo_b = reinterpret_cast<const __nv_bfloat16*>(p_lo);
    const act_t* base = kv + (u64)index * time * width;
    float null_score = (d.null_on != 0 && row_ok) ? aux[row] : neg_inf();
    double lq = 0.0;
    double beta = 0.0;
    double offset = 0.0;
    if (d.score != 0) {
        if (row_ok) lq = query_lift[row];
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
    }
    uint last_key = min(tt * 16 + 15, time - 1);
    uint blocks = last_key / FLASH_TC_KEYS + 1;
    for (uint v0 = 0; v0 < d.value; v0 += FLASH_VALUE_CHUNK) {
        float maximum = null_score;
        float total = d.null_on != 0 ? 1.0f : 0.0f;
        float acc_out[FLASH_VALUE_CHUNK / 16];
        #pragma unroll
        for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) acc_out[i] = 0.0f;
        for (uint jb = 0; jb < blocks; ++jb) {
            uint k0 = jb * FLASH_TC_KEYS;
            uint live = min(last_key + 1 - k0, (uint)FLASH_TC_KEYS);
            bool warp_keys = warp * 16 < live;
            wmma::fragment<wmma::accumulator, 16, 16, 16, float> s_frag;
            wmma::fill_fragment(s_frag, 0.0f);
            for (uint c0 = 0; c0 < d.key; c0 += FLASH_TC_KCHUNK) {
                uint cw = min(d.key - c0, (uint)FLASH_TC_KCHUNK);
                for (uint e = tid; e < 16 * FLASH_TC_KCHUNK; e += 256) {
                    uint r = e / FLASH_TC_KCHUNK;
                    uint c = e % FLASH_TC_KCHUNK;
                    uint qr = tt * 16 + r;
                    q_s[e] = (qr < time && c < cw)
                        ? query[((u64)index * time + qr) * d.key + c0 + c] : (act_t)0;
                }
                for (uint e = tid; e < live * FLASH_TC_KCHUNK; e += 256) {
                    uint r = e / FLASH_TC_KCHUNK;
                    uint c = e % FLASH_TC_KCHUNK;
                    kv_s[e] = c < cw ? base[(u64)(k0 + r) * width + c0 + c] : (act_t)0;
                }
                __syncthreads();
                if (warp_keys) {
                    for (uint s = 0; s * 16 < cw; ++s) {
                        wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::row_major> a;
                        wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::col_major> b;
                        wmma::load_matrix_sync(a, q_b + s * 16, FLASH_TC_KCHUNK);
                        wmma::load_matrix_sync(b, kv_b + warp * 16 * FLASH_TC_KCHUNK + s * 16, FLASH_TC_KCHUNK);
                        wmma::mma_sync(s_frag, a, b, s_frag);
                    }
                }
                __syncthreads();
            }
            wmma::store_matrix_sync(s_s + warp * 16, s_frag, FLASH_TC_KEYS, wmma::mem_row_major);
            for (uint e = tid; e < FLASH_TC_KEYS * FLASH_VALUE_CHUNK; e += 256) {
                uint r = e / FLASH_VALUE_CHUNK;
                uint c = e % FLASH_VALUE_CHUNK;
                kv_s[e] = (r < live && v0 + c < d.value)
                    ? base[(u64)(k0 + r) * width + d.key + v0 + c] : (act_t)0;
            }
            __syncthreads();
            float sc[FLASH_TC_KEYS / 16];
            float block_max = neg_inf();
            #pragma unroll
            for (uint i = 0; i < FLASH_TC_KEYS / 16; ++i) {
                uint j = k0 + lx + 16 * i;
                float acc = s_s[ly * FLASH_TC_KEYS + lx + 16 * i];
                // Rows past the end score 0 (finite, never written); j > t is masked.
                float score = 0.0f;
                if (row_ok) {
                    if (j > t) {
                        score = neg_inf();
                    } else {
                        float age = d.age_on != 0
                            ? aux[read_age_offset(d) + (u64)head * time + (t - j)] : 0.0f;
                        if (d.score == 0) {
                            float scale = 1.0f / sqrtf((float)d.key);
                            score = acc * scale + age;
                        } else {
                            double lk = key_lift[(u64)index * time + j];
                            double e = d.score == 1 ? lq * lk - (double)acc - 1.0
                                                    : lq + lk - 2.0 * (double)acc;
                            score = (float)(-beta * (read_distance(d, e) - offset)) + age;
                        }
                    }
                }
                sc[i] = score;
                block_max = fmaxf(block_max, score);
            }
            // Block 0 holds j = 0 <= t for every valid row, so the running
            // maximum is finite from the first block on.
            float next = fmaxf(maximum, half_warp_max(block_max));
            float psum = 0.0f;
            // Every S read above completes before the weights overwrite it.
            __syncthreads();
            #pragma unroll
            for (uint i = 0; i < FLASH_TC_KEYS / 16; ++i) {
                float p = expf(sc[i] - next);
                psum += p;
                act_t hi = act_from_f(p);
                p_hi[ly * FLASH_TC_KEYS + lx + 16 * i] = hi;
                p_lo[ly * FLASH_TC_KEYS + lx + 16 * i] = act_from_f(p - act_to_f(hi));
            }
            float rescale = expf(maximum - next);
            total = total * rescale + half_warp_sum(psum);
            maximum = next;
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) acc_out[i] *= rescale;
            __syncthreads();
            bool warp_values = v0 + warp * 16 < d.value;
            wmma::fragment<wmma::accumulator, 16, 16, 16, float> o_frag;
            wmma::fill_fragment(o_frag, 0.0f);
            if (warp_values) {
                for (uint s = 0; s * 16 < live; ++s) {
                    wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::row_major> a;
                    wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::row_major> b;
                    wmma::load_matrix_sync(b, kv_b + s * 16 * FLASH_VALUE_CHUNK + warp * 16, FLASH_VALUE_CHUNK);
                    wmma::load_matrix_sync(a, hi_b + s * 16, FLASH_TC_KEYS);
                    wmma::mma_sync(o_frag, a, b, o_frag);
                    wmma::load_matrix_sync(a, lo_b + s * 16, FLASH_TC_KEYS);
                    wmma::mma_sync(o_frag, a, b, o_frag);
                }
            }
            // Every weight load completes before the slices overwrite them.
            __syncthreads();
            if (warp_values) {
                wmma::store_matrix_sync(s_s + warp * 16, o_frag, FLASH_TC_KEYS, wmma::mem_row_major);
            }
            __syncthreads();
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) {
                if (v0 + 16 * i < d.value) acc_out[i] += s_s[ly * FLASH_TC_KEYS + lx + 16 * i];
            }
            __syncthreads();
        }
        if (row_ok) {
            float inverse = 1.0f / total;
            #pragma unroll
            for (uint i = 0; i < FLASH_VALUE_CHUNK / 16; ++i) {
                uint c = v0 + lx + 16 * i;
                if (c < d.value) out[row * d.value + c] = act_from_f(acc_out[i] * inverse);
            }
        }
    }
}
#endif

// ---------------------------------------------------------------------------
// Flash read backward: no T x T buffer. Each block recomputes its 16 x 16
// score tiles; per row it uses the max, total and D = sum_j p_j dp_j, which
// equals dO . out because the NoRead slot carries a zero value.
// read_flash_bwd_query (per query tile) writes those row statistics, dq,
// the NoRead gradient and the per-row beta/offset and per-tile age partials;
// read_flash_bwd_key (per key tile) writes dk and dv; read_flash_bwd_reduce
// sums the partials in a fixed order. No atomics: the result is
// deterministic.
// ---------------------------------------------------------------------------
#define FLASH_GRAD_CHUNK 128
#define FLASH_KV_CHUNK 160

__device__ __forceinline__ double half_warp_sum_d(double v) {
    for (int o = 8; o > 0; o >>= 1) v += __shfl_xor_sync(0xffffffffu, v, o);
    return v;
}

// sum_c a[c] b'[c] over `length` columns, where thread (ly, lx) supplies row
// ly of each 16-row tile (`a_row`, `b_row`) and receives (a row ly, b row lx),
// in the same tile order as read_flash_fwd. Every thread of the block calls it.
__device__ __forceinline__ float flash_tile_dot(
    const act_t* a_row, bool a_ok, const act_t* b_row, bool b_ok, uint length,
    float (*tile_a)[17], float (*tile_b)[17], uint lx, uint ly
) {
    float acc = 0.0f;
    for (uint c0 = 0; c0 < length; c0 += 16) {
        uint c = c0 + lx;
        tile_a[ly][lx] = (a_ok && c < length) ? act_to_f(a_row[c]) : 0.0f;
        tile_b[ly][lx] = (b_ok && c < length) ? act_to_f(b_row[c]) : 0.0f;
        __syncthreads();
        for (uint k = 0; k < 16; ++k) acc += tile_a[ly][k] * tile_b[lx][k];
        __syncthreads();
    }
    return acc;
}

// The read score of a valid (t, j) from its inner product, as read_flash_fwd;
// for Lorentz/L2 also the excess and the clamped distance.
__device__ __forceinline__ float flash_score(
    ReadDims d, const float* aux, uint head, uint t, uint j, float acc,
    double lq, double lk, double beta, double offset, double* excess, double* distance
) {
    float age = d.age_on != 0 ? aux[read_age_offset(d) + (u64)head * d.time + (t - j)] : 0.0f;
    if (d.score == 0) {
        float scale = 1.0f / sqrtf((float)d.key);
        return acc * scale + age;
    }
    double e = d.score == 1 ? lq * lk - (double)acc - 1.0 : lq + lk - 2.0 * (double)acc;
    double dist = read_distance(d, e);
    *excess = e;
    *distance = dist;
    return (float)(-beta * (dist - offset)) + age;
}

// The inner-product gradient of one score from its f64 ds, as
// read_row_grad_warp, with the query and key self coefficients.
__device__ __forceinline__ float flash_inner_grad(
    ReadDims d, double ds, double e, double distance, double beta, double lq, double lk,
    double* self_q, double* self_k
) {
    *self_q = 0.0;
    *self_k = 0.0;
    if (d.score == 0) return (float)(ds * (1.0 / sqrt((double)d.key)));
    if (d.score == 1) {
        if (e > LORENTZ_MIN_EXCESS) {
            double de = -beta * ds / sqrt(e * (e + 2.0));
            *self_q = de * lk / lq;
            *self_k = de * lq / lk;
            return (float)(-de);
        }
        return 0.0f;
    }
    if (e > L2_MIN_SQUARED) {
        double ds_ds = -beta * ds / (2.0 * distance);
        *self_q = 2.0 * ds_ds;
        *self_k = 2.0 * ds_ds;
        return (float)(-2.0 * ds_ds);
    }
    return 0.0f;
}

#ifndef UOR_STORAGE_BF16
// Per (16-row query tile, index). rowstat is [rows, 3] = (max, total, D);
// age_part is [index, tiles, time] (entry delta written by tile tt for
// delta <= 16 tt + 15).
extern "C" __global__ void read_flash_bwd_query(
    const act_t* query, const act_t* kv, const act_t* d_out, const float* aux,
    const double* query_lift, const double* key_lift, double* rowstat, act_t* dq,
    float* d_aux, double* row_beta, double* row_offset, double* age_part, ReadDims d
) {
    uint tt = blockIdx.x;
    uint tiles = gridDim.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint t = tt * 16 + ly;
    bool row_ok = t < time;
    u64 row = (u64)index * time + t;
    __shared__ float tile_a[16][17];
    __shared__ float tile_b[16][17];
    __shared__ float grads[16][17];
    __shared__ double ds_tile[16][17];
    __shared__ double age_carry[16];
    __shared__ float tile_k[16][FLASH_GRAD_CHUNK];
    const act_t* q_row = query + row * d.key;
    const act_t* g_row = d_out + row * d.value;
    float null_score = (d.null_on != 0 && row_ok) ? aux[row] : neg_inf();
    double lq = 0.0;
    double beta = 0.0;
    double offset = 0.0;
    if (d.score != 0) {
        if (row_ok) lq = query_lift[row];
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
    }
    // Pass 1: the online max and total, and the unnormalised D in f64.
    float maximum = null_score;
    float total = d.null_on != 0 ? 1.0f : 0.0f;
    double weighted = 0.0;
    for (uint jt = 0; jt <= tt; ++jt) {
        uint j = jt * 16 + lx;
        uint b_row = jt * 16 + ly;
        const act_t* k_row = kv + ((u64)index * time + b_row) * width;
        float acc = flash_tile_dot(q_row, row_ok, k_row, b_row < time, d.key, tile_a, tile_b, lx, ly);
        float dpv = flash_tile_dot(g_row, row_ok, k_row + d.key, b_row < time, d.value, tile_a, tile_b, lx, ly);
        float score = 0.0f;
        if (row_ok) {
            if (j > t) {
                score = neg_inf();
            } else {
                double lk = d.score != 0 ? key_lift[(u64)index * time + j] : 0.0;
                double e = 0.0;
                double dist = 0.0;
                score = flash_score(d, aux, head, t, j, acc, lq, lk, beta, offset, &e, &dist);
            }
        }
        float next = fmaxf(maximum, half_warp_max(score));
        float p = expf(score - next);
        float rescale = expf(maximum - next);
        total = total * rescale + half_warp_sum(p);
        weighted = weighted * (double)rescale + half_warp_sum_d((double)p * (double)dpv);
        maximum = next;
    }
    float inverse = 1.0f / total;
    double row_dot = weighted / (double)total;
    if (row_ok && lx == 0) {
        rowstat[row * 3] = (double)maximum;
        rowstat[row * 3 + 1] = (double)total;
        rowstat[row * 3 + 2] = row_dot;
        if (d.null_on != 0) {
            float null_p = expf(null_score - maximum) * inverse;
            d_aux[row] = (float)(-(double)null_p * row_dot);
        }
    }
    // Pass 2, per key-column chunk: ds, the inner gradients and dq; on the
    // first chunk also the beta/offset row partials and the age diagonals.
    for (uint c0 = 0; c0 < d.key; c0 += FLASH_GRAD_CHUNK) {
        bool first = c0 == 0;
        float acc_q[FLASH_GRAD_CHUNK / 16];
        #pragma unroll
        for (uint i = 0; i < FLASH_GRAD_CHUNK / 16; ++i) acc_q[i] = 0.0f;
        double self_q = 0.0;
        double d_beta = 0.0;
        double d_offset = 0.0;
        if (tid < 16) age_carry[tid] = 0.0;
        for (uint jt = 0; jt <= tt; ++jt) {
            uint j = jt * 16 + lx;
            uint b_row = jt * 16 + ly;
            const act_t* k_row = kv + ((u64)index * time + b_row) * width;
            float acc = flash_tile_dot(q_row, row_ok, k_row, b_row < time, d.key, tile_a, tile_b, lx, ly);
            float dpv = flash_tile_dot(g_row, row_ok, k_row + d.key, b_row < time, d.value, tile_a, tile_b, lx, ly);
            float ig = 0.0f;
            double ds = 0.0;
            if (row_ok && j <= t) {
                double lk = d.score != 0 ? key_lift[(u64)index * time + j] : 0.0;
                double e = 0.0;
                double dist = 0.0;
                float score = flash_score(d, aux, head, t, j, acc, lq, lk, beta, offset, &e, &dist);
                float p = expf(score - maximum) * inverse;
                ds = (double)p * ((double)dpv - row_dot);
                double sq = 0.0;
                double sk = 0.0;
                ig = flash_inner_grad(d, ds, e, dist, beta, lq, lk, &sq, &sk);
                self_q += sq;
                if (d.score != 0) {
                    d_beta -= ds * (dist - offset);
                    d_offset += ds * beta;
                }
            }
            grads[ly][lx] = ig;
            ds_tile[ly][lx] = ds;
            for (uint e = tid; e < 16 * FLASH_GRAD_CHUNK; e += 256) {
                uint jj = e / FLASH_GRAD_CHUNK;
                uint c = e % FLASH_GRAD_CHUNK;
                uint key_row = jt * 16 + jj;
                tile_k[jj][c] = (key_row < time && c0 + c < d.key)
                    ? act_to_f(kv[((u64)index * time + key_row) * width + c0 + c]) : 0.0f;
            }
            __syncthreads();
            #pragma unroll
            for (uint i = 0; i < FLASH_GRAD_CHUNK / 16; ++i) {
                float s = 0.0f;
                for (uint jj = 0; jj < 16; ++jj) s += grads[ly][jj] * tile_k[jj][lx + 16 * i];
                acc_q[i] += s;
            }
            // Diagonal delta = 16 (tt - jt) + (ly - lx); delta_local in
            // [-15, 15] by thread tid < 31. The negative ones carry into the
            // next tile's block of 16 distances.
            double diag = 0.0;
            int local = (int)tid - 15;
            if (first && d.age_on != 0 && tid < 31) {
                for (uint y = 0; y < 16; ++y) {
                    int x = (int)y - local;
                    if (x >= 0 && x < 16) diag += ds_tile[y][x];
                }
                if (local >= 0) {
                    uint delta = 16 * (tt - jt) + (uint)local;
                    if (delta < time) {
                        age_part[((u64)index * tiles + tt) * time + delta]
                            = diag + age_carry[local];
                    }
                }
            }
            __syncthreads();
            if (first && d.age_on != 0 && tid < 15) age_carry[16 + local] = diag;
        }
        double sq = half_warp_sum_d(self_q);
        if (row_ok) {
            #pragma unroll
            for (uint i = 0; i < FLASH_GRAD_CHUNK / 16; ++i) {
                uint c = c0 + lx + 16 * i;
                if (c < d.key) {
                    float v = acc_q[i];
                    if (d.score != 0) v += (float)sq * act_to_f(q_row[c]);
                    dq[row * d.key + c] = act_from_f(v);
                }
            }
        }
        if (first && d.score != 0) {
            double db = half_warp_sum_d(d_beta);
            double doff = half_warp_sum_d(d_offset);
            if (row_ok && lx == 0) {
                row_beta[row] = db;
                row_offset[row] = doff;
            }
        }
        __syncthreads();
    }
}
#endif

#ifndef UOR_STORAGE_BF16
// Per (16-key tile, index): dk = sum_{t >= j} g q (+ the key self term) and
// dv = sum_{t >= j} p dO, over the combined key/value columns in chunks.
extern "C" __global__ void read_flash_bwd_key(
    const act_t* query, const act_t* kv, const act_t* d_out, const float* aux,
    const double* query_lift, const double* key_lift, const double* rowstat, act_t* dkv,
    ReadDims d
) {
    uint jtile = blockIdx.x;
    uint tiles = gridDim.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint j = jtile * 16 + lx;
    uint b_row = jtile * 16 + ly;
    const act_t* k_row = kv + ((u64)index * time + b_row) * width;
    __shared__ float tile_a[16][17];
    __shared__ float tile_b[16][17];
    __shared__ float grads[16][17];
    __shared__ float probs[16][17];
    __shared__ double self_tile[16][17];
    __shared__ float tile_x[16][FLASH_KV_CHUNK];
    double beta = 0.0;
    double offset = 0.0;
    double lk = 0.0;
    if (d.score != 0) {
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
        if (j < time) lk = key_lift[(u64)index * time + j];
    }
    for (uint c0 = 0; c0 < width; c0 += FLASH_KV_CHUNK) {
        float acc_kv[FLASH_KV_CHUNK / 16];
        #pragma unroll
        for (uint i = 0; i < FLASH_KV_CHUNK / 16; ++i) acc_kv[i] = 0.0f;
        double key_self = 0.0;
        for (uint tt = jtile; tt < tiles; ++tt) {
            uint t = tt * 16 + ly;
            bool row_ok = t < time;
            u64 row = (u64)index * time + t;
            const act_t* q_row = query + row * d.key;
            const act_t* g_row = d_out + row * d.value;
            float acc = flash_tile_dot(q_row, row_ok, k_row, b_row < time, d.key, tile_a, tile_b, lx, ly);
            float dpv = flash_tile_dot(g_row, row_ok, k_row + d.key, b_row < time, d.value, tile_a, tile_b, lx, ly);
            float ig = 0.0f;
            float p = 0.0f;
            double sk = 0.0;
            if (row_ok && j <= t) {
                double lq = d.score != 0 ? query_lift[row] : 0.0;
                double e = 0.0;
                double dist = 0.0;
                float score = flash_score(d, aux, head, t, j, acc, lq, lk, beta, offset, &e, &dist);
                float maximum = (float)rowstat[row * 3];
                float inverse = 1.0f / (float)rowstat[row * 3 + 1];
                p = expf(score - maximum) * inverse;
                double ds = (double)p * ((double)dpv - rowstat[row * 3 + 2]);
                double sq = 0.0;
                ig = flash_inner_grad(d, ds, e, dist, beta, lq, lk, &sq, &sk);
            }
            grads[ly][lx] = ig;
            probs[ly][lx] = p;
            self_tile[ly][lx] = sk;
            for (uint e = tid; e < 16 * FLASH_KV_CHUNK; e += 256) {
                uint tl = e / FLASH_KV_CHUNK;
                uint c = c0 + e % FLASH_KV_CHUNK;
                uint q_t = tt * 16 + tl;
                float x = 0.0f;
                if (q_t < time && c < width) {
                    u64 q_r = (u64)index * time + q_t;
                    x = c < d.key ? act_to_f(query[q_r * d.key + c])
                                  : act_to_f(d_out[q_r * d.value + (c - d.key)]);
                }
                tile_x[tl][e % FLASH_KV_CHUNK] = x;
            }
            __syncthreads();
            // Thread (ly, lx) owns key row ly and columns lx + 16 i.
            #pragma unroll
            for (uint i = 0; i < FLASH_KV_CHUNK / 16; ++i) {
                uint c = c0 + lx + 16 * i;
                float s = 0.0f;
                if (c < d.key) {
                    for (uint tl = 0; tl < 16; ++tl) s += grads[tl][ly] * tile_x[tl][lx + 16 * i];
                } else {
                    for (uint tl = 0; tl < 16; ++tl) s += probs[tl][ly] * tile_x[tl][lx + 16 * i];
                }
                acc_kv[i] += s;
            }
            for (uint tl = 0; tl < 16; ++tl) key_self += self_tile[tl][ly];
            __syncthreads();
        }
        if (b_row < time) {
            #pragma unroll
            for (uint i = 0; i < FLASH_KV_CHUNK / 16; ++i) {
                uint c = c0 + lx + 16 * i;
                if (c < width) {
                    float v = acc_kv[i];
                    if (c < d.key && d.score != 0) v += (float)key_self * act_to_f(k_row[c]);
                    dkv[((u64)index * time + b_row) * width + c] = act_from_f(v);
                }
            }
        }
    }
}
#else
// bf16 storage: the same dk/dv on tensor cores (bf16 16x16x16 WMMA, f32
// accumulation). The block keeps its 16-key tile and walks query blocks of
// FLASH_TCB_ROWS rows from t = 16 jtile. Warps 0-3 compute S = Q K^T and
// warps 4-7 dP = dO V^T, one 16 x 16 tile each; the score transform, p and
// ds = p (dP - D) then run per element exactly as the scalar kernel, from the
// same row statistics. The weights P and IG (the inner-product gradient) are
// split into bf16 hi + lo (about 16 mantissa bits, as read_flash_fwd), and
// each warp accumulates FLASH_TCB_FRAGS 16-column slices of dv += P^T dO and
// dk += IG^T Q (two MMAs per operand). Key and value columns are each padded
// to a multiple of 16 so no slice straddles the two. The key self term is
// summed in f64 per thread and reduced in a fixed order. No atomics.
#define FLASH_TCB_ROWS 64
#define FLASH_TCB_KCHUNK 64
#define FLASH_TCB_XCOLS 128
#define FLASH_TCB_FRAGS 2

extern "C" __global__ void read_flash_bwd_key(
    const act_t* query, const act_t* kv, const act_t* d_out, const float* aux,
    const double* query_lift, const double* key_lift, const double* rowstat, act_t* dkv,
    ReadDims d
) {
    using namespace nvcuda;
    uint jtile = blockIdx.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint warp = tid >> 5;
    uint j = jtile * 16 + lx;
    uint kp = (d.key + 15) / 16 * 16;
    uint vp = (d.value + 15) / 16 * 16;
    uint cols = kp + vp;
    uint inner = max(d.key, d.value);
    const act_t* base = kv + (u64)index * time * width;
    // Staging, reused: Q | dO | K | V chunks; then the X = [Q | dO] columns;
    // then the f32 dk/dv slices.
    __shared__ __align__(32) act_t stage[2 * FLASH_TCB_ROWS * FLASH_TCB_KCHUNK + 32 * FLASH_TCB_KCHUNK];
    __shared__ __align__(32) float sd_s[2 * FLASH_TCB_ROWS * 16];
    __shared__ __align__(32) act_t w_s[4 * FLASH_TCB_ROWS * 16];
    __shared__ double red[16][17];
    __shared__ double self_k[16];
    act_t* q_s = stage;
    act_t* g_s = q_s + FLASH_TCB_ROWS * FLASH_TCB_KCHUNK;
    act_t* k_s = g_s + FLASH_TCB_ROWS * FLASH_TCB_KCHUNK;
    act_t* v_s = k_s + 16 * FLASH_TCB_KCHUNK;
    float* out_s = reinterpret_cast<float*>(stage);
    float* s_s = sd_s;
    float* dp_s = sd_s + FLASH_TCB_ROWS * 16;
    act_t* p_hi = w_s;
    act_t* p_lo = p_hi + FLASH_TCB_ROWS * 16;
    act_t* g_hi = p_lo + FLASH_TCB_ROWS * 16;
    act_t* g_lo = g_hi + FLASH_TCB_ROWS * 16;
    double beta = 0.0;
    double offset = 0.0;
    double lk = 0.0;
    if (d.score != 0) {
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
        if (j < time) lk = key_lift[(u64)index * time + j];
    }
    for (uint c0 = 0; c0 < cols; c0 += FLASH_TCB_FRAGS * FLASH_TCB_XCOLS) {
        wmma::fragment<wmma::accumulator, 16, 16, 16, float> acc_f[FLASH_TCB_FRAGS];
        #pragma unroll
        for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) wmma::fill_fragment(acc_f[f], 0.0f);
        double key_self = 0.0;
        for (uint tq0 = jtile * 16; tq0 < time; tq0 += FLASH_TCB_ROWS) {
            // S (warps 0-3) and dP (warps 4-7), one 16-row query tile each.
            uint sub = warp & 3;
            bool is_dp = warp >= 4;
            bool sub_ok = tq0 + sub * 16 < time;
            wmma::fragment<wmma::accumulator, 16, 16, 16, float> s_frag;
            wmma::fill_fragment(s_frag, 0.0f);
            for (uint i0 = 0; i0 < inner; i0 += FLASH_TCB_KCHUNK) {
                uint kw = d.key > i0 ? min(d.key - i0, (uint)FLASH_TCB_KCHUNK) : 0;
                uint vw = d.value > i0 ? min(d.value - i0, (uint)FLASH_TCB_KCHUNK) : 0;
                for (uint e = tid; e < FLASH_TCB_ROWS * FLASH_TCB_KCHUNK; e += 256) {
                    uint r = e / FLASH_TCB_KCHUNK;
                    uint c = e % FLASH_TCB_KCHUNK;
                    uint qt = tq0 + r;
                    bool ok = qt < time;
                    u64 qr = (u64)index * time + qt;
                    q_s[e] = (ok && c < kw) ? query[qr * d.key + i0 + c] : (act_t)0;
                    g_s[e] = (ok && c < vw) ? d_out[qr * d.value + i0 + c] : (act_t)0;
                }
                for (uint e = tid; e < 16 * FLASH_TCB_KCHUNK; e += 256) {
                    uint r = e / FLASH_TCB_KCHUNK;
                    uint c = e % FLASH_TCB_KCHUNK;
                    uint kr = jtile * 16 + r;
                    bool ok = kr < time;
                    k_s[e] = (ok && c < kw) ? base[(u64)kr * width + i0 + c] : (act_t)0;
                    v_s[e] = (ok && c < vw) ? base[(u64)kr * width + d.key + i0 + c] : (act_t)0;
                }
                __syncthreads();
                uint cw = is_dp ? vw : kw;
                if (sub_ok) {
                    const __nv_bfloat16* a_b = reinterpret_cast<const __nv_bfloat16*>(
                        (is_dp ? g_s : q_s) + sub * 16 * FLASH_TCB_KCHUNK);
                    const __nv_bfloat16* b_b = reinterpret_cast<const __nv_bfloat16*>(is_dp ? v_s : k_s);
                    for (uint s = 0; s * 16 < cw; ++s) {
                        wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::row_major> a;
                        wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::col_major> b;
                        wmma::load_matrix_sync(a, a_b + s * 16, FLASH_TCB_KCHUNK);
                        wmma::load_matrix_sync(b, b_b + s * 16, FLASH_TCB_KCHUNK);
                        wmma::mma_sync(s_frag, a, b, s_frag);
                    }
                }
                __syncthreads();
            }
            wmma::store_matrix_sync((is_dp ? dp_s : s_s) + sub * 256, s_frag, 16, wmma::mem_row_major);
            __syncthreads();
            // Thread (ly, lx) owns key column lx and query rows ly + 16 i.
            #pragma unroll
            for (uint i = 0; i < FLASH_TCB_ROWS / 16; ++i) {
                uint r = ly + 16 * i;
                uint t = tq0 + r;
                float ig = 0.0f;
                float p = 0.0f;
                if (t < time && j <= t) {
                    u64 row = (u64)index * time + t;
                    double lq = d.score != 0 ? query_lift[row] : 0.0;
                    double e = 0.0;
                    double dist = 0.0;
                    float score = flash_score(d, aux, head, t, j, s_s[r * 16 + lx], lq, lk, beta, offset, &e, &dist);
                    float maximum = (float)rowstat[row * 3];
                    float inverse = 1.0f / (float)rowstat[row * 3 + 1];
                    p = expf(score - maximum) * inverse;
                    double ds = (double)p * ((double)dp_s[r * 16 + lx] - rowstat[row * 3 + 2]);
                    double sq = 0.0;
                    double sk = 0.0;
                    ig = flash_inner_grad(d, ds, e, dist, beta, lq, lk, &sq, &sk);
                    key_self += sk;
                }
                act_t ph = act_from_f(p);
                act_t gh = act_from_f(ig);
                p_hi[r * 16 + lx] = ph;
                p_lo[r * 16 + lx] = act_from_f(p - act_to_f(ph));
                g_hi[r * 16 + lx] = gh;
                g_lo[r * 16 + lx] = act_from_f(ig - act_to_f(gh));
            }
            // dk/dv: for each fragment group stage X columns, then MMA.
            #pragma unroll
            for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) {
                uint x0 = c0 + f * FLASH_TCB_XCOLS;
                __syncthreads();
                for (uint e = tid; e < FLASH_TCB_ROWS * FLASH_TCB_XCOLS; e += 256) {
                    uint r = e / FLASH_TCB_XCOLS;
                    uint vc = x0 + e % FLASH_TCB_XCOLS;
                    uint qt = tq0 + r;
                    act_t x = (act_t)0;
                    if (qt < time) {
                        u64 qr = (u64)index * time + qt;
                        if (vc < kp) {
                            if (vc < d.key) x = query[qr * d.key + vc];
                        } else if (vc - kp < d.value) {
                            x = d_out[qr * d.value + (vc - kp)];
                        }
                    }
                    stage[e] = x;
                }
                __syncthreads();
                uint vc0 = x0 + warp * 16;
                if (vc0 < cols) {
                    bool key_slice = vc0 < kp;
                    const __nv_bfloat16* hi_b = reinterpret_cast<const __nv_bfloat16*>(key_slice ? g_hi : p_hi);
                    const __nv_bfloat16* lo_b = reinterpret_cast<const __nv_bfloat16*>(key_slice ? g_lo : p_lo);
                    const __nv_bfloat16* x_b = reinterpret_cast<const __nv_bfloat16*>(stage);
                    for (uint s = 0; s < FLASH_TCB_ROWS / 16 && tq0 + s * 16 < time; ++s) {
                        wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::col_major> a;
                        wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::row_major> b;
                        wmma::load_matrix_sync(b, x_b + s * 16 * FLASH_TCB_XCOLS + warp * 16, FLASH_TCB_XCOLS);
                        wmma::load_matrix_sync(a, hi_b + s * 256, 16);
                        wmma::mma_sync(acc_f[f], a, b, acc_f[f]);
                        wmma::load_matrix_sync(a, lo_b + s * 256, 16);
                        wmma::mma_sync(acc_f[f], a, b, acc_f[f]);
                    }
                }
            }
            // Every S/dP/weight/X read completes before the next block writes.
            __syncthreads();
        }
        red[ly][lx] = key_self;
        __syncthreads();
        if (ly == 0) {
            double total = 0.0;
            for (uint r = 0; r < 16; ++r) total += red[r][lx];
            self_k[lx] = total;
        }
        #pragma unroll
        for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) {
            wmma::store_matrix_sync(out_s + f * 16 * FLASH_TCB_XCOLS + warp * 16, acc_f[f],
                                    FLASH_TCB_XCOLS, wmma::mem_row_major);
        }
        __syncthreads();
        for (uint e = tid; e < FLASH_TCB_FRAGS * 16 * FLASH_TCB_XCOLS; e += 256) {
            uint f = e / (16 * FLASH_TCB_XCOLS);
            uint rem = e % (16 * FLASH_TCB_XCOLS);
            uint kr = rem / FLASH_TCB_XCOLS;
            uint vc = c0 + f * FLASH_TCB_XCOLS + rem % FLASH_TCB_XCOLS;
            uint key_row = jtile * 16 + kr;
            if (key_row >= time || vc >= cols) continue;
            uint c;
            if (vc < kp) {
                if (vc >= d.key) continue;
                c = vc;
            } else {
                if (vc - kp >= d.value) continue;
                c = d.key + (vc - kp);
            }
            u64 kv_row = (u64)index * time + key_row;
            float v = out_s[e];
            if (c < d.key && d.score != 0) v += (float)self_k[kr] * act_to_f(kv[kv_row * width + c]);
            dkv[kv_row * width + c] = act_from_f(v);
        }
        __syncthreads();
    }
}

// S = Q K^T (warps 0-3) and dP = dO V^T (warps 4-7) of the 16-row query tile
// at tq against the `ntile` 16-key tiles from jb, into s_s / dp_s laid out
// [tile][16 rows][16 keys]. The same staging and k-step order as
// read_flash_bwd_key, so both kernels see the same f32 scores.
__device__ __forceinline__ void flash_tcq_scores(
    const act_t* query, const act_t* base, const act_t* d_out, ReadDims d, uint index,
    uint tq, uint jb, uint ntile, act_t* stage, float* s_s, float* dp_s, uint tid
) {
    using namespace nvcuda;
    uint time = d.time;
    uint width = d.key + d.value;
    uint inner = max(d.key, d.value);
    uint warp = tid >> 5;
    uint sub = warp & 3;
    bool is_dp = warp >= 4;
    act_t* q_s = stage;
    act_t* g_s = q_s + 16 * FLASH_TCB_KCHUNK;
    act_t* k_s = g_s + 16 * FLASH_TCB_KCHUNK;
    act_t* v_s = k_s + FLASH_TCB_ROWS * FLASH_TCB_KCHUNK;
    wmma::fragment<wmma::accumulator, 16, 16, 16, float> s_frag;
    wmma::fill_fragment(s_frag, 0.0f);
    for (uint i0 = 0; i0 < inner; i0 += FLASH_TCB_KCHUNK) {
        uint kw = d.key > i0 ? min(d.key - i0, (uint)FLASH_TCB_KCHUNK) : 0;
        uint vw = d.value > i0 ? min(d.value - i0, (uint)FLASH_TCB_KCHUNK) : 0;
        for (uint e = tid; e < 16 * FLASH_TCB_KCHUNK; e += 256) {
            uint r = e / FLASH_TCB_KCHUNK;
            uint c = e % FLASH_TCB_KCHUNK;
            uint qt = tq + r;
            bool ok = qt < time;
            u64 qr = (u64)index * time + qt;
            q_s[e] = (ok && c < kw) ? query[qr * d.key + i0 + c] : (act_t)0;
            g_s[e] = (ok && c < vw) ? d_out[qr * d.value + i0 + c] : (act_t)0;
        }
        for (uint e = tid; e < FLASH_TCB_ROWS * FLASH_TCB_KCHUNK; e += 256) {
            uint r = e / FLASH_TCB_KCHUNK;
            uint c = e % FLASH_TCB_KCHUNK;
            uint kr = jb + r;
            bool ok = kr < time && r < ntile * 16;
            k_s[e] = (ok && c < kw) ? base[(u64)kr * width + i0 + c] : (act_t)0;
            v_s[e] = (ok && c < vw) ? base[(u64)kr * width + d.key + i0 + c] : (act_t)0;
        }
        __syncthreads();
        uint cw = is_dp ? vw : kw;
        if (sub < ntile) {
            const __nv_bfloat16* a_b = reinterpret_cast<const __nv_bfloat16*>(is_dp ? g_s : q_s);
            const __nv_bfloat16* b_b = reinterpret_cast<const __nv_bfloat16*>(
                (is_dp ? v_s : k_s) + sub * 16 * FLASH_TCB_KCHUNK);
            for (uint s = 0; s * 16 < cw; ++s) {
                wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::row_major> a;
                wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::col_major> b;
                wmma::load_matrix_sync(a, a_b + s * 16, FLASH_TCB_KCHUNK);
                wmma::load_matrix_sync(b, b_b + s * 16, FLASH_TCB_KCHUNK);
                wmma::mma_sync(s_frag, a, b, s_frag);
            }
        }
        __syncthreads();
    }
    wmma::store_matrix_sync((is_dp ? dp_s : s_s) + sub * 256, s_frag, 16, wmma::mem_row_major);
    __syncthreads();
}

// bf16 storage: read_flash_bwd_query on tensor cores. The block keeps its
// 16-row query tile and walks key blocks of FLASH_TCB_ROWS keys (four 16-key
// tiles, j <= t only) through flash_tcq_scores. Thread (ly, lx) owns query
// row ly and key lx of each tile, so the online max/total, the f64 D, the
// NoRead gradient, ds, the query self term, the beta/offset row partials and
// the age diagonals follow the scalar kernel tile by tile in the same order.
// dq += IG K runs on tensor cores with IG split into bf16 hi + lo; each warp
// owns FLASH_TCB_FRAGS 16-column slices of the key columns. No atomics.
extern "C" __global__ void read_flash_bwd_query(
    const act_t* query, const act_t* kv, const act_t* d_out, const float* aux,
    const double* query_lift, const double* key_lift, double* rowstat, act_t* dq,
    float* d_aux, double* row_beta, double* row_offset, double* age_part, ReadDims d
) {
    using namespace nvcuda;
    uint tt = blockIdx.x;
    uint tiles = gridDim.x;
    uint index = blockIdx.z;
    uint time = d.time;
    uint width = d.key + d.value;
    uint head = index % d.heads;
    uint lx = threadIdx.x;
    uint ly = threadIdx.y;
    uint tid = ly * 16 + lx;
    uint warp = tid >> 5;
    uint t = tt * 16 + ly;
    bool row_ok = t < time;
    u64 row = (u64)index * time + t;
    uint kp = (d.key + 15) / 16 * 16;
    const act_t* base = kv + (u64)index * time * width;
    // Staging, reused: Q | dO | K | V chunks; then the K columns of dq;
    // then the f32 dq slices.
    __shared__ __align__(32) act_t stage[32 * FLASH_TCB_KCHUNK + 2 * FLASH_TCB_ROWS * FLASH_TCB_KCHUNK];
    __shared__ __align__(32) float sd_s[2 * FLASH_TCB_ROWS * 16];
    __shared__ __align__(32) act_t w_s[2 * 16 * FLASH_TCB_ROWS];
    __shared__ double ds_s[FLASH_TCB_ROWS / 16][16][17];
    __shared__ double age_carry[16];
    __shared__ double self_sh[16];
    float* s_s = sd_s;
    float* dp_s = sd_s + FLASH_TCB_ROWS * 16;
    act_t* g_hi = w_s;
    act_t* g_lo = w_s + 16 * FLASH_TCB_ROWS;
    float* out_s = reinterpret_cast<float*>(stage);
    float null_score = (d.null_on != 0 && row_ok) ? aux[row] : neg_inf();
    double lq = 0.0;
    double beta = 0.0;
    double offset = 0.0;
    if (d.score != 0) {
        if (row_ok) lq = query_lift[row];
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
    }
    // Pass 1: the online max and total, and the unnormalised D in f64.
    float maximum = null_score;
    float total = d.null_on != 0 ? 1.0f : 0.0f;
    double weighted = 0.0;
    for (uint jb = 0; jb <= tt * 16; jb += FLASH_TCB_ROWS) {
        uint ntile = min((uint)(FLASH_TCB_ROWS / 16), tt - jb / 16 + 1);
        flash_tcq_scores(query, base, d_out, d, index, tt * 16, jb, ntile, stage, s_s, dp_s, tid);
        for (uint i = 0; i < ntile; ++i) {
            uint j = jb + 16 * i + lx;
            float acc = s_s[i * 256 + ly * 16 + lx];
            float dpv = dp_s[i * 256 + ly * 16 + lx];
            float score = 0.0f;
            if (row_ok) {
                if (j > t) {
                    score = neg_inf();
                } else {
                    double lk = d.score != 0 ? key_lift[(u64)index * time + j] : 0.0;
                    double e = 0.0;
                    double dist = 0.0;
                    score = flash_score(d, aux, head, t, j, acc, lq, lk, beta, offset, &e, &dist);
                }
            }
            float next = fmaxf(maximum, half_warp_max(score));
            float p = expf(score - next);
            float rescale = expf(maximum - next);
            total = total * rescale + half_warp_sum(p);
            weighted = weighted * (double)rescale + half_warp_sum_d((double)p * (double)dpv);
            maximum = next;
        }
    }
    float inverse = 1.0f / total;
    double row_dot = weighted / (double)total;
    if (row_ok && lx == 0) {
        rowstat[row * 3] = (double)maximum;
        rowstat[row * 3 + 1] = (double)total;
        rowstat[row * 3 + 2] = row_dot;
        if (d.null_on != 0) {
            float null_p = expf(null_score - maximum) * inverse;
            d_aux[row] = (float)(-(double)null_p * row_dot);
        }
    }
    // Pass 2, per key-column chunk: ds, the inner gradients and dq; on the
    // first chunk also the beta/offset row partials and the age diagonals.
    for (uint c0 = 0; c0 < kp; c0 += FLASH_TCB_FRAGS * FLASH_TCB_XCOLS) {
        bool first = c0 == 0;
        wmma::fragment<wmma::accumulator, 16, 16, 16, float> acc_f[FLASH_TCB_FRAGS];
        #pragma unroll
        for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) wmma::fill_fragment(acc_f[f], 0.0f);
        double self_q = 0.0;
        double d_beta = 0.0;
        double d_offset = 0.0;
        if (tid < 16) age_carry[tid] = 0.0;
        for (uint jb = 0; jb <= tt * 16; jb += FLASH_TCB_ROWS) {
            uint ntile = min((uint)(FLASH_TCB_ROWS / 16), tt - jb / 16 + 1);
            flash_tcq_scores(query, base, d_out, d, index, tt * 16, jb, ntile, stage, s_s, dp_s, tid);
            #pragma unroll
            for (uint i = 0; i < FLASH_TCB_ROWS / 16; ++i) {
                uint j = jb + 16 * i + lx;
                float ig = 0.0f;
                double ds = 0.0;
                if (i < ntile && row_ok && j <= t) {
                    double lk = d.score != 0 ? key_lift[(u64)index * time + j] : 0.0;
                    double e = 0.0;
                    double dist = 0.0;
                    float score = flash_score(
                        d, aux, head, t, j, s_s[i * 256 + ly * 16 + lx], lq, lk, beta, offset, &e, &dist);
                    float p = expf(score - maximum) * inverse;
                    ds = (double)p * ((double)dp_s[i * 256 + ly * 16 + lx] - row_dot);
                    double sq = 0.0;
                    double sk = 0.0;
                    ig = flash_inner_grad(d, ds, e, dist, beta, lq, lk, &sq, &sk);
                    self_q += sq;
                    if (d.score != 0) {
                        d_beta -= ds * (dist - offset);
                        d_offset += ds * beta;
                    }
                }
                act_t gh = act_from_f(ig);
                g_hi[ly * FLASH_TCB_ROWS + i * 16 + lx] = gh;
                g_lo[ly * FLASH_TCB_ROWS + i * 16 + lx] = act_from_f(ig - act_to_f(gh));
                ds_s[i][ly][lx] = ds;
            }
            __syncthreads();
            // Diagonal delta = 16 (tt - jt) + (ly - lx), per tile in jt order
            // as the scalar kernel, with the same carry.
            if (first && d.age_on != 0) {
                for (uint i = 0; i < ntile; ++i) {
                    uint jt = jb / 16 + i;
                    double diag = 0.0;
                    int local = (int)tid - 15;
                    if (tid < 31) {
                        for (uint y = 0; y < 16; ++y) {
                            int x = (int)y - local;
                            if (x >= 0 && x < 16) diag += ds_s[i][y][x];
                        }
                        if (local >= 0) {
                            uint delta = 16 * (tt - jt) + (uint)local;
                            if (delta < time) {
                                age_part[((u64)index * tiles + tt) * time + delta]
                                    = diag + age_carry[local];
                            }
                        }
                    }
                    __syncthreads();
                    if (tid < 15) age_carry[16 + local] = diag;
                    __syncthreads();
                }
            }
            // dq += IG K: stage the K columns of this key block, then MMA.
            #pragma unroll
            for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) {
                uint x0 = c0 + f * FLASH_TCB_XCOLS;
                __syncthreads();
                for (uint e = tid; e < FLASH_TCB_ROWS * FLASH_TCB_XCOLS; e += 256) {
                    uint r = e / FLASH_TCB_XCOLS;
                    uint c = x0 + e % FLASH_TCB_XCOLS;
                    uint kr = jb + r;
                    stage[e] = (kr < time && r < ntile * 16 && c < d.key)
                        ? base[(u64)kr * width + c] : (act_t)0;
                }
                __syncthreads();
                if (x0 + warp * 16 < kp) {
                    const __nv_bfloat16* hi_b = reinterpret_cast<const __nv_bfloat16*>(g_hi);
                    const __nv_bfloat16* lo_b = reinterpret_cast<const __nv_bfloat16*>(g_lo);
                    const __nv_bfloat16* x_b = reinterpret_cast<const __nv_bfloat16*>(stage);
                    for (uint s = 0; s < ntile; ++s) {
                        wmma::fragment<wmma::matrix_a, 16, 16, 16, __nv_bfloat16, wmma::row_major> a;
                        wmma::fragment<wmma::matrix_b, 16, 16, 16, __nv_bfloat16, wmma::row_major> b;
                        wmma::load_matrix_sync(b, x_b + s * 16 * FLASH_TCB_XCOLS + warp * 16, FLASH_TCB_XCOLS);
                        wmma::load_matrix_sync(a, hi_b + s * 16, FLASH_TCB_ROWS);
                        wmma::mma_sync(acc_f[f], a, b, acc_f[f]);
                        wmma::load_matrix_sync(a, lo_b + s * 16, FLASH_TCB_ROWS);
                        wmma::mma_sync(acc_f[f], a, b, acc_f[f]);
                    }
                }
            }
            // Every weight/K read completes before the next block stages.
            __syncthreads();
        }
        double sq = half_warp_sum_d(self_q);
        if (lx == 0) self_sh[ly] = sq;
        if (first && d.score != 0) {
            double db = half_warp_sum_d(d_beta);
            double doff = half_warp_sum_d(d_offset);
            if (row_ok && lx == 0) {
                row_beta[row] = db;
                row_offset[row] = doff;
            }
        }
        #pragma unroll
        for (uint f = 0; f < FLASH_TCB_FRAGS; ++f) {
            wmma::store_matrix_sync(out_s + f * 16 * FLASH_TCB_XCOLS + warp * 16, acc_f[f],
                                    FLASH_TCB_XCOLS, wmma::mem_row_major);
        }
        __syncthreads();
        for (uint e = tid; e < FLASH_TCB_FRAGS * 16 * FLASH_TCB_XCOLS; e += 256) {
            uint f = e / (16 * FLASH_TCB_XCOLS);
            uint rem = e % (16 * FLASH_TCB_XCOLS);
            uint r = rem / FLASH_TCB_XCOLS;
            uint c = c0 + f * FLASH_TCB_XCOLS + rem % FLASH_TCB_XCOLS;
            uint qt = tt * 16 + r;
            if (qt >= time || c >= d.key) continue;
            u64 q_row = (u64)index * time + qt;
            float v = out_s[e];
            if (d.score != 0) v += (float)self_sh[r] * act_to_f(query[q_row * d.key + c]);
            dq[q_row * d.key + c] = act_from_f(v);
        }
        __syncthreads();
    }
}
#endif

// Ordered sums of the partials: ids [0, heads * time) the age table (when
// on), then [heads * time, + 2 heads) beta and offset (Lorentz/L2).
extern "C" __global__ void read_flash_bwd_reduce(
    const double* age_part, const double* row_beta, const double* row_offset, float* d_aux,
    ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    uint tiles = (time + 15) / 16;
    uint age_len = d.heads * time;
    if (id < age_len) {
        if (d.age_on == 0) return;
        uint delta = id % time;
        uint head = id / time;
        double sum = 0.0;
        for (uint b = 0; b < d.batch; ++b) {
            u64 index = (u64)b * d.heads + head;
            for (uint tt = delta / 16; tt < tiles; ++tt) {
                sum += age_part[(index * tiles + tt) * time + delta];
            }
        }
        d_aux[read_age_offset(d) + id] = (float)sum;
        return;
    }
    uint k = id - age_len;
    if (d.score == 0 || k >= 2 * d.heads) return;
    uint head = k % d.heads;
    const double* source = k < d.heads ? row_beta : row_offset;
    double sum = 0.0;
    for (uint b = 0; b < d.batch; ++b) {
        u64 index = (u64)b * d.heads + head;
        for (uint t = 0; t < time; ++t) sum += source[index * time + t];
    }
    d_aux[read_beta_offset(d) + k] = (float)sum;
}

// Per row, one warp: the softmax backward in f64 as the CPU. Writes
// ds = p (dp - <p, dp>) (f64), the inner-product gradients (f32), the NoRead
// logit gradient (into d_aux), and for Lorentz/L2 the query self coefficient
// and the row's beta and offset partials (f64). `dp` and `inner_grad` are
// score-space buffers and stay f32 in both storages.
extern "C" __global__ void read_row_grad_warp(
    const float* probabilities, const float* dp, double* ds_out, float* inner_grad,
    const double* excess, const float* null_probability, const float* aux,
    const double* query_lift, const double* key_lift,
    double* query_self, double* row_beta, double* row_offset, float* d_aux, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    uint row = id / 32;
    uint lane = id % 32;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    uint index = row / time;
    uint head = index % d.heads;
    const float* p = probabilities + (u64)row * time;
    const float* g = dp + (u64)row * time;
    double* dsr = ds_out + (u64)row * time;
    float* ig = inner_grad + (u64)row * time;
    double row_dot = 0.0;
    for (uint j = lane; j <= t; j += 32) row_dot += (double)p[j] * (double)g[j];
    row_dot = warp_sum_d(row_dot);
    if (d.null_on != 0 && lane == 0) {
        d_aux[row] = (float)(-(double)null_probability[row] * row_dot);
    }
    double scale = 1.0 / sqrt((double)d.key);
    double beta = 0.0;
    double offset = 0.0;
    if (d.score != 0) {
        u64 beta_offset = read_beta_offset(d);
        beta = (double)aux[beta_offset + head];
        offset = (double)aux[beta_offset + d.heads + head];
    }
    double self_q = 0.0;
    double d_beta = 0.0;
    double d_offset = 0.0;
    for (uint j = lane; j <= t; j += 32) {
        double ds = (double)p[j] * ((double)g[j] - row_dot);
        dsr[j] = ds;
        if (d.score == 0) {
            ig[j] = (float)(ds * scale);
        } else {
            double e = excess[(u64)row * time + j];
            double distance = read_distance(d, e);
            d_beta -= ds * (distance - offset);
            d_offset += ds * beta;
            if (d.score == 1) {
                if (e > LORENTZ_MIN_EXCESS) {
                    double de = -beta * ds / sqrt(e * (e + 2.0));
                    self_q += de * key_lift[(u64)index * time + j] / query_lift[row];
                    ig[j] = (float)(-de);
                } else {
                    ig[j] = 0.0f;
                }
            } else {
                if (e > L2_MIN_SQUARED) {
                    double ds_ds = -beta * ds / (2.0 * distance);
                    self_q += 2.0 * ds_ds;
                    ig[j] = (float)(-2.0 * ds_ds);
                } else {
                    ig[j] = 0.0f;
                }
            }
        }
    }
    if (d.score != 0) {
        self_q = warp_sum_d(self_q);
        d_beta = warp_sum_d(d_beta);
        d_offset = warp_sum_d(d_offset);
        if (lane == 0) {
            query_self[row] = self_q;
            row_beta[row] = d_beta;
            row_offset[row] = d_offset;
        }
    }
}

// Lorentz/L2 key self coefficient per (index, j), recomputed in f64 from ds:
// Lorentz sum_{t >= j} de[t, j] lift_q[t] / lift_k[j]; L2 sum 2 dscore/ds.
extern "C" __global__ void read_key_self(
    const double* ds, const double* excess, const float* aux,
    const double* query_lift, const double* key_lift, double* key_self, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    if (id >= d.batch * d.heads * time) return;
    uint j = id % time;
    uint index = id / time;
    uint head = index % d.heads;
    u64 beta_offset = read_beta_offset(d);
    double beta = (double)aux[beta_offset + head];
    double sum = 0.0;
    for (uint t = j; t < time; ++t) {
        u64 slot = ((u64)index * time + t) * time + j;
        double e = excess[slot];
        if (d.score == 1) {
            if (e > LORENTZ_MIN_EXCESS) {
                double de = -beta * ds[slot] / sqrt(e * (e + 2.0));
                sum += de * query_lift[(u64)index * time + t] / key_lift[id];
            }
        } else if (e > L2_MIN_SQUARED) {
            double ds_ds = -beta * ds[slot] / (2.0 * read_distance(d, e));
            sum += 2.0 * ds_ds;
        }
    }
    key_self[id] = sum;
}

// dq[index, t, c] = sum_{j <= t} g[t, j] key[j, c] (+ the self term).
extern "C" __global__ void read_dq(
    const float* inner_grad, const act_t* query, const act_t* kv, const double* query_self,
    act_t* dq, ReadDims d
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    if (id >= (u64)d.batch * d.heads * time * d.key) return;
    uint c = (uint)(id % d.key);
    u64 row = id / d.key;
    uint t = (uint)(row % time);
    u64 index = row / time;
    uint width = d.key + d.value;
    const float* g = inner_grad + row * time;
    const act_t* keys = kv + index * time * width + c;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) accum += g[j] * act_to_f(keys[(u64)j * width]);
    if (d.score != 0) {
        accum += (float)query_self[row] * act_to_f(query[id]);
    }
    dq[id] = act_from_f(accum);
}

// dkv[index, j, c]: keys sum_{t >= j} g[t, j] query[t, c] (+ the self term);
// values sum_{t >= j} p[t, j] d_out[t, c - key].
extern "C" __global__ void read_dkv(
    const float* inner_grad, const float* probabilities, const act_t* query, const act_t* kv,
    const act_t* d_out, const double* key_self, act_t* dkv, ReadDims d
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    uint width = d.key + d.value;
    if (id >= (u64)d.batch * d.heads * time * width) return;
    uint c = (uint)(id % width);
    u64 row = id / width;
    uint j = (uint)(row % time);
    u64 index = row / time;
    float accum = 0.0f;
    if (c < d.key) {
        for (uint t = j; t < time; ++t) {
            accum += inner_grad[(index * time + t) * time + j]
                     * act_to_f(query[(index * time + t) * d.key + c]);
        }
        if (d.score != 0) {
            accum += (float)key_self[row] * act_to_f(kv[id]);
        }
    } else {
        uint v = c - d.key;
        for (uint t = j; t < time; ++t) {
            accum += probabilities[(index * time + t) * time + j]
                     * act_to_f(d_out[(index * time + t) * d.value + v]);
        }
    }
    dkv[id] = act_from_f(accum);
}

// Age-table gradient per (head, distance): the f64 sum over windows and
// positions of ds[t, t - distance].
extern "C" __global__ void read_dage(const double* ds, float* d_aux, ReadDims d) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint time = d.time;
    if (id >= d.heads * time) return;
    uint distance = id % time;
    uint head = id / time;
    double sum = 0.0;
    for (uint b = 0; b < d.batch; ++b) {
        u64 index = (u64)b * d.heads + head;
        for (uint t = distance; t < time; ++t) {
            sum += ds[(index * time + t) * time + (t - distance)];
        }
    }
    d_aux[read_age_offset(d) + id] = (float)sum;
}

// Lorentz/L2 beta and offset gradients per head from the rows' f64 partials.
extern "C" __global__ void read_dbeta(
    const double* row_beta, const double* row_offset, float* d_aux, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= 2 * d.heads) return;
    uint head = id % d.heads;
    const double* source = id < d.heads ? row_beta : row_offset;
    double sum = 0.0;
    for (uint b = 0; b < d.batch; ++b) {
        u64 index = (u64)b * d.heads + head;
        for (uint t = 0; t < d.time; ++t) sum += source[index * d.time + t];
    }
    d_aux[read_beta_offset(d) + id] = (float)sum;
}

// Diagnostic only (UOR_CUDA_POISON_UNINIT=1): fills a fresh unspecified
// buffer with all-ones words, a NaN in f32 and f64, so a read of an element
// no kernel wrote shows up as a nonfinite result.
extern "C" __global__ void poison_words(uint* dst, uint words) {
    uint i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < words) dst[i] = 0xFFFFFFFFu;
}

// ---------------------------------------------------------------------------
// 8. AdamW step in place, the CPU `adam_step`'s f32 operations in its order
// (non-contracting intrinsics, IEEE sqrt and division).
// ---------------------------------------------------------------------------
extern "C" __global__ void adam_update(
    float* p, float* m, float* v, const float* g, AdamConstants k, uint n
) {
    uint i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const float* c = k.c;
    float grad = __fmul_rn(g[i], c[0]);
    float first = __fadd_rn(__fmul_rn(m[i], c[1]), __fmul_rn(grad, c[2]));
    float second = __fadd_rn(__fmul_rn(v[i], c[3]), __fmul_rn(__fmul_rn(grad, grad), c[4]));
    m[i] = first;
    v[i] = second;
    float step = __fdiv_rn(__fmul_rn(first, c[5]), __fadd_rn(__fsqrt_rn(__fmul_rn(second, c[6])), c[7]));
    p[i] = __fadd_rn(__fmul_rn(p[i], c[8]), -__fmul_rn(step, c[9]));
}

// ---------------------------------------------------------------------------
// 9. Pointer-copy mixture loss (no selection, no route; Dot or Lorentz).
// Rows are windows * time; side rows are [query(dim) | key(dim) | gate logit].
// The scratch is [row, j] (j <= t used): the attention, then in the backward
// d loss / d score (Dot) or d loss / d excess (Lorentz), all f64.
// ---------------------------------------------------------------------------
struct PointerDims {
    uint rows;
    uint time;
    uint dim;
    uint vocab;
    uint score;
    uint backward;
    // Copy-gate supervision (`PointerMixture::supervise`): with it, the
    // forward also writes each row's gate BCE into scale_z and its pointer
    // NLL into row_beta (both otherwise backward-only), and the backward reads
    // the upstream gradients of those two terms from grad_in[1] and grad_in[2].
    uint supervise;
    uint unused1;
};

// sqrt(1 + |x|^2) in f64 for every row's query and key, as `pointer_lift`.
extern "C" __global__ void pointer_lift(
    const act_t* side, double* query_lift, double* key_lift, PointerDims d
) {
    uint row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= d.rows) return;
    const act_t* q = side + (u64)row * (2 * d.dim + 1);
    const act_t* k = q + d.dim;
    double qq = 0.0;
    double kk = 0.0;
    for (uint c = 0; c < d.dim; ++c) {
        double qc = (double)act_to_f(q[c]);
        double kc = (double)act_to_f(k[c]);
        qq += qc * qc;
        kk += kc * kc;
    }
    query_lift[row] = sqrt(1.0 + qq);
    key_lift[row] = sqrt(1.0 + kk);
}

// The f32 Dot score `dot(q, k) / sqrt(dim)` with the CPU `dot`'s sixteen
// partial sums.
__device__ __forceinline__ float pointer_dot(const act_t* q, const act_t* k, uint dim) {
    float partial[16];
    for (uint i = 0; i < 16; ++i) partial[i] = 0.0f;
    uint whole = dim - dim % 16;
    for (uint c = 0; c < whole; c += 16) {
        for (uint i = 0; i < 16; ++i) {
            partial[i] += act_to_f(q[c + i]) * act_to_f(k[c + i]);
        }
    }
    float total = 0.0f;
    for (uint i = 0; i < 16; ++i) total += partial[i];
    for (uint c = whole; c < dim; ++c) total += act_to_f(q[c]) * act_to_f(k[c]);
    return total;
}

// The Lorentz excess lift_q lift_k - <q, k> - 1 in f64.
__device__ __forceinline__ double pointer_excess(
    const act_t* q, const act_t* k, uint dim, double lift_q, double lift_k
) {
    double inner = 0.0;
    for (uint c = 0; c < dim; ++c) {
        inner += (double)act_to_f(q[c]) * (double)act_to_f(k[c]);
    }
    return lift_q * lift_k - inner - 1.0;
}

__device__ __forceinline__ double pointer_distance(double e) {
    double x = fmax(e, LORENTZ_MIN_EXCESS);
    return log1p(x + sqrt(x * (x + 2.0)));
}

__device__ __forceinline__ double softplus_d(double x) {
    return x > 0.0 ? x + log1p(exp(-x)) : log1p(exp(x));
}

// One warp per row: the pointer's attention, p_copy and the mixture's log
// probability, as `PointerMixture::evaluate`. Forward (d.backward == 0):
// row_value[row] = -weight log mixture. Backward: scale_z[row] = c share_gen
// (the logit gradient's factor), the gate logit's gradient into d_side, the
// scratch's per-source gradient, the row's beta partial and (Lorentz) the
// query's self coefficient. Rows of weight zero write nothing (zeros).
extern "C" __global__ void pointer_rows(
    const act_t* logits, const act_t* side, const float* beta_in, const uint* ids, const uint* targets,
    const float* weights, const double* lse, const double* query_lift,
    const double* key_lift, const act_t* grad_in, const double* total_in,
    double* scratch, double* row_value, double* scale_z, act_t* d_side,
    double* row_beta, double* query_self, PointerDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint row = id / 32;
    uint lane = id % 32;
    if (row >= d.rows) return;
    double weight = (double)weights[row];
    if (weight == 0.0) return;
    uint time = d.time;
    uint dim = d.dim;
    u64 stride = 2 * (u64)dim + 1;
    uint t = row % time;
    uint first = row - t;
    uint target = targets[row];
    const act_t* q = side + (u64)row * stride;
    double beta = (double)beta_in[0];
    double lift_q = d.score == 1 ? query_lift[row] : 1.0;
    float scale_f = 1.0f / sqrtf((float)dim);
    double* s = scratch + (u64)row * time;

    float maximum = neg_inf();
    for (uint j = lane; j <= t; j += 32) {
        const act_t* k = side + (u64)(first + j) * stride + dim;
        float score;
        if (d.score == 0) {
            score = pointer_dot(q, k, dim) * scale_f;
        } else {
            double e = pointer_excess(q, k, dim, lift_q, key_lift[first + j]);
            score = (float)(-beta * pointer_distance(e));
        }
        s[j] = (double)score;
        maximum = fmaxf(maximum, score);
    }
    maximum = warp_max(maximum);
    double sum = 0.0;
    for (uint j = lane; j <= t; j += 32) {
        double a = exp((double)((float)s[j] - maximum));
        s[j] = a;
        sum += a;
    }
    sum = warp_sum_d(sum);
    double copy = 0.0;
    uint held = 0;
    for (uint j = lane; j <= t; j += 32) {
        double a = s[j] / sum;
        s[j] = a;
        if (ids[first + j] == target) {
            copy += a;
            held = 1;
        }
    }
    copy = warp_sum_d(copy);
    // Whether any source 0..=t holds the target id (the supervised "present").
    bool present = __any_sync(0xffffffffu, held != 0);

    double logit = (double)act_to_f(q[2 * dim]);
    double generate =
        -softplus_d(logit) + (double)act_to_f(logits[(u64)row * d.vocab + target]) - lse[row];
    // No source holds the target: the copy branch is exactly 0 (no floor).
    double copied = copy > 0.0
        ? -softplus_d(-logit) + log(copy)
        : __longlong_as_double(0xfff0000000000000ULL);
    double high = fmax(generate, copied);
    double log_mixture = high + log(exp(generate - high) + exp(copied - high));

    if (d.backward == 0) {
        if (lane == 0) {
            row_value[row] = -weight * log_mixture;
            if (d.supervise != 0) {
                // BCE(gate, present) and -log p_copy(target) when present.
                scale_z[row] = weight * (present ? softplus_d(-logit) : softplus_d(logit));
                row_beta[row] = present && copy > 0.0 ? -weight * log(copy) : 0.0;
            }
        }
        return;
    }
    double generate_share = exp(generate - log_mixture);
    double copy_share = exp(copied - log_mixture);
    double gate = 1.0 / (1.0 + exp(-logit));
    double c = (double)act_to_f(grad_in[0]) * weight / total_in[0];
    // The per-source coefficient: the mixture's c copy_share, plus the
    // pointer-NLL term's upstream coefficient on a supervised present row.
    double k_src = c * copy_share;
    if (d.supervise != 0 && present) {
        k_src += (double)act_to_f(grad_in[2]) * weight / total_in[0];
    }
    if (lane == 0) {
        scale_z[row] = c * generate_share;
        double d_gate = c * (generate_share * gate - copy_share * (1.0 - gate));
        if (d.supervise != 0) {
            double c_gate = (double)act_to_f(grad_in[1]) * weight / total_in[0];
            d_gate += c_gate * (gate - (present ? 1.0 : 0.0));
        }
        d_side[(u64)row * stride + 2 * dim] = act_from_f((float)d_gate);
    }
    double d_beta = 0.0;
    double self_q = 0.0;
    for (uint j = lane; j <= t; j += 32) {
        double a = s[j];
        double out = 0.0;
        if (copy > 0.0 && a != 0.0) {
            double d_source = ids[first + j] == target
                ? -k_src * (a / copy) * (1.0 - copy)
                : k_src * a;
            if (d.score == 0) {
                out = d_source;
            } else {
                const act_t* k = side + (u64)(first + j) * stride + dim;
                double lift_k = key_lift[first + j];
                double e = pointer_excess(q, k, dim, lift_q, lift_k);
                d_beta -= d_source * pointer_distance(e);
                if (e > LORENTZ_MIN_EXCESS) {
                    double de = -beta * d_source / sqrt(e * (e + 2.0));
                    out = de;
                    self_q += de * lift_k / lift_q;
                }
            }
        }
        s[j] = out;
    }
    d_beta = warp_sum_d(d_beta);
    self_q = warp_sum_d(self_q);
    if (lane == 0) {
        row_beta[row] = d_beta;
        query_self[row] = self_q;
    }
}

// d_side[row, c] for c < 2 dim from the scratch's per-source gradients:
// the query (c < dim) over its sources j <= t, the key over the positions
// t >= j that read it.
extern "C" __global__ void pointer_side_grad(
    const act_t* side, const double* scratch, const double* query_lift,
    const double* key_lift, const double* query_self, act_t* d_side, PointerDims d
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    uint dim = d.dim;
    uint time = d.time;
    if (id >= (u64)d.rows * 2 * dim) return;
    uint c = (uint)(id % (2 * dim));
    uint row = (uint)(id / (2 * dim));
    u64 stride = 2 * (u64)dim + 1;
    uint t = row % time;
    uint first = row - t;
    double scale = 1.0 / sqrt((double)dim);
    double acc = 0.0;
    if (c < dim) {
        const double* s = scratch + (u64)row * time;
        for (uint j = 0; j <= t; ++j) {
            double g = s[j];
            if (g == 0.0) continue;
            double k = (double)act_to_f(side[(u64)(first + j) * stride + dim + c]);
            if (d.score == 0) {
                acc += g * scale * k;
            } else {
                acc -= g * k;
            }
        }
        if (d.score != 0) {
            acc += query_self[row] * (double)act_to_f(side[(u64)row * stride + c]);
        }
    } else {
        uint cc = c - dim;
        double own_key = (double)act_to_f(side[(u64)row * stride + dim + cc]);
        for (uint tt = t; tt < time; ++tt) {
            double g = scratch[(u64)(first + tt) * time + t];
            if (g == 0.0) continue;
            double qv = (double)act_to_f(side[(u64)(first + tt) * stride + cc]);
            if (d.score == 0) {
                acc += g * scale * qv;
            } else {
                double own = g * query_lift[first + tt] / key_lift[row];
                acc += own * own_key - g * qv;
            }
        }
    }
    d_side[(u64)row * stride + c] = act_from_f((float)acc);
}

// out[0] = (sum of values in index order) / divisor[0], as the CPU's ordered
// f64 sums. One thread.
extern "C" __global__ void pointer_sum(
    const double* values, const double* divisor, float* out, uint n
) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    double sum = 0.0;
    for (uint i = 0; i < n; ++i) sum += values[i];
    out[0] = (float)(sum / divisor[0]);
}

// ---------------------------------------------------------------------------
// 10. Global gradient squared norm, bit-identical to Candle's
// `x.sqr()?.sum_all()` per tensor and `cat(..).sum_all()` over the tensors.
// Candle's `fast_sum` runs one block of `width = min(1024, n).next_power_of_two()`
// threads: thread `t` adds elements t, t + width, ... into a zeroed f32 in
// index order, then a shared-memory tree halves `width` down to one value.
// `sq_lanes` computes those per-thread sums for `width` lanes spread over
// many blocks (the same f32 additions in the same order; `__fmul_rn` and
// `__fadd_rn` forbid contraction), and `sq_tree` replays the tree, one block
// per segment of `width` lane sums.
// ---------------------------------------------------------------------------
extern "C" __global__ void sq_lanes(
    const float* x, float* lanes_out, uint n, uint width, uint square
) {
    uint lane = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane >= width) return;
    float acc = 0.0f;
    u64 i = lane;
    u64 stride = width;
    u64 total = n;
    // Eight loads in flight, then their additions in index order.
    while (i + 7 * stride < total) {
        float v[8];
#pragma unroll
        for (int k = 0; k < 8; ++k) v[k] = x[i + (u64)k * stride];
#pragma unroll
        for (int k = 0; k < 8; ++k) {
            float term = square ? __fmul_rn(v[k], v[k]) : v[k];
            acc = __fadd_rn(acc, term);
        }
        i += 8 * stride;
    }
    while (i < total) {
        float value = x[i];
        float term = square ? __fmul_rn(value, value) : value;
        acc = __fadd_rn(acc, term);
        i += stride;
    }
    lanes_out[lane] = acc;
}

// segments[2 * b] = offset of block b's lane sums, segments[2 * b + 1] = its
// power-of-two width (<= 1024, <= blockDim.x). out[b] = the tree's result.
extern "C" __global__ void sq_tree(
    const float* lane_sums, const uint* segments, float* out
) {
    __shared__ float shr[1024];
    uint b = blockIdx.x;
    uint offset = segments[2 * b];
    uint width = segments[2 * b + 1];
    uint t = threadIdx.x;
    if (t < width) shr[t] = lane_sums[(u64)offset + t];
    for (uint s = width / 2; s > 0; s >>= 1) {
        __syncthreads();
        if (t < s) shr[t] = __fadd_rn(shr[t], shr[t + s]);
    }
    if (t == 0) out[b] = shr[0];
}
"#;

    /// The storage of the activation buffers a kernel launch reads and writes.
    /// Parameters, statistics, scores, partials and carried states are f32 or
    /// f64 in both storages.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Storage {
        /// Every buffer is f32: the source compiled without UOR_STORAGE_BF16.
        F32,
        /// Activation buffers are bf16 (`act_t` is 16 bits wide): the source
        /// compiled with UOR_STORAGE_BF16.
        Bf16,
    }

    impl Storage {
        /// The Candle custom module the storage's PTX is loaded as.
        pub fn module(self) -> &'static str {
            match self {
                Storage::F32 => MODULE,
                Storage::Bf16 => MODULE_BF16,
            }
        }
    }

    /// The CUDA toolkit include directory NVRTC reads <mma.h> from:
    /// `$CUDA_HOME/include` (or `$CUDA_PATH`), else `/usr/local/cuda/include`.
    fn cuda_include_dir() -> String {
        let root = std::env::var("CUDA_HOME")
            .or_else(|_| std::env::var("CUDA_PATH"))
            .unwrap_or_else(|_| "/usr/local/cuda".to_string());
        format!("{root}/include")
    }

    /// The stack kernels' PTX, compiled once per process and per storage by
    /// NVRTC from the same source (the bf16 build defines UOR_STORAGE_BF16).
    fn ptx(storage: Storage) -> Result<&'static str> {
        static PTX_F32: OnceLock<std::result::Result<String, String>> = OnceLock::new();
        static PTX_BF16: OnceLock<std::result::Result<String, String>> = OnceLock::new();
        let cell = match storage {
            Storage::F32 => &PTX_F32,
            Storage::Bf16 => &PTX_BF16,
        };
        let compiled = cell.get_or_init(|| {
            let (name, defines) = match storage {
                Storage::F32 => ("uor_r4_geometric_stack.cu".to_string(), Vec::new()),
                Storage::Bf16 => (
                    "uor_r4_geometric_stack_bf16.cu".to_string(),
                    vec!["-DUOR_STORAGE_BF16".to_string()],
                ),
            };
            let bf16 = matches!(storage, Storage::Bf16);
            let options = cudarc::nvrtc::CompileOptions {
                use_fast_math: Some(false),
                prec_sqrt: Some(true),
                prec_div: Some(true),
                ftz: Some(false),
                name: Some(name),
                options: defines,
                // The bf16 build's flash read uses bf16 WMMA (sm_80 and later)
                // from <mma.h> in the toolkit's include directory.
                arch: bf16.then_some("compute_80"),
                include_paths: if bf16 {
                    vec![cuda_include_dir()]
                } else {
                    Vec::new()
                },
                ..Default::default()
            };
            cudarc::nvrtc::compile_ptx_with_opts(CUDA_STACK_SOURCE, options)
                .map(|ptx| ptx.to_src())
                .map_err(|error| error.to_string())
        });
        match compiled {
            Ok(source) => Ok(source.as_str()),
            Err(error) => Err(Error::Msg(format!(
                "NVRTC compilation of the geometric stack kernels failed: {error}"
            ))),
        }
    }

    /// Compiles both storages of the kernel source with NVRTC (no device
    /// needed): a check that the CUDA C source is valid for the installed
    /// toolkit.
    pub fn compile_check() -> Result<()> {
        ptx(Storage::F32)?;
        ptx(Storage::Bf16).map(|_| ())
    }

    /// One kernel argument, in parameter order.
    pub enum Arg<'a> {
        /// An f32 device buffer (read, written or both).
        F(CudaView<'a, f32>),
        /// A bf16 activation buffer (read, written or both): the same device
        /// pointer a kernel's `act_t*` parameter takes in the bf16 module.
        B(CudaView<'a, bf16>),
        /// An f64 device buffer.
        D(CudaView<'a, f64>),
        /// A u32 device buffer.
        U(CudaView<'a, u32>),
        U32(u32),
        /// `ReadDims` by value.
        Dims([u32; 8]),
        /// `Geom` by value.
        Geom([u32; 7]),
        /// `AdamConstants` (ten f32 bit patterns) by value.
        Adam([u32; 10]),
    }

    impl<'a> Arg<'a> {
        pub fn f(slice: &'a CudaSlice<f32>) -> Self {
            Arg::F(slice.as_view())
        }

        pub fn b(slice: &'a CudaSlice<bf16>) -> Self {
            Arg::B(slice.as_view())
        }

        pub fn d(slice: &'a CudaSlice<f64>) -> Self {
            Arg::D(slice.as_view())
        }

        pub fn u(slice: &'a CudaSlice<u32>) -> Self {
            Arg::U(slice.as_view())
        }
    }

    /// A zero-filled device buffer of `len` elements (at least one, so a
    /// placeholder argument is always a valid pointer).
    pub fn zeros<T>(device: &CudaDevice, len: usize) -> Result<CudaSlice<T>>
    where
        T: cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits,
    {
        device.alloc_zeros::<T>(len.max(1))
    }

    /// A device buffer of `len` elements (at least one) whose contents are
    /// unspecified, for an output the following kernels write in full before
    /// anything reads it: it skips [`zeros`]'s fill. Each call site names the
    /// kernel that covers the buffer; a buffer a kernel writes only in part
    /// (or not at all in some configuration) and later reads must stay
    /// [`zeros`].
    ///
    /// `UOR_CUDA_POISON_UNINIT=1` (diagnostic) fills each such buffer with
    /// NaN words first, so a kernel that leaves an element unwritten and a
    /// later read of it surface as nonfinite values in the parity tests.
    pub fn uninit<T>(device: &CudaDevice, len: usize) -> Result<CudaSlice<T>>
    where
        T: cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits,
    {
        // SAFETY: `alloc` returns device memory that is not initialized.
        // The element types used here (f32, f64, u32) have no invalid bit
        // patterns, so unspecified device bytes are never undefined
        // behaviour on the host; callers pass only buffers whose every
        // element their kernels write before any read.
        let slice = unsafe { device.alloc::<T>(len.max(1)) }?;
        if poison_uninit() {
            let words = slice.len() * std::mem::size_of::<T>() / 4;
            // SAFETY: the view covers `words` whole 4-byte words inside the
            // allocation (f32, f64 and u32 sizes are multiples of 4), and
            // any bit pattern is a valid u32.
            let view = unsafe { slice.transmute::<u32>(words) }
                .ok_or_else(|| Error::Msg("poison view exceeds its buffer".into()))?;
            let size = u32::try_from(words)
                .map_err(|_| Error::Msg("poisoned buffer exceeds u32 words".into()))?;
            launch(
                device,
                "poison_words",
                words,
                &[Arg::U(view), Arg::U32(size)],
            )?;
        }
        Ok(slice)
    }

    /// `UOR_CUDA_POISON_UNINIT=1`: [`uninit`] buffers start as NaN words.
    fn poison_uninit() -> bool {
        static POISON: OnceLock<bool> = OnceLock::new();
        *POISON.get_or_init(|| std::env::var("UOR_CUDA_POISON_UNINIT").is_ok_and(|v| v == "1"))
    }

    /// Threads per block of a 1-D launch.
    const BLOCK: usize = 256;

    /// Launches `name` over `threads` threads (a 1-D grid of 256-thread blocks).
    pub fn launch(device: &CudaDevice, name: &str, threads: usize, args: &[Arg<'_>]) -> Result<()> {
        launch_storage(device, Storage::F32, name, threads, args)
    }

    /// [`launch`] from the bf16 module.
    pub fn launch_bf16(
        device: &CudaDevice,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_storage(device, Storage::Bf16, name, threads, args)
    }

    fn launch_storage(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        if threads == 0 {
            return Ok(());
        }
        let blocks = threads.div_ceil(BLOCK);
        dispatch(device, storage, name, (blocks, 1, 1), (BLOCK, 1, 1), args)
    }

    /// Launches `name` over `groups` blocks of `group` threads each.
    pub fn launch_groups(
        device: &CudaDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_groups_storage(device, Storage::F32, name, groups, group, args)
    }

    /// [`launch_groups`] from the bf16 module.
    pub fn launch_groups_bf16(
        device: &CudaDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_groups_storage(device, Storage::Bf16, name, groups, group, args)
    }

    fn launch_groups_storage(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        if groups.0 * groups.1 * groups.2 == 0 {
            return Ok(());
        }
        dispatch(device, storage, name, groups, group, args)
    }

    fn dim(value: usize, limit: usize, what: &str) -> Result<u32> {
        if value == 0 || value > limit {
            return Err(Error::Msg(format!(
                "CUDA launch {what} {value} outside 1..={limit}"
            )));
        }
        Ok(value as u32)
    }

    fn dispatch(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        let config = LaunchConfig {
            grid_dim: (
                dim(groups.0, i32::MAX as usize, "grid x")?,
                dim(groups.1, 65_535, "grid y")?,
                dim(groups.2, 65_535, "grid z")?,
            ),
            block_dim: (
                dim(group.0, 1024, "block x")?,
                dim(group.1, 1024, "block y")?,
                dim(group.2, 64, "block z")?,
            ),
            shared_mem_bytes: 0,
        };
        let function = device.get_or_load_custom_func(name, storage.module(), ptx(storage)?)?;
        let started = profiling().then(std::time::Instant::now);
        let mut builder = function.builder();
        for arg in args {
            match arg {
                Arg::F(view) => {
                    builder.arg(view);
                }
                Arg::B(view) => {
                    builder.arg(view);
                }
                Arg::D(view) => {
                    builder.arg(view);
                }
                Arg::U(view) => {
                    builder.arg(view);
                }
                Arg::U32(value) => {
                    builder.arg(value);
                }
                Arg::Dims(words) => {
                    builder.arg(words);
                }
                Arg::Geom(words) => {
                    builder.arg(words);
                }
                Arg::Adam(words) => {
                    builder.arg(words);
                }
            }
        }
        // SAFETY: every kernel in CUDA_STACK_SOURCE takes exactly the
        // arguments its Rust caller passes, in order (device pointers for
        // buffers, `uint`s, and `ReadDims`/`Geom`/`AdamConstants` structs of
        // 4-byte words matching the `[u32; N]` arrays); callers size every
        // buffer for the kernel's index range and bounds-check the grid.
        unsafe { builder.launch(config) }.map_err(|error| Error::Cuda(Box::new(error)))?;
        if let Some(started) = started {
            device
                .cuda_stream()
                .synchronize()
                .map_err(|error| Error::Cuda(Box::new(error)))?;
            eprintln!("cuda-kernel {name} {} us", started.elapsed().as_micros());
        }
        Ok(())
    }

    /// `UOR_CUDA_PROFILE=1` synchronizes after each launched kernel and
    /// reports its wall time on stderr (diagnostic only; it serializes the GPU).
    fn profiling() -> bool {
        static PROFILE: OnceLock<bool> = OnceLock::new();
        *PROFILE.get_or_init(|| std::env::var_os("UOR_CUDA_PROFILE").is_some())
    }
}
