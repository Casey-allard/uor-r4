//! Native Metal Forward and Backward Kernels for Geometric Stack Ops.
//!
//! Implements Apple Silicon GPU shaders for the 7 operations in `geometric_stack.rs`:
//! 1. StraightThrough (fwd + bwd)
//! 2. SwiGlu (fwd + bwd)
//! 3. RmsNorm (fwd + bwd dx/dw)
//! 4. QuaternionScan (fwd + bwd using native float4 Hamilton product)
//! 5. CrossEntropy (fwd + bwd)
//! 6. RecurrenceCore (fwd)
//! 7. FusedRead (fwd)

#[cfg(feature = "metal")]
pub mod metal {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    use candle_core::{Error, MetalDevice, Result};
    use candle_metal_kernels::metal::{Buffer, ComputePipeline};
    use candle_metal_kernels::utils::set_param;
    use objc2_metal::{MTLResourceUsage, MTLSize};

    #[inline]
    fn linear_split(pipeline: &ComputePipeline, length: usize) -> (MTLSize, MTLSize) {
        let size = length;
        let max_threads = pipeline.max_total_threads_per_threadgroup();
        let width = std::cmp::max(1, std::cmp::min(max_threads, size));
        let count = size.div_ceil(width);
        let thread_group_count = MTLSize {
            width: count,
            height: 1,
            depth: 1,
        };
        let thread_group_size = MTLSize {
            width,
            height: 1,
            depth: 1,
        };
        (thread_group_count, thread_group_size)
    }

