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

    /// The Candle custom-module name the stack kernels are loaded under.
    const MODULE: &str = "uor_r4_geometric_stack";

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
extern "C" __global__ void straight_through_fwd(const float* src, float* dst, uint total) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id < total) dst[id] = src[id];
}

// ---------------------------------------------------------------------------
// 2. SwiGLU
// ---------------------------------------------------------------------------
extern "C" __global__ void swiglu_fwd(const float* gate, const float* up, float* out, uint total) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float g = gate[id];
    float u = up[id];
    out[id] = g * sigmoid_f(g) * u;
}

extern "C" __global__ void swiglu_bwd(
    const float* gate, const float* up, const float* grad,
    float* d_gate, float* d_up, uint total
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    float g = gate[id];
    float u = up[id];
    float d = grad[id];
    float s = sigmoid_f(g);
    d_up[id] = d * g * s;
    d_gate[id] = d * u * s * (1.0f + g * (1.0f - s));
}

// ---------------------------------------------------------------------------
// 3. RMSNorm: f64 statistics as the CPU; one block per row.
// ---------------------------------------------------------------------------
extern "C" __global__ void rms_norm_fwd(
    const float* x, const float* w, float* out, uint width
) {
    __shared__ double shared[32];
    u64 offset = (u64)blockIdx.x * width;
    double sum_sq = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double v = (double)x[offset + i];
        sum_sq += v * v;
    }
    double total = block_sum_d(sum_sq, shared);
    float r = (float)(1.0 / sqrt(total / (double)width + RMS_EPSILON));
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        out[offset + i] = x[offset + i] * r * w[i];
    }
}

// dx = r (g w - xhat mean(g w xhat)); also stores the row's r for dw.
extern "C" __global__ void rms_norm_bwd_dx(
    const float* x, const float* w, const float* grad, float* dx,
    double* row_r, uint width
) {
    __shared__ double shared[32];
    u64 offset = (u64)blockIdx.x * width;
    double sum_sq = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double v = (double)x[offset + i];
        sum_sq += v * v;
    }
    double total = block_sum_d(sum_sq, shared);
    double r = 1.0 / sqrt(total / (double)width + RMS_EPSILON);
    double projection = 0.0;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double xhat = (double)x[offset + i] * r;
        projection += (double)grad[offset + i] * (double)w[i] * xhat;
    }
    projection = block_sum_d(projection, shared) / (double)width;
    for (uint i = threadIdx.x; i < width; i += blockDim.x) {
        double xhat = (double)x[offset + i] * r;
        dx[offset + i] = (float)(r * ((double)grad[offset + i] * (double)w[i] - xhat * projection));
    }
    if (threadIdx.x == 0) row_r[blockIdx.x] = r;
}

