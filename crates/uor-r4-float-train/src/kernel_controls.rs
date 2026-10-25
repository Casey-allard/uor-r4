//! Finite CPU arithmetic controls for the 24 CUDA kernels in the `uor-r4-float-train` extract
//! (`22` in `src/cuda_stack.cu` + `2` in `src/spiralcore_fbs.cu`).
//!
//! Each function reproduces the exact per-element / per-row formula of its corresponding `__global__`
//! CUDA kernel so the module is qualified at finite CPU control level on the laptop without NVRTC or
//! host-fallback model execution (`CHARTER §3.4, §3.5, §3.7`).

const RMS_EPSILON: f64 = 1e-5;
const RATE_ENTRIES: usize = 16;
const SC_FBS_ABSENT: u32 = 0xffff_ffff;

/// Exact 16-entry keep table `sqrt(1 - lambda^2)` matching `keep_entry` in `src/cuda_stack.cu`.
const KEEP_TABLE: [f32; RATE_ENTRIES] = [
    0.866_025_4,
    0.968_245_86,
    0.992_156_74,
    0.998_044_97,
    0.999_511_6,
    0.999_877_9,
    0.999_969_5,
    0.999_992_4,
    0.999_998_1,
    0.999_999_5,
    0.999_999_9,
    0.999_999_94,
    1.0,
    1.0,
    1.0,
    1.0,
];

#[inline]
fn rate_entry(index: usize) -> f32 {
    let bits: u32 = (127u32 - (index as u32 + 1)) << 23;
    f32::from_bits(bits)
}

#[inline]
fn keep_entry(index: usize) -> f32 {
    KEEP_TABLE[index]
}

#[inline]
fn rate_index_of(score: f32) -> usize {
    let scaled = (score * (RATE_ENTRIES as f32)).max(0.0) as usize;
    scaled.min(RATE_ENTRIES - 1)
}

#[inline]
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

#[inline]
fn quat_conj(q: [f32; 4]) -> [f32; 4] {
    [q[0], -q[1], -q[2], -q[3]]
}

#[inline]
fn quat_add(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

#[inline]
fn recurrence_step(q: [f32; 4], h: [f32; 4], c: [f32; 4], keep: f32) -> [f32; 4] {
    let qh = quat_mul(q, h);
    [
        qh[0] + keep * c[0],
        qh[1] + keep * c[1],
        qh[2] + keep * c[2],
        qh[3] + keep * c[3],
    ]
}

/// Kernel 1 (`src/cuda_stack.cu:160`): `straight_through_fwd`.
pub fn straight_through_fwd_cpu(src: &[f32]) -> Vec<f32> {
    src.to_vec()
}

/// Kernel 2 (`src/cuda_stack.cu:172`): `hamilton_gate_fwd`.
pub fn hamilton_gate_fwd_cpu(gate: &[f32], up: &[f32]) -> Vec<f32> {
    assert_eq!(gate.len(), up.len());
    assert_eq!(gate.len() % 4, 0);
    let mut out = vec![0.0f32; gate.len()];
    for i in (0..gate.len()).step_by(4) {
        let (g0, g1, g2, g3) = (gate[i], gate[i + 1], gate[i + 2], gate[i + 3]);
        let (u0, u1, u2, u3) = (up[i], up[i + 1], up[i + 2], up[i + 3]);
        let r = (g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f32).sqrt();
        let (q0, q1, q2, q3) = (g0 / r, g1 / r, g2 / r, g3 / r);
        out[i] = q0 * u0 - q1 * u1 - q2 * u2 - q3 * u3;
        out[i + 1] = q0 * u1 + q1 * u0 + q2 * u3 - q3 * u2;
        out[i + 2] = q0 * u2 - q1 * u3 + q2 * u0 + q3 * u1;
        out[i + 3] = q0 * u3 + q1 * u2 - q2 * u1 + q3 * u0;
    }
    out
}

/// Kernel 3 (`src/cuda_stack.cu:192`): `hamilton_gate_bwd`.
pub fn hamilton_gate_bwd_cpu(gate: &[f32], up: &[f32], grad: &[f32]) -> (Vec<f32>, Vec<f32>) {
    assert_eq!(gate.len(), up.len());
    assert_eq!(gate.len(), grad.len());
    assert_eq!(gate.len() % 4, 0);
    let mut d_gate = vec![0.0f32; gate.len()];
    let mut d_up = vec![0.0f32; up.len()];
    for i in (0..gate.len()).step_by(4) {
        let (g0, g1, g2, g3) = (gate[i], gate[i + 1], gate[i + 2], gate[i + 3]);
        let (u0, u1, u2, u3) = (up[i], up[i + 1], up[i + 2], up[i + 3]);
        let (d0, d1, d2, d3) = (grad[i], grad[i + 1], grad[i + 2], grad[i + 3]);
        let r = (g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f32).sqrt();
        let (q0, q1, q2, q3) = (g0 / r, g1 / r, g2 / r, g3 / r);
        d_up[i] = q0 * d0 + q1 * d1 + q2 * d2 + q3 * d3;
        d_up[i + 1] = q0 * d1 - q1 * d0 - q2 * d3 + q3 * d2;
        d_up[i + 2] = q0 * d2 + q1 * d3 - q2 * d0 - q3 * d1;
        d_up[i + 3] = q0 * d3 - q1 * d2 + q2 * d1 - q3 * d0;
        let s0 = d0 * u0 + d1 * u1 + d2 * u2 + d3 * u3;
        let s1 = -d0 * u1 + d1 * u0 - d2 * u3 + d3 * u2;
        let s2 = -d0 * u2 + d1 * u3 + d2 * u0 - d3 * u1;
        let s3 = -d0 * u3 - d1 * u2 + d2 * u1 + d3 * u0;
        let dot = q0 * s0 + q1 * s1 + q2 * s2 + q3 * s3;
        d_gate[i] = (s0 - q0 * dot) / r;
        d_gate[i + 1] = (s1 - q1 * dot) / r;
        d_gate[i + 2] = (s2 - q2 * dot) / r;
        d_gate[i + 3] = (s3 - q3 * dot) / r;
    }
    (d_gate, d_up)
}

/// Kernel 4 (`src/cuda_stack.cu:259`): `rate_select_fwd`.
pub fn rate_select_fwd_cpu(score: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; score.len() * 2];
    for (id, &s) in score.iter().enumerate() {
        let index = rate_index_of(s);
        out[2 * id] = rate_entry(index);
        out[2 * id + 1] = keep_entry(index);
    }
    out
}