    pub const METAL_STACK_SHADERS: &str = r#"
#include <metal_stdlib>
using namespace metal;

// ---------------------------------------------------------------------------
// 1. Straight-Through
// ---------------------------------------------------------------------------
kernel void straight_through_fwd(
    device const float* src [[buffer(0)]],
    device float* dst [[buffer(1)]],
    constant uint& total [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    if (id < total) {
        dst[id] = src[id];
    }
}

// ---------------------------------------------------------------------------
// 2. SwiGLU Forward & Backward
// ---------------------------------------------------------------------------
kernel void swiglu_fwd(
    device const float* gate [[buffer(0)]],
    device const float* up [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant uint& total [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= total) return;
    float g = gate[id];
    float u = up[id];
    float s = 1.0f / (1.0f + exp(-g));
    out[id] = g * s * u;
}

kernel void swiglu_bwd(
    device const float* gate [[buffer(0)]],
    device const float* up [[buffer(1)]],
    device const float* grad [[buffer(2)]],
    device float* d_gate [[buffer(3)]],
    device float* d_up [[buffer(4)]],
    constant uint& total [[buffer(5)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= total) return;
    float g = gate[id];
    float u = up[id];
    float d = grad[id];
    float s = 1.0f / (1.0f + exp(-g));
    d_up[id] = d * g * s;
    d_gate[id] = d * u * s * (1.0f + g * (1.0f - s));
}

// ---------------------------------------------------------------------------
// 3. RMSNorm Forward & Backward
// ---------------------------------------------------------------------------
kernel void rms_norm_fwd(
    device const float* x [[buffer(0)]],
    device const float* w [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant uint& width [[buffer(3)]],
    constant float& eps [[buffer(4)]],
    uint row [[threadgroup_position_in_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint threads [[threads_per_threadgroup]]
) {
    uint offset = row * width;
    float sum_sq = 0.0f;
    for (uint i = tid; i < width; i += threads) {
        float v = x[offset + i];
        sum_sq += v * v;
    }
    threadgroup float shared[32];
    sum_sq = simd_sum(sum_sq);
    if ((tid & 31) == 0) shared[tid / 32] = sum_sq;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float total = 0.0f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) total += shared[i];
        shared[0] = 1.0f / sqrt(total / float(width) + eps);
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    float r = shared[0];
    for (uint i = tid; i < width; i += threads) {
        out[offset + i] = x[offset + i] * r * w[i];
    }
}

kernel void rms_norm_bwd_dx(
    device const float* x [[buffer(0)]],
    device const float* w [[buffer(1)]],
    device const float* grad [[buffer(2)]],
    device float* dx [[buffer(3)]],
    constant uint& width [[buffer(4)]],
    constant float& eps [[buffer(5)]],
    uint row [[threadgroup_position_in_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint threads [[threads_per_threadgroup]]
) {
    uint offset = row * width;
    float sum_sq = 0.0f;
    float sum_gw_x = 0.0f;
    for (uint i = tid; i < width; i += threads) {
        float xi = x[offset + i];
        float gi = grad[offset + i];
        float wi = w[i];
        sum_sq += xi * xi;
        sum_gw_x += gi * wi * xi;
    }
    sum_sq = simd_sum(sum_sq);
    sum_gw_x = simd_sum(sum_gw_x);

    threadgroup float shared_sq[32];
    threadgroup float shared_proj[32];
    if ((tid & 31) == 0) {
        shared_sq[tid / 32] = sum_sq;
        shared_proj[tid / 32] = sum_gw_x;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float tot_sq = 0.0f;
        float tot_proj = 0.0f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) {
            tot_sq += shared_sq[i];
            tot_proj += shared_proj[i];
        }
        float r = 1.0f / sqrt(tot_sq / float(width) + eps);
        shared_sq[0] = r;
        shared_proj[0] = tot_proj * r / float(width);
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    float r = shared_sq[0];
    float proj = shared_proj[0];
    for (uint i = tid; i < width; i += threads) {
        float xhat = x[offset + i] * r;
        dx[offset + i] = r * (grad[offset + i] * w[i] - xhat * proj);
    }
}

kernel void rms_norm_bwd_dw(
    device const float* x [[buffer(0)]],
    device const float* grad [[buffer(1)]],
    device float* dw [[buffer(2)]],
    constant uint& rows [[buffer(3)]],
    constant uint& width [[buffer(4)]],
    constant float& eps [[buffer(5)]],
    uint col [[thread_position_in_grid]]
) {
    if (col >= width) return;
    float sum = 0.0f;
    for (uint r = 0; r < rows; ++r) {
        uint offset = r * width;
        float sum_sq = 0.0f;
        for (uint i = 0; i < width; ++i) {
            float v = x[offset + i];
            sum_sq += v * v;
        }
        float scale = 1.0f / sqrt(sum_sq / float(width) + eps);
        sum += grad[offset + col] * x[offset + col] * scale;
    }
    dw[col] = sum;
}

// ---------------------------------------------------------------------------
// 4. Quaternion Transport Scan Forward & Backward
// ---------------------------------------------------------------------------
inline float4 quat_mul(float4 a, float4 b) {
    return float4(
        a.x * b.x - a.y * b.y - a.z * b.z - a.w * b.w,
        a.x * b.y + a.y * b.x + a.z * b.w - a.w * b.z,
        a.x * b.z - a.y * b.w + a.z * b.x + a.w * b.y,
        a.x * b.w + a.y * b.z - a.z * b.y + a.w * b.x
    );
}

inline float4 quat_conj(float4 a) {
    return float4(a.x, -a.y, -a.z, -a.w);
}

kernel void quaternion_scan_fwd(
    device const float4* transition [[buffer(0)]],
    device const float4* drive [[buffer(1)]],
    device float4* state_out [[buffer(2)]],
    constant uint& time [[buffer(3)]],
    constant uint& lanes [[buffer(4)]],
    constant uint& total_seqs [[buffer(5)]],
    uint seq_id [[thread_position_in_grid]]
) {
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 held = float4(0.0f);
    for (uint t = 0; t < time; ++t) {
        uint idx = (b * time + t) * lanes + lane;
        float4 moved = quat_mul(transition[idx], held);
        held = moved + drive[idx];
        state_out[idx] = held;
    }
}

kernel void quaternion_scan_bwd(
    device const float4* transition [[buffer(0)]],
    device const float4* state [[buffer(1)]],
    device const float4* grad [[buffer(2)]],
    device float4* dq [[buffer(3)]],
    device float4* db [[buffer(4)]],
    constant uint& time [[buffer(5)]],
    constant uint& lanes [[buffer(6)]],
    constant uint& total_seqs [[buffer(7)]],
    uint seq_id [[thread_position_in_grid]]
) {
    if (seq_id >= total_seqs) return;
    uint b = seq_id / lanes;
    uint lane = seq_id % lanes;
    float4 carried = float4(0.0f);
    for (int t = int(time) - 1; t >= 0; --t) {
        uint idx = (b * time + uint(t)) * lanes + lane;
        float4 total = grad[idx];
        if (uint(t + 1) < time) {
            uint next_idx = (b * time + uint(t + 1)) * lanes + lane;
            float4 back = quat_mul(quat_conj(transition[next_idx]), carried);
            total += back;
        }
        db[idx] = total;
        if (t > 0) {
            uint prev_idx = (b * time + uint(t - 1)) * lanes + lane;
            dq[idx] = quat_mul(total, quat_conj(state[prev_idx]));
        } else {
            dq[idx] = float4(0.0f);
        }
        carried = total;
    }
}

// ---------------------------------------------------------------------------
// 5. Cross-Entropy Forward & Backward
// ---------------------------------------------------------------------------
kernel void cross_entropy_fwd(
    device const float* logits [[buffer(0)]],
    device const uint* targets [[buffer(1)]],
    device float* loss [[buffer(2)]],
    constant uint& vocab [[buffer(3)]],
    uint row [[threadgroup_position_in_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint threads [[threads_per_threadgroup]]
) {
    uint offset = row * vocab;
    float max_val = -1e30f;
    for (uint i = tid; i < vocab; i += threads) {
        max_val = max(max_val, logits[offset + i]);
    }
    max_val = simd_max(max_val);
    threadgroup float shared_max[32];
    if ((tid & 31) == 0) shared_max[tid / 32] = max_val;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float m = -1e30f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) m = max(m, shared_max[i]);
        shared_max[0] = m;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);
    float row_max = shared_max[0];

    float sum_exp = 0.0f;
    for (uint i = tid; i < vocab; i += threads) {
        sum_exp += exp(logits[offset + i] - row_max);
    }
    sum_exp = simd_sum(sum_exp);
    threadgroup float shared_sum[32];
    if ((tid & 31) == 0) shared_sum[tid / 32] = sum_exp;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float tot = 0.0f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) tot += shared_sum[i];
        float log_z = row_max + log(tot);
        uint target = targets[row];
        loss[row] = log_z - logits[offset + target];
    }
}

kernel void cross_entropy_bwd(
    device const float* logits [[buffer(0)]],
    device const uint* targets [[buffer(1)]],
    device float* grad [[buffer(2)]],
    constant uint& vocab [[buffer(3)]],
    constant float& scale [[buffer(4)]],
    uint row [[threadgroup_position_in_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint threads [[threads_per_threadgroup]]
) {
    uint offset = row * vocab;
    float max_val = -1e30f;
    for (uint i = tid; i < vocab; i += threads) {
        max_val = max(max_val, logits[offset + i]);
    }
    max_val = simd_max(max_val);
    threadgroup float shared_max[32];
    if ((tid & 31) == 0) shared_max[tid / 32] = max_val;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float m = -1e30f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) m = max(m, shared_max[i]);
        shared_max[0] = m;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);
    float row_max = shared_max[0];

    float sum_exp = 0.0f;
    for (uint i = tid; i < vocab; i += threads) {
        sum_exp += exp(logits[offset + i] - row_max);
    }
    sum_exp = simd_sum(sum_exp);
    threadgroup float shared_sum[32];
    if ((tid & 31) == 0) shared_sum[tid / 32] = sum_exp;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (tid == 0) {
        float tot = 0.0f;
        uint num_warps = (threads + 31) / 32;
        for (uint i = 0; i < num_warps; ++i) tot += shared_sum[i];
        shared_sum[0] = 1.0f / tot;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);
    float inv_z = shared_sum[0];
    uint target = targets[row];

    for (uint i = tid; i < vocab; i += threads) {
        float p = exp(logits[offset + i] - row_max) * inv_z;
        float diff = (i == target) ? (p - 1.0f) : p;
        grad[offset + i] = diff * scale;
    }
}

inline float gelu_fwd(float x) {
    const float K = 0.7978846f;
    const float C = 0.044715f;
    float v = K * (x + C * x * x * x);
    // precise::tanh of a clamped argument: the fast tanh overflows to NaN
    // for large |v| (a ratio of overflowing exponentials), while tanh is
    // exactly +-1 in f32 beyond |v| ~ 9, as the CPU computes it.
    float t = precise::tanh(clamp(v, -20.0f, 20.0f));
    return 0.5f * x * (1.0f + t);
}

// ---------------------------------------------------------------------------
// 6. Recurrence Core Forward
// ---------------------------------------------------------------------------
kernel void recurrence_core_fwd(
    device const float* branches [[buffer(0)]],
    device const float* gates [[buffer(1)]],
    device const float* params [[buffer(2)]],
    device const float* log_a [[buffer(3)]],
    device float* state_out [[buffer(4)]],
    device float* drive_out [[buffer(5)]],
    device float* out [[buffer(6)]],
    constant uint& time [[buffer(7)]],
    constant uint& width [[buffer(8)]],
    constant uint& lanes [[buffer(9)]],
    constant uint& gate_width [[buffer(10)]],
    constant uint& rotation [[buffer(11)]],
    constant uint& total_lanes [[buffer(12)]],
    uint lane_id [[thread_position_in_grid]]
) {
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;

    device const float* taps = params;
    device const float* bias = params + 4 * width;

    float4 held = float4(0.0f);
    for (uint t = 0; t < time; ++t) {
        float4 d_val = float4(0.0f);
        for (uint k = 0; k < 4; ++k) {
            uint ch = 4 * lane + k;
            float c_val = bias[ch];
            for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
                float w = taps[shift * width + ch];
                float a = branches[(b * time + (t - shift)) * (2 * width) + ch];
                c_val += w * a;
            }
            d_val[k] = c_val;
        }
        uint drive_idx = (b * time + t) * width + 4 * lane;
        drive_out[drive_idx + 0] = d_val.x;
        drive_out[drive_idx + 1] = d_val.y;
        drive_out[drive_idx + 2] = d_val.z;
        drive_out[drive_idx + 3] = d_val.w;

        uint gate_base = (b * time + t) * gate_width;
        float r_val = 1.0f / (1.0f + exp(-gates[gate_base + lane]));
        float decay = exp(8.0f * r_val * log_a[lane]);
        float complement = 1.0f - decay * decay;
        float keep = (complement < 1e-6f) ? 1e-3f : sqrt(complement);

        float4 q;
        if (rotation != 0) {
            float u0 = gates[gate_base + lanes + 4 * lane + 0];
            float u1 = gates[gate_base + lanes + 4 * lane + 1];
            float u2 = gates[gate_base + lanes + 4 * lane + 2];
            float u3 = gates[gate_base + lanes + 4 * lane + 3];
            float norm = sqrt(u0 * u0 + u1 * u1 + u2 * u2 + u3 * u3 + 1e-6f);
            float s = decay / norm;
            q = float4(u0 * s, u1 * s, u2 * s, u3 * s);
        } else {
            q = float4(decay, 0.0f, 0.0f, 0.0f);
        }

        held = quat_mul(q, held) + keep * d_val;
        state_out[drive_idx + 0] = held.x;
        state_out[drive_idx + 1] = held.y;
        state_out[drive_idx + 2] = held.z;
        state_out[drive_idx + 3] = held.w;

        uint branch_base = (b * time + t) * (2 * width) + 4 * lane;
        float4 g_val = float4(
            branches[branch_base + width + 0],
            branches[branch_base + width + 1],
            branches[branch_base + width + 2],
            branches[branch_base + width + 3]
        );
        out[drive_idx + 0] = held.x * gelu_fwd(g_val.x);
        out[drive_idx + 1] = held.y * gelu_fwd(g_val.y);
        out[drive_idx + 2] = held.z * gelu_fwd(g_val.z);
        out[drive_idx + 3] = held.w * gelu_fwd(g_val.w);
    }
}

// ---------------------------------------------------------------------------
// 7. Fused Causal Read Forward
// ---------------------------------------------------------------------------
kernel void fused_read_fwd(
    device const float* query [[buffer(0)]],
    device const float* kv [[buffer(1)]],
    device const float* aux [[buffer(2)]],
    device float* out [[buffer(3)]],
    constant uint& time [[buffer(4)]],
    constant uint& key_dim [[buffer(5)]],
    constant uint& val_dim [[buffer(6)]],
    constant uint& kv_width [[buffer(7)]],
    constant float& scale [[buffer(8)]],
    constant uint& total_heads [[buffer(9)]],
    uint head_id [[thread_position_in_grid]]
) {
    if (head_id >= total_heads) return;
    for (uint t = 0; t < time; ++t) {
        uint q_offset = (head_id * time + t) * key_dim;
        float max_score = -1e30f;
        for (uint s = 0; s <= t; ++s) {
            uint kv_offset = (head_id * time + s) * kv_width;
            float dot = 0.0f;
            for (uint d = 0; d < key_dim; ++d) {
                dot += query[q_offset + d] * kv[kv_offset + d];
            }
            max_score = max(max_score, dot * scale);
        }

        float sum_exp = 0.0f;
        for (uint s = 0; s <= t; ++s) {
            uint kv_offset = (head_id * time + s) * kv_width;
            float dot = 0.0f;
            for (uint d = 0; d < key_dim; ++d) {
                dot += query[q_offset + d] * kv[kv_offset + d];
            }
            sum_exp += exp(dot * scale - max_score);
        }
        float inv_z = 1.0f / sum_exp;

        uint out_offset = (head_id * time + t) * val_dim;
        for (uint v = 0; v < val_dim; ++v) {
            float accum = 0.0f;
            for (uint s = 0; s <= t; ++s) {
                uint kv_offset = (head_id * time + s) * kv_width;
                float dot = 0.0f;
                for (uint d = 0; d < key_dim; ++d) {
                    dot += query[q_offset + d] * kv[kv_offset + d];
                }
                float w = exp(dot * scale - max_score) * inv_z;
                accum += w * kv[kv_offset + key_dim + v];
            }
            out[out_offset + v] = accum;
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Recurrence Core: log a on the device, exact backward, parameter reduction
// ---------------------------------------------------------------------------

// log(1 + x), accurate for small x (Goldberg's correction).
inline float log1p_accurate(float x) {
    float u = 1.0f + x;
    if (u == 1.0f) return x;
    return precise::log(u) * x / (u - 1.0f);
}

// GELU (tanh approximation) and its derivative, as the CPU `gelu`.
inline float2 gelu_value_slope(float x) {
    const float K = 0.7978846f;
    const float C = 0.044715f;
    float v = K * (x + C * x * x * x);
    float t = precise::tanh(clamp(v, -20.0f, 20.0f));
    float value = 0.5f * x * (1.0f + t);
    float slope = 0.5f * (1.0f + t) + 0.5f * x * (1.0f - t * t) * K * (1.0f + 3.0f * C * x * x);
    return float2(value, slope);
}

// log a = -softplus(-decay) per lane.
kernel void recurrence_log_a(
    device const float* params [[buffer(0)]],
    device float* log_a [[buffer(1)]],
    constant uint& width [[buffer(2)]],
    constant uint& lanes [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= lanes) return;
    float x = -params[5 * width + id];
    float softplus = max(x, 0.0f) + log1p_accurate(precise::exp(-fabs(x)));
    log_a[id] = -softplus;
}

// One thread per (window, lane): the reverse sweep of the CPU backward.
// Writes d_branches and d_gates in full and the window's parameter partials
// (taps, bias, d log a) into partials[window * param_len ..].
kernel void recurrence_core_bwd(
    device const float* branches [[buffer(0)]],
    device const float* gates [[buffer(1)]],
    device const float* params [[buffer(2)]],
    device const float* log_a [[buffer(3)]],
    device const float* state [[buffer(4)]],
    device const float* drive [[buffer(5)]],
    device const float* d_out [[buffer(6)]],
    device float* d_branches [[buffer(7)]],
    device float* d_gates [[buffer(8)]],
    device float* partials [[buffer(9)]],
    constant uint& time [[buffer(10)]],
    constant uint& width [[buffer(11)]],
    constant uint& lanes [[buffer(12)]],
    constant uint& gate_width [[buffer(13)]],
    constant uint& rotation [[buffer(14)]],
    constant uint& total_lanes [[buffer(15)]],
    constant uint& param_len [[buffer(16)]],
    uint lane_id [[thread_position_in_grid]]
) {
    if (lane_id >= total_lanes) return;
    uint b = lane_id / lanes;
    uint lane = lane_id % lanes;
    uint ch = 4 * lane;
    uint two_w = 2 * width;
    device const float* taps = params;
    float la = log_a[lane];

    for (uint t = 0; t < time; ++t) {
        uint src = (b * time + t) * two_w + ch;
        d_branches[src + 0] = 0.0f;
        d_branches[src + 1] = 0.0f;
        d_branches[src + 2] = 0.0f;
        d_branches[src + 3] = 0.0f;
    }

    float4 held = float4(0.0f);
    float4 q_next = float4(0.0f);
    float4 d_bias = float4(0.0f);
    float4 d_tap0 = float4(0.0f);
    float4 d_tap1 = float4(0.0f);
    float4 d_tap2 = float4(0.0f);
    float4 d_tap3 = float4(0.0f);
    float d_log_a = 0.0f;
    float4 tap0 = float4(taps[0 * width + ch], taps[0 * width + ch + 1], taps[0 * width + ch + 2], taps[0 * width + ch + 3]);
    float4 tap1 = float4(taps[1 * width + ch], taps[1 * width + ch + 1], taps[1 * width + ch + 2], taps[1 * width + ch + 3]);
    float4 tap2 = float4(taps[2 * width + ch], taps[2 * width + ch + 1], taps[2 * width + ch + 2], taps[2 * width + ch + 3]);
    float4 tap3 = float4(taps[3 * width + ch], taps[3 * width + ch + 1], taps[3 * width + ch + 2], taps[3 * width + ch + 3]);

    for (int ti = int(time) - 1; ti >= 0; --ti) {
        uint t = uint(ti);
        uint row = b * time + t;
        uint base = row * width + ch;
        uint branch_row = row * two_w;
        float4 st = float4(state[base], state[base + 1], state[base + 2], state[base + 3]);
        float4 dy = float4(d_out[base], d_out[base + 1], d_out[base + 2], d_out[base + 3]);
        float4 direct;
        for (uint k = 0; k < 4; ++k) {
            float2 vs = gelu_value_slope(branches[branch_row + width + ch + k]);
            d_branches[branch_row + width + ch + k] = dy[k] * st[k] * vs.y;
            direct[k] = dy[k] * vs.x;
        }
        // This position's transition.
        uint gate_row = row * gate_width;
        float opening = 1.0f / (1.0f + precise::exp(-gates[gate_row + lane]));
        float lambda = precise::exp(8.0f * opening * la);
        float complement = 1.0f - lambda * lambda;
        bool clamped = complement < 1e-6f;
        float keep = clamped ? 1e-3f : precise::sqrt(complement);
        float4 unit = float4(1.0f, 0.0f, 0.0f, 0.0f);
        float norm = 1.0f;
        if (rotation != 0) {
            uint r = gate_row + lanes + ch;
            float4 raw = float4(gates[r], gates[r + 1], gates[r + 2], gates[r + 3]);
            norm = precise::sqrt(raw.x * raw.x + raw.y * raw.y + raw.z * raw.z + raw.w * raw.w + 1e-6f);
            unit = raw / norm;
        }
        float4 total = direct;
        if (t + 1 < time) {
            total += quat_mul(quat_conj(q_next), held);
        }
        held = total;
        float4 c = float4(drive[base], drive[base + 1], drive[base + 2], drive[base + 3]);
        float d_keep = dot(total, c);
        float4 dd = keep * total;
        float4 dq = float4(0.0f);
        if (t > 0) {
            uint prev = base - width;
            float4 earlier = float4(state[prev], state[prev + 1], state[prev + 2], state[prev + 3]);
            dq = quat_mul(total, quat_conj(earlier));
        }
        float d_lambda = dot(dq, unit);
        if (!clamped) {
            d_lambda -= d_keep * lambda / keep;
        }
        float d_log_lambda = d_lambda * lambda;
        float d_opening = d_log_lambda * 8.0f * la;
        d_log_a += d_log_lambda * 8.0f * opening;
        d_gates[gate_row + lane] = d_opening * opening * (1.0f - opening);
        if (rotation != 0) {
            float4 du = dq * lambda;
            float projection = dot(du, unit);
            float4 dr = (du - unit * projection) / norm;
            uint r = gate_row + lanes + ch;
            d_gates[r] = dr.x;
            d_gates[r + 1] = dr.y;
            d_gates[r + 2] = dr.z;
            d_gates[r + 3] = dr.w;
        }
        q_next = unit * lambda;
        // Convolution: c_t = bias + sum_shift taps_shift * a_{t - shift}.
        d_bias += dd;
        for (uint shift = 0; shift < 4 && shift <= t; ++shift) {
            uint src = (row - shift) * two_w + ch;
            float4 a = float4(branches[src], branches[src + 1], branches[src + 2], branches[src + 3]);
            float4 w = shift == 0 ? tap0 : (shift == 1 ? tap1 : (shift == 2 ? tap2 : tap3));
            float4 contribution = dd * a;
            if (shift == 0) d_tap0 += contribution;
            else if (shift == 1) d_tap1 += contribution;
            else if (shift == 2) d_tap2 += contribution;
            else d_tap3 += contribution;
            float4 back = w * dd;
            d_branches[src] += back.x;
            d_branches[src + 1] += back.y;
            d_branches[src + 2] += back.z;
            d_branches[src + 3] += back.w;
        }
    }
    device float* p = partials + b * param_len;
    for (uint k = 0; k < 4; ++k) {
        p[0 * width + ch + k] = d_tap0[k];
        p[1 * width + ch + k] = d_tap1[k];
        p[2 * width + ch + k] = d_tap2[k];
        p[3 * width + ch + k] = d_tap3[k];
        p[4 * width + ch + k] = d_bias[k];
    }
    p[5 * width + lane] = d_log_a;
}

// Sums the windows' parameter partials; d log a / d decay = sigma(-decay).
kernel void recurrence_param_reduce(
    device const float* partials [[buffer(0)]],
    device const float* params [[buffer(1)]],
    device float* d_params [[buffer(2)]],
    constant uint& batch [[buffer(3)]],
    constant uint& param_len [[buffer(4)]],
    constant uint& width [[buffer(5)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= param_len) return;
    float sum = 0.0f;
    for (uint b = 0; b < batch; ++b) {
        sum += partials[b * param_len + id];
    }
    if (id >= 5 * width) {
        sum *= 1.0f / (1.0f + precise::exp(params[id]));
    }
    d_params[id] = sum;
}

// ---------------------------------------------------------------------------
// 9. General Fused Read: Dot or Lorentz score, NoRead slot, age table
// ---------------------------------------------------------------------------
// dims: [batch, heads, time, key, value, null_on, age_on, lorentz]
// Blocks are index = window * heads + head; rows are index * time + t; the
// square scratch is [index, t, j] with j <= t used.

struct ReadDims {
    uint batch;
    uint heads;
    uint time;
    uint key;
    uint value;
    uint null_on;
    uint age_on;
    uint lorentz;
};

inline uint read_age_offset(constant ReadDims& d) {
    return d.null_on != 0 ? d.batch * d.heads * d.time : 0;
}

inline uint read_beta_offset(constant ReadDims& d) {
    return read_age_offset(d) + (d.age_on != 0 ? d.heads * d.time : 0);
}

inline float read_distance(float e) {
    float x = max(e, 1e-7f);
    return log1p_accurate(x + precise::sqrt(x * (x + 2.0f)));
}

// Lifts sqrt(1 + |x|^2) of every query and key row.
kernel void read_lift(
    device const float* query [[buffer(0)]],
    device const float* kv [[buffer(1)]],
    device float* query_lift [[buffer(2)]],
    device float* key_lift [[buffer(3)]],
    constant ReadDims& d [[buffer(4)]],
    uint id [[thread_position_in_grid]]
) {
    uint rows = d.batch * d.heads * d.time;
    if (id >= rows) return;
    device const float* q = query + id * d.key;
    device const float* k = kv + id * (d.key + d.value);
    float qq = 0.0f;
    float kk = 0.0f;
    for (uint c = 0; c < d.key; ++c) {
        qq += q[c] * q[c];
        kk += k[c] * k[c];
    }
    query_lift[id] = precise::sqrt(1.0f + qq);
    key_lift[id] = precise::sqrt(1.0f + kk);
}

// Scores of every (index, t, j <= t); with `write_excess`, Lorentz excesses too.
kernel void read_scores(
    device const float* query [[buffer(0)]],
    device const float* kv [[buffer(1)]],
    device const float* aux [[buffer(2)]],
    device const float* query_lift [[buffer(3)]],
    device const float* key_lift [[buffer(4)]],
    device float* scores [[buffer(5)]],
    device float* excess [[buffer(6)]],
    constant ReadDims& d [[buffer(7)]],
    constant uint& write_excess [[buffer(8)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    uint total = d.batch * d.heads * time * time;
    if (id >= total) return;
    uint j = id % time;
    uint row = id / time;
    uint t = row % time;
    if (j > t) return;
    uint index = row / time;
    uint head = index % d.heads;
    uint width = d.key + d.value;
    device const float* q = query + row * d.key;
    device const float* k = kv + (index * time + j) * width;
    float inner = 0.0f;
    for (uint c = 0; c < d.key; ++c) {
        inner += q[c] * k[c];
    }
    float age = d.age_on != 0 ? aux[read_age_offset(d) + head * time + (t - j)] : 0.0f;
    float score;
    if (d.lorentz != 0) {
        float e = query_lift[row] * key_lift[index * time + j] - inner - 1.0f;
        if (write_excess != 0) {
            excess[id] = e;
        }
        uint beta_offset = read_beta_offset(d);
        float beta = aux[beta_offset + head];
        float offset = aux[beta_offset + d.heads + head];
        score = -beta * (read_distance(e) - offset) + age;
    } else {
        score = inner * rsqrt(float(d.key)) + age;
    }
    scores[id] = score;
}

// Softmax of each row over j <= t and the NoRead slot, in place; the NoRead
// probability goes to null_probability[row].
kernel void read_softmax(
    device float* scores [[buffer(0)]],
    device const float* aux [[buffer(1)]],
    device float* null_probability [[buffer(2)]],
    constant ReadDims& d [[buffer(3)]],
    uint row [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    device float* s = scores + row * time;
    float null_score = d.null_on != 0 ? aux[row] : -INFINITY;
    float maximum = null_score;
    for (uint j = 0; j <= t; ++j) {
        maximum = max(maximum, s[j]);
    }
    float null_weight = d.null_on != 0 ? precise::exp(null_score - maximum) : 0.0f;
    float total = null_weight;
    for (uint j = 0; j <= t; ++j) {
        float w = precise::exp(s[j] - maximum);
        s[j] = w;
        total += w;
    }
    float inverse = 1.0f / total;
    for (uint j = 0; j <= t; ++j) {
        s[j] *= inverse;
    }
    null_probability[row] = null_weight * inverse;
}

// out[index, t, v] = sum_{j <= t} p[t, j] value[j, v].
kernel void read_mix(
    device const float* probabilities [[buffer(0)]],
    device const float* kv [[buffer(1)]],
    device float* out [[buffer(2)]],
    constant ReadDims& d [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (id >= d.batch * d.heads * time * d.value) return;
    uint v = id % d.value;
    uint row = id / d.value;
    uint t = row % time;
    uint index = row / time;
    uint width = d.key + d.value;
    device const float* p = probabilities + row * time;
    device const float* values = kv + index * time * width + d.key + v;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) {
        accum += p[j] * values[j * width];
    }
    out[id] = accum;
}

// dp[index, t, j] = sum_v d_out[t, v] value[j, v], j <= t.
kernel void read_dp(
    device const float* d_out [[buffer(0)]],
    device const float* kv [[buffer(1)]],
    device float* dp [[buffer(2)]],
    constant ReadDims& d [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (id >= d.batch * d.heads * time * time) return;
    uint j = id % time;
    uint row = id / time;
    uint t = row % time;
    if (j > t) return;
    uint index = row / time;
    uint width = d.key + d.value;
    device const float* g = d_out + row * d.value;
    device const float* v = kv + (index * time + j) * width + d.key;
    float accum = 0.0f;
    for (uint c = 0; c < d.value; ++c) {
        accum += g[c] * v[c];
    }
    dp[id] = accum;
}

// Per row: the softmax backward. Overwrites dp with ds = p (dp - <p, dp>),
// writes the inner-product gradients g, the NoRead logit gradient (into
// d_aux), and for Lorentz the query self coefficient and the row's beta and
// offset partials.
kernel void read_row_grad(
    device const float* probabilities [[buffer(0)]],
    device float* dp [[buffer(1)]],
    device float* inner_grad [[buffer(2)]],
    device const float* excess [[buffer(3)]],
    device const float* null_probability [[buffer(4)]],
    device const float* aux [[buffer(5)]],
    device const float* query_lift [[buffer(6)]],
    device const float* key_lift [[buffer(7)]],
    device float* query_self [[buffer(8)]],
    device float* row_beta [[buffer(9)]],
    device float* row_offset [[buffer(10)]],
    device float* d_aux [[buffer(11)]],
    constant ReadDims& d [[buffer(12)]],
    uint row [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    uint index = row / time;
    uint head = index % d.heads;
    device const float* p = probabilities + row * time;
    device float* g = dp + row * time;
    device float* ig = inner_grad + row * time;
    float row_dot = 0.0f;
    for (uint j = 0; j <= t; ++j) {
        row_dot += p[j] * g[j];
    }
    if (d.null_on != 0) {
        d_aux[row] = -null_probability[row] * row_dot;
    }
    float scale = rsqrt(float(d.key));
    float beta = 0.0f;
    float offset = 0.0f;
    if (d.lorentz != 0) {
        uint beta_offset = read_beta_offset(d);
        beta = aux[beta_offset + head];
        offset = aux[beta_offset + d.heads + head];
    }
    float self_q = 0.0f;
    float d_beta = 0.0f;
    float d_offset = 0.0f;
    float lq = d.lorentz != 0 ? query_lift[row] : 1.0f;
    for (uint j = 0; j <= t; ++j) {
        float ds = p[j] * (g[j] - row_dot);
        g[j] = ds;
        if (d.lorentz != 0) {
            float e = excess[row * time + j];
            d_beta -= ds * (read_distance(e) - offset);
            d_offset += ds * beta;
            if (e > 1e-7f) {
                float de = -beta * ds / precise::sqrt(e * (e + 2.0f));
                self_q += de * key_lift[index * time + j] / lq;
                ig[j] = -de;
            } else {
                ig[j] = 0.0f;
            }
        } else {
            ig[j] = ds * scale;
        }
    }
    if (d.lorentz != 0) {
        query_self[row] = self_q;
        row_beta[row] = d_beta;
        row_offset[row] = d_offset;
    }
}

// Lorentz key self coefficient: sum_{t >= j} de[t, j] lift_q[t] / lift_k[j],
// with de = -inner_grad.
kernel void read_key_self(
    device const float* inner_grad [[buffer(0)]],
    device const float* query_lift [[buffer(1)]],
    device const float* key_lift [[buffer(2)]],
    device float* key_self [[buffer(3)]],
    constant ReadDims& d [[buffer(4)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (id >= d.batch * d.heads * time) return;
    uint j = id % time;
    uint index = id / time;
    float sum = 0.0f;
    for (uint t = j; t < time; ++t) {
        sum -= inner_grad[(index * time + t) * time + j] * query_lift[index * time + t];
    }
    key_self[id] = sum / key_lift[id];
}

// dq[index, t, c] = sum_{j <= t} g[t, j] key[j, c] (+ Lorentz self term).
kernel void read_dq(
    device const float* inner_grad [[buffer(0)]],
    device const float* query [[buffer(1)]],
    device const float* kv [[buffer(2)]],
    device const float* query_self [[buffer(3)]],
    device float* dq [[buffer(4)]],
    constant ReadDims& d [[buffer(5)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (id >= d.batch * d.heads * time * d.key) return;
    uint c = id % d.key;
    uint row = id / d.key;
    uint t = row % time;
    uint index = row / time;
    uint width = d.key + d.value;
    device const float* g = inner_grad + row * time;
    device const float* keys = kv + index * time * width + c;
    float accum = 0.0f;
    for (uint j = 0; j <= t; ++j) {
        accum += g[j] * keys[j * width];
    }
    if (d.lorentz != 0) {
        accum += query_self[row] * query[id];
    }
    dq[id] = accum;
}

// dkv[index, j, c]: keys sum_{t >= j} g[t, j] query[t, c] (+ Lorentz self
// term); values sum_{t >= j} p[t, j] d_out[t, c - key].
kernel void read_dkv(
    device const float* inner_grad [[buffer(0)]],
    device const float* probabilities [[buffer(1)]],
    device const float* query [[buffer(2)]],
    device const float* kv [[buffer(3)]],
    device const float* d_out [[buffer(4)]],
    device const float* key_self [[buffer(5)]],
    device float* dkv [[buffer(6)]],
    constant ReadDims& d [[buffer(7)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    uint width = d.key + d.value;
    if (id >= d.batch * d.heads * time * width) return;
    uint c = id % width;
    uint row = id / width;
    uint j = row % time;
    uint index = row / time;
    float accum = 0.0f;
    if (c < d.key) {
        for (uint t = j; t < time; ++t) {
            accum += inner_grad[(index * time + t) * time + j] * query[(index * time + t) * d.key + c];
        }
        if (d.lorentz != 0) {
            accum += key_self[row] * kv[id];
        }
    } else {
        uint v = c - d.key;
        for (uint t = j; t < time; ++t) {
            accum += probabilities[(index * time + t) * time + j] * d_out[(index * time + t) * d.value + v];
        }
    }
    dkv[id] = accum;
}

// Age-table gradient per (head, distance): sum over windows and positions
// of ds[t, t - distance].
kernel void read_dage(
    device const float* ds [[buffer(0)]],
    device float* d_aux [[buffer(1)]],
    constant ReadDims& d [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    if (id >= d.heads * time) return;
    uint distance = id % time;
    uint head = id / time;
    float sum = 0.0f;
    for (uint b = 0; b < d.batch; ++b) {
        uint index = b * d.heads + head;
        for (uint t = distance; t < time; ++t) {
            sum += ds[(index * time + t) * time + (t - distance)];
        }
    }
    d_aux[read_age_offset(d) + id] = sum;
}

// Lorentz beta and offset gradients per head from the rows' partials.
kernel void read_dbeta(
    device const float* row_beta [[buffer(0)]],
    device const float* row_offset [[buffer(1)]],
    device float* d_aux [[buffer(2)]],
    constant ReadDims& d [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= 2 * d.heads) return;
    uint head = id % d.heads;
    device const float* source = id < d.heads ? row_beta : row_offset;
    float sum = 0.0f;
    for (uint b = 0; b < d.batch; ++b) {
        uint index = b * d.heads + head;
        for (uint t = 0; t < d.time; ++t) {
            sum += source[index * d.time + t];
        }
    }
    d_aux[read_beta_offset(d) + id] = sum;
}

// Tiled causal inner products of one block: out[index, t, j] for j <= t of
// sum_c a[index * time + t][c] b[index * time + j][c] over `length`
// columns. geom = [a_stride, a_offset, b_stride, b_offset, length, mode,
// write_excess]. Mode 0 stores the raw product; mode 1 stores the read score
// (Dot or Lorentz, with age) and, with write_excess, the Lorentz excess.
// Threadgroups are 16 x 16 over (j tile, t tile, index); tiles wholly above
// the diagonal exit at once.
kernel void read_tile_inner(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device const float* aux [[buffer(2)]],
    device const float* query_lift [[buffer(3)]],
    device const float* key_lift [[buffer(4)]],
    device float* out [[buffer(5)]],
    device float* excess [[buffer(6)]],
    constant ReadDims& d [[buffer(7)]],
    constant uint* geom [[buffer(8)]],
    uint3 group [[threadgroup_position_in_grid]],
    uint3 local [[thread_position_in_threadgroup]]
) {
    uint jt = group.x;
    uint tt = group.y;
    if (jt > tt) return;
    uint index = group.z;
    uint time = d.time;
    uint a_stride = geom[0];
    uint a_offset = geom[1];
    uint b_stride = geom[2];
    uint b_offset = geom[3];
    uint length = geom[4];
    threadgroup float tile_a[16][17];
    threadgroup float tile_b[16][17];
    uint t = tt * 16 + local.y;
    uint j = jt * 16 + local.x;
    uint b_row = jt * 16 + local.y;
    float acc = 0.0f;
    for (uint c0 = 0; c0 < length; c0 += 16) {
        uint c = c0 + local.x;
        tile_a[local.y][local.x] = (t < time && c < length)
            ? a[(index * time + t) * a_stride + a_offset + c] : 0.0f;
        tile_b[local.y][local.x] = (b_row < time && c < length)
            ? b[(index * time + b_row) * b_stride + b_offset + c] : 0.0f;
        threadgroup_barrier(mem_flags::mem_threadgroup);
        for (uint k = 0; k < 16; ++k) {
            acc += tile_a[local.y][k] * tile_b[local.x][k];
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }
    if (t >= time || j > t) return;
    uint slot = (index * time + t) * time + j;
    if (geom[5] == 0) {
        out[slot] = acc;
        return;
    }
    uint head = index % d.heads;
    float age = d.age_on != 0 ? aux[read_age_offset(d) + head * time + (t - j)] : 0.0f;
    float score;
    if (d.lorentz != 0) {
        float e = query_lift[index * time + t] * key_lift[index * time + j] - acc - 1.0f;
        if (geom[6] != 0) {
            excess[slot] = e;
        }
        uint beta_offset = read_beta_offset(d);
        float beta = aux[beta_offset + head];
        float offset = aux[beta_offset + d.heads + head];
        score = -beta * (read_distance(e) - offset) + age;
    } else {
        score = acc * rsqrt(float(d.key)) + age;
    }
    out[slot] = score;
}

// AdamW step in place; c = [scale, beta1, rest1, beta2, rest2, correct1,
// correct2, epsilon, keep, lr], as the CPU `adam_step`.
kernel void adam_update(
    device float* p [[buffer(0)]],
    device float* m [[buffer(1)]],
    device float* v [[buffer(2)]],
    device const float* g [[buffer(3)]],
    constant float* c [[buffer(4)]],
    constant uint& n [[buffer(5)]],
    uint i [[thread_position_in_grid]]
) {
    if (i >= n) return;
    float grad = g[i] * c[0];
    float first = m[i] * c[1] + grad * c[2];
    float second = v[i] * c[3] + (grad * grad) * c[4];
    m[i] = first;
    v[i] = second;
    float step = (first * c[5]) / (precise::sqrt(second * c[6]) + c[7]);
    p[i] = p[i] * c[8] - step * c[9];
}

// read_softmax with one SIMD group (32 threads) per row.
kernel void read_softmax_simd(
    device float* scores [[buffer(0)]],
    device const float* aux [[buffer(1)]],
    device float* null_probability [[buffer(2)]],
    constant ReadDims& d [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    uint row = id / 32;
    uint lane = id % 32;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    device float* s = scores + row * time;
    float null_score = d.null_on != 0 ? aux[row] : -INFINITY;
    float maximum = null_score;
    for (uint j = lane; j <= t; j += 32) {
        maximum = max(maximum, s[j]);
    }
    maximum = simd_max(maximum);
    float total = 0.0f;
    for (uint j = lane; j <= t; j += 32) {
        float w = precise::exp(s[j] - maximum);
        s[j] = w;
        total += w;
    }
    total = simd_sum(total);
    float null_weight = d.null_on != 0 ? precise::exp(null_score - maximum) : 0.0f;
    total += null_weight;
    float inverse = 1.0f / total;
    for (uint j = lane; j <= t; j += 32) {
        s[j] *= inverse;
    }
    if (lane == 0) {
        null_probability[row] = null_weight * inverse;
    }
}

// read_row_grad with one SIMD group (32 threads) per row.
kernel void read_row_grad_simd(
    device const float* probabilities [[buffer(0)]],
    device float* dp [[buffer(1)]],
    device float* inner_grad [[buffer(2)]],
    device const float* excess [[buffer(3)]],
    device const float* null_probability [[buffer(4)]],
    device const float* aux [[buffer(5)]],
    device const float* query_lift [[buffer(6)]],
    device const float* key_lift [[buffer(7)]],
    device float* query_self [[buffer(8)]],
    device float* row_beta [[buffer(9)]],
    device float* row_offset [[buffer(10)]],
    device float* d_aux [[buffer(11)]],
    constant ReadDims& d [[buffer(12)]],
    uint id [[thread_position_in_grid]]
) {
    uint time = d.time;
    uint row = id / 32;
    uint lane = id % 32;
    if (row >= d.batch * d.heads * time) return;
    uint t = row % time;
    uint index = row / time;
    uint head = index % d.heads;
    device const float* p = probabilities + row * time;
    device float* g = dp + row * time;
    device float* ig = inner_grad + row * time;
    float row_dot = 0.0f;
    for (uint j = lane; j <= t; j += 32) {
        row_dot += p[j] * g[j];
    }
    row_dot = simd_sum(row_dot);
    if (d.null_on != 0 && lane == 0) {
        d_aux[row] = -null_probability[row] * row_dot;
    }
    float scale = rsqrt(float(d.key));
    float beta = 0.0f;
    float offset = 0.0f;
    if (d.lorentz != 0) {
        uint beta_offset = read_beta_offset(d);
        beta = aux[beta_offset + head];
        offset = aux[beta_offset + d.heads + head];
    }
    float self_q = 0.0f;
    float d_beta = 0.0f;
    float d_offset = 0.0f;
    float lq = d.lorentz != 0 ? query_lift[row] : 1.0f;
    for (uint j = lane; j <= t; j += 32) {
        float ds = p[j] * (g[j] - row_dot);
        g[j] = ds;
        if (d.lorentz != 0) {
            float e = excess[row * time + j];
            d_beta -= ds * (read_distance(e) - offset);
            d_offset += ds * beta;
            if (e > 1e-7f) {
                float de = -beta * ds / precise::sqrt(e * (e + 2.0f));
                self_q += de * key_lift[index * time + j] / lq;
                ig[j] = -de;
            } else {
                ig[j] = 0.0f;
            }
        } else {
            ig[j] = ds * scale;
        }
    }
    if (d.lorentz != 0) {
        self_q = simd_sum(self_q);
        d_beta = simd_sum(d_beta);
        d_offset = simd_sum(d_offset);
        if (lane == 0) {
            query_self[row] = self_q;
            row_beta[row] = d_beta;
            row_offset[row] = d_offset;
        }
    }
}
"#;

    // -----------------------------------------------------------------------
    // Pipeline Cache
    // -----------------------------------------------------------------------

    pub struct MetalPipelineCache {
        pipelines: Mutex<HashMap<String, ComputePipeline>>,
    }

    impl MetalPipelineCache {
        pub fn new() -> Self {
            Self {
                pipelines: Mutex::new(HashMap::new()),
            }
        }

        pub fn get_or_compile(
            &self,
            device: &MetalDevice,
            kernel_name: &str,
        ) -> Result<ComputePipeline> {
            let mut map = self
                .pipelines
                .lock()
                .map_err(|e| Error::Msg(format!("Lock error: {e}")))?;
            if let Some(pl) = map.get(kernel_name) {
                return Ok(pl.clone());
            }
            let raw_device = device.metal_device();
            let lib = raw_device
                .new_library_with_source(METAL_STACK_SHADERS, None)
                .map_err(|e| Error::Msg(format!("Metal shader compilation error: {e}")))?;
            let func = lib
                .get_function(kernel_name, None)
                .map_err(|e| Error::Msg(format!("Kernel {kernel_name} not found: {e}")))?;
            let pl = raw_device
                .new_compute_pipeline_state_with_function(&func)
                .map_err(|e| Error::Msg(format!("Pipeline creation error: {e}")))?;
            map.insert(kernel_name.to_string(), pl.clone());
            Ok(pl)
        }
    }

    static CACHE: OnceLock<MetalPipelineCache> = OnceLock::new();

    pub fn get_cache() -> &'static MetalPipelineCache {
        CACHE.get_or_init(MetalPipelineCache::new)
    }

    // -----------------------------------------------------------------------
    // Dispatch Helpers
    // -----------------------------------------------------------------------

    pub fn call_straight_through(
        device: &MetalDevice,
        src: &Buffer,
        dst: &Buffer,
        length: usize,
    ) -> Result<()> {
        if length == 0 {
            candle_core::bail!("call_straight_through requires non-empty buffer (length > 0)");
        }
        let pipeline = get_cache().get_or_compile(device, "straight_through_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total = length as u32;
        set_param(&encoder, 0, src);
        set_param(&encoder, 1, dst);
        set_param(&encoder, 2, total);
        let (grid, group) = linear_split(&pipeline, length);
        encoder.use_resource(src, MTLResourceUsage::Read);
        encoder.use_resource(dst, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_swiglu_fwd(
        device: &MetalDevice,
        gate: &Buffer,
        up: &Buffer,
        out: &Buffer,
        length: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "swiglu_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total = length as u32;
        set_param(&encoder, 0, gate);
        set_param(&encoder, 1, up);
        set_param(&encoder, 2, out);
        set_param(&encoder, 3, total);
        let (grid, group) = linear_split(&pipeline, length);
        encoder.use_resource(gate, MTLResourceUsage::Read);
        encoder.use_resource(up, MTLResourceUsage::Read);
        encoder.use_resource(out, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_swiglu_bwd(
        device: &MetalDevice,
        gate: &Buffer,
        up: &Buffer,
        grad: &Buffer,
        d_gate: &Buffer,
        d_up: &Buffer,
        length: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "swiglu_bwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total = length as u32;
        set_param(&encoder, 0, gate);
        set_param(&encoder, 1, up);
        set_param(&encoder, 2, grad);
        set_param(&encoder, 3, d_gate);
        set_param(&encoder, 4, d_up);
        set_param(&encoder, 5, total);
        let (grid, group) = linear_split(&pipeline, length);
        encoder.use_resource(gate, MTLResourceUsage::Read);
        encoder.use_resource(up, MTLResourceUsage::Read);
        encoder.use_resource(grad, MTLResourceUsage::Read);
        encoder.use_resource(d_gate, MTLResourceUsage::Write);
        encoder.use_resource(d_up, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_rms_norm_fwd(
        device: &MetalDevice,
        x: &Buffer,
        w: &Buffer,
        out: &Buffer,
        rows: usize,
        width: usize,
        eps: f32,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "rms_norm_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        set_param(&encoder, 0, x);
        set_param(&encoder, 1, w);
        set_param(&encoder, 2, out);
        set_param(&encoder, 3, width as u32);
        set_param(&encoder, 4, eps);
        let grid = MTLSize {
            width: rows,
            height: 1,
            depth: 1,
        };
        let group_threads = 64.min(width);
        let group = MTLSize {
            width: group_threads,
            height: 1,
            depth: 1,
        };
        encoder.use_resource(x, MTLResourceUsage::Read);
        encoder.use_resource(w, MTLResourceUsage::Read);
        encoder.use_resource(out, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_rms_norm_bwd(
        device: &MetalDevice,
        x: &Buffer,
        w: &Buffer,
        grad: &Buffer,
        dx: &Buffer,
        dw: &Buffer,
        rows: usize,
        width: usize,
        eps: f32,
    ) -> Result<()> {
        let pipeline_dx = get_cache().get_or_compile(device, "rms_norm_bwd_dx")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline_dx);
        set_param(&encoder, 0, x);
        set_param(&encoder, 1, w);
        set_param(&encoder, 2, grad);
        set_param(&encoder, 3, dx);
        set_param(&encoder, 4, width as u32);
        set_param(&encoder, 5, eps);
        let grid = MTLSize {
            width: rows,
            height: 1,
            depth: 1,
        };
        let group_threads = 64.min(width);
        let group = MTLSize {
            width: group_threads,
            height: 1,
            depth: 1,
        };
        encoder.use_resource(x, MTLResourceUsage::Read);
        encoder.use_resource(w, MTLResourceUsage::Read);
        encoder.use_resource(grad, MTLResourceUsage::Read);
        encoder.use_resource(dx, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);

        let pipeline_dw = get_cache().get_or_compile(device, "rms_norm_bwd_dw")?;
        encoder.set_compute_pipeline_state(&pipeline_dw);
        set_param(&encoder, 0, x);
        set_param(&encoder, 1, grad);
        set_param(&encoder, 2, dw);
        set_param(&encoder, 3, rows as u32);
        set_param(&encoder, 4, width as u32);
        set_param(&encoder, 5, eps);
        let (grid_dw, group_dw) = linear_split(&pipeline_dw, width);
        encoder.use_resource(dw, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid_dw, group_dw);

        Ok(())
    }

    pub fn call_quaternion_scan_fwd(
        device: &MetalDevice,
        transition: &Buffer,
        drive: &Buffer,
        out: &Buffer,
        batch: usize,
        time: usize,
        lanes: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "quaternion_scan_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total_seqs = (batch * lanes) as u32;
        set_param(&encoder, 0, transition);
        set_param(&encoder, 1, drive);
        set_param(&encoder, 2, out);
        set_param(&encoder, 3, time as u32);
        set_param(&encoder, 4, lanes as u32);
        set_param(&encoder, 5, total_seqs);
        let (grid, group) = linear_split(&pipeline, batch * lanes);
        encoder.use_resource(transition, MTLResourceUsage::Read);
        encoder.use_resource(drive, MTLResourceUsage::Read);
        encoder.use_resource(out, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_quaternion_scan_bwd(
        device: &MetalDevice,
        transition: &Buffer,
        state: &Buffer,
        grad: &Buffer,
        dq: &Buffer,
        db: &Buffer,
        batch: usize,
        time: usize,
        lanes: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "quaternion_scan_bwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total_seqs = (batch * lanes) as u32;
        set_param(&encoder, 0, transition);
        set_param(&encoder, 1, state);
        set_param(&encoder, 2, grad);
        set_param(&encoder, 3, dq);
        set_param(&encoder, 4, db);
        set_param(&encoder, 5, time as u32);
        set_param(&encoder, 6, lanes as u32);
        set_param(&encoder, 7, total_seqs);
        let (grid, group) = linear_split(&pipeline, batch * lanes);
        encoder.use_resource(transition, MTLResourceUsage::Read);
        encoder.use_resource(state, MTLResourceUsage::Read);
        encoder.use_resource(grad, MTLResourceUsage::Read);
        encoder.use_resource(dq, MTLResourceUsage::Write);
        encoder.use_resource(db, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_cross_entropy_fwd(
        device: &MetalDevice,
        logits: &Buffer,
        targets: &Buffer,
        loss_per_row: &Buffer,
        rows: usize,
        vocab: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "cross_entropy_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        set_param(&encoder, 0, logits);
        set_param(&encoder, 1, targets);
        set_param(&encoder, 2, loss_per_row);
        set_param(&encoder, 3, vocab as u32);
        let grid = MTLSize {
            width: rows,
            height: 1,
            depth: 1,
        };
        let group = MTLSize {
            width: 128.min(vocab),
            height: 1,
            depth: 1,
        };
        encoder.use_resource(logits, MTLResourceUsage::Read);
        encoder.use_resource(targets, MTLResourceUsage::Read);
        encoder.use_resource(loss_per_row, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_cross_entropy_bwd(
        device: &MetalDevice,
        logits: &Buffer,
        targets: &Buffer,
        grad: &Buffer,
        rows: usize,
        vocab: usize,
        scale: f32,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "cross_entropy_bwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        set_param(&encoder, 0, logits);
        set_param(&encoder, 1, targets);
        set_param(&encoder, 2, grad);
        set_param(&encoder, 3, vocab as u32);
        set_param(&encoder, 4, scale);
        let grid = MTLSize {
            width: rows,
            height: 1,
            depth: 1,
        };
        let group = MTLSize {
            width: 128.min(vocab),
            height: 1,
            depth: 1,
        };
        encoder.use_resource(logits, MTLResourceUsage::Read);
        encoder.use_resource(targets, MTLResourceUsage::Read);
        encoder.use_resource(grad, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_recurrence_core_fwd(
        device: &MetalDevice,
        branches: &Buffer,
        gates: &Buffer,
        params: &Buffer,
        log_a: &Buffer,
        state_out: &Buffer,
        drive_out: &Buffer,
        out: &Buffer,
        batch: usize,
        time: usize,
        width: usize,
        rotation: bool,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "recurrence_core_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let lanes = width / 4;
        let gate_width = lanes + if rotation { width } else { 0 };
        let total_lanes = (batch * lanes) as u32;

        set_param(&encoder, 0, branches);
        set_param(&encoder, 1, gates);
        set_param(&encoder, 2, params);
        set_param(&encoder, 3, log_a);
        set_param(&encoder, 4, state_out);
        set_param(&encoder, 5, drive_out);
        set_param(&encoder, 6, out);
        set_param(&encoder, 7, time as u32);
        set_param(&encoder, 8, width as u32);
        set_param(&encoder, 9, lanes as u32);
        set_param(&encoder, 10, gate_width as u32);
        set_param(&encoder, 11, if rotation { 1u32 } else { 0u32 });
        set_param(&encoder, 12, total_lanes);

        let (grid, group) = linear_split(&pipeline, batch * lanes);
        encoder.use_resource(branches, MTLResourceUsage::Read);
        encoder.use_resource(gates, MTLResourceUsage::Read);
        encoder.use_resource(params, MTLResourceUsage::Read);
        encoder.use_resource(log_a, MTLResourceUsage::Read);
        encoder.use_resource(state_out, MTLResourceUsage::Write);
        encoder.use_resource(drive_out, MTLResourceUsage::Write);
        encoder.use_resource(out, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    pub fn call_fused_read_fwd(
        device: &MetalDevice,
        query: &Buffer,
        kv: &Buffer,
        aux: &Buffer,
        out: &Buffer,
        batch: usize,
        heads: usize,
        time: usize,
        key_dim: usize,
        val_dim: usize,
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, "fused_read_fwd")?;
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        let total_heads = (batch * heads) as u32;
        let kv_width = key_dim + val_dim;
        let scale = 1.0f32 / (key_dim as f32).sqrt();

        set_param(&encoder, 0, query);
        set_param(&encoder, 1, kv);
        set_param(&encoder, 2, aux);
        set_param(&encoder, 3, out);
        set_param(&encoder, 4, time as u32);
        set_param(&encoder, 5, key_dim as u32);
        set_param(&encoder, 6, val_dim as u32);
        set_param(&encoder, 7, kv_width as u32);
        set_param(&encoder, 8, scale);
        set_param(&encoder, 9, total_heads);

        let (grid, group) = linear_split(&pipeline, batch * heads);
        encoder.use_resource(query, MTLResourceUsage::Read);
        encoder.use_resource(kv, MTLResourceUsage::Read);
        encoder.use_resource(aux, MTLResourceUsage::Read);
        encoder.use_resource(out, MTLResourceUsage::Write);
        encoder.dispatch_thread_groups(grid, group);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Generic launcher for the recurrence backward and general read kernels.
    // -----------------------------------------------------------------------

    /// One bound argument of a kernel, in buffer-index order.
    pub enum Arg<'a> {
        /// A buffer the kernel only reads.
        In(&'a Buffer),
        /// A buffer the kernel only reads, bound from a byte offset.
        InAt(&'a Buffer, usize),
        /// A buffer the kernel writes (and possibly reads).
        Out(&'a Buffer),
        U32(u32),
        /// A small constant struct of `uint`s (e.g. `ReadDims`).
        Words(&'a [u32]),
    }

    /// Dispatches `name` over `threads` threads (a 1-D grid) with `args`
    /// bound at indices 0, 1, ... in order.
    pub fn launch(
        device: &MetalDevice,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        if threads == 0 {
            return Ok(());
        }
        dispatch(device, name, None, threads, args)
    }

    /// Dispatches `name` over `groups` threadgroups of `group` threads each.
    pub fn launch_groups(
        device: &MetalDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        if groups.0 * groups.1 * groups.2 == 0 {
            return Ok(());
        }
        dispatch(device, name, Some((groups, group)), 0, args)
    }

    fn dispatch(
        device: &MetalDevice,
        name: &str,
        shape: Option<((usize, usize, usize), (usize, usize, usize))>,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        let pipeline = get_cache().get_or_compile(device, name)?;
        if profiling() {
            device.wait_until_completed()?;
        }
        let encoder = device.command_encoder()?;
        encoder.set_compute_pipeline_state(&pipeline);
        for (index, arg) in args.iter().enumerate() {
            match arg {
                Arg::In(buffer) => {
                    set_param(&encoder, index, *buffer);
                    encoder.use_resource(*buffer, MTLResourceUsage::Read);
                }
                Arg::InAt(buffer, offset) => {
                    set_param(&encoder, index, (*buffer, *offset));
                    encoder.use_resource(*buffer, MTLResourceUsage::Read);
                }
                Arg::Out(buffer) => {
                    set_param(&encoder, index, *buffer);
                    encoder.use_resource(*buffer, MTLResourceUsage::Read | MTLResourceUsage::Write);
                }
                Arg::U32(value) => set_param(&encoder, index, *value),
                Arg::Words(words) => set_param(&encoder, index, *words),
            }
        }
        let (grid, group) = match shape {
            None => linear_split(&pipeline, threads),
            Some((groups, group)) => (
                MTLSize {
                    width: groups.0,
                    height: groups.1,
                    depth: groups.2,
                },
                MTLSize {
                    width: group.0,
                    height: group.1,
                    depth: group.2,
                },
            ),
        };
        encoder.dispatch_thread_groups(grid, group);
        if profiling() {
            drop(encoder);
            let start = std::time::Instant::now();
            device.wait_until_completed()?;
            eprintln!("metal-kernel {name} {} us", start.elapsed().as_micros());
        }
        Ok(())
    }

    /// `UOR_METAL_PROFILE=1` synchronizes after each launched kernel and
    /// reports its wall time on stderr (diagnostic only; it serializes the GPU).
    fn profiling() -> bool {
        static PROFILE: OnceLock<bool> = OnceLock::new();
        *PROFILE.get_or_init(|| std::env::var_os("UOR_METAL_PROFILE").is_some())
    }
}