// partials[chunk, col] = sum over the chunk's rows of g xhat, in f64.
extern "C" __global__ void rms_norm_dw_partial(
    const float* x, const float* grad, const double* row_r, double* partials,
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
        sum += (double)grad[at] * ((double)x[at] * row_r[r]);
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
// 4. Quaternion transport scan
// ---------------------------------------------------------------------------
extern "C" __global__ void quaternion_scan_fwd(
    const float* transition, const float* drive, float* state_out,
    uint time, uint lanes, uint total_seqs
) {
    uint seq_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 held = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (uint t = 0; t < time; ++t) {
        u64 idx = ((u64)(b * time + t) * lanes + lane) * 4;
        float4 moved = quat_mul(ld4(transition + idx), held);
        held = add4(moved, ld4(drive + idx));
        st4(state_out + idx, held);
    }
}

extern "C" __global__ void quaternion_scan_bwd(
    const float* transition, const float* state, const float* grad,
    float* dq, float* db, uint time, uint lanes, uint total_seqs
) {
    uint seq_id = blockIdx.x * blockDim.x + threadIdx.x;
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 carried = make_float4(0.0f, 0.0f, 0.0f, 0.0f);
    for (int t = (int)time - 1; t >= 0; --t) {
        u64 idx = ((u64)(b * time + (uint)t) * lanes + lane) * 4;
        float4 total = ld4(grad + idx);
        if ((uint)(t + 1) < time) {
            u64 next_idx = ((u64)(b * time + (uint)(t + 1)) * lanes + lane) * 4;
            total = add4(total, quat_mul(quat_conj(ld4(transition + next_idx)), carried));
        }
        st4(db + idx, total);
        if (t > 0) {
            u64 prev_idx = ((u64)(b * time + (uint)(t - 1)) * lanes + lane) * 4;
            st4(dq + idx, quat_mul(total, quat_conj(ld4(state + prev_idx))));
        } else {
            st4(dq + idx, make_float4(0.0f, 0.0f, 0.0f, 0.0f));
        }
        carried = total;
    }
}

// ---------------------------------------------------------------------------
// 5. Cross-entropy: f64 log-sum-exp per row as the CPU; elementwise gradient.
// ---------------------------------------------------------------------------
extern "C" __global__ void cross_entropy_rows(
    const float* logits, const uint* targets, double* lse, double* loss, uint vocab
) {
    __shared__ float shared_max[32];
    __shared__ double shared_sum[32];
    u64 offset = (u64)blockIdx.x * vocab;
    float m = neg_inf();
    for (uint i = threadIdx.x; i < vocab; i += blockDim.x) m = fmaxf(m, logits[offset + i]);
    float row_max = block_max(m, shared_max);
    double sum = 0.0;
    for (uint i = threadIdx.x; i < vocab; i += blockDim.x) {
        sum += exp((double)(logits[offset + i] - row_max));
    }
    double total = block_sum_d(sum, shared_sum);
    if (threadIdx.x == 0) {
        double z = (double)row_max + log(total);
        lse[blockIdx.x] = z;
        loss[blockIdx.x] = z - (double)logits[offset + targets[blockIdx.x]];
    }
}

extern "C" __global__ void cross_entropy_grad(
    const float* logits, const uint* targets, const double* lse, const double* scale,
    float* grad, uint rows, uint vocab
) {
    u64 id = (u64)blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= (u64)rows * vocab) return;
    uint row = (uint)(id / vocab);
    uint i = (uint)(id % vocab);
    double s = scale[row];
    float slot = (float)(exp((double)logits[id] - lse[row]) * s);
    if (i == targets[row]) slot -= (float)s;
    grad[id] = slot;
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
// `unit_quaternion` (U(1) zeroes j and k first).
__device__ __forceinline__ float4 transport_unit(
    const float* gates, u64 gate_row, uint lanes, uint lane, uint rotation, float* norm
) {
    if (rotation == 0) {
        *norm = 1.0f;
        return make_float4(1.0f, 0.0f, 0.0f, 0.0f);
    }
    u64 r = gate_row + lanes + 4 * lane;
    float4 raw = ld4(gates + r);
    if (rotation == 2) {
        raw.z = 0.0f;
        raw.w = 0.0f;
    }
    float n = sqrtf(raw.x * raw.x + raw.y * raw.y + raw.z * raw.z + raw.w * raw.w + 1e-6f);
    *norm = n;
    return make_float4(raw.x / n, raw.y / n, raw.z / n, raw.w / n);
}

extern "C" __global__ void recurrence_core_fwd(
    const float* branches, const float* gates, const float* params, const float* log_a,
    float* state_out, float* drive_out, float* out,
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
                c_val += taps[shift * width + ch + k] * branches[(row - shift) * two_w + ch + k];
            }
            c[k] = c_val;
        }
        float4 drive = make_float4(c[0], c[1], c[2], c[3]);
        u64 base = row * width + ch;
        st4(drive_out + base, drive);

        u64 gate_row = row * gate_width;
        float opening = sigmoid_f(gates[gate_row + lane]);
        float lambda = expf(8.0f * opening * la);
        float complement = 1.0f - lambda * lambda;
        float keep = (complement < 1e-6f) ? 1e-3f : sqrtf(complement);
        float norm;
        float4 unit = transport_unit(gates, gate_row, lanes, lane, rotation, &norm);
        float4 q = scale4(unit, lambda);
        held = add4(quat_mul(q, held), scale4(drive, keep));
        st4(state_out + base, held);

        u64 branch_base = row * two_w + width + ch;
        out[base + 0] = held.x * gelu_value(branches[branch_base + 0]);
        out[base + 1] = held.y * gelu_value(branches[branch_base + 1]);
        out[base + 2] = held.z * gelu_value(branches[branch_base + 2]);
        out[base + 3] = held.w * gelu_value(branches[branch_base + 3]);
    }
}

