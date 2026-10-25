//! Pure-integer D11 recurrence export and receipt verifier for `recurrence-gates-six-recurrence-fit` (`#2890`).
//!
//! Implements the multiplier-free and division-free D11 integer recurrence operations (`CHARTER.md` §3.6 / D11):
//! 1. 16-entry dyadic fade bit-shift `>> (index + 1)` for `lambda_i = 2^{-(i+1)}`.
//! 2. Canonical Signed Digit (CSD) shift-and-add Q12 keep scaling `csd_keep_q12` for `KEEP_TABLE_Q12`.
//! 3. Shift-and-add Q8 4-block Hamilton product `hamilton_shift_add_q8` for state transport and output gating.
//! 4. Pure-integer Q12 16-root E8 / SpiralCore vector writer oracle `pure_i32_q12_writer_oracle`.

#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RATE_ENTRIES: usize = 16;
pub const Q12_SHIFT: u32 = 12;
pub const Q12_ONE: i32 = 4096;
pub const Q8_SHIFT: u32 = 8;
pub const Q8_ONE: i32 = 256;
pub const EXPECTED_DEV_DOCS: u64 = 400;
pub const EXPECTED_DEV_TARGETS: u64 = 14287;
pub const EXPECTED_COUNT_TABLE_TOP1: u64 = 1226;

/// Exact Q12 (`scale = 4096 = 2^12`) keep table for `i in 0..16`.
/// Every entry is a Canonical Signed Digit (CSD) sum/difference of powers of two, so scaling by
/// `KEEP_TABLE_Q12[i]` executes with pure bit shifts and additions/subtractions (`csd_keep_q12`).
pub const KEEP_TABLE_Q12: [i32; RATE_ENTRIES] = [
    3547, // 4096 - 512 - 32 - 4 - 1
    3966, // 4096 - 128 - 2
    4064, // 4096 - 32
    4088, // 4096 - 8
    4094, // 4096 - 2
    4096, // 4096
    4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096,
];

/// Fixed 16-root E8 / paired-H4 table (`roots[16][8]` in `{-1, 0, +1}`) scaled to Q12 (`{-4096, 0, +4096}`).
pub const SPIRALCORE_ROOTS_Q12: [[i32; 8]; 16] = [
    [4096, 0, 0, 0, 4096, 0, 0, 0],
    [0, 4096, 0, 0, 0, 4096, 0, 0],
    [0, 0, 4096, 0, 0, 0, 4096, 0],
    [0, 0, 0, 4096, 0, 0, 0, 4096],
    [4096, 4096, 0, 0, 0, 0, 0, 0],
    [0, 0, 4096, 4096, 0, 0, 0, 0],
    [0, 0, 0, 0, 4096, 4096, 0, 0],
    [0, 0, 0, 0, 0, 0, 4096, 4096],
    [4096, -4096, 0, 0, 0, 0, 0, 0],
    [0, 0, 4096, -4096, 0, 0, 0, 0],
    [0, 0, 0, 0, 4096, -4096, 0, 0],
    [0, 0, 0, 0, 0, 0, 4096, -4096],
    [4096, 0, 4096, 0, 0, 0, 0, 0],
    [0, 4096, 0, 4096, 0, 0, 0, 0],
    [0, 0, 0, 0, 4096, 0, 4096, 0],
    [0, 0, 0, 0, 0, 4096, 0, 4096],
];

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Pure shift-and-add Q12 keep scaling `(val_q12 * KEEP_TABLE_Q12[index]) >> 12` with zero
/// multiplier instructions.
#[inline]
pub fn csd_keep_q12(index: usize, val_q12: i32) -> i32 {
    let x = i64::from(val_q12);
    let scaled = match index {
        0 => (x << 12) - (x << 9) - (x << 5) - (x << 2) - x,
        1 => (x << 12) - (x << 7) - (x << 1),
        2 => (x << 12) - (x << 5),
        3 => (x << 12) - (x << 3),
        4 => (x << 12) - (x << 1),
        _ => x << 12,
    };
    (scaled >> Q12_SHIFT) as i32
}

