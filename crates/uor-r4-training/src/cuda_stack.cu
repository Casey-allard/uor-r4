// The geometric stack's CUDA kernels, compiled AHEAD OF TIME by nvcc in
// build.rs (the `cuda` feature) into two PTX modules: f32, and bf16 storage
// with -DUOR_STORAGE_BF16. The owner banned NVRTC on 13 October 2026, and D29
// makes the compiled module the compliance boundary: no normalised exponential, softplus,
// sigmoid, tanh, exp-family squashing or tensor-core contraction may exist in
// it. The families that did (the fused sigmoid/exp/GELU recurrence,
// SwiGLU, cross-entropy, the general fused read and its flash variants, and the
// pointer-copy mixture) are kept as evidence only, never compiled, in
// docs/history/d29-excluded-kernels-2026-10-13/. `spiralcore_fbs.cu` is
// appended to this source by build.rs.

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


__device__ __forceinline__ double warp_sum_d(double v) {
    for (int o = 16; o > 0; o >>= 1) v += __shfl_xor_sync(0xffffffffu, v, o);
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


// ---------------------------------------------------------------------------
// 1. Straight-through
// ---------------------------------------------------------------------------
extern "C" __global__ void straight_through_fwd(const act_t* src, act_t* dst, uint total) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id < total) dst[id] = src[id];
}

// ---------------------------------------------------------------------------
// 2b. The Hamilton rotation gate (the owner rule of 2026-10-12, G2 only after
// the owner's amendment of 2026-10-13): no exponential enters either direction.
// ---------------------------------------------------------------------------
// The Hamilton gate, one thread per 4-block: the gate block is a quaternion,
// unit-normalized (r carries a 1e-12 guard, exactly as the CPU), and the up
// block is rotated by it. total is the element count, a multiple of 4.
extern "C" __global__ void hamilton_gate_fwd(
    const act_t* gate, const act_t* up, act_t* out, uint total
) {
    uint block = blockIdx.x * blockDim.x + threadIdx.x;
    uint i = block * 4;
    if (i >= total) return;
    float g0 = act_to_f(gate[i]), g1 = act_to_f(gate[i + 1]);
    float g2 = act_to_f(gate[i + 2]), g3 = act_to_f(gate[i + 3]);
    float u0 = act_to_f(up[i]), u1 = act_to_f(up[i + 1]);
    float u2 = act_to_f(up[i + 2]), u3 = act_to_f(up[i + 3]);
    float r = sqrtf(g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f);
    float q0 = g0 / r, q1 = g1 / r, q2 = g2 / r, q3 = g3 / r;
    out[i] = act_from_f(q0 * u0 - q1 * u1 - q2 * u2 - q3 * u3);
    out[i + 1] = act_from_f(q0 * u1 + q1 * u0 + q2 * u3 - q3 * u2);
    out[i + 2] = act_from_f(q0 * u2 - q1 * u3 + q2 * u0 + q3 * u1);
    out[i + 3] = act_from_f(q0 * u3 + q1 * u2 - q2 * u1 + q3 * u0);
}

// Backward: du = conj(q) * d and dq = d * conj(u) (the Hamilton adjoints),
// then the normalization Jacobian dg = (dq - q (q . dq)) / r.
extern "C" __global__ void hamilton_gate_bwd(
    const act_t* gate, const act_t* up, const act_t* grad,
    act_t* d_gate, act_t* d_up, uint total
) {
    uint block = blockIdx.x * blockDim.x + threadIdx.x;
    uint i = block * 4;
    if (i >= total) return;
    float g0 = act_to_f(gate[i]), g1 = act_to_f(gate[i + 1]);
    float g2 = act_to_f(gate[i + 2]), g3 = act_to_f(gate[i + 3]);
    float u0 = act_to_f(up[i]), u1 = act_to_f(up[i + 1]);
    float u2 = act_to_f(up[i + 2]), u3 = act_to_f(up[i + 3]);
    float d0 = act_to_f(grad[i]), d1 = act_to_f(grad[i + 1]);
    float d2 = act_to_f(grad[i + 2]), d3 = act_to_f(grad[i + 3]);
    float r = sqrtf(g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f);
    float q0 = g0 / r, q1 = g1 / r, q2 = g2 / r, q3 = g3 / r;
    d_up[i] = act_from_f(q0 * d0 + q1 * d1 + q2 * d2 + q3 * d3);
    d_up[i + 1] = act_from_f(q0 * d1 - q1 * d0 - q2 * d3 + q3 * d2);
    d_up[i + 2] = act_from_f(q0 * d2 + q1 * d3 - q2 * d0 - q3 * d1);
    d_up[i + 3] = act_from_f(q0 * d3 - q1 * d2 + q2 * d1 - q3 * d0);
    float s0 = d0 * u0 + d1 * u1 + d2 * u2 + d3 * u3;
    float s1 = -d0 * u1 + d1 * u0 - d2 * u3 + d3 * u2;
    float s2 = -d0 * u2 + d1 * u3 + d2 * u0 - d3 * u1;
    float s3 = -d0 * u3 - d1 * u2 + d2 * u1 + d3 * u0;
    float dot = q0 * s0 + q1 * s1 + q2 * s2 + q3 * s3;
    d_gate[i] = act_from_f((s0 - q0 * dot) / r);
    d_gate[i + 1] = act_from_f((s1 - q1 * dot) / r);
    d_gate[i + 2] = act_from_f((s2 - q2 * dot) / r);
    d_gate[i + 3] = act_from_f((s3 - q3 * dot) / r);
}