// One thread per (window, lane): the reverse sweep of the CPU backward.
// Writes d_branches and d_gates in full and the window's f64 parameter
// partials (taps, bias, d log a) into partials[window * param_len ..].
extern "C" __global__ void recurrence_core_bwd(
    const float* branches, const float* gates, const float* params, const float* log_a,
    const float* state, const float* drive, const float* d_out,
    float* d_branches, float* d_gates, double* partials,
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
        st4(d_branches + src, make_float4(0.0f, 0.0f, 0.0f, 0.0f));
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
        float4 dy = ld4(d_out + base);
        float direct[4];
        for (uint k = 0; k < 4; ++k) {
            float2 vs = gelu_value_slope(branches[branch_row + width + ch + k]);
            d_branches[branch_row + width + ch + k] = comp4(dy, k) * comp4(st, k) * vs.y;
            direct[k] = comp4(dy, k) * vs.x;
        }
        u64 gate_row = row * gate_width;
        float opening = sigmoid_f(gates[gate_row + lane]);
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
        d_gates[gate_row + lane] = d_opening * opening * (1.0f - opening);
        if (rotation != 0) {
            float4 du = scale4(dq, lambda);
            float projection = dot4(du, unit);
            u64 r = gate_row + lanes + ch;
            d_gates[r + 0] = (du.x - unit.x * projection) / norm;
            d_gates[r + 1] = (du.y - unit.y * projection) / norm;
            if (rotation == 2) {
                // j and k were zeroed before normalization.
                d_gates[r + 2] = 0.0f;
                d_gates[r + 3] = 0.0f;
            } else {
                d_gates[r + 2] = (du.z - unit.z * projection) / norm;
                d_gates[r + 3] = (du.w - unit.w * projection) / norm;
            }
        }
        q_next = scale4(unit, lambda);
        // Convolution: c_t = bias + sum_shift taps_shift * a_{t - shift}.
        for (uint k = 0; k < 4; ++k) d_bias[k] += (double)comp4(dd, k);
        for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
            u64 src = (row - shift) * two_w + ch;
            for (uint k = 0; k < 4; ++k) {
                float ddk = comp4(dd, k);
                d_tap[shift][k] += (double)(ddk * branches[src + k]);
                d_branches[src + k] += taps[shift * width + ch + k] * ddk;
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
    const float* query, const float* kv, double* query_lift, double* key_lift, ReadDims d
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    uint rows = d.batch * d.heads * d.time;
    if (id >= rows) return;
    const float* q = query + (u64)id * d.key;
    const float* k = kv + (u64)id * (d.key + d.value);
    float qq = 0.0f;
    float kk = 0.0f;
    for (uint c = 0; c < d.key; ++c) {
        qq += q[c] * q[c];
        kk += k[c] * k[c];
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
    const float* a, const float* b, const float* aux,
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
            ? a[((u64)index * time + t) * geom.a_stride + geom.a_offset + c] : 0.0f;
        tile_b[ly][lx] = (b_row < time && c < geom.length)
            ? b[((u64)index * time + b_row) * geom.b_stride + geom.b_offset + c] : 0.0f;
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
    const float* probabilities, const float* kv, float* out, ReadDims d
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
    const float* values = kv + index * time * width + d.key + v;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) accum += p[j] * values[(u64)j * width];
    out[id] = accum;
}

// Per row, one warp: the softmax backward in f64 as the CPU. Writes
// ds = p (dp - <p, dp>) (f64), the inner-product gradients (f32), the NoRead
// logit gradient (into d_aux), and for Lorentz/L2 the query self coefficient
// and the row's beta and offset partials (f64).
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
    const float* inner_grad, const float* query, const float* kv, const double* query_self,
    float* dq, ReadDims d
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
    const float* keys = kv + index * time * width + c;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) accum += g[j] * keys[(u64)j * width];
    if (d.score != 0) {
        accum += (float)query_self[row] * query[id];
    }
    dq[id] = accum;
}