/// Kernel 5 (`src/cuda_stack.cu:269`): `rate_select_bwd`.
pub fn rate_select_bwd_cpu(score: &[f32], grad: &[f32]) -> Vec<f32> {
    assert_eq!(grad.len(), score.len() * 2);
    let mut d_score = vec![0.0f32; score.len()];
    for (id, &s) in score.iter().enumerate() {
        let index = rate_index_of(s);
        let d_lambda = grad[2 * id];
        let d_keep = grad[2 * id + 1];
        let mut credit = 0.0f32;
        let mut sides = 0.0f32;
        if index > 0 {
            credit -= d_lambda * (rate_entry(index - 1) - rate_entry(index))
                + d_keep * (keep_entry(index - 1) - keep_entry(index));
            sides += 1.0;
        }
        if index + 1 < RATE_ENTRIES {
            credit += d_lambda * (rate_entry(index + 1) - rate_entry(index))
                + d_keep * (keep_entry(index + 1) - keep_entry(index));
            sides += 1.0;
        }
        if sides > 0.0 {
            credit /= sides;
        }
        d_score[id] = credit * (RATE_ENTRIES as f32);
    }
    d_score
}

/// Kernel 6 (`src/cuda_stack.cu:297`): `rms_norm_fwd`.
pub fn rms_norm_fwd_cpu(x: &[f32], w: &[f32], rows: usize, width: usize) -> Vec<f32> {
    assert_eq!(x.len(), rows * width);
    assert_eq!(w.len(), width);
    let mut out = vec![0.0f32; x.len()];
    for r in 0..rows {
        let offset = r * width;
        let sum_sq: f64 = x[offset..offset + width]
            .iter()
            .map(|&v| {
                let d = v as f64;
                d * d
            })
            .sum();
        let inv_rms = (1.0 / (sum_sq / (width as f64) + RMS_EPSILON).sqrt()) as f32;
        for i in 0..width {
            out[offset + i] = x[offset + i] * inv_rms * w[i];
        }
    }
    out
}

/// Kernel 7 (`src/cuda_stack.cu:316`): `rms_norm_bwd_dx`.
pub fn rms_norm_bwd_dx_cpu(
    x: &[f32],
    w: &[f32],
    grad: &[f32],
    rows: usize,
    width: usize,
) -> (Vec<f32>, Vec<f64>) {
    assert_eq!(x.len(), rows * width);
    assert_eq!(grad.len(), rows * width);
    assert_eq!(w.len(), width);
    let mut dx = vec![0.0f32; x.len()];
    let mut row_r = vec![0.0f64; rows];
    for (row, r_slot) in row_r.iter_mut().enumerate().take(rows) {
        let offset = row * width;
        let sum_sq: f64 = x[offset..offset + width]
            .iter()
            .map(|&v| {
                let d = v as f64;
                d * d
            })
            .sum();
        let r = 1.0 / (sum_sq / (width as f64) + RMS_EPSILON).sqrt();
        let mut projection = 0.0f64;
        for i in 0..width {
            let xhat = (x[offset + i] as f64) * r;
            projection += (grad[offset + i] as f64) * (w[i] as f64) * xhat;
        }
        projection /= width as f64;
        for i in 0..width {
            let xhat = (x[offset + i] as f64) * r;
            dx[offset + i] =
                (r * ((grad[offset + i] as f64) * (w[i] as f64) - xhat * projection)) as f32;
        }
        *r_slot = r;
    }
    (dx, row_r)
}