/// Pure 8-bit shift-and-add integer product `q_q8 * val` for `q_q8 in -256..=256` with zero
/// multiplier instructions.
#[inline]
pub fn shift_add_mul_q8(q_q8: i32, val: i32) -> i64 {
    let neg = q_q8 < 0;
    let mag = q_q8.unsigned_abs();
    let v = i64::from(val);
    let mut acc = 0i64;
    let mut bit = 0u32;
    while bit <= Q8_SHIFT {
        if ((mag >> bit) & 1) != 0 {
            acc += v << bit;
        }
        bit += 1;
    }
    if neg {
        -acc
    } else {
        acc
    }
}

/// Pure shift-and-add 4-block Hamilton product `(q_q8 (x) x) >> 8` with zero multiplier instructions.
#[inline]
pub fn hamilton_shift_add_q8(q_q8: [i32; 4], x: [i32; 4]) -> [i32; 4] {
    let [a1, b1, c1, d1] = q_q8;
    let [a2, b2, c2, d2] = x;
    let r0 = shift_add_mul_q8(a1, a2)
        - shift_add_mul_q8(b1, b2)
        - shift_add_mul_q8(c1, c2)
        - shift_add_mul_q8(d1, d2);
    let r1 = shift_add_mul_q8(a1, b2) + shift_add_mul_q8(b1, a2) + shift_add_mul_q8(c1, d2)
        - shift_add_mul_q8(d1, c2);
    let r2 = shift_add_mul_q8(a1, c2) - shift_add_mul_q8(b1, d2)
        + shift_add_mul_q8(c1, a2)
        + shift_add_mul_q8(d1, b2);
    let r3 = shift_add_mul_q8(a1, d2) + shift_add_mul_q8(b1, c2) - shift_add_mul_q8(c1, b2)
        + shift_add_mul_q8(d1, a2);
    [
        (r0 >> Q8_SHIFT) as i32,
        (r1 >> Q8_SHIFT) as i32,
        (r2 >> Q8_SHIFT) as i32,
        (r3 >> Q8_SHIFT) as i32,
    ]
}

/// Pure-integer D11 recurrence scan + Hamilton output gate over `time` steps for one 4-lane:
/// `h_t = (hamilton_shift_add_q8(u_q8[t], h_{t-1}) >> (rate_idx[t] + 1)) + csd_keep_q12(rate_idx[t], conv_q12[t])`
/// `out_t = hamilton_shift_add_q8(gate_q8[t], h_t)`
pub fn d11_recurrence_scan_and_gate_i32(
    rate_indices: &[usize],
    u_q8: &[[i32; 4]],
    conv_q12: &[[i32; 4]],
    gate_q8: &[[i32; 4]],
) -> Vec<[i32; 4]> {
    let mut h = [0i32; 4];
    let mut out = Vec::with_capacity(rate_indices.len());
    for t in 0..rate_indices.len() {
        let idx = rate_indices[t].min(RATE_ENTRIES - 1);
        let shift = (idx + 1) as u32;
        let rot = hamilton_shift_add_q8(u_q8[t], h);
        let faded = [
            rot[0] >> shift,
            rot[1] >> shift,
            rot[2] >> shift,
            rot[3] >> shift,
        ];
        let c = conv_q12[t];
        let drive = [
            csd_keep_q12(idx, c[0]),
            csd_keep_q12(idx, c[1]),
            csd_keep_q12(idx, c[2]),
            csd_keep_q12(idx, c[3]),
        ];
        h = [
            faded[0] + drive[0],
            faded[1] + drive[1],
            faded[2] + drive[2],
            faded[3] + drive[3],
        ];
        out.push(hamilton_shift_add_q8(gate_q8[t], h));
    }
    out
}

