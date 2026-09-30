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
    float t = tanh(v);
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
}