/// Kernel 8 (`src/cuda_stack.cu:344`): `rms_norm_dw_partial`.
pub fn rms_norm_dw_partial_cpu(
    x: &[f32],
    grad: &[f32],
    row_r: &[f64],
    rows: usize,
    width: usize,
    chunk_rows: usize,
) -> Vec<f64> {
    let chunks = rows.div_ceil(chunk_rows);
    let mut partials = vec![0.0f64; chunks * width];
    for (id, partial_slot) in partials.iter_mut().enumerate().take(chunks * width) {
        let col = id % width;
        let chunk = id / width;
        let end = ((chunk + 1) * chunk_rows).min(rows);
        let mut sum = 0.0f64;
        for (r, &r_val) in row_r.iter().enumerate().take(end).skip(chunk * chunk_rows) {
            let at = r * width + col;
            sum += (grad[at] as f64) * ((x[at] as f64) * r_val);
        }
        *partial_slot = sum;
    }
    partials
}

/// Kernel 9 (`src/cuda_stack.cu:362`): `rms_norm_dw_reduce`.
pub fn rms_norm_dw_reduce_cpu(partials: &[f64], chunks: usize, width: usize) -> Vec<f32> {
    assert_eq!(partials.len(), chunks * width);
    let mut dw = vec![0.0f32; width];
    for col in 0..width {
        let mut sum = 0.0f64;
        for c in 0..chunks {
            sum += partials[c * width + col];
        }
        dw[col] = sum as f32;
    }
    dw
}

/// Kernel 10 (`src/cuda_stack.cu:377`): `quaternion_scan_fwd`.
pub fn quaternion_scan_fwd_cpu(
    transition: &[f32],
    drive: &[f32],
    batch: usize,
    time: usize,
    lanes: usize,
) -> Vec<f32> {
    let total = batch * time * lanes * 4;
    assert_eq!(transition.len(), total);
    assert_eq!(drive.len(), total);
    let mut state_out = vec![0.0f32; total];
    let total_seqs = batch * lanes;
    for seq_id in 0..total_seqs {
        let b = seq_id / lanes;
        let lane = seq_id % lanes;
        let mut held = [0.0f32; 4];
        for t in 0..time {
            let idx = ((b * time + t) * lanes + lane) * 4;
            let tr = [
                transition[idx],
                transition[idx + 1],
                transition[idx + 2],
                transition[idx + 3],
            ];
            let dr = [drive[idx], drive[idx + 1], drive[idx + 2], drive[idx + 3]];
            let moved = quat_mul(tr, held);
            held = quat_add(moved, dr);
            state_out[idx..idx + 4].copy_from_slice(&held);
        }
    }
    state_out
}

/// Kernel 11 (`src/cuda_stack.cu:394`): `quaternion_scan_bwd`.
pub fn quaternion_scan_bwd_cpu(
    transition: &[f32],
    state: &[f32],
    grad: &[f32],
    batch: usize,
    time: usize,
    lanes: usize,
) -> (Vec<f32>, Vec<f32>) {
    let total = batch * time * lanes * 4;
    assert_eq!(transition.len(), total);
    assert_eq!(state.len(), total);
    assert_eq!(grad.len(), total);
    let mut dq = vec![0.0f32; total];
    let mut db = vec![0.0f32; total];
    let total_seqs = batch * lanes;
    for seq_id in 0..total_seqs {
        let b = seq_id / lanes;
        let lane = seq_id % lanes;
        let mut carried = [0.0f32; 4];
        for t in (0..time).rev() {
            let idx = ((b * time + t) * lanes + lane) * 4;
            let mut total_g = [grad[idx], grad[idx + 1], grad[idx + 2], grad[idx + 3]];
            if t + 1 < time {
                let next_idx = ((b * time + t + 1) * lanes + lane) * 4;
                let tr_next = [
                    transition[next_idx],
                    transition[next_idx + 1],
                    transition[next_idx + 2],
                    transition[next_idx + 3],
                ];
                total_g = quat_add(total_g, quat_mul(quat_conj(tr_next), carried));
            }
            db[idx..idx + 4].copy_from_slice(&total_g);
            if t > 0 {
                let prev_idx = ((b * time + t - 1) * lanes + lane) * 4;
                let st_prev = [
                    state[prev_idx],
                    state[prev_idx + 1],
                    state[prev_idx + 2],
                    state[prev_idx + 3],
                ];
                let dq_t = quat_mul(total_g, quat_conj(st_prev));
                dq[idx..idx + 4].copy_from_slice(&dq_t);
            } else {
                dq[idx..idx + 4].copy_from_slice(&[0.0; 4]);
            }
            carried = total_g;
        }
    }
    (dq, db)
}