/// Pure-integer Q12 16-root E8 cell-admitted vector writer oracle (`CHARTER.md` §3.6 / D11).
#[allow(clippy::too_many_arguments)]
pub fn pure_i32_q12_writer_oracle(
    queries_q12: &[i32],
    table_q12: &[i32],
    roots_q12: &[[i32; 8]; 16],
    rows: usize,
    width: usize,
    vocab: usize,
    kept_cells: usize,
    targets: &[u32],
) -> (Vec<u32>, usize, usize) {
    let mut token_cells = vec![0usize; vocab];
    for (v, cell_slot) in token_cells.iter_mut().enumerate().take(vocab) {
        let row = &table_q12[v * width..(v + 1) * width];
        let mut best_cell = 0usize;
        let mut best_score = i64::MIN;
        for (c, root) in roots_q12.iter().enumerate() {
            let mut s = 0i64;
            for j in 0..8 {
                if root[j] > 0 {
                    s += i64::from(row[j]);
                } else if root[j] < 0 {
                    s -= i64::from(row[j]);
                }
            }
            if s > best_score {
                best_score = s;
                best_cell = c;
            }
        }
        *cell_slot = best_cell;
    }

    let mut preds = Vec::with_capacity(rows);
    let mut top1_hits = 0usize;
    let mut admitted_hits = 0usize;

    for (r, &target) in targets.iter().enumerate().take(rows) {
        let q = &queries_q12[r * width..(r + 1) * width];
        let mut cell_scores: [(i64, usize); 16] = std::array::from_fn(|c| {
            let root = &roots_q12[c];
            let mut s = 0i64;
            for j in 0..8 {
                if root[j] > 0 {
                    s += i64::from(q[j]);
                } else if root[j] < 0 {
                    s -= i64::from(q[j]);
                }
            }
            (s, c)
        });
        cell_scores.sort_by(|left, right| {
            let (sa, ca) = *left;
            let (sb, cb) = *right;
            sb.cmp(&sa).then_with(|| ca.cmp(&cb))
        });
        let mut admitted = [false; 16];
        for item in cell_scores.iter().take(kept_cells.min(16)) {
            let (_, cell_idx) = *item;
            admitted[cell_idx] = true;
        }
        let tgt = target as usize;
        if tgt < vocab && admitted[token_cells[tgt]] {
            admitted_hits += 1;
        }
        let mut best_tok = 0u32;
        let mut best_neg_dist = i64::MIN;
        for (v, &tc) in token_cells.iter().enumerate().take(vocab) {
            if !admitted[tc] {
                continue;
            }
            let trow = &table_q12[v * width..(v + 1) * width];
            let mut l1 = 0i64;
            for j in 0..width {
                l1 += (i64::from(q[j]) - i64::from(trow[j])).abs();
            }
            let neg_dist = -l1;
            if neg_dist > best_neg_dist {
                best_neg_dist = neg_dist;
                best_tok = v as u32;
            }
        }
        if best_tok == target {
            top1_hits += 1;
        }
        preds.push(best_tok);
    }

    (preds, top1_hits, admitted_hits)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegerExportExactnessSummary {
    pub passed: bool,
    pub csd_keep_q12_exact_entries: usize,
    pub shift_add_q8_exact_pairs: usize,
    pub synthetic_arms_tested: usize,
    pub max_scan_lsb_diff: i32,
}

/// Verifies `I-Export-Exactness` using pure integer arithmetic (`CHARTER.md` §3.6 / D11 / D32).
pub fn verify_d11_integer_export_exactness() -> Result<IntegerExportExactnessSummary, String> {
    let mut csd_ok = 0usize;
    for (idx, &k_q12) in KEEP_TABLE_Q12.iter().enumerate() {
        for sample in [-32768i32, -4096, -1023, -1, 0, 1, 1023, 4096, 32767] {
            let expected = ((i64::from(sample) * i64::from(k_q12)) >> Q12_SHIFT) as i32;
            let actual = csd_keep_q12(idx, sample);
            if actual != expected {
                return Err(format!(
                    "csd_keep_q12 mismatch at idx={idx}, sample={sample}: {actual} != {expected}"
                ));
            }
        }
        csd_ok += 1;
    }

    let mut q8_pairs = 0usize;
    for q in [-256i32, -181, -128, -63, -1, 0, 1, 63, 128, 181, 256] {
        for v in [-32768i32, -4096, -333, 0, 333, 4096, 32767] {
            if shift_add_mul_q8(q, v) != i64::from(q) * i64::from(v) {
                return Err(format!("shift_add_mul_q8 mismatch at q={q}, v={v}"));
            }
            q8_pairs += 1;
        }
    }

    let rows = 16usize;
    let lanes = 8usize;
    let mut max_scan_lsb_diff = 0i32;

    for arm_idx in 0..4usize {
        for lane in 0..lanes {
            let rate_indices: Vec<usize> = (0..rows)
                .map(|t| {
                    if arm_idx == 3 {
                        lane % RATE_ENTRIES
                    } else {
                        (lane + t + arm_idx) % RATE_ENTRIES
                    }
                })
                .collect();
            let u_q8: Vec<[i32; 4]> = (0..rows)
                .map(|t| match arm_idx {
                    2 => [Q8_ONE, 0, 0, 0],
                    1 => {
                        if (t + lane).is_multiple_of(2) {
                            [181, 181, 0, 0]
                        } else {
                            [Q8_ONE, 0, 0, 0]
                        }
                    }
                    _ => {
                        if (t + lane).is_multiple_of(2) {
                            [128, 128, 128, 128]
                        } else {
                            [181, 0, 181, 0]
                        }
                    }
                })
                .collect();
            let conv_q12: Vec<[i32; 4]> = (0..rows)
                .map(|t| {
                    let b = ((t * 31 + lane * 13 + arm_idx * 7) % 1024) as i32 - 512;
                    [b, -b / 2, b / 3, -b / 4]
                })
                .collect();
            let gate_q8: Vec<[i32; 4]> = (0..rows)
                .map(|t| {
                    if (t + lane).is_multiple_of(3) {
                        [128, -128, 128, -128]
                    } else {
                        [Q8_ONE, 0, 0, 0]
                    }
                })
                .collect();

            let scan_i32 =
                d11_recurrence_scan_and_gate_i32(&rate_indices, &u_q8, &conv_q12, &gate_q8);
            let mut h_ref = [0i32; 4];
            for t in 0..rows {
                let idx = rate_indices[t];
                let shift = (idx + 1) as u32;
                let [u0, u1, u2, u3] = u_q8[t].map(i64::from);
                let [h0, h1, h2, h3] = h_ref.map(i64::from);
                let rot = [
                    ((u0 * h0 - u1 * h1 - u2 * h2 - u3 * h3) >> Q8_SHIFT) as i32,
                    ((u0 * h1 + u1 * h0 + u2 * h3 - u3 * h2) >> Q8_SHIFT) as i32,
                    ((u0 * h2 - u1 * h3 + u2 * h0 + u3 * h1) >> Q8_SHIFT) as i32,
                    ((u0 * h3 + u1 * h2 - u2 * h1 + u3 * h0) >> Q8_SHIFT) as i32,
                ];
                let k_q12 = i64::from(KEEP_TABLE_Q12[idx]);
                for c in 0..4 {
                    let faded = rot[c] >> shift;
                    let drive = ((i64::from(conv_q12[t][c]) * k_q12) >> Q12_SHIFT) as i32;
                    h_ref[c] = faded + drive;
                }
                let [g0, g1, g2, g3] = gate_q8[t].map(i64::from);
                let [hn0, hn1, hn2, hn3] = h_ref.map(i64::from);
                let out_ref = [
                    ((g0 * hn0 - g1 * hn1 - g2 * hn2 - g3 * hn3) >> Q8_SHIFT) as i32,
                    ((g0 * hn1 + g1 * hn0 + g2 * hn3 - g3 * hn2) >> Q8_SHIFT) as i32,
                    ((g0 * hn2 - g1 * hn3 + g2 * hn0 + g3 * hn1) >> Q8_SHIFT) as i32,
                    ((g0 * hn3 + g1 * hn2 - g2 * hn1 + g3 * hn0) >> Q8_SHIFT) as i32,
                ];
                for c in 0..4 {
                    let diff = (scan_i32[t][c] - out_ref[c]).abs();
                    if diff > max_scan_lsb_diff {
                        max_scan_lsb_diff = diff;
                    }
                }
            }
        }
    }

    let passed = csd_ok == RATE_ENTRIES && max_scan_lsb_diff == 0;
    Ok(IntegerExportExactnessSummary {
        passed,
        csd_keep_q12_exact_entries: csd_ok,
        shift_add_q8_exact_pairs: q8_pairs,
        synthetic_arms_tested: 4,
        max_scan_lsb_diff,
    })
}

/// Integer square-root ceiling `ceil(sqrt(n))` with zero floating-point operations.
pub fn isqrt_ceil(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut lo = 1u64;
    let mut hi = 1u64 << 32;
    while lo < hi {
        let mid = lo + ((hi - lo) >> 1);
        if mid.saturating_mul(mid) >= n {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// Paired McNemar threshold `F_I = max(24, ceil(2 * sqrt(b + c))) = max(24, ceil(sqrt(4 * (b + c))))`
/// computed in exact integer arithmetic.
pub fn mcnemar_f_i_threshold_int(discordant_b_plus_c: u64) -> u64 {
    let bound = isqrt_ceil(discordant_b_plus_c.saturating_mul(4));
    if bound > 24 {
        bound
    } else {
        24
    }
}

/// Verifies a sealed `receipt.json` from `recurrence-gates-six-recurrence-fit` (`#2890`).
pub fn verify_receipt_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let sha = sha256_bytes(&bytes);
    let v: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON: {e}"))?;

    if v["item_id"].as_str() != Some("recurrence-gates-six-recurrence-fit") {
        return Err("receipt item_id mismatch".into());
    }
    if v["issue"].as_u64() != Some(2890) {
        return Err("receipt issue != 2890".into());
    }
    if v["dev_documents"].as_u64() != Some(EXPECTED_DEV_DOCS) {
        return Err("dev_documents != 400".into());
    }
    if v["dev_eligible_targets"].as_u64() != Some(EXPECTED_DEV_TARGETS) {
        return Err("dev_eligible_targets != 14287".into());
    }
    if v["train_split_count_table_ref"]["dev_top1_hits"].as_u64() != Some(EXPECTED_COUNT_TABLE_TOP1)
    {
        return Err("train_split_count_table_ref.dev_top1_hits != 1226".into());
    }
    for flag in [
        "i_standalone_cuda_parity_pass",
        "i_dev_denominator_pass",
        "i_rate_mobility_and_plateau_pass",
        "i_export_exactness_pass",
    ] {
        if v[flag].as_bool() != Some(true) {
            return Err(format!("instrument check {flag} is not true"));
        }
    }
    let runs = v["runs"]
        .as_array()
        .ok_or_else(|| String::from("missing runs array"))?;
    if runs.len() != 8 {
        return Err(format!("expected 8 arm-seed runs, got {}", runs.len()));
    }
    for run in runs {
        let scored = run["dev_step1024_d11_int"]["scored_rows"]
            .as_u64()
            .unwrap_or(0);
        if scored != EXPECTED_DEV_TARGETS {
            return Err(format!("run scored_rows {scored} != 14287"));
        }
        let writer_mm = run["dev_step1024_d11_int"]["q12_i32_writer_oracle_mismatches"]
            .as_u64()
            .unwrap_or(1);
        if writer_mm != 0 {
            return Err(format!("q12_i32_writer_oracle_mismatches = {writer_mm}"));
        }
    }
    Ok(sha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d11_integer_export_exactness_passes() {
        let summary = verify_d11_integer_export_exactness().expect("exactness check");
        assert!(summary.passed);
        assert_eq!(summary.csd_keep_q12_exact_entries, 16);
        assert_eq!(summary.max_scan_lsb_diff, 0);
    }

    #[test]
    fn mcnemar_f_i_threshold_int_matches_formula() {
        assert_eq!(mcnemar_f_i_threshold_int(0), 24);
        assert_eq!(mcnemar_f_i_threshold_int(50), 24);
        assert_eq!(mcnemar_f_i_threshold_int(144), 24);
        assert_eq!(mcnemar_f_i_threshold_int(145), 25);
        assert_eq!(mcnemar_f_i_threshold_int(400), 40);
    }

    #[test]
    fn pure_i32_writer_oracle_selects_exact_match() {
        let width = 8usize;
        let vocab = 4usize;
        let table_q12: Vec<i32> = vec![
            4096, 0, 0, 0, 4096, 0, 0, 0, // token 0 in cell 0
            0, 4096, 0, 0, 0, 4096, 0, 0, // token 1 in cell 1
            0, 0, 4096, 0, 0, 0, 4096, 0, // token 2 in cell 2
            0, 0, 0, 4096, 0, 0, 0, 4096, // token 3 in cell 3
        ];
        let queries_q12: Vec<i32> = vec![
            4096, 0, 0, 0, 4096, 0, 0, 0, // query 0 -> token 0
            0, 0, 4096, 0, 0, 0, 4096, 0, // query 1 -> token 2
        ];
        let targets = vec![0u32, 2u32];
        let (preds, top1, adm) = pure_i32_q12_writer_oracle(
            &queries_q12,
            &table_q12,
            &SPIRALCORE_ROOTS_Q12,
            2,
            width,
            vocab,
            8,
            &targets,
        );
        assert_eq!(preds, vec![0u32, 2u32]);
        assert_eq!(top1, 2);
        assert_eq!(adm, 2);
    }
}