// dkv[index, j, c]: keys sum_{t >= j} g[t, j] query[t, c] (+ the self term);
// values sum_{t >= j} p[t, j] d_out[t, c - key].
extern "C" __global__ void read_dkv(
    const float* inner_grad, const float* probabilities, const float* query, const float* kv,
    const float* d_out, const double* key_self, float* dkv, ReadDims d
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
            accum += inner_grad[(index * time + t) * time + j] * query[(index * time + t) * d.key + c];
        }
        if (d.score != 0) {
            accum += (float)key_self[row] * kv[id];
        }
    } else {
        uint v = c - d.key;
        for (uint t = j; t < time; ++t) {
            accum += probabilities[(index * time + t) * time + j] * d_out[(index * time + t) * d.value + v];
        }
    }
    dkv[id] = accum;
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
    uint unused0;
    uint unused1;
};

// sqrt(1 + |x|^2) in f64 for every row's query and key, as `pointer_lift`.
extern "C" __global__ void pointer_lift(
    const float* side, double* query_lift, double* key_lift, PointerDims d
) {
    uint row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= d.rows) return;
    const float* q = side + (u64)row * (2 * d.dim + 1);
    const float* k = q + d.dim;
    double qq = 0.0;
    double kk = 0.0;
    for (uint c = 0; c < d.dim; ++c) {
        qq += (double)q[c] * (double)q[c];
        kk += (double)k[c] * (double)k[c];
    }
    query_lift[row] = sqrt(1.0 + qq);
    key_lift[row] = sqrt(1.0 + kk);
}

// The f32 Dot score `dot(q, k) / sqrt(dim)` with the CPU `dot`'s sixteen
// partial sums.
__device__ __forceinline__ float pointer_dot(const float* q, const float* k, uint dim) {
    float partial[16];
    for (uint i = 0; i < 16; ++i) partial[i] = 0.0f;
    uint whole = dim - dim % 16;
    for (uint c = 0; c < whole; c += 16) {
        for (uint i = 0; i < 16; ++i) partial[i] += q[c + i] * k[c + i];
    }
    float total = 0.0f;
    for (uint i = 0; i < 16; ++i) total += partial[i];
    for (uint c = whole; c < dim; ++c) total += q[c] * k[c];
    return total;
}