fn top_rivals(row: &[f32], target: usize, k: usize) -> Vec<(usize, f32)> {
    let mut candidates: Vec<(usize, f32)> = row
        .iter()
        .copied()
        .enumerate()
        .filter(|&(i, _)| i != target)
        .collect();
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    candidates.truncate(k);
    candidates
}

/// Kernel 12 (`src/cuda_stack.cu:508`): `margin_rank_rows`.
pub fn margin_rank_rows_cpu(
    logits: &[f32],
    targets: &[u32],
    vocab: usize,
    rivals: usize,
) -> Vec<f64> {
    let rows = targets.len();
    assert_eq!(logits.len(), rows * vocab);
    let take = rivals.min(vocab.saturating_sub(1));
    let mut losses = vec![0.0f64; rows];
    for (r, &target_u32) in targets.iter().enumerate() {
        let target = target_u32 as usize;
        let row = &logits[r * vocab..(r + 1) * vocab];
        let zt = row[target] as f64;
        let top = top_rivals(row, target, take);
        let mut total = 0.0f64;
        for (_, gap_val) in top {
            let gap = 1.0 + (gap_val as f64) - zt;
            if gap > 0.0 {
                total += gap;
            }
        }
        losses[r] = total / (take as f64);
    }
    losses
}

/// Kernel 13 (`src/cuda_stack.cu:514`): `margin_rank_grad`.
pub fn margin_rank_grad_cpu(
    logits: &[f32],
    targets: &[u32],
    scale: &[f64],
    vocab: usize,
    rivals: usize,
) -> Vec<f32> {
    let rows = targets.len();
    assert_eq!(logits.len(), rows * vocab);
    assert_eq!(scale.len(), rows);
    let take = rivals.min(vocab.saturating_sub(1));
    let mut grad = vec![0.0f32; logits.len()];
    for r in 0..rows {
        let target = targets[r] as usize;
        let offset = r * vocab;
        let row = &logits[offset..offset + vocab];
        let zt = row[target] as f64;
        let top = top_rivals(row, target, take);
        let mut violated = 0usize;
        for (idx, gap_val) in top {
            let gap = 1.0 + (gap_val as f64) - zt;
            if gap > 0.0 {
                grad[offset + idx] = (scale[r] / (take as f64)) as f32;
                violated += 1;
            }
        }
        grad[offset + target] = (-((violated as f64) * scale[r] / (take as f64))) as f32;
    }
    grad
}

/// Kernel 14 (`src/cuda_stack.cu:587`): `recurrence_scan_fwd`.
pub fn recurrence_scan_fwd_cpu(
    q: &[f32],
    drive: &[f32],
    keep: &[f32],
    batch: usize,
    time: usize,
    width: usize,
    lanes: usize,
) -> Vec<f32> {
    assert_eq!(width, lanes * 4);
    assert_eq!(q.len(), batch * time * width);
    assert_eq!(drive.len(), batch * time * width);
    assert_eq!(keep.len(), batch * time * lanes);
    let mut state_out = vec![0.0f32; q.len()];
    for b in 0..batch {
        for lane in 0..lanes {
            let ch = 4 * lane;
            let mut held = [0.0f32; 4];
            for t in 0..time {
                let row = b * time + t;
                let base = row * width + ch;
                let qi = [q[base], q[base + 1], q[base + 2], q[base + 3]];
                let di = [
                    drive[base],
                    drive[base + 1],
                    drive[base + 2],
                    drive[base + 3],
                ];
                let ki = keep[row * lanes + lane];
                held = recurrence_step(qi, held, di, ki);
                state_out[base..base + 4].copy_from_slice(&held);
            }
        }
    }
    state_out
}