// ---------------------------------------------------------------------------
// 2d. Exact rate selection (the geometric recurrence's decay): the score picks
// one entry of the fixed dyadic table `lambda = 2^-(s+1)` with
// `keep = sqrt(1 - lambda^2)`. The forward is a threshold and a table read -
// `lambda` is built from its exact f32 bit pattern, never by an exponential -
// and the backward is the declared geometric secant surrogate over the selected
// entry's lawful neighbours at the fixed edge `1/16` (the same rule the host op
// documents). No exponential of any kind appears in either direction.
// ---------------------------------------------------------------------------
#define RATE_ENTRIES 16u

__device__ __forceinline__ float rate_entry(uint index) {
    // 2^-(index+1) as an exact f32 bit pattern (index < 16, no subnormals).
    uint bits = ((127u - (index + 1u)) << 23);
    return __uint_as_float(bits);
}

__device__ __forceinline__ float keep_entry(uint index) {
    // Same precomputed F32 table as KEEP_TABLE; no backend-dependent
    // subtraction/square-root rounding changes the declared table entries.
    const float entries[16] = {
        0.8660253882408142f, 0.9682458639144897f,
        0.9921567440032959f, 0.9980449676513672f,
        0.9995115995407104f, 0.9998779296875f,
        0.999969482421875f, 0.9999923706054688f,
        0.9999980926513672f, 0.9999995231628418f,
        0.9999998807907104f, 0.9999999403953552f,
        1.0f, 1.0f, 1.0f, 1.0f
    };
    return entries[index];
}

__device__ __forceinline__ uint rate_index_of(float score) {
    uint index = (uint)(score * (float)RATE_ENTRIES);
    return index < RATE_ENTRIES ? index : RATE_ENTRIES - 1u;
}

extern "C" __global__ void rate_select_fwd(
    const act_t* score, act_t* out, uint total
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    uint index = rate_index_of(act_to_f(score[id]));
    out[2 * id] = act_from_f(rate_entry(index));
    out[2 * id + 1] = act_from_f(keep_entry(index));
}