// The Lorentz excess lift_q lift_k - <q, k> - 1 in f64.
__device__ __forceinline__ double pointer_excess(
    const float* q, const float* k, uint dim, double lift_q, double lift_k
) {
    double inner = 0.0;
    for (uint c = 0; c < dim; ++c) inner += (double)q[c] * (double)k[c];
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
    const float* logits, const float* side, const float* beta_in, const uint* ids, const uint* targets,
    const float* weights, const double* lse, const double* query_lift,
    const double* key_lift, const float* grad_in, const double* total_in,
    double* scratch, double* row_value, double* scale_z, float* d_side,
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
    const float* q = side + (u64)row * stride;
    double beta = (double)beta_in[0];
    double lift_q = d.score == 1 ? query_lift[row] : 1.0;
    float scale_f = 1.0f / sqrtf((float)dim);
    double* s = scratch + (u64)row * time;

    float maximum = neg_inf();
    for (uint j = lane; j <= t; j += 32) {
        const float* k = side + (u64)(first + j) * stride + dim;
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
    for (uint j = lane; j <= t; j += 32) {
        double a = s[j] / sum;
        s[j] = a;
        if (ids[first + j] == target) copy += a;
    }
    copy = warp_sum_d(copy);

    double logit = (double)q[2 * dim];
    double generate =
        -softplus_d(logit) + (double)logits[(u64)row * d.vocab + target] - lse[row];
    // No source holds the target: the copy branch is exactly 0 (no floor).
    double copied = copy > 0.0
        ? -softplus_d(-logit) + log(copy)
        : __longlong_as_double(0xfff0000000000000ULL);
    double high = fmax(generate, copied);
    double log_mixture = high + log(exp(generate - high) + exp(copied - high));

    if (d.backward == 0) {
        if (lane == 0) row_value[row] = -weight * log_mixture;
        return;
    }
    double generate_share = exp(generate - log_mixture);
    double copy_share = exp(copied - log_mixture);
    double gate = 1.0 / (1.0 + exp(-logit));
    double c = (double)grad_in[0] * weight / total_in[0];
    if (lane == 0) {
        scale_z[row] = c * generate_share;
        d_side[(u64)row * stride + 2 * dim] =
            (float)(c * (generate_share * gate - copy_share * (1.0 - gate)));
    }
    double d_beta = 0.0;
    double self_q = 0.0;
    for (uint j = lane; j <= t; j += 32) {
        double a = s[j];
        double out = 0.0;
        if (copy > 0.0 && a != 0.0) {
            double d_source = ids[first + j] == target
                ? -c * copy_share * (a / copy) * (1.0 - copy)
                : c * copy_share * a;
            if (d.score == 0) {
                out = d_source;
            } else {
                const float* k = side + (u64)(first + j) * stride + dim;
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
    const float* side, const double* scratch, const double* query_lift,
    const double* key_lift, const double* query_self, float* d_side, PointerDims d
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
            double k = (double)side[(u64)(first + j) * stride + dim + c];
            if (d.score == 0) {
                acc += g * scale * k;
            } else {
                acc -= g * k;
            }
        }
        if (d.score != 0) {
            acc += query_self[row] * (double)side[(u64)row * stride + c];
        }
    } else {
        uint cc = c - dim;
        double own_key = (double)side[(u64)row * stride + dim + cc];
        for (uint tt = t; tt < time; ++tt) {
            double g = scratch[(u64)(first + tt) * time + t];
            if (g == 0.0) continue;
            double qv = (double)side[(u64)(first + tt) * stride + cc];
            if (d.score == 0) {
                acc += g * scale * qv;
            } else {
                double own = g * query_lift[first + tt] / key_lift[row];
                acc += own * own_key - g * qv;
            }
        }
    }
    d_side[(u64)row * stride + c] = (float)acc;
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

    /// The stack kernels' PTX, compiled once per process by NVRTC.
    fn ptx() -> Result<&'static str> {
        static PTX: OnceLock<std::result::Result<String, String>> = OnceLock::new();
        let compiled = PTX.get_or_init(|| {
            let options = cudarc::nvrtc::CompileOptions {
                use_fast_math: Some(false),
                prec_sqrt: Some(true),
                prec_div: Some(true),
                ftz: Some(false),
                name: Some("uor_r4_geometric_stack.cu".to_string()),
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

    /// Compiles the kernel source with NVRTC (no device needed): a check that
    /// the CUDA C source is valid for the installed toolkit.
    pub fn compile_check() -> Result<()> {
        ptx().map(|_| ())
    }

    /// One kernel argument, in parameter order.
    pub enum Arg<'a> {
        /// An f32 device buffer (read, written or both).
        F(CudaView<'a, f32>),
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

    /// Threads per block of a 1-D launch.
    const BLOCK: usize = 256;

    /// Launches `name` over `threads` threads (a 1-D grid of 256-thread blocks).
    pub fn launch(device: &CudaDevice, name: &str, threads: usize, args: &[Arg<'_>]) -> Result<()> {
        if threads == 0 {
            return Ok(());
        }
        let blocks = threads.div_ceil(BLOCK);
        dispatch(device, name, (blocks, 1, 1), (BLOCK, 1, 1), args)
    }

    /// Launches `name` over `groups` blocks of `group` threads each.
    pub fn launch_groups(
        device: &CudaDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        if groups.0 * groups.1 * groups.2 == 0 {
            return Ok(());
        }
        dispatch(device, name, groups, group, args)
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
        let function = device.get_or_load_custom_func(name, MODULE, ptx()?)?;
        let started = profiling().then(std::time::Instant::now);
        let mut builder = function.builder();
        for arg in args {
            match arg {
                Arg::F(view) => {
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