/// Kernel 15 (`src/cuda_stack.cu:646`): `recurrence_tile_fold`.
#[allow(clippy::too_many_arguments)]
pub fn recurrence_tile_fold_cpu(
    q: &[f32],
    drive: &[f32],
    keep: &[f32],
    batch: usize,
    time: usize,
    width: usize,
    lanes: usize,
    ntiles: usize,
    tile: usize,
) -> (Vec<f32>, Vec<f32>) {
    let total = batch * ntiles * lanes;
    let mut tile_q = vec![0.0f32; total * 4];
    let mut tile_d = vec![0.0f32; total * 4];
    for id in 0..total {
        let lane = id % lanes;
        let tile_id = (id / lanes) % ntiles;
        let b = id / (lanes * ntiles);
        let ch = 4 * lane;
        let t0 = tile_id * tile;
        let t1 = (t0 + tile).min(time);
        let mut p = [1.0f32, 0.0, 0.0, 0.0];
        let mut d = [0.0f32; 4];
        for t in t0..t1 {
            let row = b * time + t;
            let base = row * width + ch;
            let qi = [q[base], q[base + 1], q[base + 2], q[base + 3]];
            let di = [
                drive[base],
                drive[base + 1],
                drive[base + 2],
                drive[base + 3],
            ];
            d = recurrence_step(qi, d, di, keep[row * lanes + lane]);
            p = quat_mul(qi, p);
        }
        tile_q[id * 4..id * 4 + 4].copy_from_slice(&p);
        tile_d[id * 4..id * 4 + 4].copy_from_slice(&d);
    }
    (tile_q, tile_d)
}

/// Kernel 16 (`src/cuda_stack.cu:674`): `recurrence_tile_carry`.
pub fn recurrence_tile_carry_cpu(
    tile_q: &[f32],
    tile_d: &[f32],
    batch: usize,
    lanes: usize,
    ntiles: usize,
) -> Vec<f32> {
    let mut entry = vec![0.0f32; tile_q.len()];
    let total_lanes = batch * lanes;
    for id in 0..total_lanes {
        let lane = id % lanes;
        let b = id / lanes;
        let mut held = [0.0f32; 4];
        for tile_id in 0..ntiles {
            let slot = ((b * ntiles + tile_id) * lanes + lane) * 4;
            let p = [
                tile_q[slot],
                tile_q[slot + 1],
                tile_q[slot + 2],
                tile_q[slot + 3],
            ];
            let d = [
                tile_d[slot],
                tile_d[slot + 1],
                tile_d[slot + 2],
                tile_d[slot + 3],
            ];
            entry[slot..slot + 4].copy_from_slice(&held);
            held = quat_add(quat_mul(p, held), d);
        }
    }
    entry
}

/// Kernel 17 (`src/cuda_stack.cu:693`): `recurrence_tile_expand`.
#[allow(clippy::too_many_arguments)]
pub fn recurrence_tile_expand_cpu(
    q: &[f32],
    drive: &[f32],
    keep: &[f32],
    entry: &[f32],
    batch: usize,
    time: usize,
    width: usize,
    lanes: usize,
    ntiles: usize,
    tile: usize,
) -> Vec<f32> {
    let mut state_out = vec![0.0f32; q.len()];
    let total = batch * ntiles * lanes;
    for id in 0..total {
        let lane = id % lanes;
        let tile_id = (id / lanes) % ntiles;
        let b = id / (lanes * ntiles);
        let ch = 4 * lane;
        let t0 = tile_id * tile;
        let t1 = (t0 + tile).min(time);
        let mut held = [
            entry[id * 4],
            entry[id * 4 + 1],
            entry[id * 4 + 2],
            entry[id * 4 + 3],
        ];
        for t in t0..t1 {
            let row = b * time + t;
            let base = row * width + ch;
            let qi = [q[base], q[base + 1], q[base + 2], q[base + 3]];
            let di = [
                drive[base],
                drive[base + 1],
                drive[base + 2],
                drive[base + 3],
            ];
            held = recurrence_step(qi, held, di, keep[row * lanes + lane]);
            state_out[base..base + 4].copy_from_slice(&held);
        }
    }
    state_out
}