extern "C" __global__ void rate_select_bwd(
    const act_t* score, const act_t* grad, act_t* d_score, uint total
) {
    uint id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= total) return;
    uint index = rate_index_of(act_to_f(score[id]));
    float d_lambda = act_to_f(grad[2 * id]);
    float d_keep = act_to_f(grad[2 * id + 1]);
    float credit = 0.0f;
    float sides = 0.0f;
    if (index > 0u) {
        credit -= d_lambda * (rate_entry(index - 1u) - rate_entry(index))
                + d_keep * (keep_entry(index - 1u) - keep_entry(index));
        sides += 1.0f;
    }
    if (index + 1u < RATE_ENTRIES) {
        credit += d_lambda * (rate_entry(index + 1u) - rate_entry(index))
                + d_keep * (keep_entry(index + 1u) - keep_entry(index));
        sides += 1.0f;
    }
    if (sides > 0.0f) credit /= sides;
    d_score[id] = act_from_f(credit * (float)RATE_ENTRIES);
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
// Rank-margin (the D23 exponential-free objective): row losses and gradients. One
// warp per row, at most 8 rivals, logits compared in f32 and losses
// accumulated in f64 like the CPU's `margin_row`. Ties resolve to the lower
// index, the CPU stable-sort's order. No exponential of any kind anywhere.
// ---------------------------------------------------------------------------
// 5. Rank margin over logits: a hinge over the declared rivals, raw-score
// units, no exponential (the historical cross-entropy of this section was
// removed by D29).
// ---------------------------------------------------------------------------
__device__ __forceinline__ void margin_select_impl(
    const float* row, uint target, uint vocab, uint k,
    float* gaps, uint* indices, float* shared_gaps, uint* shared_indices
) {
    // Each lane keeps its strided scan's top-k (descending, ties to the lower index).
    float local_gap[8];
    uint local_idx[8];
    uint n = 0;
    for (uint i = threadIdx.x; i < vocab; i += 32) {
        if (i == target) continue;
        float gap = row[i];
        if (n < k) {
            uint j = n++;
            while (j > 0 && (local_gap[j-1] < gap || (local_gap[j-1] == gap && local_idx[j-1] > i))) {
                local_gap[j] = local_gap[j-1]; local_idx[j] = local_idx[j-1]; j--;
            }
            local_gap[j] = gap; local_idx[j] = i;
        } else if (gap > local_gap[k-1] || (gap == local_gap[k-1] && i < local_idx[k-1])) {
            uint j = k - 1;
            while (j > 0 && (local_gap[j-1] < gap || (local_gap[j-1] == gap && local_idx[j-1] > i))) {
                local_gap[j] = local_gap[j-1]; local_idx[j] = local_idx[j-1]; j--;
            }
            local_gap[j] = gap; local_idx[j] = i;
        }
    }
    uint base = (threadIdx.x & 31) * 8;
    #pragma unroll
    for (uint j = 0; j < 8; j++) {
        shared_gaps[base + j] = (j < n) ? local_gap[j] : -3.402823466e38f;
        shared_indices[base + j] = (j < n) ? local_idx[j] : 0u;
    }
    __syncwarp();
    if ((threadIdx.x & 31) == 0) {
        // Merge the warp's candidates: k selection rounds, ties to the lower index.
        for (uint s = 0; s < k; s++) {
            float best = -3.402823466e38f;
            uint bi = 0xffffffffu;
            for (uint j = 0; j < 256; j++) {
                if (shared_gaps[j] > best ||
                    (bi != 0xffffffffu && shared_gaps[j] == best && shared_indices[j] < shared_indices[bi])) {
                    best = shared_gaps[j]; bi = j;
                }
            }
            if (bi == 0xffffffffu) { gaps[s] = -3.402823466e38f; indices[s] = target; continue; }
            gaps[s] = best; indices[s] = shared_indices[bi];
            shared_gaps[bi] = -3.402823466e38f;
        }
    }
    __syncwarp();
}

__device__ __forceinline__ void margin_rank_rows_impl(
    const float* logits, const uint* targets, double* loss, uint vocab, uint rivals
) {
    __shared__ float shared_gaps[256];
    __shared__ uint shared_indices[256];
    u64 offset = (u64)blockIdx.x * vocab;
    float gaps[8]; uint indices[8];
    margin_select_impl(logits + offset, targets[blockIdx.x], vocab, rivals, gaps, indices, shared_gaps, shared_indices);
    if (threadIdx.x == 0) {
        float zt = logits[offset + targets[blockIdx.x]];
        double total = 0.0;
        uint take = rivals < vocab - 1 ? rivals : vocab - 1;
        for (uint s = 0; s < rivals; s++) {
            double gap = 1.0 + (double)gaps[s] - (double)zt;
            if (gap > 0.0) total += gap;
        }
        loss[blockIdx.x] = total / (double)take;
    }
}

extern "C" __global__ void margin_rank_rows(
    const float* logits, const uint* targets, double* loss, uint vocab, uint rivals
) {
    margin_rank_rows_impl(logits, targets, loss, vocab, rivals);
}

extern "C" __global__ void margin_rank_grad(
    const float* logits, const uint* targets, const double* scale,
    float* grad, uint vocab, uint rivals
) {
    __shared__ float shared_gaps[256];
    __shared__ uint shared_indices[256];
    u64 offset = (u64)blockIdx.x * vocab;
    float gaps[8]; uint indices[8];
    margin_select_impl(logits + offset, targets[blockIdx.x], vocab, rivals, gaps, indices, shared_gaps, shared_indices);
    uint target = targets[blockIdx.x];
    float zt = logits[offset + target];
    uint take = rivals < vocab - 1 ? rivals : vocab - 1;
    if (threadIdx.x == 0) {
        uint violated = 0;
        for (uint s = 0; s < rivals; s++) {
            double gap = 1.0 + (double)gaps[s] - (double)zt;
            if (gap > 0.0) {
                grad[offset + indices[s]] = (float)(scale[blockIdx.x] / (double)take);
                violated++;
            }
        }
        grad[offset + target] = (float)(-((double)violated) * scale[blockIdx.x] / (double)take);
    }
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