/// Kernel 18 (`src/cuda_stack.cu:724`): `recurrence_bwd_params`.
#[allow(clippy::too_many_arguments)]
pub fn recurrence_bwd_params_cpu(
    branches: &[f32],
    params: &[f32],
    dd: &[f32],
    log_term: &[f32],
    batch: usize,
    time: usize,
    width: usize,
    lanes: usize,
) -> (Vec<f32>, Vec<f64>) {
    let total_channels = batch * width;
    let param_len = 5 * width + lanes;
    let two_w = 2 * width;
    let mut d_branches = vec![0.0f32; batch * time * two_w];
    let mut partials = vec![0.0f64; batch * param_len];
    for id in 0..total_channels {
        let b = id / width;
        let c = id % width;
        let lane = c / 4;
        let lane_owner = c.is_multiple_of(4);
        let tap0 = params[c];
        let tap1 = params[width + c];
        let tap2 = params[2 * width + c];
        let tap3 = params[3 * width + c];
        let mut d_bias = 0.0f64;
        let (mut d_tap0, mut d_tap1, mut d_tap2, mut d_tap3) = (0.0f64, 0.0, 0.0, 0.0);
        let mut d_log_a = 0.0f64;
        let (mut dd1, mut dd2, mut dd3) = (0.0f32, 0.0, 0.0);
        for t in (0..time).rev() {
            let row = b * time + t;
            let dd0 = dd[row * width + c];
            d_bias += dd0 as f64;
            d_tap0 += (dd0 * branches[row * two_w + c]) as f64;
            if t >= 1 {
                d_tap1 += (dd0 * branches[(row - 1) * two_w + c]) as f64;
            }
            if t >= 2 {
                d_tap2 += (dd0 * branches[(row - 2) * two_w + c]) as f64;
            }
            if t >= 3 {
                d_tap3 += (dd0 * branches[(row - 3) * two_w + c]) as f64;
            }
            let mut acc = 0.0f32;
            if t + 3 < time {
                acc += tap3 * dd3;
            }
            if t + 2 < time {
                acc += tap2 * dd2;
            }
            if t + 1 < time {
                acc += tap1 * dd1;
            }
            acc += tap0 * dd0;
            d_branches[row * two_w + c] = acc;
            if lane_owner {
                d_log_a += log_term[row * lanes + lane] as f64;
            }
            dd3 = dd2;
            dd2 = dd1;
            dd1 = dd0;
        }
        let p_base = b * param_len;
        partials[p_base + c] = d_tap0;
        partials[p_base + width + c] = d_tap1;
        partials[p_base + 2 * width + c] = d_tap2;
        partials[p_base + 3 * width + c] = d_tap3;
        partials[p_base + 4 * width + c] = d_bias;
        if lane_owner {
            partials[p_base + 5 * width + lane] = d_log_a;
        }
    }
    (d_branches, partials)
}

/// Kernel 19 (`src/cuda_stack.cu:778`): `poison_words`.
pub fn poison_words_cpu(words: usize) -> Vec<u32> {
    vec![0xFFFF_FFFFu32; words]
}

/// Kernel 20 (`src/cuda_stack.cu:787`): `adam_update`.
pub fn adam_update_cpu(p: &mut [f32], m: &mut [f32], v: &mut [f32], g: &[f32], c: &[f32; 10]) {
    let n = p.len();
    assert_eq!(m.len(), n);
    assert_eq!(v.len(), n);
    assert_eq!(g.len(), n);
    for i in 0..n {
        let grad = g[i] * c[0];
        let first = m[i] * c[1] + grad * c[2];
        let second = v[i] * c[3] + (grad * grad) * c[4];
        m[i] = first;
        v[i] = second;
        let step = (first * c[5]) / ((second * c[6]).sqrt() + c[7]);
        p[i] = p[i] * c[8] - step * c[9];
    }
}

/// Kernel 21 (`src/cuda_stack.cu:813`): `sq_lanes`.
pub fn sq_lanes_cpu(x: &[f32], width: usize, square: bool) -> Vec<f32> {
    let mut lanes_out = vec![0.0f32; width];
    for (lane, out_slot) in lanes_out.iter_mut().enumerate() {
        let mut acc = 0.0f32;
        let mut i = lane;
        while i < x.len() {
            let val = x[i];
            let term = if square { val * val } else { val };
            acc += term;
            i += width;
        }
        *out_slot = acc;
    }
    lanes_out
}

/// Kernel 22 (`src/cuda_stack.cu:845`): `sq_tree`.
pub fn sq_tree_cpu(lane_sums: &[f32], segments: &[u32]) -> Vec<f32> {
    assert_eq!(segments.len() % 2, 0);
    let blocks = segments.len() / 2;
    let mut out = vec![0.0f32; blocks];
    for b in 0..blocks {
        let offset = segments[2 * b] as usize;
        let width = segments[2 * b + 1] as usize;
        assert!(width.is_power_of_two());
        let mut shr = vec![0.0f32; width];
        shr.copy_from_slice(&lane_sums[offset..offset + width]);
        let mut s = width / 2;
        while s > 0 {
            for t in 0..s {
                shr[t] += shr[t + s];
            }
            s >>= 1;
        }
        out[b] = shr[0];
    }
    out
}

/// Host fixture for the SpiralCore FBS routing tables (`src/spiralcore_fbs.cu`).
#[derive(Clone, Debug)]
pub struct ScFbsHostTables {
    pub nodes: Vec<u32>,
    pub frontier: Vec<u32>,
    pub axes: Vec<u32>,
    pub compose: Vec<u32>,
    pub root_of_state: Vec<u32>,
    pub bfs_actions: Vec<u32>,
    pub branch_targets: Vec<u32>,
    pub roots: Vec<f32>,
    pub stages: usize,
    pub frontier_count: usize,
    pub identity: u32,
}

impl ScFbsHostTables {
    /// Two-leaf 1-stage routing fixture exercising exact signed action and secant query pullback.
    pub fn fixture_two_leaf_tree() -> Self {
        // Two frontier leaves: node 0 -> atom 0, node 1 -> atom 1
        let nodes = vec![
            SC_FBS_ABSENT,
            SC_FBS_ABSENT,
            SC_FBS_ABSENT,
            0, // node 0: leaf atom 0
            SC_FBS_ABSENT,
            SC_FBS_ABSENT,
            SC_FBS_ABSENT,
            1, // node 1: leaf atom 1
        ];
        let frontier = vec![0u32, 1u32];
        // State 0 = identity axis map, State 1 = negated/permuted axis map
        let mut axes = vec![0u32; 64 * 8];
        for s in 0..64u32 {
            for a in 0..8u32 {
                axes[(s * 8 + a) as usize] = if s == 1 { (a ^ 1) | 8 } else { a };
            }
        }
        let mut compose = vec![0u32; 64 * 64];
        for action in 0..64u32 {
            for state in 0..64u32 {
                compose[(action * 64 + state) as usize] = (action ^ state) & 63;
            }
        }
        let root_of_state: Vec<u32> = (0..64u32).map(|s| s & 15).collect();
        let mut bfs_actions = vec![0u32; 16 * 16];
        for from in 0..16u32 {
            for target in 0..16u32 {
                bfs_actions[(from * 16 + target) as usize] = target;
            }
        }
        let branch_targets: Vec<u32> = (0..16u32)
            .flat_map(|r| [(r + 1) & 15, (r + 2) & 15, (r + 3) & 15])
            .collect();
        let mut roots = vec![0.0f32; 16 * 8];
        // Root 0 along +e0, Root 1 along -e0
        roots[0] = 1.0;
        roots[8] = -1.0;
        for r in 2..16 {
            roots[r * 8 + (r % 8)] = 1.0;
        }
        Self {
            nodes,
            frontier,
            axes,
            compose,
            root_of_state,
            bfs_actions,
            branch_targets,
            roots,
            stages: 1,
            frontier_count: 2,
            identity: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct ScFbsRoute {
    atom: u32,
    state: u32,
    count: usize,
    before: [u32; 8],
    choice: [u32; 8],
    goal: [u32; 8],
}

fn sc_fbs_distance(q: &[f32], root: &[f32]) -> f32 {
    q[0..8]
        .iter()
        .zip(&root[0..8])
        .map(|(a, b)| {
            let d = a - b;
            d * d
        })
        .sum()
}

fn sc_fbs_advance(t: &ScFbsHostTables, state: u32, target: u32) -> u32 {
    let from = t.root_of_state[state as usize];
    let action = t.bfs_actions[(from * 16 + target) as usize];
    t.compose[(action * 64 + state) as usize]
}

fn sc_fbs_route(
    t: &ScFbsHostTables,
    q: &[f32],
    forced_stage: u32,
    forced_choice: u32,
) -> ScFbsRoute {
    let mut r = ScFbsRoute {
        atom: SC_FBS_ABSENT,
        state: t.identity,
        count: 0,
        before: [0; 8],
        choice: [0; 8],
        goal: [0; 8],
    };
    let mut node = SC_FBS_ABSENT;
    for stage in 0..t.stages {
        let root = t.root_of_state[r.state as usize];
        let count = if stage == 0 { t.frontier_count } else { 3 };
        let mut selected = 0u32;
        let mut best = f32::MAX;
        for candidate in 0..(count as u32) {
            let goal = if stage == 0 {
                candidate
            } else {
                t.branch_targets[(root * 3 + candidate) as usize]
            };
            let dist = sc_fbs_distance(
                &q[stage * 8..(stage + 1) * 8],
                &t.roots[(goal as usize) * 8..(goal as usize + 1) * 8],
            );
            if !dist.is_finite() {
                return r;
            }
            if dist < best {
                best = dist;
                selected = candidate;
            }
        }
        if (stage as u32) == forced_stage {
            selected = forced_choice;
        }
        let goal = if stage == 0 {
            selected
        } else {
            t.branch_targets[(root * 3 + selected) as usize]
        };
        r.before[stage] = r.state;
        r.choice[stage] = selected;
        r.goal[stage] = goal;
        r.count = stage + 1;
        r.state = sc_fbs_advance(t, r.state, goal);
        node = if stage == 0 {
            t.frontier[selected as usize]
        } else {
            t.nodes[(node * 4 + selected) as usize]
        };
        let atom = t.nodes[(node * 4 + 3) as usize];
        if atom != SC_FBS_ABSENT {
            r.atom = atom;
            return r;
        }
    }
    r
}

fn sc_fbs_coordinate(t: &ScFbsHostTables, x: &[f32], atom: u32, state: u32, axis: usize) -> f32 {
    let code = t.axes[(state as usize) * 8 + axis];
    let val = x[(atom as usize) * 8 + ((code & 7) as usize)];
    if (code & 8) != 0 {
        -val
    } else {
        val
    }
}

/// Kernel 23 (`src/spiralcore_fbs.cu:91`): `spiralcore_fbs_fwd`.
pub fn spiralcore_fbs_fwd_cpu(
    queries: &[f32],
    input: &[f32],
    tables: &ScFbsHostTables,
    rows: usize,
    outputs: usize,
    inputs: usize,
) -> (Vec<f32>, u32) {
    let mut out = vec![0.0f32; rows * outputs * 8];
    let mut error = 0u32;
    for item in 0..(rows * outputs) {
        let row = item / outputs;
        let output = item % outputs;
        let q = &queries[output * tables.stages * 8..(output + 1) * tables.stages * 8];
        if q.iter().any(|v| !v.is_finite()) {
            error = 1;
            continue;
        }
        let r = sc_fbs_route(tables, q, SC_FBS_ABSENT, 0);
        if r.atom == SC_FBS_ABSENT || (r.atom as usize) >= inputs {
            error = 2;
            continue;
        }
        let x = &input[row * inputs * 8..(row + 1) * inputs * 8];
        for axis in 0..8 {
            let value = sc_fbs_coordinate(tables, x, r.atom, r.state, axis);
            if !value.is_finite() {
                error = 3;
                break;
            }
            out[item * 8 + axis] = value;
        }
    }
    (out, error)
}

/// Kernel 24 (`src/spiralcore_fbs.cu:124`): `spiralcore_fbs_bwd`.
pub fn spiralcore_fbs_bwd_cpu(
    queries: &[f32],
    input: &[f32],
    grad: &[f32],
    tables: &ScFbsHostTables,
    rows: usize,
    outputs: usize,
    inputs: usize,
) -> (Vec<f32>, Vec<f32>, u32) {
    let mut d_queries = vec![0.0f32; queries.len()];
    let mut d_input = vec![0.0f32; input.len()];
    let mut error = 0u32;
    for item in 0..(rows * outputs) {
        let row = item / outputs;
        let output = item % outputs;
        let q = &queries[output * tables.stages * 8..(output + 1) * tables.stages * 8];
        if q.iter().any(|v| !v.is_finite()) {
            error = 1;
            continue;
        }
        let selected = sc_fbs_route(tables, q, SC_FBS_ABSENT, 0);
        if selected.atom == SC_FBS_ABSENT || (selected.atom as usize) >= inputs {
            error = 2;
            continue;
        }
        let x = &input[row * inputs * 8..(row + 1) * inputs * 8];
        let mut dy = [0.0f32; 8];
        let mut y = [0.0f32; 8];
        for axis in 0..8 {
            dy[axis] = grad[item * 8 + axis];
            y[axis] = sc_fbs_coordinate(tables, x, selected.atom, selected.state, axis);
            if !dy[axis].is_finite() || !y[axis].is_finite() {
                error = 3;
                break;
            }
            let code = tables.axes[(selected.state as usize) * 8 + axis];
            let value = if (code & 8) != 0 { -dy[axis] } else { dy[axis] };
            let idx = row * inputs * 8 + (selected.atom as usize) * 8 + ((code & 7) as usize);
            d_input[idx] += value;
        }
        for stage in 0..selected.count {
            let count = if stage == 0 { tables.frontier_count } else { 3 };
            let root = tables.root_of_state[selected.before[stage] as usize];
            let old_goal = selected.goal[stage] as usize;
            for sibling in 0..(count as u32) {
                if sibling == selected.choice[stage] {
                    continue;
                }
                let alternative = sc_fbs_route(tables, q, stage as u32, sibling);
                if alternative.atom == SC_FBS_ABSENT || (alternative.atom as usize) >= inputs {
                    error = 2;
                    break;
                }
                let goal = if stage == 0 {
                    sibling as usize
                } else {
                    tables.branch_targets[(root * 3 + sibling) as usize] as usize
                };
                let mut contrast = 0.0f32;
                let mut squared_edge = 0.0f32;
                for axis in 0..8 {
                    let other =
                        sc_fbs_coordinate(tables, x, alternative.atom, alternative.state, axis);
                    contrast += dy[axis] * (other - y[axis]);
                    let edge = tables.roots[goal * 8 + axis] - tables.roots[old_goal * 8 + axis];
                    squared_edge += edge * edge;
                }
                if squared_edge <= 0.0 || !contrast.is_finite() {
                    error = 4;
                    break;
                }
                let directional = contrast / squared_edge;
                for axis in 0..8 {
                    let edge = tables.roots[goal * 8 + axis] - tables.roots[old_goal * 8 + axis];
                    d_queries[output * tables.stages * 8 + stage * 8 + axis] += directional * edge;
                }
            }
        }
    }
    (d_queries, d_input, error)
}
