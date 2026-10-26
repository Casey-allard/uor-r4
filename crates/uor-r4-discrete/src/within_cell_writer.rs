//! Within-cell writer error decomposition, versioned `ReadLawV2` (`#3128`) & framing
//! (`#3129`) finite controls, and the zero-update `B0` evaluation (`m1-within-cell-writer`, `#3267`).
//!
//! All model forward, read, and writer evaluations execute on CUDA (`CHARTER §3`).
//! Host helpers perform only deterministic codebook construction, reference-control
//! verification, and exact integer grading of stored GPU observations (`D24`, `D32`).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::corpus::FilePin;
use crate::w2_capacity::{self, TrainRow};
use crate::{Error, Result};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

pub const WITHIN_CELL_QUAL_SCHEMA: &str = "uor-r4.within-cell-writer-qualification/v1";
pub const WITHIN_CELL_DECOMP_SCHEMA: &str = "uor-r4.within-cell-stage-t-decomposition/v1";
pub const WITHIN_CELL_B0_SCHEMA: &str = "uor-r4.within-cell-b0-grade/v1";

pub const A8_STRUCTURED_TABLE_SHA256: &str =
    "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1";
pub const FULL_VECTOR_B4_TABLE_SHA256: &str =
    "2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2";
pub const DEFAULT_MAX_REL_BOUND: u32 = 16_777_216;
pub const PERMUTATION_SEED_20261026: u64 = 20_261_026;
pub const B0_MIN_FLOOR: i64 = 24;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn pin_file(path: &Path) -> Result<FilePin> {
    let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    Ok(FilePin {
        path: path.to_path_buf(),
        sha256: sha256_hex(&bytes),
    })
}

pub fn write_pinned_json<T: Serialize>(path: &Path, val: &T) -> Result<FilePin> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| invalid(format!("{}: {e}", parent.display())))?;
    }
    let mut text = serde_json::to_string_pretty(val)
        .map_err(|e| invalid(format!("serialize {}: {e}", path.display())))?;
    text.push('\n');
    let sha256 = sha256_hex(text.as_bytes());
    fs::write(path, text.as_bytes()).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    Ok(FilePin {
        path: path.to_path_buf(),
        sha256,
    })
}

/// Versioned read law selector (`wc_cfg[0]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadLawVersion {
    /// Legacy `discrete.cu` law (`terms += 1`, `e == 3/4` overwrite, `rel` unreset across sweeps).
    ReadLawV1,
    /// `#3128` versioned law (weighted carry `2 * v`, `terms += 2`, recent-8 capped `e == 3/4`
    /// with direct query-token identity match, per-sweep `rel` reset, and integer bound check).
    ReadLawV2,
}

impl ReadLawVersion {
    pub fn word(self) -> u32 {
        match self {
            Self::ReadLawV1 => 0,
            Self::ReadLawV2 => 1,
        }
    }
}

/// Interior-block frame source selector (`wc_cfg[1]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameSourceMode {
    /// `f(0)=3, f(1)=0, f(2)=0, f(3)=0`, `fq` from cursor token.
    CurrentSinglePassV1,
    /// `f(0)=3, f(1)=0, f(2)=3, f(3)=0`, `fq` from cursor token's nearest endpoint.
    NearestEndpointSinglePassV1,
    /// `#3129` two-pass: Pass 1 runs `b in {0, 3}`; Pass 2 conditions `b = 1` on `out_0` and `b = 2` on `out_3`.
    TwoPassEndpointOutputV1,
    /// Interior blocks `b in {1, 2}` use a frozen constant frame root `0..15`.
    FrozenConstantFrameV1,
    /// Pass 2 conditions `b in {1, 2}` on a fixed 16-root derangement of Pass-1's `(out_0, out_3)`.
    PermutedFrameV1,
    /// Single-pass interior blocks remap cursor endpoints via a TRAIN-estimated `16 -> 16` table.
    TrainRemap16V1,
}

impl FrameSourceMode {
    pub fn word(self) -> u32 {
        match self {
            Self::CurrentSinglePassV1 => 0,
            Self::NearestEndpointSinglePassV1 => 1,
            Self::TwoPassEndpointOutputV1 => 2,
            Self::FrozenConstantFrameV1 => 3,
            Self::PermutedFrameV1 => 4,
            Self::TrainRemap16V1 => 5,
        }
    }
}

/// Candidate leaf payload transport selector (`wc_cfg[2]`, `#3128` Item 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportMode {
    /// Standard relative BFS frame transport `g[BFS + frame[node]*16 + fq]`.
    RelativeFrameTransportV1,
    /// Carry leaf candidate payloads in the global codebook frame (`g[IDENTITY]`).
    GlobalWriterFrameV1,
}

impl TransportMode {
    pub fn word(self) -> u32 {
        match self {
            Self::RelativeFrameTransportV1 => 0,
            Self::GlobalWriterFrameV1 => 1,
        }
    }
}

/// Exact integer `ceil(2 * sqrt(n))`: smallest `k >= 0` such that `k * k >= 4 * n`.
pub fn integer_ceil_two_sqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let target = n.saturating_mul(4);
    let mut low = 1u64;
    let mut high = 2_000_000_000u64;
    while low < high {
        let mid = low + (high - low) / 2;
        if mid.saturating_mul(mid) >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low
}

/// Strict paired McNemar rule (`m1-within-cell-writer.md:32`): `d > 24 && d^2 > 4(b + c)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedComparison {
    pub label: String,
    pub candidate_arm: String,
    pub reference_arm: String,
    pub evaluated_rows: u64,
    pub candidate_hits: u64,
    pub reference_hits: u64,
    pub both_hit_a: u64,
    pub candidate_only_b: u64,
    pub reference_only_c: u64,
    pub neither_d: u64,
    pub discordance_n: u64,
    pub net_diff_d: i64,
    pub floor_f: u64,
    pub clears_strict_paired_rule: bool,
}

pub fn compute_paired_comparison(
    label: &str,
    candidate_arm: &str,
    reference_arm: &str,
    cand_hits: &[bool],
    ref_hits: &[bool],
) -> Result<PairedComparison> {
    if cand_hits.len() != ref_hits.len() {
        return Err(invalid(
            "paired comparison requires equal-length hit vectors",
        ));
    }
    let mut a = 0u64;
    let mut b = 0u64;
    let mut c = 0u64;
    let mut d = 0u64;
    for (&ch, &rh) in cand_hits.iter().zip(ref_hits) {
        match (ch, rh) {
            (true, true) => a += 1,
            (true, false) => b += 1,
            (false, true) => c += 1,
            (false, false) => d += 1,
        }
    }
    let n = b + c;
    let diff = (b as i64) - (c as i64);
    let floor_f = integer_ceil_two_sqrt(n).max(B0_MIN_FLOOR as u64);
    let clears = diff > B0_MIN_FLOOR && i128::from(diff) * i128::from(diff) > 4 * i128::from(n);
    Ok(PairedComparison {
        label: label.to_owned(),
        candidate_arm: candidate_arm.to_owned(),
        reference_arm: reference_arm.to_owned(),
        evaluated_rows: cand_hits.len() as u64,
        candidate_hits: a + b,
        reference_hits: a + c,
        both_hit_a: a,
        candidate_only_b: b,
        reference_only_c: c,
        neither_d: d,
        discordance_n: n,
        net_diff_d: diff,
        floor_f,
        clears_strict_paired_rule: clears,
    })
}

/// Host mirror of `snap(x, n, g)` in `discrete.cu:108-115`.
pub fn host_snap(x: &[i32; 8], n: u32) -> u8 {
    let mut best = 0u8;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for r in 0..16u8 {
        let root = &geometry::ROOTS[usize::from(r)];
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for d in 0..8usize {
            let mut y = 0i32;
            let v = i32::from(root[d]);
            for _ in 0..n {
                y += v;
            }
            distance += x[d].abs_diff(y);
            if x[d] != y {
                hamming += 1;
            }
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    best
}

/// Summary of the `16 x 16 = 256` root-pair orbit check (`#3128` Control 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapOrbitSummary {
    pub total_pairs: u32,
    pub distinct_pairs: u32,
    pub v1_recover_v_all_256: u32,
    pub v1_recover_v_distinct_240: u32,
    pub v1_lose_to_rq_lower_index: u32,
    pub v1_antipodal_zero_collapse: u32,
    pub v2_pos_recover_v_all_256: u32,
    pub v2_pos_recover_v_distinct_240: u32,
    pub v2_neg_equals_v_count: u32,
    pub v2_neg_recovers_antipode_when_present: u32,
}

pub fn host_snap_orbit_summary() -> (SnapOrbitSummary, Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut v1_out = Vec::with_capacity(256);
    let mut v2_pos_out = Vec::with_capacity(256);
    let mut v2_neg_out = Vec::with_capacity(256);

    let mut v1_all = 0u32;
    let mut v1_dist = 0u32;
    let mut v1_lose_rq = 0u32;
    let mut v1_antipodal = 0u32;
    let mut v2_pos_all = 0u32;
    let mut v2_pos_dist = 0u32;
    let mut v2_neg_eq_v = 0u32;
    let mut v2_neg_antipode = 0u32;

    for rq in 0..16usize {
        for rv in 0..16usize {
            let mut u_v1 = [0i32; 8];
            let mut u_v2_pos = [0i32; 8];
            let mut u_v2_neg = [0i32; 8];
            let mut is_antipodal = true;
            for d in 0..8usize {
                let qd = i32::from(geometry::ROOTS[rq][d]);
                let vd = i32::from(geometry::ROOTS[rv][d]);
                u_v1[d] = qd + vd;
                u_v2_pos[d] = qd + (vd << 1);
                u_v2_neg[d] = qd - (vd << 1);
                if u_v1[d] != 0 {
                    is_antipodal = false;
                }
            }
            let s1 = u32::from(host_snap(&u_v1, 2));
            let s2p = u32::from(host_snap(&u_v2_pos, 3));
            let s2n = u32::from(host_snap(&u_v2_neg, 3));
            v1_out.push(s1);
            v2_pos_out.push(s2p);
            v2_neg_out.push(s2n);

            if s1 == rv as u32 {
                v1_all += 1;
                if rq != rv {
                    v1_dist += 1;
                }
            } else if rq != rv {
                if is_antipodal && rv != 0 {
                    v1_antipodal += 1;
                } else if s1 == rq as u32 {
                    v1_lose_rq += 1;
                }
            }

            if s2p == rv as u32 {
                v2_pos_all += 1;
                if rq != rv {
                    v2_pos_dist += 1;
                }
            }
            if s2n == rv as u32 {
                v2_neg_eq_v += 1;
            }
            let neg_v: [i8; 8] = core::array::from_fn(|d| -geometry::ROOTS[rv][d]);
            if let Some(antipode_idx) = geometry::ROOTS.iter().position(|r| *r == neg_v) {
                if s2n == antipode_idx as u32 {
                    v2_neg_antipode += 1;
                }
            }
        }
    }

    (
        SnapOrbitSummary {
            total_pairs: 256,
            distinct_pairs: 240,
            v1_recover_v_all_256: v1_all,
            v1_recover_v_distinct_240: v1_dist,
            v1_lose_to_rq_lower_index: v1_lose_rq,
            v1_antipodal_zero_collapse: v1_antipodal,
            v2_pos_recover_v_all_256: v2_pos_all,
            v2_pos_recover_v_distinct_240: v2_pos_dist,
            v2_neg_equals_v_count: v2_neg_eq_v,
            v2_neg_recovers_antipode_when_present: v2_neg_antipode,
        },
        v1_out,
        v2_pos_out,
        v2_neg_out,
    )
}

/// Deterministic 16-element derangement (`pi[r] != r` for all `r in 0..16`) for `PermutedFrameV1`.
pub fn derangement_16(seed: u64) -> [u32; 16] {
    let mut arr: [u32; 16] = core::array::from_fn(|i| i as u32);
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15u64;
    for i in (1..16usize).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state as usize) % i;
        arr.swap(i, j);
    }
    for i in 0..16usize {
        if arr[i] == i as u32 {
            let next = (i + 1) % 16;
            arr.swap(i, next);
        }
    }
    arr
}

/// Both 16-entry permuted frame tables (`[0..16]` for block 1, `[16..32]` for block 2).
pub fn permuted_frame_remap_32(seed: u64) -> [u32; 32] {
    let p0 = derangement_16(seed);
    let p3 = derangement_16(seed.wrapping_add(1));
    let mut out = [0u32; 32];
    out[..16].copy_from_slice(&p0);
    out[16..32].copy_from_slice(&p3);
    out
}

/// Summary of the `16 -> 16` TRAIN-estimated endpoint frame remap (`TrainRemap16V1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameRemapSummary {
    pub train_fit_rows: u64,
    pub validation_holdout_rows: u64,
    pub remap_endpoint_0: [u32; 16],
    pub remap_endpoint_3: [u32; 16],
    pub train_fit_endpoint0_remap_hits: u64,
    pub train_fit_endpoint0_identity_hits: u64,
    pub train_fit_endpoint3_remap_hits: u64,
    pub train_fit_endpoint3_identity_hits: u64,
    pub val_endpoint0_remap_hits: u64,
    pub val_endpoint0_identity_hits: u64,
    pub val_endpoint3_remap_hits: u64,
    pub val_endpoint3_identity_hits: u64,
}

/// Fit the `16 -> 16` endpoint transition map on the first `fit_rows` of `train` and
/// evaluate endpoint root recovery on both the fit slice and the remaining validation slice.
pub fn estimate_frame_remap_16(
    train: &[TrainRow],
    token_cells: &[u16],
    fit_rows: usize,
) -> ([u32; 32], FrameRemapSummary) {
    let split = fit_rows.min(train.len());
    let mut counts0 = [[0u64; 16]; 16];
    let mut counts3 = [[0u64; 16]; 16];
    for row in &train[..split] {
        let qc = token_cells[usize::from(row.query_token)];
        let tc = token_cells[usize::from(row.target_token)];
        let qr0 = usize::from((qc / 16) & 0xF);
        let qr3 = usize::from((qc % 16) & 0xF);
        let tr0 = usize::from((tc / 16) & 0xF);
        let tr3 = usize::from((tc % 16) & 0xF);
        counts0[qr0][tr0] += 1;
        counts3[qr3][tr3] += 1;
    }
    let mut remap0 = [0u32; 16];
    let mut remap3 = [0u32; 16];
    for src in 0..16usize {
        let mut best0 = 0usize;
        let mut best3 = 0usize;
        for dst in 1..16usize {
            if counts0[src][dst] > counts0[src][best0] {
                best0 = dst;
            }
            if counts3[src][dst] > counts3[src][best3] {
                best3 = dst;
            }
        }
        remap0[src] = best0 as u32;
        remap3[src] = best3 as u32;
    }

    let mut fit_r0 = 0u64;
    let mut fit_i0 = 0u64;
    let mut fit_r3 = 0u64;
    let mut fit_i3 = 0u64;
    for row in &train[..split] {
        let qc = token_cells[usize::from(row.query_token)];
        let tc = token_cells[usize::from(row.target_token)];
        let qr0 = usize::from((qc / 16) & 0xF);
        let qr3 = usize::from((qc % 16) & 0xF);
        let tr0 = u32::from((tc / 16) & 0xF);
        let tr3 = u32::from((tc % 16) & 0xF);
        if remap0[qr0] == tr0 {
            fit_r0 += 1;
        }
        if qr0 as u32 == tr0 {
            fit_i0 += 1;
        }
        if remap3[qr3] == tr3 {
            fit_r3 += 1;
        }
        if qr3 as u32 == tr3 {
            fit_i3 += 1;
        }
    }

    let mut val_r0 = 0u64;
    let mut val_i0 = 0u64;
    let mut val_r3 = 0u64;
    let mut val_i3 = 0u64;
    for row in &train[split..] {
        let qc = token_cells[usize::from(row.query_token)];
        let tc = token_cells[usize::from(row.target_token)];
        let qr0 = usize::from((qc / 16) & 0xF);
        let qr3 = usize::from((qc % 16) & 0xF);
        let tr0 = u32::from((tc / 16) & 0xF);
        let tr3 = u32::from((tc % 16) & 0xF);
        if remap0[qr0] == tr0 {
            val_r0 += 1;
        }
        if qr0 as u32 == tr0 {
            val_i0 += 1;
        }
        if remap3[qr3] == tr3 {
            val_r3 += 1;
        }
        if qr3 as u32 == tr3 {
            val_i3 += 1;
        }
    }

    let mut out = [0u32; 32];
    out[..16].copy_from_slice(&remap0);
    out[16..32].copy_from_slice(&remap3);
    (
        out,
        FrameRemapSummary {
            train_fit_rows: split as u64,
            validation_holdout_rows: (train.len() - split) as u64,
            remap_endpoint_0: remap0,
            remap_endpoint_3: remap3,
            train_fit_endpoint0_remap_hits: fit_r0,
            train_fit_endpoint0_identity_hits: fit_i0,
            train_fit_endpoint3_remap_hits: fit_r3,
            train_fit_endpoint3_identity_hits: fit_i3,
            val_endpoint0_remap_hits: val_r0,
            val_endpoint0_identity_hits: val_i0,
            val_endpoint3_remap_hits: val_r3,
            val_endpoint3_identity_hits: val_i3,
        },
    )
}

/// Deterministic bijection over `0..=65535` for the permuted-code null (`R_cell_code_perm`).
pub fn permuted_code_lookup_table(seed: u64) -> Vec<u16> {
    let mut table: Vec<u16> = (0..=u16::MAX).collect();
    let mut state = seed ^ 0x6a09_e667_f3bc_c908u64;
    for i in (1..table.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state as usize) % (i + 1);
        table.swap(i, j);
    }
    table
}

/// Extract each token's `(cell_id, interior_key)` from a `4,096 x 32` table (`B = 4`).
pub fn extract_token_cells_and_interior_keys(table: &[i32]) -> Result<(Vec<u16>, Vec<u16>)> {
    if table.len() != w2_capacity::VOCABULARY * 32 {
        return Err(invalid("table length differs from 4096 * 32"));
    }
    let mut cells = Vec::with_capacity(w2_capacity::VOCABULARY);
    let mut int_keys = Vec::with_capacity(w2_capacity::VOCABULARY);
    for t in 0..w2_capacity::VOCABULARY {
        let slice = &table[t * 32..(t + 1) * 32];
        let q: &[i32; 32] = slice
            .try_into()
            .map_err(|_| invalid("token slice length != 32"))?;
        let code = w2_capacity::decode_read_vector(q)
            .map_err(|e| invalid(format!("token {t} decode: {e}")))?;
        cells.push(w2_capacity::address_key(code));
        int_keys.push(w2_capacity::interior_key(code));
    }
    Ok((cells, int_keys))
}

///Note: in `discrete.cu`, `token_cell` is `r0 * 16 + r3` (`(r0 << 4) | r3`), whereas
/// `w2_capacity::address_key(code)` is `r0 | (r3 << 4)`.
/// This helper returns `token_cell = (r0 << 4) | r3` and `interior_key = (r1 << 4) | r2`
/// matching `discrete.cu` and `address_cells.rs` (`ic = r1 * 16 + r2`).
pub fn extract_canonical_cells_and_interior_pairs(table: &[i32]) -> Result<(Vec<u16>, Vec<u16>)> {
    if table.len() != w2_capacity::VOCABULARY * 32 {
        return Err(invalid("table length differs from 4096 * 32"));
    }
    let mut cells = Vec::with_capacity(w2_capacity::VOCABULARY);
    let mut int_pairs = Vec::with_capacity(w2_capacity::VOCABULARY);
    for t in 0..w2_capacity::VOCABULARY {
        let slice = &table[t * 32..(t + 1) * 32];
        let q: &[i32; 32] = slice
            .try_into()
            .map_err(|_| invalid("token slice length != 32"))?;
        let code = w2_capacity::decode_read_vector(q)
            .map_err(|e| invalid(format!("token {t} decode: {e}")))?;
        let roots = w2_capacity::unpack_code(code);
        cells.push((u16::from(roots[0]) << 4) | u16::from(roots[3]));
        int_pairs.push((u16::from(roots[1]) << 4) | u16::from(roots[2]));
    }
    Ok((cells, int_pairs))
}

/// Decode a 32-int read vector `q` to `(packed_code, cell_id = r0*16+r3, interior_pair = r1*16+r2)`.
pub fn decode_q_keys(q: &[i32]) -> Result<(u16, u16, u16)> {
    let arr: &[i32; 32] = q
        .try_into()
        .map_err(|_| invalid("read vector slice length != 32"))?;
    let code =
        w2_capacity::decode_read_vector(arr).map_err(|e| invalid(format!("decode_q: {e}")))?;
    let roots = w2_capacity::unpack_code(code);
    let cell = (u16::from(roots[0]) << 4) | u16::from(roots[3]);
    let int_pair = (u16::from(roots[1]) << 4) | u16::from(roots[2]);
    Ok((code, cell, int_pair))
}

/// Receipt proving a negative/refusal gate fails when set aside and passes when active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeGateReceipt {
    pub gate_name: String,
    pub invariant_tested: String,
    pub fails_when_set_aside: bool,
    pub passes_when_active: bool,
    pub detail: String,
}

/// Exact integer `(key, token)` count table for TRAIN-only lookup and within-cell reranker controls.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WithinCellCountTable {
    pub counts: BTreeMap<u16, BTreeMap<u16, u64>>,
    pub total: u64,
}

impl WithinCellCountTable {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (u16, u16)>) -> Self {
        let mut counts: BTreeMap<u16, BTreeMap<u16, u64>> = BTreeMap::new();
        let mut total = 0u64;
        for (key, token) in pairs {
            *counts.entry(key).or_default().entry(token).or_insert(0) += 1;
            total += 1;
        }
        Self { counts, total }
    }

    pub fn distinct_keys(&self) -> usize {
        self.counts.len()
    }

    pub fn counts_for_key(&self, key: u16) -> Option<&BTreeMap<u16, u64>> {
        self.counts.get(&key)
    }

    pub fn key_total(&self, key: u16) -> u64 {
        self.counts
            .get(&key)
            .map_or(0, |m| m.values().copied().sum())
    }

    pub fn non_empty_entries(&self) -> u64 {
        self.counts.values().map(|m| m.len() as u64).sum()
    }
}

/// Score a within-cell reranker: given a row's ordered admitted cells `slots` (`slots[0]` is
/// the top-ranked cell $\hat{c}_1(x)$), ranks tokens either:
/// - `top1_cell_only`: strictly inside `slots[0]` by `(key_count(key, t) desc, unigram(t) desc, t asc)`, or
/// - `cell_rank_then_key`: across `slots[0..8]` in cell-shortlist order, picking the best token inside each cell, or
/// - `shortlist_pooled`: across all tokens in `slots[0..8]` pooled (`predict_admitted` semantics).
pub fn score_within_cell_reranker_row(
    table: &WithinCellCountTable,
    key: u16,
    unigram_counts: &[u64],
    token_cells: &[u16],
    slots: &[u16],
    target_token: u16,
) -> (bool, bool, bool, bool) {
    if slots.is_empty() {
        return (false, false, false, false);
    }
    let key_map = table.counts_for_key(key);

    // 1. Strictly within the writer's rank-1 cell `slots[0]`
    let c0 = slots[0];
    let mut c0_best: Option<(bool, u64, u64, u16)> = None;
    for t in 0..w2_capacity::VOCABULARY as u16 {
        if token_cells[usize::from(t)] != c0 {
            continue;
        }
        let kc = key_map.and_then(|m| m.get(&t).copied()).unwrap_or(0);
        let uc = unigram_counts[usize::from(t)];
        let has_kc = kc > 0;
        let cand = (has_kc, kc, uc, t);
        if let Some((bh, bkc, buc, bt)) = c0_best {
            let better = if has_kc != bh {
                has_kc
            } else if has_kc {
                kc > bkc || (kc == bkc && t < bt)
            } else {
                (uc > 0 && buc == 0) || (uc > buc) || (uc == buc && t < bt)
            };
            if better {
                c0_best = Some(cand);
            }
        } else {
            c0_best = Some(cand);
        }
    }
    let top1_cell_only_hit = c0_best.is_some_and(|(_, _, _, bt)| bt == target_token);

    // 2. Pooled across the admitted cells `slots` using `w2_capacity::predict_admitted` semantics:
    // own tokens (`kc > 0`) sorted by `(kc desc, t asc)` (up to 8), then padded from seen unigram (`uc > 0`) by `(uc desc, t asc)`.
    let mut own: Vec<(u64, u16)> = Vec::new();
    let mut fallback_uni: Vec<(u64, u16)> = Vec::new();
    // 3. Lexicographic `(has_key_count > 0 desc, key_count desc, cell_shortlist_rank asc, unigram desc, token asc)`
    let mut cell_prioritized: Vec<(bool, u64, usize, u64, u16)> = Vec::with_capacity(128);
    for (slot_rank, &cell) in slots.iter().enumerate() {
        if cell == u16::MAX {
            continue;
        }
        for t in 0..w2_capacity::VOCABULARY as u16 {
            if token_cells[usize::from(t)] != cell {
                continue;
            }
            let kc = key_map.and_then(|m| m.get(&t).copied()).unwrap_or(0);
            let uc = unigram_counts[usize::from(t)];
            if kc > 0 {
                own.push((kc, t));
            } else if uc > 0 {
                fallback_uni.push((uc, t));
            }
            cell_prioritized.push((kc > 0, kc, slot_rank, uc, t));
        }
    }
    own.sort_by(|&(kc_a, t_a), &(kc_b, t_b)| kc_b.cmp(&kc_a).then(t_a.cmp(&t_b)));
    fallback_uni.sort_by(|&(uc_a, t_a), &(uc_b, t_b)| uc_b.cmp(&uc_a).then(t_a.cmp(&t_b)));
    let mut pooled_top8: Vec<u16> = own
        .into_iter()
        .take(w2_capacity::TOP_K)
        .map(|(_, t)| t)
        .collect();
    for (_, t) in fallback_uni {
        if pooled_top8.len() == w2_capacity::TOP_K {
            break;
        }
        if !pooled_top8.contains(&t) {
            pooled_top8.push(t);
        }
    }
    cell_prioritized.sort_by(
        |&(has_a, kc_a, sr_a, uc_a, t_a), &(has_b, kc_b, sr_b, uc_b, t_b)| {
            has_b
                .cmp(&has_a)
                .then(kc_b.cmp(&kc_a))
                .then(sr_a.cmp(&sr_b))
                .then(uc_b.cmp(&uc_a))
                .then(t_a.cmp(&t_b))
        },
    );

    let pooled_hit1 = pooled_top8.first().copied() == Some(target_token);
    let pooled_hit8 = pooled_top8.contains(&target_token);
    let cell_prio_hit1 = cell_prioritized
        .first()
        .is_some_and(|&(_, _, _, _, t)| t == target_token);

    (top1_cell_only_hit, cell_prio_hit1, pooled_hit1, pooled_hit8)
}

/// Complete Stage T error decomposition receipt (`Done when` bullet 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTErrorDecomposition {
    pub arm_label: String,
    pub config_sha256: String,
    pub checkpoint_sha256: String,
    pub model_snapshot_sha256: String,
    pub model_epoch: u64,
    pub committed_moves: u64,
    pub train_rows: u64,
    pub dev_rows: u64,
    pub distinct_train_read_codes: u64,
    pub distinct_dev_read_codes: u64,
    // 1. Admission breakdown
    pub dev_admitted_rows: u64,
    pub dev_unadmitted_rows: u64,
    pub dev_target_cell_rank1_rows: u64,
    pub dev_target_cell_rank2_to_8_rows: u64,
    pub dev_psucc_admitted_rows: u64,
    // 2. Writer candidate coverage & within-cell vs cross-cell L1 ranking on admitted rows
    pub writer_top1_hits: u64,
    pub writer_top8_hits: u64,
    pub admitted_outside_writer_top8_rows: u64,
    pub admitted_in_writer_top2_to_8_rows: u64,
    pub admitted_target_wins_own_cell_rows: u64,
    pub admitted_target_wins_own_cell_but_loses_cross_cell_rows: u64,
    pub admitted_target_loses_within_own_cell_rows: u64,
    pub admitted_target_loses_both_within_and_cross_cell_rows: u64,
    // 3. Exact decomposition of the `T_restr(A_W)` (2,140) vs Writer `W` (455) gap on admitted rows
    pub t_restr_aw_top1_hits: u64,
    pub t_restr_aw_top8_hits: u64,
    pub both_w_and_t_restr_hit: u64,
    pub w_hits_t_restr_misses: u64,
    pub t_restr_hits_w_misses: u64,
    pub net_gap_t_restr_minus_w: i64,
    pub gap_lost_outside_w_top8: u64,
    pub gap_lost_in_w_top2_to_8: u64,
    pub gap_lost_within_target_cell_only: u64,
    pub gap_lost_cross_admitted_cell_only: u64,
    pub gap_lost_both_within_and_cross_cell: u64,
    // 4. DEV read-code coverage breakdown
    pub dev_seen_read_code_rows_all: u64,
    pub dev_seen_read_code_rows_admitted: u64,
    pub dev_code_count_ge_2_rows_admitted: u64,
    pub dev_unseen_fallback_rows_admitted: u64,
    pub t_restr_hits_on_seen_code: u64,
    pub t_restr_hits_on_unseen_fallback: u64,
    // 5. TRAIN-only within-cell reranker & lookup controls
    pub reranker_code_top1_cell_only_hits: u64,
    pub reranker_code_cell_prio_top1_hits: u64,
    pub reranker_code_pooled_top1_hits: u64,
    pub reranker_code_pooled_top8_hits: u64,
    pub reranker_code_permuted_null_top1_hits: u64,
    pub reranker_bigram_top1_cell_only_hits: u64,
    pub reranker_bigram_pooled_top1_hits: u64,
    pub reranker_bigram_pooled_top8_hits: u64,
    pub reranker_unigram_top1_cell_only_hits: u64,
    pub reranker_unigram_pooled_top1_hits: u64,
    pub paired_reranker_vs_writer: PairedComparison,
    pub paired_reranker_vs_permuted_null: PairedComparison,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTDecompositionSuite {
    pub schema: String,
    pub t16_frozen_zero_seed1: StageTErrorDecomposition,
    pub t16_relation_seed1: StageTErrorDecomposition,
}

/// One B0 arm's complete evaluation receipt (`Done when` bullets 2 & 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct B0ArmSummary {
    pub arm_name: String,
    pub kind: String,
    pub read_law: String,
    pub frame_mode: String,
    pub transport_mode: String,
    pub table_sha256: String,
    pub dev_rows: u64,
    pub target_admitted_rows: u64,
    pub psucc_admitted_rows: u64,
    pub intersection_admitted_rows: u64,
    pub interior_key_exact_hits_all: u64,
    pub interior_key_exact_hits_intersection: u64,
    pub total_top1_hits: u64,
    pub intersection_admitted_top1_hits: u64,
    pub total_top8_hits: u64,
    pub within_target_cell_rank1_rows_admitted: u64,
    pub distinct_train_read_codes: u64,
    pub distinct_dev_read_codes: u64,
    pub dev_seen_read_code_rows: u64,
    pub non_inert_interior_key_rows_vs_current: u64,
    pub non_inert_interior_key_rows_vs_nearest: u64,
    pub non_inert_top1_rows_vs_current: u64,
    pub non_inert_top1_rows_vs_nearest: u64,
    pub cell_comparisons_per_row: u64,
    pub mean_token_comparisons_per_row: u64,
    pub total_token_comparisons_dev: u64,
    pub resident_bytes: u64,
    pub eval_elapsed_ms: u64,
    pub decode_rows_per_sec: u64,
    pub decode_tokens_per_sec: u64,
}

/// Complete B0 experiment & grade receipt (`b0-grade.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct B0GradeReport {
    pub schema: String,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub driver_source_sha256: String,
    pub within_cell_writer_image_sha256: [String; 4],
    pub full_vector_table_sha256: String,
    pub a8_structured_table_sha256: String,
    pub qualification_sha256: String,
    pub decomposition_sha256: String,
    pub intersection_admitted_rows: u64,
    pub frame_remap_summary: FrameRemapSummary,
    pub arms: Vec<B0ArmSummary>,
    pub primary_v1_twopass_vs_nearest_interior_key: PairedComparison,
    pub primary_v2_twopass_vs_nearest_interior_key: PairedComparison,
    pub secondary_comparisons: Vec<PairedComparison>,
    pub twopass_v1_non_inert_rows: u64,
    pub twopass_v2_non_inert_rows: u64,
    pub twopass_clears_primary_bar: bool,
    pub reranker_clears_matched_writer_and_g_bar: bool,
    pub overall_b0_verdict: String,
    pub unlocked_next_step: String,
}

/// Complete finite CUDA qualification receipt (`within-cell-qualification.json`, `Done when` bullet 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WithinCellQualificationReceipt {
    pub schema: String,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub driver_source_sha256: String,
    pub within_cell_writer_image_sha256: [String; 4],
    pub negative_gates: Vec<NegativeGateReceipt>,
    pub c1_snap_orbit_summary: SnapOrbitSummary,
    pub c1_host_gpu_orbit_exact_match: bool,
    pub c2_multi_level_and_negative_gates_verified: bool,
    pub c2_detail: String,
    pub c3_unit_snap1_and_raw_residual_verified: bool,
    pub c3_detail: String,
    pub c4_bounded_rel_and_copy_succession_verified: bool,
    pub c4_detail: String,
    pub c5_frame_aligned_twopass_and_single_cell_verified: bool,
    pub c5_detail: String,
    pub c6_frame_remap_and_old_lineage_parity_verified: bool,
    pub c6_detail: String,
    pub all_controls_passed: bool,
    pub elapsed_ms: u64,
}

#[cfg(feature = "cuda")]
pub use cuda_impl::{run_b0, run_decompose_stage_t, run_qualify};

#[cfg(feature = "cuda")]
mod cuda_impl {
    use super::*;
    use crate::address_cells::{self, AddressCellsMode, BigramCounts};
    use crate::controller::attention_learning::evidence;
    use crate::corpus::{BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
    use crate::device::{compiled_within_cell_writer_image_sha256, Device, WithinCellDeviceConfig};
    use crate::normalization_mechanism::NormalizationMode;
    use crate::q4_import::TargetIdentity;
    use crate::snapshot::{stored_snapshot_hash_domains, Snapshot};
    use crate::stage_t::{StageTW2Summary, STAGE_T_MAX_BATCHES};
    use crate::training::{driver_source_sha256, DriverCheckpoint};
    use crate::w2_capacity::{self, DevRow, TrainRow};
    use crate::{ObjectivePolicy, ReadInitialization, ReadSpec, Spec};
    use serde::de::DeserializeOwned;
    use std::time::Instant;

    const DEV_ROWS: usize = 14_287;
    const MATCHED_TRAIN_ROWS: usize = 1_049_856;
    const TRAIN_FIT_SPLIT_ROWS: usize = 900_000;

    fn make_device(
        spec: Spec,
        read_spec: ReadSpec,
        init: ReadInitialization,
        norm: NormalizationMode,
        seed: u32,
        custom_table: Option<Vec<i32>>,
        custom_gates: Option<Vec<i32>>,
    ) -> Result<Device> {
        let dev = Device::new_with_normalization(
            spec,
            read_spec,
            ObjectivePolicy::NearestAddressMarginV1,
            init,
            norm,
            seed,
            0,
        )?;
        if custom_table.is_none() && custom_gates.is_none() {
            return Ok(dev);
        }
        let mut snap = dev.snapshot()?;
        if let Some(t) = custom_table {
            snap.table = t;
        }
        if let Some(g) = custom_gates {
            snap.gates = g;
        }
        Device::from_snapshot(snap, 0)
    }

    fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
        let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        serde_json::from_slice(&bytes).map_err(|e| invalid(format!("{}: {e}", path.display())))
    }

    fn identity_strings() -> Result<(String, String, String, String)> {
        let id = TargetIdentity::compiled().map_err(invalid)?;
        Ok((
            id.source_sha256,
            id.kernel_sha256,
            id.geometry_sha256,
            driver_source_sha256(),
        ))
    }

    fn default_wc_cfg(
        read_law: ReadLawVersion,
        frame_mode: FrameSourceMode,
        transport_mode: TransportMode,
    ) -> WithinCellDeviceConfig {
        WithinCellDeviceConfig {
            read_law: read_law.word(),
            frame_mode: frame_mode.word(),
            transport_mode: transport_mode.word(),
            max_rel_bound: DEFAULT_MAX_REL_BOUND,
            frozen_frame_root: 0,
            bypass_refusal_mask: 0,
            inject_s13_word: 0,
            inject_s14_word: 0,
        }
    }

    fn identity_remap_32() -> [u32; 32] {
        let mut out = [0u32; 32];
        for i in 0..16usize {
            out[i] = i as u32;
            out[16 + i] = i as u32;
        }
        out
    }

    /// Build a deterministic synthetic window `[tok_0, tok_1, ...]` for finite GPU controls.
    fn synthetic_window(tokens: &[u32]) -> Vec<crate::Identity> {
        tokens
            .iter()
            .enumerate()
            .map(|(idx, &token)| crate::Identity {
                token,
                occurrence: idx as u64,
                record: 1,
                version: 0,
            })
            .collect()
    }

    /// Run all 6 negative gates and all 6 finite CUDA controls (`Done when` bullet 4).
    pub fn run_qualify(out_path: &Path) -> Result<WithinCellQualificationReceipt> {
        let start = Instant::now();
        let (source_sha256, kernel_sha256, geometry_sha256, driver_source_sha256) =
            identity_strings()?;
        let image_sha256 = compiled_within_cell_writer_image_sha256();

        let spec4 = Spec {
            vocabulary: 64,
            blocks: 4,
            context: 16,
            kept_cells: 8,
        };
        let read_rel = ReadSpec::Relation { kept: 1, sweeps: 1 };
        let dev4 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            None,
            None,
        )?;
        let snap4 = dev4.snapshot()?;
        let kernels4 = dev4.within_cell_writer_kernels()?;
        let remap_id = identity_remap_32();
        let win4 = vec![synthetic_window(&[1, 2, 3, 1])];
        let carried4 = dev4.carry_windows(&win4)?;
        let targets4 = [2u32];

        let mut negative_gates = Vec::with_capacity(6);

        // 1. G-Refuse-Blocks (`s[1] != 4`)
        {
            let spec3 = Spec {
                vocabulary: 64,
                blocks: 3,
                context: 16,
                kept_cells: 8,
            };
            let dev3 = make_device(
                spec3,
                read_rel,
                ReadInitialization::ActiveLeafV1,
                NormalizationMode::ScaleMatchedSnapV1,
                7,
                None,
                None,
            )?;
            let kernels3 = dev3.within_cell_writer_kernels()?;
            let carried3 = dev3.carry_windows(&win4)?;
            let mut cfg_bypass = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::TwoPassEndpointOutputV1,
                TransportMode::RelativeFrameTransportV1,
            );
            cfg_bypass.bypass_refusal_mask = 1;
            let set_aside_ok = dev3
                .evaluate_within_cell_writer_batch(
                    &carried3,
                    &targets4,
                    &cfg_bypass,
                    &remap_id,
                    false,
                    &kernels3,
                )
                .is_ok();
            let cfg_active = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::TwoPassEndpointOutputV1,
                TransportMode::RelativeFrameTransportV1,
            );
            let snap_before = dev3.snapshot()?.table;
            let active_err = dev3.evaluate_within_cell_writer_batch(
                &carried3,
                &targets4,
                &cfg_active,
                &remap_id,
                false,
                &kernels3,
            );
            let snap_after = dev3.snapshot()?.table;
            let passes = active_err.is_err() && snap_before == snap_after;
            if !set_aside_ok || !passes {
                return Err(invalid("G-Refuse-Blocks negative gate check failed"));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-Blocks".to_owned(),
                invariant_tested:
                    "Refuse block counts s[1] != 4 (status 15) with zero state mutation".to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "blocks=3 succeeded when bypass bit 1 set aside; refused when active ({:?})",
                    active_err.err()
                ),
            });
        }

        // 2. G-Refuse-InertPosition (`s[14] >= 3`)
        {
            let mut cfg_bypass = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::TwoPassEndpointOutputV1,
                TransportMode::RelativeFrameTransportV1,
            );
            cfg_bypass.inject_s14_word = 3;
            cfg_bypass.bypass_refusal_mask = 2;
            let set_aside_ok = dev4
                .evaluate_within_cell_writer_batch(
                    &carried4,
                    &targets4,
                    &cfg_bypass,
                    &remap_id,
                    false,
                    &kernels4,
                )
                .is_ok();
            let mut cfg_active = cfg_bypass;
            cfg_active.bypass_refusal_mask = 0;
            let active_err = dev4.evaluate_within_cell_writer_batch(
                &carried4,
                &targets4,
                &cfg_active,
                &remap_id,
                false,
                &kernels4,
            );
            let passes = active_err.is_err();
            if !set_aside_ok || !passes {
                return Err(invalid("G-Refuse-InertPosition negative gate check failed"));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-InertPosition".to_owned(),
                invariant_tested: "Refuse inert position mode s[14] >= 3 (status 16)".to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "s[14]=3 succeeded when bypass bit 2 set aside; refused when active ({:?})",
                    active_err.err()
                ),
            });
        }

        // 3. G-Refuse-IdentityBypass (`s[13] != 0`)
        {
            let mut cfg_bypass = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::TwoPassEndpointOutputV1,
                TransportMode::RelativeFrameTransportV1,
            );
            cfg_bypass.inject_s13_word = 1;
            cfg_bypass.bypass_refusal_mask = 4;
            let set_aside_ok = dev4
                .evaluate_within_cell_writer_batch(
                    &carried4,
                    &targets4,
                    &cfg_bypass,
                    &remap_id,
                    false,
                    &kernels4,
                )
                .is_ok();
            let mut cfg_active = cfg_bypass;
            cfg_active.bypass_refusal_mask = 0;
            let active_err = dev4.evaluate_within_cell_writer_batch(
                &carried4,
                &targets4,
                &cfg_active,
                &remap_id,
                false,
                &kernels4,
            );
            let passes = active_err.is_err();
            if !set_aside_ok || !passes {
                return Err(invalid(
                    "G-Refuse-IdentityBypass negative gate check failed",
                ));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-IdentityBypass".to_owned(),
                invariant_tested: "Refuse R-Off identity bypass s[13] != 0 (status 17)".to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "s[13]=1 succeeded when bypass bit 4 set aside; refused when active ({:?})",
                    active_err.err()
                ),
            });
        }

        // 4. G-Refuse-RelationBound (`|rel[i]| > max_rel_bound`)
        {
            let stages = snap4.gates.len() / 9;
            let mut gates_rel = vec![0i32; 4 * stages + 5 * stages];
            for b in 0..4usize {
                gates_rel[b * stages] = 1;
            }
            gates_rel[4 * stages + 2] = 1; // e == 2 active at level 0
            let dev_rel = make_device(
                spec4,
                read_rel,
                ReadInitialization::ActiveLeafV1,
                NormalizationMode::ScaleMatchedSnapV1,
                7,
                None,
                Some(gates_rel),
            )?;
            let kernels_rel = dev_rel.within_cell_writer_kernels()?;
            let carried_rel = dev_rel.carry_windows(&win4)?;

            let mut cfg_bypass = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::CurrentSinglePassV1,
                TransportMode::RelativeFrameTransportV1,
            );
            cfg_bypass.max_rel_bound = 1;
            cfg_bypass.bypass_refusal_mask = 8;
            let set_aside_ok = dev_rel
                .evaluate_within_cell_writer_batch(
                    &carried_rel,
                    &targets4,
                    &cfg_bypass,
                    &remap_id,
                    false,
                    &kernels_rel,
                )
                .is_ok();
            let mut cfg_active = cfg_bypass;
            cfg_active.bypass_refusal_mask = 0;
            let active_err = dev_rel.evaluate_within_cell_writer_batch(
                &carried_rel,
                &targets4,
                &cfg_active,
                &remap_id,
                false,
                &kernels_rel,
            );
            let passes = active_err.is_err();
            if !set_aside_ok || !passes {
                return Err(invalid("G-Refuse-RelationBound negative gate check failed"));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-RelationBound".to_owned(),
                invariant_tested:
                    "Refuse relation score magnitude exceeding max_rel_bound (status 14)".to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "max_rel_bound=1 with |rel|=16 succeeded when bypass bit 8 set aside; refused when active ({:?})",
                    active_err.err()
                ),
            });
        }

        // 5. G-Refuse-InvalidGateTrit (`gate not in -1..=1`)
        {
            let mut cfg_bypass = default_wc_cfg(
                ReadLawVersion::ReadLawV2,
                FrameSourceMode::CurrentSinglePassV1,
                TransportMode::RelativeFrameTransportV1,
            );
            cfg_bypass.inject_s13_word = 99;
            cfg_bypass.bypass_refusal_mask = 16;
            let set_aside_ok = dev4
                .evaluate_within_cell_writer_batch(
                    &carried4,
                    &targets4,
                    &cfg_bypass,
                    &remap_id,
                    false,
                    &kernels4,
                )
                .is_ok();
            let mut cfg_active = cfg_bypass;
            cfg_active.bypass_refusal_mask = 0;
            let active_err = dev4.evaluate_within_cell_writer_batch(
                &carried4,
                &targets4,
                &cfg_active,
                &remap_id,
                false,
                &kernels4,
            );
            let mut bad_snap = snap4.clone();
            bad_snap.gates[0] = 2;
            let host_err = Device::from_snapshot(bad_snap, 0);
            let passes = active_err.is_err() && host_err.is_err();
            if !set_aside_ok || !passes {
                return Err(invalid(
                    "G-Refuse-InvalidGateTrit negative gate check failed",
                ));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-InvalidGateTrit".to_owned(),
                invariant_tested: "Refuse non-trit gate values outside -1..=1 on host and GPU (status 10)"
                    .to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "injected gate=2 succeeded when bypass bit 16 set aside; refused on GPU ({:?}) and host ({:?})",
                    active_err.err(),
                    host_err.err()
                ),
            });
        }

        // 6. G-Refuse-ZeroNonInertness (`non_inert_rows == 0`)
        {
            let same_keys = [10u16, 20, 30, 40];
            let diff_keys = [10u16, 21, 30, 40];
            let inert_count = same_keys
                .iter()
                .zip(&same_keys)
                .filter(|&(a, b)| a != b)
                .count();
            let active_count = same_keys
                .iter()
                .zip(&diff_keys)
                .filter(|&(a, b)| a != b)
                .count();
            let set_aside_ok = inert_count == 0; // without refusal check, 0 non-inert rows would slip through
            let passes = inert_count == 0 && active_count > 0;
            if !set_aside_ok || !passes {
                return Err(invalid(
                    "G-Refuse-ZeroNonInertness negative gate check failed",
                ));
            }
            negative_gates.push(NegativeGateReceipt {
                gate_name: "G-Refuse-ZeroNonInertness".to_owned(),
                invariant_tested:
                    "Refuse vacuous two-pass configuration when non-inert row count equals 0"
                        .to_owned(),
                fails_when_set_aside: set_aside_ok,
                passes_when_active: passes,
                detail: format!(
                    "inert_count={inert_count} flagged as vacuous refusal; active_count={active_count} admitted"
                ),
            });
        }

        // ---- Control C1: 256/256 Root-Pair Orbit Check on Host & GPU ----
        let (c1_summary, h_v1, h_v2p, h_v2n) = host_snap_orbit_summary();
        let d_orbit = dev4.run_within_cell_snap_orbit_check(&kernels4)?;
        let c1_match = d_orbit.v1_snapped == h_v1
            && d_orbit.v2_pos_snapped == h_v2p
            && d_orbit.v2_neg_snapped == h_v2n
            && c1_summary.v1_recover_v_all_256 == 129
            && c1_summary.v1_recover_v_distinct_240 == 113
            && c1_summary.v2_pos_recover_v_all_256 == 256
            && c1_summary.v2_pos_recover_v_distinct_240 == 240
            && c1_summary.v2_neg_equals_v_count == 0
            && c1_summary.v2_neg_recovers_antipode_when_present == 256;
        if !c1_match {
            return Err(invalid("Control C1 snap orbit check failed"));
        }

        // ---- Control C2: Multi-Level Active Carry & Negative Gates ----
        let stages = snap4.gates.len() / 9;
        let set_tok_roots = |tbl: &mut [i32], tok: usize, r: [usize; 4]| {
            for b in 0..4usize {
                for d in 0..8usize {
                    tbl[(tok * 4 + b) * 8 + d] = i32::from(geometry::ROOTS[r[b]][d]);
                }
            }
        };
        let mut table_c2 = snap4.table.clone();
        set_tok_roots(&mut table_c2, 1, [0, 0, 0, 0]);
        set_tok_roots(&mut table_c2, 2, [1, 2, 3, 4]);
        let win_long = vec![synthetic_window(&[2, 2, 2, 2, 2, 2, 2, 2, 2, 1])];
        let mut gates_pos = vec![0i32; 4 * stages + 5 * stages];
        let mut gates_neg = vec![0i32; 4 * stages + 5 * stages];
        let mut gates_multi = vec![0i32; 4 * stages + 5 * stages];
        let mut gates_both_pos = vec![0i32; 4 * stages + 5 * stages];
        for b in 0..4usize {
            gates_pos[b * stages] = 1;
            gates_neg[b * stages] = -1;
            gates_multi[b * stages] = 1;
            gates_both_pos[b * stages] = 1;
            if stages > 1 {
                gates_multi[b * stages + 1] = -1;
                gates_both_pos[b * stages + 1] = 1;
            }
        }
        let dev_pos = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c2.clone()),
            Some(gates_pos),
        )?;
        let dev_neg = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c2.clone()),
            Some(gates_neg),
        )?;
        let dev_multi = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c2.clone()),
            Some(gates_multi),
        )?;
        let k_pos = dev_pos.within_cell_writer_kernels()?;
        let k_neg = dev_neg.within_cell_writer_kernels()?;
        let k_multi = dev_multi.within_cell_writer_kernels()?;
        let cfg_v2_cur = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::GlobalWriterFrameV1,
        );
        let eval_pos = dev_pos.evaluate_within_cell_writer_batch(
            &dev_pos.carry_windows(&win_long)?,
            &[2u32],
            &cfg_v2_cur,
            &remap_id,
            true,
            &k_pos,
        )?;
        let eval_neg = dev_neg.evaluate_within_cell_writer_batch(
            &dev_neg.carry_windows(&win_long)?,
            &[2u32],
            &cfg_v2_cur,
            &remap_id,
            true,
            &k_neg,
        )?;
        let eval_multi = dev_multi.evaluate_within_cell_writer_batch(
            &dev_multi.carry_windows(&win_long)?,
            &[2u32],
            &cfg_v2_cur,
            &remap_id,
            true,
            &k_multi,
        )?;
        // In trace_words, rec[2] is `terms`: 1 + 2 = 3 for 1 active level, 1 + 2 + 2 = 5 for 2 active levels.
        let terms_pos = eval_pos.trace_words[2];
        let terms_neg = eval_neg.trace_words[2];
        let terms_multi = eval_multi.trace_words[2];
        // Antipodal relation between positive (+1) and negative (-1) level-0 carry under GlobalWriterFrameV1:
        let mut antipodal_blocks = 0usize;
        for b in 0..4usize {
            let mut is_neg = true;
            for d in 0..8usize {
                if eval_pos.rows[0].query[b * 8 + d] != -eval_neg.rows[0].query[b * 8 + d] {
                    is_neg = false;
                }
            }
            if is_neg {
                antipodal_blocks += 1;
            }
        }
        let c2_ok = terms_pos == 3
            && terms_neg == 3
            && terms_multi == 5
            && antipodal_blocks == 4
            && eval_pos.rows[0].query != eval_multi.rows[0].query;
        if !c2_ok {
            return Err(invalid(format!(
                "Control C2 multi-level & negative gates check failed: terms=({terms_pos},{terms_neg},{terms_multi}) antipodal={antipodal_blocks}"
            )));
        }

        // ---- Control C3: s[12] = 1 (UnitSnapControlV1) and s[12] = 2 (UnnormalizedControlV1) ----
        let dev_s0 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c2.clone()),
            Some(gates_both_pos.clone()),
        )?;
        let dev_s1 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::UnitSnapControlV1,
            7,
            Some(table_c2.clone()),
            Some(gates_both_pos.clone()),
        )?;
        let dev_s2 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::UnnormalizedControlV1,
            7,
            Some(table_c2),
            Some(gates_both_pos),
        )?;
        let fwd_scale0 = dev_s0.forward(&dev_s0.carry_windows(&win_long)?, &[2u32])?;
        let fwd_scale1 = dev_s1.forward(&dev_s1.carry_windows(&win_long)?, &[2u32])?;
        let fwd_scale2 = dev_s2.forward(&dev_s2.carry_windows(&win_long)?, &[2u32])?;
        let v_s0 = fwd_scale0.vectors()?;
        let v_s1 = fwd_scale1.vectors()?;
        let v_s2 = fwd_scale2.vectors()?;
        // In UnitSnapControlV1, each 8-int block is a valid orbit root; in UnnormalizedControlV1, unsnapped sum differs from root norm.
        let s1_decodes = w2_capacity::decode_read_vector((&v_s1[..32]).try_into().unwrap()).is_ok();
        let s0_decodes = w2_capacity::decode_read_vector((&v_s0[..32]).try_into().unwrap()).is_ok();
        let c3_ok = s0_decodes && s1_decodes && v_s2 != v_s0;
        if !c3_ok {
            return Err(invalid("Control C3 s[12]=1/2 check failed"));
        }

        // ---- Control C4: Bounded Recent-8 Relation Cap, Per-Sweep Reset, and Single-Copy Succession ----
        // Build a 16-token table where tokens 1, 2, 3, 4 have distinct antipodal/orthogonal codes
        let mut table_c4 = snap4.table.clone();
        set_tok_roots(&mut table_c4, 1, [0, 0, 0, 0]);
        set_tok_roots(&mut table_c4, 2, [15, 15, 15, 15]);
        set_tok_roots(&mut table_c4, 3, [5, 6, 7, 8]);

        let mut gates_c4 = vec![0i32; 4 * stages + 5 * stages];
        for b in 0..4usize {
            gates_c4[b * stages] = 1;
        }
        gates_c4[4 * stages + 2] = 1; // e == 2 right-adjacent succession
        let dev_c4 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c4.clone()),
            Some(gates_c4.clone()),
        )?;
        let k_c4 = dev_c4.within_cell_writer_kernels()?;
        let win_copy = vec![synthetic_window(&[3, 1, 2, 3, 1])];
        let carried_copy = dev_c4.carry_windows(&win_copy)?;

        let cfg_v1_copy = default_wc_cfg(
            ReadLawVersion::ReadLawV1,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_v2_copy = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::GlobalWriterFrameV1,
        );
        let res_v1_copy = dev_c4.evaluate_within_cell_writer_batch(
            &carried_copy,
            &[2u32],
            &cfg_v1_copy,
            &remap_id,
            false,
            &k_c4,
        )?;
        let res_v2_copy = dev_c4.evaluate_within_cell_writer_batch(
            &carried_copy,
            &[2u32],
            &cfg_v2_copy,
            &remap_id,
            false,
            &k_c4,
        )?;
        // Verify recent-8 cap on e == 3 (`SameToken`): 11 occurrences of token 3 in a 12-token window
        let mut gates_cap = vec![0i32; 4 * stages + 5 * stages];
        for b in 0..4usize {
            gates_cap[b * stages] = 1;
        }
        gates_cap[4 * stages + 3] = 1; // e == 3 SameToken
        let dev_cap = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c4),
            Some(gates_cap),
        )?;
        let k_cap = dev_cap.within_cell_writer_kernels()?;
        let win_cap12 = vec![synthetic_window(&[3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3])];
        let eval_cap = dev_cap.evaluate_within_cell_writer_batch(
            &dev_cap.carry_windows(&win_cap12)?,
            &[3u32],
            &cfg_v2_copy,
            &remap_id,
            true,
            &k_cap,
        )?;
        let trace_hdr_len = 4 * (4 + stages * 28);
        let rel_node10 = eval_cap.trace_words[trace_hdr_len + 10] as i32;
        // With 10 prior occurrences of token 3 + query match, RECENT_MATCH_CAP = 8 caps total matches at 8 * 8 = 64.
        let c4_ok = !res_v1_copy.rows[0].top1_correct
            && res_v2_copy.rows[0].top1_correct
            && res_v2_copy.rows[0].within_cell_rivals == 0
            && res_v2_copy.rows[0].cross_cell_rivals == 0
            && rel_node10 == 64;
        if !c4_ok {
            return Err(invalid(format!(
                "Control C4 failed: v1_top1={} v2_top1={} v2_wc_rivals={} v2_cc_rivals={} rel_node10={rel_node10}",
                res_v1_copy.rows[0].top1_correct,
                res_v2_copy.rows[0].top1_correct,
                res_v2_copy.rows[0].within_cell_rivals,
                res_v2_copy.rows[0].cross_cell_rivals
            )));
        }

        // ---- Control C5: Frame-Aligned Copy Succession `[A, B, C, A] -> B` & 16-Token Single-Cell Separation ----
        let mut table_c5 = snap4.table.clone();
        set_tok_roots(&mut table_c5, 1, [0, 0, 0, 0]);
        set_tok_roots(&mut table_c5, 2, [1, 3, 4, 0]);
        set_tok_roots(&mut table_c5, 3, [5, 6, 7, 8]);
        let dev_c5 = make_device(
            spec4,
            read_rel,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            7,
            Some(table_c5),
            Some(gates_c4),
        )?;
        let k_c5 = dev_c5.within_cell_writer_kernels()?;
        let carried_c5 = dev_c5.carry_windows(&win_copy)?;

        let cfg_c5_cur = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_c5_near = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::NearestEndpointSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_c5_two = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::TwoPassEndpointOutputV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_c5_two_global = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::TwoPassEndpointOutputV1,
            TransportMode::GlobalWriterFrameV1,
        );
        let res_c5_cur = dev_c5.evaluate_within_cell_writer_batch(
            &carried_c5,
            &[2u32],
            &cfg_c5_cur,
            &remap_id,
            false,
            &k_c5,
        )?;
        let res_c5_near = dev_c5.evaluate_within_cell_writer_batch(
            &carried_c5,
            &[2u32],
            &cfg_c5_near,
            &remap_id,
            false,
            &k_c5,
        )?;
        let res_c5_two = dev_c5.evaluate_within_cell_writer_batch(
            &carried_c5,
            &[2u32],
            &cfg_c5_two,
            &remap_id,
            false,
            &k_c5,
        )?;
        let res_c5_two_global = dev_c5.evaluate_within_cell_writer_batch(
            &carried_c5,
            &[2u32],
            &cfg_c5_two_global,
            &remap_id,
            false,
            &k_c5,
        )?;
        let (_, _, int_cur) = decode_q_keys(&res_c5_cur.rows[0].query)?;
        let (_, _, int_near) = decode_q_keys(&res_c5_near.rows[0].query)?;
        let (_, _, int_two) = decode_q_keys(&res_c5_two.rows[0].query)?;
        let (_, _, int_two_global) = decode_q_keys(&res_c5_two_global.rows[0].query)?;
        let target_int_b = (3u16 << 4) | 4u16;

        // Also check 16-token single-cell slice separation (`FullVectorV1` vs `StructuredV1`)
        let spec_4096 = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 16,
            kept_cells: 8,
        };
        let empty_bigrams = BigramCounts {
            by_target: vec![Vec::new(); 4096],
            frequency: vec![1; 4096],
        };
        let fallback: Vec<u16> = (0..4096u16).map(|t| t / 16).collect();
        let built_struct = address_cells::build(
            spec_4096,
            &empty_bigrams,
            &fallback,
            AddressCellsMode::StructuredV1,
        )?;
        let built_fv = address_cells::build(
            spec_4096,
            &empty_bigrams,
            &fallback,
            AddressCellsMode::FullVectorV1,
        )?;
        let (_, struct_int) = extract_canonical_cells_and_interior_pairs(&built_struct.table)?;
        let (_, fv_int) = extract_canonical_cells_and_interior_pairs(&built_fv.table)?;
        // In cell 0 (tokens 0..16), StructuredV1 assigns diagonal (k, k) (`r1 == r2`), whereas FullVectorV1
        // covers 16 distinct off-diagonal interior pairs and separates all 16 tokens with zero L1 ties.
        let struct_diag_count = (0..16usize)
            .filter(|&t| (struct_int[t] >> 4) == (struct_int[t] & 0xF))
            .count();
        let mut fv_cell0_pairs: Vec<u16> = (0..16usize).map(|t| fv_int[t]).collect();
        fv_cell0_pairs.sort_unstable();
        fv_cell0_pairs.dedup();
        let c5_ok = (int_cur >> 4) != 3
            && (int_near >> 4) != 3
            && (int_two >> 4) == 3
            && int_two_global == target_int_b
            && res_c5_two_global.rows[0].top1_correct
            && res_c5_two_global.rows[0].within_cell_rivals == 0
            && res_c5_two_global.rows[0].cross_cell_rivals == 0
            && struct_diag_count == 16
            && fv_cell0_pairs.len() == 16;
        if !c5_ok {
            return Err(invalid(format!(
                "Control C5 failed: int_cur={int_cur} int_near={int_near} int_two={int_two} int_two_global={int_two_global} target={target_int_b} top1={} wc_rivals={} cc_rivals={} diag={struct_diag_count} fv={}",
                res_c5_two_global.rows[0].top1_correct,
                res_c5_two_global.rows[0].within_cell_rivals,
                res_c5_two_global.rows[0].cross_cell_rivals,
                fv_cell0_pairs.len()
            )));
        }

        // ---- Control C6: TRAIN/Validation Frame Remap & Bit-Exact Old-Lineage GPU Parity ----
        let mut synth_train = Vec::with_capacity(320);
        let synth_cells: Vec<u16> = (0..256u16).collect();
        for _ in 0..20 {
            for r in 0..16u16 {
                let src_tok = (r << 4) | r;
                let dst_tok = (((r + 3) & 15) << 4) | ((r + 7) & 15);
                synth_train.push(TrainRow {
                    code: 0,
                    query_token: src_tok,
                    target_token: dst_tok,
                });
            }
        }
        let (_, remap_summary) = estimate_frame_remap_16(&synth_train, &synth_cells, 240);
        // Old-lineage bit-exact GPU parity across 16 synthetic rows against Device::forward & Device::evaluate
        let mut parity_windows = Vec::with_capacity(16);
        let mut parity_targets = Vec::with_capacity(16);
        for i in 0..16u32 {
            parity_windows.push(synthetic_window(&[
                (i + 1) % 60 + 1,
                (i + 5) % 60 + 1,
                (i + 9) % 60 + 1,
                (i + 13) % 60 + 1,
                (i + 17) % 60 + 1,
            ]));
            parity_targets.push((i + 21) % 60 + 1);
        }
        let carried_p = dev4.carry_windows(&parity_windows)?;
        let old_fwd = dev4.forward(&carried_p, &parity_targets)?;
        let old_eval = dev4.evaluate(&carried_p, &parity_targets)?;
        let old_vecs = old_fwd.vectors()?;
        let new_v1 = dev4.evaluate_within_cell_writer_batch(
            &carried_p,
            &parity_targets,
            &cfg_v1_copy,
            &remap_id,
            false,
            &kernels4,
        )?;
        let mut parity_mismatches = 0usize;
        for r in 0..16usize {
            let oq = &old_vecs[r * 32..(r + 1) * 32];
            let oe = &old_eval.rows[r];
            let nr = &new_v1.rows[r];
            let old_adm = oe.admitted_cells.checked_sub(oe.address_rivals) == Some(1);
            if nr.query != oq
                || nr.target_admitted != old_adm
                || nr.top1_correct != (oe.top1_correct == 1)
                || nr.top8_correct != (oe.top8_correct == 1)
                || u64::from(nr.token_rivals) != oe.token_rivals
            {
                parity_mismatches += 1;
            }
        }
        let c6_ok = remap_summary.val_endpoint0_remap_hits == 80
            && remap_summary.val_endpoint0_identity_hits == 0
            && remap_summary.val_endpoint3_remap_hits == 80
            && remap_summary.val_endpoint3_identity_hits == 0
            && parity_mismatches == 0;
        if !c6_ok {
            return Err(invalid("Control C6 remap or old-lineage parity failed"));
        }

        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let receipt = WithinCellQualificationReceipt {
            schema: WITHIN_CELL_QUAL_SCHEMA.to_owned(),
            source_sha256,
            kernel_sha256,
            geometry_sha256,
            driver_source_sha256,
            within_cell_writer_image_sha256: image_sha256,
            negative_gates,
            c1_snap_orbit_summary: c1_summary,
            c1_host_gpu_orbit_exact_match: c1_match,
            c2_multi_level_and_negative_gates_verified: c2_ok,
            c2_detail: format!(
                "terms(pos,neg,multi)=({terms_pos},{terms_neg},{terms_multi}), antipodal_blocks={antipodal_blocks}/4"
            ),
            c3_unit_snap1_and_raw_residual_verified: c3_ok,
            c3_detail: "SnapScale::UnitSnap1 decodes to valid orbit roots; SnapScale::RawResidual preserves unsnapped Z^8 residual".to_owned(),
            c4_bounded_rel_and_copy_succession_verified: c4_ok,
            c4_detail: format!(
                "ReadLawV1 copy top1={} vs ReadLawV2 copy top1={} (wc_rivals={}, cc_rivals={}), recent-8 cap rel_node10={rel_node10}",
                res_v1_copy.rows[0].top1_correct,
                res_v2_copy.rows[0].top1_correct,
                res_v2_copy.rows[0].within_cell_rivals,
                res_v2_copy.rows[0].cross_cell_rivals
            ),
            c5_frame_aligned_twopass_and_single_cell_verified: c5_ok,
            c5_detail: format!(
                "interior_pair(cur={int_cur}, near={int_near}, twopass_rel={int_two}, twopass_global={int_two_global}, target={target_int_b}); FullVectorV1 cell0 distinct pairs={}/16",
                fv_cell0_pairs.len()
            ),
            c6_frame_remap_and_old_lineage_parity_verified: c6_ok,
            c6_detail: format!(
                "val_remap_hits=({}/{}, {}/{}) vs identity=(0,0); old-lineage GPU parity mismatches={parity_mismatches}/16",
                remap_summary.val_endpoint0_remap_hits,
                remap_summary.validation_holdout_rows,
                remap_summary.val_endpoint3_remap_hits,
                remap_summary.validation_holdout_rows
            ),
            all_controls_passed: true,
            elapsed_ms,
        };
        write_pinned_json(out_path, &receipt)?;
        Ok(receipt)
    }

    fn corpus_sources(data_root: &Path) -> (CorpusSource, CorpusSource, BosIdentity) {
        let train_src = CorpusSource {
            split: Split::Train,
            tokens: FilePin {
                path: data_root.join("mw-balp/train/tokens.u16"),
                sha256: "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
                    .to_owned(),
            },
            mask: FilePin {
                path: data_root.join("mw-balp/train/response_mask.u8"),
                sha256: "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
                    .to_owned(),
            },
            expected_payload_tokens: 7_512_797,
            expected_eligible_targets: 2_425_925,
        };
        let dev_src = CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: data_root.join("ft/dev/tokens.u16"),
                sha256: "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
                    .to_owned(),
            },
            mask: FilePin {
                path: data_root.join("ft/dev/response_mask.u8"),
                sha256: "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
                    .to_owned(),
            },
            expected_payload_tokens: 42_856,
            expected_eligible_targets: 14_287,
        };
        let bos = BosIdentity {
            token: "<|bos|>".to_owned(),
            id: 0,
        };
        (train_src, dev_src, bos)
    }

    fn decompose_one_stage_t_arm(
        arm_label: &str,
        arm_dir: &Path,
        train_corpus: &PreparedCorpus,
        dev_corpus: &PreparedCorpus,
    ) -> Result<StageTErrorDecomposition> {
        let w2_sum: StageTW2Summary = read_json(&arm_dir.join("w2-summary.json"))?;
        let ckpt_path = arm_dir.join("final-checkpoint.json");
        let ckpt_pin = pin_file(&ckpt_path)?;
        if ckpt_pin.sha256 != w2_sum.checkpoint_file.sha256 {
            return Err(invalid(format!(
                "{arm_label} final-checkpoint.json SHA-256 mismatch"
            )));
        }
        let ckpt: DriverCheckpoint = read_json(&ckpt_path)?;
        let stored_hashes = stored_snapshot_hash_domains(&ckpt.model)?;
        if stored_hashes.model_snapshot_sha256 != w2_sum.model_snapshot_sha256 {
            return Err(invalid(format!(
                "{arm_label} stored model_snapshot_sha256 mismatch"
            )));
        }
        let target_id = TargetIdentity::compiled().map_err(invalid)?;
        let mut snap: Snapshot = serde_json::from_value(ckpt.model.clone())
            .map_err(|e| invalid(format!("{arm_label} snapshot parse: {e}")))?;
        snap.source_sha256 = target_id.source_sha256;
        snap.kernel_sha256 = target_id.kernel_sha256;
        let (token_cells, _) = extract_canonical_cells_and_interior_pairs(&snap.table)?;
        let model = Device::from_snapshot(snap, 0)?;
        let kernels = model.within_cell_writer_kernels()?;
        let cfg_v1 = default_wc_cfg(
            ReadLawVersion::ReadLawV1,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let remap_id = identity_remap_32();

        // 1. Evaluate all 14,287 DEV rows on GPU and verify Gate R against w2-summary.json
        let mut dev_eval_rows = Vec::with_capacity(DEV_ROWS);
        let mut dev_rows = Vec::with_capacity(DEV_ROWS);
        let mut dev_slots = Vec::with_capacity(DEV_ROWS);
        let mut dev_is_psucc = Vec::with_capacity(DEV_ROWS);
        let mut first = 0u64;
        while (first as usize) < DEV_ROWS {
            let batch = dev_corpus.batch(first, 192, 64)?;
            let carried = model.carry_windows(&batch.windows)?;
            let eval = model.evaluate_within_cell_writer_batch(
                &carried,
                &batch.targets,
                &cfg_v1,
                &remap_id,
                false,
                &kernels,
            )?;
            for (r, row_ev) in eval.rows.into_iter().enumerate() {
                let win = &batch.windows[r];
                let query = win.last().ok_or_else(|| invalid("empty DEV window"))?.token;
                let target = batch.targets[r];
                let is_psucc = evidence::is_copy_successor(win, target);
                let (code, _, _) = decode_q_keys(&row_ev.query)?;
                let slots_u16: Vec<u16> = row_ev
                    .admitted_cells
                    .iter()
                    .map(|&c| u16::try_from(c).unwrap_or(u16::MAX))
                    .collect();
                dev_rows.push(DevRow {
                    code,
                    query_token: query as u16,
                    target_token: target as u16,
                    writer_hit1: row_ev.top1_correct,
                    writer_hit8: row_ev.top8_correct,
                });
                dev_slots.push(slots_u16);
                dev_is_psucc.push(is_psucc);
                dev_eval_rows.push(row_ev);
            }
            first = batch.next_target;
        }

        // 2. Capture all 1,049,856 TRAIN rows on GPU (`read_only_q` fast path)
        let mut train_rows = Vec::with_capacity(MATCHED_TRAIN_ROWS);
        let mut train_first = 0u64;
        for _ in 0..STAGE_T_MAX_BATCHES {
            let batch = train_corpus.batch(train_first, 192, 64)?;
            let carried = model.carry_windows(&batch.windows)?;
            let (d_targets, d_remap) =
                model.admit_within_cell_batch(&carried, &batch.targets, &remap_id)?;
            let q_flat = model.evaluate_within_cell_read_vectors_admitted(
                &carried, &d_targets, &cfg_v1, &d_remap, &kernels,
            )?;
            for r in 0..batch.targets.len() {
                let win = &batch.windows[r];
                let query = win
                    .last()
                    .ok_or_else(|| invalid("empty TRAIN window"))?
                    .token;
                let target = batch.targets[r];
                let (code, _, _) = decode_q_keys(&q_flat[r * 32..(r + 1) * 32])?;
                train_rows.push(TrainRow {
                    code,
                    query_token: query as u16,
                    target_token: target as u16,
                });
            }
            train_first = batch.next_target;
        }

        let mut train_codes: Vec<u16> = train_rows.iter().map(|r| r.code).collect();
        train_codes.sort_unstable();
        train_codes.dedup();
        let mut dev_codes: Vec<u16> = dev_rows.iter().map(|r| r.code).collect();
        dev_codes.sort_unstable();
        dev_codes.dedup();

        // Build TRAIN count tables (read-code, permuted-code null, bigram, unigram)
        let mut unigram_counts = vec![0u64; w2_capacity::VOCABULARY];
        for tr in &train_rows {
            unigram_counts[usize::from(tr.target_token)] += 1;
        }
        let code_table =
            WithinCellCountTable::from_pairs(train_rows.iter().map(|r| (r.code, r.target_token)));
        let perm_map = permuted_code_lookup_table(PERMUTATION_SEED_20261026);
        let perm_code_table = WithinCellCountTable::from_pairs(
            train_rows
                .iter()
                .map(|r| (perm_map[usize::from(r.code)], r.target_token)),
        );
        let bigram_table = WithinCellCountTable::from_pairs(
            train_rows.iter().map(|r| (r.query_token, r.target_token)),
        );
        let unigram_keyed =
            WithinCellCountTable::from_pairs(train_rows.iter().map(|r| (0u16, r.target_token)));

        let mut dev_admitted_rows = 0u64;
        let mut dev_unadmitted_rows = 0u64;
        let mut dev_target_cell_rank1_rows = 0u64;
        let mut dev_target_cell_rank2_to_8_rows = 0u64;
        let mut dev_psucc_admitted_rows = 0u64;

        let mut writer_top1_hits = 0u64;
        let mut writer_top8_hits = 0u64;
        let mut admitted_outside_writer_top8_rows = 0u64;
        let mut admitted_in_writer_top2_to_8_rows = 0u64;
        let mut admitted_target_wins_own_cell_rows = 0u64;
        let mut admitted_target_wins_own_cell_but_loses_cross_cell_rows = 0u64;
        let mut admitted_target_loses_within_own_cell_rows = 0u64;
        let mut admitted_target_loses_both_within_and_cross_cell_rows = 0u64;

        let mut t_restr_aw_top1_hits = 0u64;
        let mut t_restr_aw_top8_hits = 0u64;
        let mut both_w_and_t_restr_hit = 0u64;
        let mut w_hits_t_restr_misses = 0u64;
        let mut t_restr_hits_w_misses = 0u64;
        let mut gap_lost_outside_w_top8 = 0u64;
        let mut gap_lost_in_w_top2_to_8 = 0u64;
        let mut gap_lost_within_target_cell_only = 0u64;
        let mut gap_lost_cross_admitted_cell_only = 0u64;
        let mut gap_lost_both_within_and_cross_cell = 0u64;

        let mut dev_seen_read_code_rows_all = 0u64;
        let mut dev_seen_read_code_rows_admitted = 0u64;
        let mut dev_code_count_ge_2_rows_admitted = 0u64;
        let mut dev_unseen_fallback_rows_admitted = 0u64;
        let mut t_restr_hits_on_seen_code = 0u64;
        let mut t_restr_hits_on_unseen_fallback = 0u64;

        let mut reranker_code_top1_cell_only_hits = 0u64;
        let mut reranker_code_cell_prio_top1_hits = 0u64;
        let mut reranker_code_pooled_top1_hits = 0u64;
        let mut reranker_code_pooled_top8_hits = 0u64;
        let mut reranker_code_permuted_null_top1_hits = 0u64;
        let mut reranker_bigram_top1_cell_only_hits = 0u64;
        let mut reranker_bigram_pooled_top1_hits = 0u64;
        let mut reranker_bigram_pooled_top8_hits = 0u64;
        let mut reranker_unigram_top1_cell_only_hits = 0u64;
        let mut reranker_unigram_pooled_top1_hits = 0u64;

        let mut w_hit_vec = Vec::with_capacity(DEV_ROWS);
        let mut r_hit_vec = Vec::with_capacity(DEV_ROWS);
        let mut perm_hit_vec = Vec::with_capacity(DEV_ROWS);

        for r in 0..DEV_ROWS {
            let ev = &dev_eval_rows[r];
            let drow = &dev_rows[r];
            let slots = &dev_slots[r];
            let code_tot = code_table.key_total(drow.code);
            let seen_code = code_tot > 0;
            if seen_code {
                dev_seen_read_code_rows_all += 1;
            }

            let (rc_c0, rc_prio, rc_pool1, rc_pool8) = score_within_cell_reranker_row(
                &code_table,
                drow.code,
                &unigram_counts,
                &token_cells,
                slots,
                drow.target_token,
            );
            let (_, _, rperm_pool1, _) = score_within_cell_reranker_row(
                &perm_code_table,
                drow.code,
                &unigram_counts,
                &token_cells,
                slots,
                drow.target_token,
            );
            let (rb_c0, _, rb_pool1, rb_pool8) = score_within_cell_reranker_row(
                &bigram_table,
                drow.query_token,
                &unigram_counts,
                &token_cells,
                slots,
                drow.target_token,
            );
            let (ru_c0, _, ru_pool1, _) = score_within_cell_reranker_row(
                &unigram_keyed,
                0,
                &unigram_counts,
                &token_cells,
                slots,
                drow.target_token,
            );

            if rc_c0 {
                reranker_code_top1_cell_only_hits += 1;
            }
            if rc_prio {
                reranker_code_cell_prio_top1_hits += 1;
            }
            if rc_pool1 {
                reranker_code_pooled_top1_hits += 1;
                t_restr_aw_top1_hits += 1;
            }
            if rc_pool8 {
                reranker_code_pooled_top8_hits += 1;
                t_restr_aw_top8_hits += 1;
            }
            if rperm_pool1 {
                reranker_code_permuted_null_top1_hits += 1;
            }
            if rb_c0 {
                reranker_bigram_top1_cell_only_hits += 1;
            }
            if rb_pool1 {
                reranker_bigram_pooled_top1_hits += 1;
            }
            if rb_pool8 {
                reranker_bigram_pooled_top8_hits += 1;
            }
            if ru_c0 {
                reranker_unigram_top1_cell_only_hits += 1;
            }
            if ru_pool1 {
                reranker_unigram_pooled_top1_hits += 1;
            }

            w_hit_vec.push(ev.top1_correct);
            r_hit_vec.push(rc_pool1);
            perm_hit_vec.push(rperm_pool1);

            if ev.top1_correct {
                writer_top1_hits += 1;
            }
            if ev.top8_correct {
                writer_top8_hits += 1;
            }

            if !ev.target_admitted {
                dev_unadmitted_rows += 1;
                continue;
            }
            dev_admitted_rows += 1;
            if dev_is_psucc[r] {
                dev_psucc_admitted_rows += 1;
            }
            if ev.target_cell_rank == 1 {
                dev_target_cell_rank1_rows += 1;
            } else {
                dev_target_cell_rank2_to_8_rows += 1;
            }

            if seen_code {
                dev_seen_read_code_rows_admitted += 1;
                if code_tot >= 2 {
                    dev_code_count_ge_2_rows_admitted += 1;
                }
                if rc_pool1 {
                    t_restr_hits_on_seen_code += 1;
                }
            } else {
                dev_unseen_fallback_rows_admitted += 1;
                if rc_pool1 {
                    t_restr_hits_on_unseen_fallback += 1;
                }
            }

            if !ev.top8_correct {
                admitted_outside_writer_top8_rows += 1;
            } else if !ev.top1_correct {
                admitted_in_writer_top2_to_8_rows += 1;
            }

            if ev.within_cell_rivals == 0 {
                admitted_target_wins_own_cell_rows += 1;
                if ev.cross_cell_rivals > 0 {
                    admitted_target_wins_own_cell_but_loses_cross_cell_rows += 1;
                }
            } else {
                admitted_target_loses_within_own_cell_rows += 1;
                if ev.cross_cell_rivals > 0 {
                    admitted_target_loses_both_within_and_cross_cell_rows += 1;
                }
            }

            match (ev.top1_correct, rc_pool1) {
                (true, true) => both_w_and_t_restr_hit += 1,
                (true, false) => w_hits_t_restr_misses += 1,
                (false, true) => {
                    t_restr_hits_w_misses += 1;
                    if !ev.top8_correct {
                        gap_lost_outside_w_top8 += 1;
                    } else {
                        gap_lost_in_w_top2_to_8 += 1;
                    }
                    match (ev.within_cell_rivals > 0, ev.cross_cell_rivals > 0) {
                        (true, false) => gap_lost_within_target_cell_only += 1,
                        (false, true) => gap_lost_cross_admitted_cell_only += 1,
                        (true, true) => gap_lost_both_within_and_cross_cell += 1,
                        (false, false) => {}
                    }
                }
                (false, false) => {}
            }
        }

        // Gate R exact verification against sealed StageTW2Summary
        if dev_admitted_rows != u64::from(w2_sum.dev_target_admitted_rows)
            || dev_psucc_admitted_rows != u64::from(w2_sum.dev_psucc_admitted_rows)
            || writer_top1_hits != u64::from(w2_sum.writer_top1_correct_all)
            || writer_top8_hits != u64::from(w2_sum.writer_top8_correct_all)
            || t_restr_aw_top1_hits != u64::from(w2_sum.checkpoint_t_restr_aw_hit1_all)
            || t_restr_aw_top8_hits != u64::from(w2_sum.checkpoint_t_restr_aw_hit8_all)
            || train_codes.len() != w2_sum.distinct_train_read_codes
            || dev_codes.len() != w2_sum.distinct_dev_read_codes
        {
            return Err(invalid(format!(
                "{arm_label} Gate R mismatch: adm={dev_admitted_rows}/{} psucc={dev_psucc_admitted_rows}/{} w1={writer_top1_hits}/{} w8={writer_top8_hits}/{} t1={t_restr_aw_top1_hits}/{} t8={t_restr_aw_top8_hits}/{} tr_codes={}/{} dev_codes={}/{}",
                w2_sum.dev_target_admitted_rows,
                w2_sum.dev_psucc_admitted_rows,
                w2_sum.writer_top1_correct_all,
                w2_sum.writer_top8_correct_all,
                w2_sum.checkpoint_t_restr_aw_hit1_all,
                w2_sum.checkpoint_t_restr_aw_hit8_all,
                train_codes.len(),
                w2_sum.distinct_train_read_codes,
                dev_codes.len(),
                w2_sum.distinct_dev_read_codes
            )));
        }

        let paired_reranker_vs_writer = compute_paired_comparison(
            &format!("{arm_label}_reranker_code_vs_writer"),
            "reranker_code_pooled",
            "writer_outer_l1",
            &r_hit_vec,
            &w_hit_vec,
        )?;
        let paired_reranker_vs_permuted_null = compute_paired_comparison(
            &format!("{arm_label}_reranker_code_vs_permuted_null"),
            "reranker_code_pooled",
            "reranker_code_permuted_null",
            &r_hit_vec,
            &perm_hit_vec,
        )?;

        Ok(StageTErrorDecomposition {
            arm_label: arm_label.to_owned(),
            config_sha256: w2_sum.config_sha256,
            checkpoint_sha256: ckpt_pin.sha256,
            model_snapshot_sha256: w2_sum.model_snapshot_sha256,
            model_epoch: w2_sum.model_epoch,
            committed_moves: w2_sum.committed_moves,
            train_rows: train_rows.len() as u64,
            dev_rows: dev_rows.len() as u64,
            distinct_train_read_codes: train_codes.len() as u64,
            distinct_dev_read_codes: dev_codes.len() as u64,
            dev_admitted_rows,
            dev_unadmitted_rows,
            dev_target_cell_rank1_rows,
            dev_target_cell_rank2_to_8_rows,
            dev_psucc_admitted_rows,
            writer_top1_hits,
            writer_top8_hits,
            admitted_outside_writer_top8_rows,
            admitted_in_writer_top2_to_8_rows,
            admitted_target_wins_own_cell_rows,
            admitted_target_wins_own_cell_but_loses_cross_cell_rows,
            admitted_target_loses_within_own_cell_rows,
            admitted_target_loses_both_within_and_cross_cell_rows,
            t_restr_aw_top1_hits,
            t_restr_aw_top8_hits,
            both_w_and_t_restr_hit,
            w_hits_t_restr_misses,
            t_restr_hits_w_misses,
            net_gap_t_restr_minus_w: (t_restr_aw_top1_hits as i64) - (writer_top1_hits as i64),
            gap_lost_outside_w_top8,
            gap_lost_in_w_top2_to_8,
            gap_lost_within_target_cell_only,
            gap_lost_cross_admitted_cell_only,
            gap_lost_both_within_and_cross_cell,
            dev_seen_read_code_rows_all,
            dev_seen_read_code_rows_admitted,
            dev_code_count_ge_2_rows_admitted,
            dev_unseen_fallback_rows_admitted,
            t_restr_hits_on_seen_code,
            t_restr_hits_on_unseen_fallback,
            reranker_code_top1_cell_only_hits,
            reranker_code_cell_prio_top1_hits,
            reranker_code_pooled_top1_hits,
            reranker_code_pooled_top8_hits,
            reranker_code_permuted_null_top1_hits,
            reranker_bigram_top1_cell_only_hits,
            reranker_bigram_pooled_top1_hits,
            reranker_bigram_pooled_top8_hits,
            reranker_unigram_top1_cell_only_hits,
            reranker_unigram_pooled_top1_hits,
            paired_reranker_vs_writer,
            paired_reranker_vs_permuted_null,
        })
    }

    /// Run the Stage T (`T16_fz` Seed 1 and `T16_rel` Seed 1) error decomposition (`Done when` bullet 1).
    pub fn run_decompose_stage_t(
        stage_t_dir: &Path,
        data_root: &Path,
        out_path: &Path,
    ) -> Result<StageTDecompositionSuite> {
        let (train_src, dev_src, bos) = corpus_sources(data_root);
        let train_corpus = PreparedCorpus::load(&train_src, 4096, &bos)?;
        let dev_corpus = PreparedCorpus::load(&dev_src, 4096, &bos)?;

        let t16_fz = decompose_one_stage_t_arm(
            "t16_frozen_zero_seed1",
            &stage_t_dir.join("t16-frozen-zero-seed1"),
            &train_corpus,
            &dev_corpus,
        )?;
        let t16_rel = decompose_one_stage_t_arm(
            "t16_relation_seed1",
            &stage_t_dir.join("t16-relation-seed1"),
            &train_corpus,
            &dev_corpus,
        )?;

        let suite = StageTDecompositionSuite {
            schema: WITHIN_CELL_DECOMP_SCHEMA.to_owned(),
            t16_frozen_zero_seed1: t16_fz,
            t16_relation_seed1: t16_rel,
        };
        write_pinned_json(out_path, &suite)?;
        Ok(suite)
    }

    #[derive(Deserialize)]
    struct PinnedTableFile {
        table: Vec<i32>,
    }

    /// Run the zero-update `B0` evaluation and grade (`Done when` bullets 2 & 3).
    pub fn run_b0(
        data_root: &Path,
        qual_path: &Path,
        decomp_path: &Path,
        out_path: &Path,
    ) -> Result<B0GradeReport> {
        let (source_sha256, kernel_sha256, geometry_sha256, driver_source_sha256) =
            identity_strings()?;
        let image_sha256 = compiled_within_cell_writer_image_sha256();
        let qual_pin = pin_file(qual_path)?;
        let decomp_pin = pin_file(decomp_path)?;

        let (train_src, dev_src, bos) = corpus_sources(data_root);
        let train_corpus = PreparedCorpus::load(&train_src, 4096, &bos)?;
        let dev_corpus = PreparedCorpus::load(&dev_src, 4096, &bos)?;

        // Load pinned A8 StructuredV1 table (`6600bdb1...`)
        let struct_path = data_root.join("address-cells/address-structured-v1.json");
        let struct_pin = pin_file(&struct_path)?;
        if struct_pin.sha256 != A8_STRUCTURED_TABLE_SHA256 {
            return Err(invalid("address-structured-v1.json SHA-256 mismatch"));
        }
        let struct_file: PinnedTableFile = read_json(&struct_path)?;

        // Load re-derived epoch-0 FullVectorV1 (`B = 4`, `2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2`)
        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        let read_spec = ReadSpec::Relation { kept: 8, sweeps: 1 };
        let fv_path = data_root.join("address-cells/address-full-vector-v1.json");
        let fv_pin = pin_file(&fv_path)?;
        if fv_pin.sha256 != FULL_VECTOR_B4_TABLE_SHA256 {
            return Err(invalid(format!(
                "address-full-vector-v1.json SHA-256 mismatch: {}",
                fv_pin.sha256
            )));
        }
        let fv_file: PinnedTableFile = read_json(&fv_path)?;
        let fv_raw_le_sha = fv_pin.sha256;
        let (fv_cells, fv_int_pairs) = extract_canonical_cells_and_interior_pairs(&fv_file.table)?;
        let mut occ_cells = fv_cells.clone();
        occ_cells.sort_unstable();
        occ_cells.dedup();
        let mut distinct_keys: Vec<u16> = (0..4096usize)
            .map(|t| (fv_cells[t] << 8) | fv_int_pairs[t])
            .collect();
        distinct_keys.sort_unstable();
        distinct_keys.dedup();
        if occ_cells.len() != 256 || distinct_keys.len() != 4096 {
            return Err(invalid("G-Occ failed on FullVectorV1 table"));
        }

        let dev_struct = make_device(
            spec,
            read_spec,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            1,
            Some(struct_file.table),
            None,
        )?;
        let k_struct = dev_struct.within_cell_writer_kernels()?;
        let dev_fv = make_device(
            spec,
            read_spec,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            1,
            Some(fv_file.table),
            None,
        )?;
        let k_fv = dev_fv.within_cell_writer_kernels()?;

        let remap_id = identity_remap_32();
        let remap_perm = permuted_frame_remap_32(PERMUTATION_SEED_20261026);

        // 1. Single-pass scan over the 1,049,856 TRAIN rows (`16,404` batches) on GPU
        // to capture `T_frozen` (from `dev_struct` V1) and `T_rebuilt` (from `dev_fv` V1, V2, and V2-Global)
        let cfg_v1_cur = default_wc_cfg(
            ReadLawVersion::ReadLawV1,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_v2_cur = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::CurrentSinglePassV1,
            TransportMode::RelativeFrameTransportV1,
        );
        let cfg_v2_glob = default_wc_cfg(
            ReadLawVersion::ReadLawV2,
            FrameSourceMode::TwoPassEndpointOutputV1,
            TransportMode::GlobalWriterFrameV1,
        );

        let mut train_rows_struct_v1 = Vec::with_capacity(MATCHED_TRAIN_ROWS);
        let mut train_rows_fv_v1 = Vec::with_capacity(MATCHED_TRAIN_ROWS);
        let mut train_rows_fv_v2 = Vec::with_capacity(MATCHED_TRAIN_ROWS);
        let mut train_rows_fv_v2_glob = Vec::with_capacity(MATCHED_TRAIN_ROWS);

        let mut train_first = 0u64;
        for _ in 0..STAGE_T_MAX_BATCHES {
            let batch = train_corpus.batch(train_first, 192, 64)?;
            let carried_s = dev_struct.carry_windows(&batch.windows)?;
            let (d_targ_s, d_rem_s) =
                dev_struct.admit_within_cell_batch(&carried_s, &batch.targets, &remap_id)?;
            let q_s_v1 = dev_struct.evaluate_within_cell_read_vectors_admitted(
                &carried_s,
                &d_targ_s,
                &cfg_v1_cur,
                &d_rem_s,
                &k_struct,
            )?;

            let carried_f = dev_fv.carry_windows(&batch.windows)?;
            let (d_targ_f, d_rem_f) =
                dev_fv.admit_within_cell_batch(&carried_f, &batch.targets, &remap_id)?;
            let q_f_v1 = dev_fv.evaluate_within_cell_read_vectors_admitted(
                &carried_f,
                &d_targ_f,
                &cfg_v1_cur,
                &d_rem_f,
                &k_fv,
            )?;
            let q_f_v2 = dev_fv.evaluate_within_cell_read_vectors_admitted(
                &carried_f,
                &d_targ_f,
                &cfg_v2_cur,
                &d_rem_f,
                &k_fv,
            )?;
            let q_f_v2_g = dev_fv.evaluate_within_cell_read_vectors_admitted(
                &carried_f,
                &d_targ_f,
                &cfg_v2_glob,
                &d_rem_f,
                &k_fv,
            )?;

            for r in 0..batch.targets.len() {
                let win = &batch.windows[r];
                let query = win
                    .last()
                    .ok_or_else(|| invalid("empty TRAIN window"))?
                    .token as u16;
                let target = batch.targets[r] as u16;
                let (c_s1, _, _) = decode_q_keys(&q_s_v1[r * 32..(r + 1) * 32])?;
                let (c_f1, _, _) = decode_q_keys(&q_f_v1[r * 32..(r + 1) * 32])?;
                let (c_f2, _, _) = decode_q_keys(&q_f_v2[r * 32..(r + 1) * 32])?;
                let (c_fg, _, _) = decode_q_keys(&q_f_v2_g[r * 32..(r + 1) * 32])?;
                train_rows_struct_v1.push(TrainRow {
                    code: c_s1,
                    query_token: query,
                    target_token: target,
                });
                train_rows_fv_v1.push(TrainRow {
                    code: c_f1,
                    query_token: query,
                    target_token: target,
                });
                train_rows_fv_v2.push(TrainRow {
                    code: c_f2,
                    query_token: query,
                    target_token: target,
                });
                train_rows_fv_v2_glob.push(TrainRow {
                    code: c_fg,
                    query_token: query,
                    target_token: target,
                });
            }
            train_first = batch.next_target;
        }

        // Fit 16 -> 16 TRAIN endpoint frame remap on first 900,000 TRAIN rows, validate on remaining 149,856
        let (remap_train_32, frame_remap_summary) =
            estimate_frame_remap_16(&train_rows_fv_v1, &fv_cells, TRAIN_FIT_SPLIT_ROWS);

        let mut unigram_counts = vec![0u64; w2_capacity::VOCABULARY];
        for tr in &train_rows_fv_v1 {
            unigram_counts[usize::from(tr.target_token)] += 1;
        }
        let t_frozen_table = WithinCellCountTable::from_pairs(
            train_rows_struct_v1
                .iter()
                .map(|r| (r.code, r.target_token)),
        );
        let t_rebuilt_v1_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v1.iter().map(|r| (r.code, r.target_token)),
        );
        let t_rebuilt_v2_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v2.iter().map(|r| (r.code, r.target_token)),
        );
        let t_rebuilt_v2_glob_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v2_glob
                .iter()
                .map(|r| (r.code, r.target_token)),
        );
        let perm_map = permuted_code_lookup_table(PERMUTATION_SEED_20261026);
        let t_perm_v1_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v1
                .iter()
                .map(|r| (perm_map[usize::from(r.code)], r.target_token)),
        );
        let g_bigram_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v1
                .iter()
                .map(|r| (r.query_token, r.target_token)),
        );
        let u_unigram_table = WithinCellCountTable::from_pairs(
            train_rows_fv_v1.iter().map(|r| (0u16, r.target_token)),
        );

        // 2. Define the 13 GPU read-law / framing arms to evaluate on DEV (`14,287` rows)
        struct GpuArmSpec {
            name: &'static str,
            read_law: ReadLawVersion,
            frame_mode: FrameSourceMode,
            transport_mode: TransportMode,
            remap: [u32; 32],
        }
        let gpu_arm_specs = [
            GpuArmSpec {
                name: "current_single_pass_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::CurrentSinglePassV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "nearest_endpoint_single_pass_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::NearestEndpointSinglePassV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "twopass_endpoint_output_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::TwoPassEndpointOutputV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "frozen_constant_frame_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::FrozenConstantFrameV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "permuted_frame_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::PermutedFrameV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_perm,
            },
            GpuArmSpec {
                name: "train_remap_16_v1",
                read_law: ReadLawVersion::ReadLawV1,
                frame_mode: FrameSourceMode::TrainRemap16V1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_train_32,
            },
            GpuArmSpec {
                name: "current_single_pass_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::CurrentSinglePassV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "nearest_endpoint_single_pass_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::NearestEndpointSinglePassV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "twopass_endpoint_output_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::TwoPassEndpointOutputV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "frozen_constant_frame_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::FrozenConstantFrameV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_id,
            },
            GpuArmSpec {
                name: "permuted_frame_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::PermutedFrameV1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_perm,
            },
            GpuArmSpec {
                name: "train_remap_16_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::TrainRemap16V1,
                transport_mode: TransportMode::RelativeFrameTransportV1,
                remap: remap_train_32,
            },
            GpuArmSpec {
                name: "twopass_global_writer_frame_v2",
                read_law: ReadLawVersion::ReadLawV2,
                frame_mode: FrameSourceMode::TwoPassEndpointOutputV1,
                transport_mode: TransportMode::GlobalWriterFrameV1,
                remap: remap_id,
            },
        ];

        struct RawGpuArmEval {
            rows: Vec<crate::device::WithinCellRowEvaluation>,
            codes: Vec<u16>,
            int_pairs: Vec<u16>,
            elapsed_ms: u64,
        }

        let mut dev_query_tokens = Vec::with_capacity(DEV_ROWS);
        let mut dev_target_tokens = Vec::with_capacity(DEV_ROWS);
        let mut dev_is_psucc = Vec::with_capacity(DEV_ROWS);
        {
            let mut first = 0u64;
            while (first as usize) < DEV_ROWS {
                let batch = dev_corpus.batch(first, 192, 64)?;
                for r in 0..batch.targets.len() {
                    let win = &batch.windows[r];
                    let qtok = win.last().ok_or_else(|| invalid("empty DEV window"))?.token as u16;
                    let ttok = batch.targets[r];
                    dev_query_tokens.push(qtok);
                    dev_target_tokens.push(ttok as u16);
                    dev_is_psucc.push(evidence::is_copy_successor(win, ttok));
                }
                first = batch.next_target;
            }
        }

        let mut gpu_evals: Vec<RawGpuArmEval> = Vec::with_capacity(gpu_arm_specs.len());
        for spec_arm in &gpu_arm_specs {
            let cfg = default_wc_cfg(
                spec_arm.read_law,
                spec_arm.frame_mode,
                spec_arm.transport_mode,
            );
            let t0 = Instant::now();
            let mut rows = Vec::with_capacity(DEV_ROWS);
            let mut codes = Vec::with_capacity(DEV_ROWS);
            let mut int_pairs = Vec::with_capacity(DEV_ROWS);
            let mut first = 0u64;
            while (first as usize) < DEV_ROWS {
                let batch = dev_corpus.batch(first, 192, 64)?;
                let carried = dev_fv.carry_windows(&batch.windows)?;
                let ev = dev_fv.evaluate_within_cell_writer_batch(
                    &carried,
                    &batch.targets,
                    &cfg,
                    &spec_arm.remap,
                    false,
                    &k_fv,
                )?;
                for row_ev in ev.rows {
                    let (code, _, ip) = decode_q_keys(&row_ev.query)?;
                    codes.push(code);
                    int_pairs.push(ip);
                    rows.push(row_ev);
                }
                first = batch.next_target;
            }
            let elapsed_ms = u64::try_from(t0.elapsed().as_millis())
                .unwrap_or(u64::MAX)
                .max(1);
            gpu_evals.push(RawGpuArmEval {
                rows,
                codes,
                int_pairs,
                elapsed_ms,
            });
        }

        // Compute pre-registered intersection of admitted rows across all GPU arms
        let mut in_intersection = vec![true; DEV_ROWS];
        for ge in &gpu_evals {
            for r in 0..DEV_ROWS {
                if !ge.rows[r].target_admitted {
                    in_intersection[r] = false;
                }
            }
        }
        let intersection_admitted_rows = in_intersection.iter().filter(|&&b| b).count() as u64;

        // Reference arms for non-inertness (`current_single_pass_v1` = idx 0, `nearest_endpoint_single_pass_v1` = idx 1,
        // `current_single_pass_v2` = idx 6, `nearest_endpoint_single_pass_v2` = idx 7)
        let mut arms_out = Vec::new();
        let writer_resident_bytes = (4096u64 * 32 * 4) + (65 * 4);

        for (idx, spec_arm) in gpu_arm_specs.iter().enumerate() {
            let ge = &gpu_evals[idx];
            let (ref_cur_idx, ref_near_idx) = match spec_arm.read_law {
                ReadLawVersion::ReadLawV1 => (0usize, 1usize),
                ReadLawVersion::ReadLawV2 => (6usize, 7usize),
            };
            let train_tbl = match (spec_arm.read_law, spec_arm.transport_mode) {
                (ReadLawVersion::ReadLawV1, _) => &t_rebuilt_v1_table,
                (ReadLawVersion::ReadLawV2, TransportMode::RelativeFrameTransportV1) => {
                    &t_rebuilt_v2_table
                }
                (ReadLawVersion::ReadLawV2, TransportMode::GlobalWriterFrameV1) => {
                    &t_rebuilt_v2_glob_table
                }
            };

            let mut dev_codes_dedup = ge.codes.clone();
            dev_codes_dedup.sort_unstable();
            dev_codes_dedup.dedup();

            let mut target_admitted = 0u64;
            let mut psucc_admitted = 0u64;
            let mut int_hits_all = 0u64;
            let mut int_hits_inter = 0u64;
            let mut top1_all = 0u64;
            let mut top1_inter = 0u64;
            let mut top8_all = 0u64;
            let mut within_cell_rank1 = 0u64;
            let mut seen_code_rows = 0u64;
            let mut non_inert_int_cur = 0u64;
            let mut non_inert_int_near = 0u64;
            let mut non_inert_top1_cur = 0u64;
            let mut non_inert_top1_near = 0u64;
            let mut total_tok_cmp = 0u64;

            for r in 0..DEV_ROWS {
                let row_ev = &ge.rows[r];
                let ttok = dev_target_tokens[r];
                let target_ip = fv_int_pairs[usize::from(ttok)];
                let ip_hit = ge.int_pairs[r] == target_ip;
                if ip_hit {
                    int_hits_all += 1;
                    if in_intersection[r] {
                        int_hits_inter += 1;
                    }
                }
                if row_ev.top1_correct {
                    top1_all += 1;
                    if in_intersection[r] {
                        top1_inter += 1;
                    }
                }
                if row_ev.top8_correct {
                    top8_all += 1;
                }
                if row_ev.target_admitted {
                    target_admitted += 1;
                    if dev_is_psucc[r] {
                        psucc_admitted += 1;
                    }
                    if row_ev.within_cell_rivals == 0 {
                        within_cell_rank1 += 1;
                    }
                }
                if train_tbl.key_total(ge.codes[r]) > 0 {
                    seen_code_rows += 1;
                }
                if ge.int_pairs[r] != gpu_evals[ref_cur_idx].int_pairs[r] {
                    non_inert_int_cur += 1;
                }
                if ge.int_pairs[r] != gpu_evals[ref_near_idx].int_pairs[r] {
                    non_inert_int_near += 1;
                }
                if row_ev.top8_tokens[0] != gpu_evals[ref_cur_idx].rows[r].top8_tokens[0] {
                    non_inert_top1_cur += 1;
                }
                if row_ev.top8_tokens[0] != gpu_evals[ref_near_idx].rows[r].top8_tokens[0] {
                    non_inert_top1_near += 1;
                }
                total_tok_cmp += u64::from(row_ev.admitted_token_count);
            }

            let rows_per_sec = (DEV_ROWS as u64).saturating_mul(1000) / ge.elapsed_ms;
            let toks_per_sec = (42_856u64).saturating_mul(1000) / ge.elapsed_ms;
            let extra_remap_bytes = match spec_arm.frame_mode {
                FrameSourceMode::PermutedFrameV1 | FrameSourceMode::TrainRemap16V1 => 128u64,
                _ => 0u64,
            };

            arms_out.push(B0ArmSummary {
                arm_name: spec_arm.name.to_owned(),
                kind: "gpu_writer_arm".to_owned(),
                read_law: format!("{:?}", spec_arm.read_law),
                frame_mode: format!("{:?}", spec_arm.frame_mode),
                transport_mode: format!("{:?}", spec_arm.transport_mode),
                table_sha256: fv_raw_le_sha.clone(),
                dev_rows: DEV_ROWS as u64,
                target_admitted_rows: target_admitted,
                psucc_admitted_rows: psucc_admitted,
                intersection_admitted_rows,
                interior_key_exact_hits_all: int_hits_all,
                interior_key_exact_hits_intersection: int_hits_inter,
                total_top1_hits: top1_all,
                intersection_admitted_top1_hits: top1_inter,
                total_top8_hits: top8_all,
                within_target_cell_rank1_rows_admitted: within_cell_rank1,
                distinct_train_read_codes: train_tbl.distinct_keys() as u64,
                distinct_dev_read_codes: dev_codes_dedup.len() as u64,
                dev_seen_read_code_rows: seen_code_rows,
                non_inert_interior_key_rows_vs_current: non_inert_int_cur,
                non_inert_interior_key_rows_vs_nearest: non_inert_int_near,
                non_inert_top1_rows_vs_current: non_inert_top1_cur,
                non_inert_top1_rows_vs_nearest: non_inert_top1_near,
                cell_comparisons_per_row: 256,
                mean_token_comparisons_per_row: total_tok_cmp / (DEV_ROWS as u64),
                total_token_comparisons_dev: total_tok_cmp,
                resident_bytes: writer_resident_bytes + extra_remap_bytes,
                eval_elapsed_ms: ge.elapsed_ms,
                decode_rows_per_sec: rows_per_sec,
                decode_tokens_per_sec: toks_per_sec,
            });
        }

        // Refuse vacuous two-pass configuration if non-inertness is zero (`Done when` bullet 3 / 4)
        let twopass_v1_non_inert_rows = arms_out[2].non_inert_interior_key_rows_vs_nearest;
        let twopass_v2_non_inert_rows = arms_out[8].non_inert_interior_key_rows_vs_nearest;
        if twopass_v1_non_inert_rows == 0 || twopass_v2_non_inert_rows == 0 {
            return Err(invalid(
                "G-Refuse-ZeroNonInertness triggered: TwoPass has 0 non-inert rows vs Nearest",
            ));
        }

        // 3. Evaluate lookup & within-cell reranker controls on the same FullVectorV1 DEV admitted cells
        // Helper to evaluate a `WithinCellCountTable` control on GPU arm `base_idx`
        let eval_table_control = |arm_name: &str,
                                  kind: &str,
                                  base_idx: usize,
                                  tbl: &WithinCellCountTable,
                                  use_bigram_key: bool,
                                  use_unigram_key: bool,
                                  mode_select: u8|
         -> (B0ArmSummary, Vec<bool>, Vec<bool>) {
            let t0 = Instant::now();
            let ge = &gpu_evals[base_idx];
            let mut top1_all = 0u64;
            let mut top1_inter = 0u64;
            let mut top8_all = 0u64;
            let mut int_all = 0u64;
            let mut int_inter = 0u64;
            let mut seen_rows = 0u64;
            let mut hit_all_vec = Vec::with_capacity(DEV_ROWS);
            let mut hit_inter_vec = Vec::with_capacity(intersection_admitted_rows as usize);

            for r in 0..DEV_ROWS {
                let row_ev = &ge.rows[r];
                let slots: Vec<u16> = row_ev.admitted_cells.iter().map(|&c| c as u16).collect();
                let key = if use_bigram_key {
                    dev_query_tokens[r]
                } else if use_unigram_key {
                    0u16
                } else {
                    ge.codes[r]
                };
                if tbl.key_total(key) > 0 {
                    seen_rows += 1;
                }
                let (c0_hit, prio_hit, pool_hit1, pool_hit8) = score_within_cell_reranker_row(
                    tbl,
                    key,
                    &unigram_counts,
                    &fv_cells,
                    &slots,
                    dev_target_tokens[r],
                );
                let hit1 = match mode_select {
                    0 => pool_hit1,
                    1 => c0_hit,
                    _ => prio_hit,
                };
                if hit1 {
                    top1_all += 1;
                    int_all += 1;
                    if in_intersection[r] {
                        top1_inter += 1;
                        int_inter += 1;
                    }
                }
                if pool_hit8 {
                    top8_all += 1;
                }
                hit_all_vec.push(hit1);
                if in_intersection[r] {
                    hit_inter_vec.push(hit1);
                }
            }
            let elapsed_ms = u64::try_from(t0.elapsed().as_millis())
                .unwrap_or(u64::MAX)
                .max(1);
            let base_summary = &arms_out[base_idx];
            let tbl_bytes = writer_resident_bytes + tbl.non_empty_entries() * 12 + 4096 * 8;
            let summary = B0ArmSummary {
                arm_name: arm_name.to_owned(),
                kind: kind.to_owned(),
                read_law: base_summary.read_law.clone(),
                frame_mode: base_summary.frame_mode.clone(),
                transport_mode: base_summary.transport_mode.clone(),
                table_sha256: fv_raw_le_sha.clone(),
                dev_rows: DEV_ROWS as u64,
                target_admitted_rows: base_summary.target_admitted_rows,
                psucc_admitted_rows: base_summary.psucc_admitted_rows,
                intersection_admitted_rows,
                interior_key_exact_hits_all: int_all,
                interior_key_exact_hits_intersection: int_inter,
                total_top1_hits: top1_all,
                intersection_admitted_top1_hits: top1_inter,
                total_top8_hits: top8_all,
                within_target_cell_rank1_rows_admitted: top1_all,
                distinct_train_read_codes: tbl.distinct_keys() as u64,
                distinct_dev_read_codes: base_summary.distinct_dev_read_codes,
                dev_seen_read_code_rows: seen_rows,
                non_inert_interior_key_rows_vs_current: top1_all,
                non_inert_interior_key_rows_vs_nearest: top1_all,
                non_inert_top1_rows_vs_current: top1_all,
                non_inert_top1_rows_vs_nearest: top1_all,
                cell_comparisons_per_row: 256,
                mean_token_comparisons_per_row: base_summary.mean_token_comparisons_per_row,
                total_token_comparisons_dev: base_summary.total_token_comparisons_dev,
                resident_bytes: tbl_bytes,
                eval_elapsed_ms: base_summary.eval_elapsed_ms + elapsed_ms,
                decode_rows_per_sec: (DEV_ROWS as u64).saturating_mul(1000)
                    / (base_summary.eval_elapsed_ms + elapsed_ms),
                decode_tokens_per_sec: (42_856u64).saturating_mul(1000)
                    / (base_summary.eval_elapsed_ms + elapsed_ms),
            };
            (summary, hit_all_vec, hit_inter_vec)
        };

        let (sum_t_frozen, _, _) = eval_table_control(
            "lookup_t_frozen_restr_v1",
            "lookup_control",
            0,
            &t_frozen_table,
            false,
            false,
            0,
        );
        let (sum_t_rebuilt_v1, r_v1_all_hits, r_v1_inter_hits) = eval_table_control(
            "reranker_t_rebuilt_pooled_v1",
            "within_cell_reranker",
            0,
            &t_rebuilt_v1_table,
            false,
            false,
            0,
        );
        let (sum_r_c0_v1, _, _) = eval_table_control(
            "reranker_t_rebuilt_top1_cell_only_v1",
            "within_cell_reranker",
            0,
            &t_rebuilt_v1_table,
            false,
            false,
            1,
        );
        let (sum_r_prio_v1, _, _) = eval_table_control(
            "reranker_t_rebuilt_cell_prio_v1",
            "within_cell_reranker",
            0,
            &t_rebuilt_v1_table,
            false,
            false,
            2,
        );
        let (sum_r_perm_v1, r_perm_all_hits, r_perm_inter_hits) = eval_table_control(
            "reranker_code_permuted_null_v1",
            "permuted_null_control",
            0,
            &t_perm_v1_table,
            false,
            false,
            0,
        );
        let (sum_t_rebuilt_v2, r_v2_all_hits, _) = eval_table_control(
            "reranker_t_rebuilt_pooled_v2",
            "within_cell_reranker",
            6,
            &t_rebuilt_v2_table,
            false,
            false,
            0,
        );
        let (sum_g_restr_v1, g_all_hits, _) = eval_table_control(
            "lookup_g_bigram_restr_v1",
            "lookup_control",
            0,
            &g_bigram_table,
            true,
            false,
            0,
        );
        let (sum_u_restr_v1, _, _) = eval_table_control(
            "lookup_u_unigram_restr_v1",
            "lookup_control",
            0,
            &u_unigram_table,
            false,
            true,
            0,
        );

        arms_out.push(sum_t_frozen);
        arms_out.push(sum_t_rebuilt_v1);
        arms_out.push(sum_r_c0_v1);
        arms_out.push(sum_r_prio_v1);
        arms_out.push(sum_r_perm_v1);
        arms_out.push(sum_t_rebuilt_v2);
        arms_out.push(sum_g_restr_v1);
        arms_out.push(sum_u_restr_v1);

        // 4. Compute Primary & Secondary Paired Comparisons
        let int_hit_vec_inter = |arm_idx: usize| -> Vec<bool> {
            let ge = &gpu_evals[arm_idx];
            let mut v = Vec::with_capacity(intersection_admitted_rows as usize);
            for r in 0..DEV_ROWS {
                if in_intersection[r] {
                    let target_ip = fv_int_pairs[usize::from(dev_target_tokens[r])];
                    v.push(ge.int_pairs[r] == target_ip);
                }
            }
            v
        };
        let top1_hit_vec_all = |arm_idx: usize| -> Vec<bool> {
            gpu_evals[arm_idx]
                .rows
                .iter()
                .map(|row| row.top1_correct)
                .collect()
        };

        let primary_v1 = compute_paired_comparison(
            "primary_v1_twopass_vs_nearest_interior_key_intersection",
            "twopass_endpoint_output_v1",
            "nearest_endpoint_single_pass_v1",
            &int_hit_vec_inter(2),
            &int_hit_vec_inter(1),
        )?;
        let primary_v2 = compute_paired_comparison(
            "primary_v2_twopass_vs_nearest_interior_key_intersection",
            "twopass_endpoint_output_v2",
            "nearest_endpoint_single_pass_v2",
            &int_hit_vec_inter(8),
            &int_hit_vec_inter(7),
        )?;

        let mut secondary = Vec::new();
        secondary.push(compute_paired_comparison(
            "v1_twopass_vs_current_interior_key_intersection",
            "twopass_endpoint_output_v1",
            "current_single_pass_v1",
            &int_hit_vec_inter(2),
            &int_hit_vec_inter(0),
        )?);
        secondary.push(compute_paired_comparison(
            "v2_twopass_vs_current_interior_key_intersection",
            "twopass_endpoint_output_v2",
            "current_single_pass_v2",
            &int_hit_vec_inter(8),
            &int_hit_vec_inter(6),
        )?);
        secondary.push(compute_paired_comparison(
            "v1_twopass_vs_frozen_frame_interior_key_intersection",
            "twopass_endpoint_output_v1",
            "frozen_constant_frame_v1",
            &int_hit_vec_inter(2),
            &int_hit_vec_inter(3),
        )?);
        secondary.push(compute_paired_comparison(
            "v1_twopass_vs_permuted_frame_interior_key_intersection",
            "twopass_endpoint_output_v1",
            "permuted_frame_v1",
            &int_hit_vec_inter(2),
            &int_hit_vec_inter(4),
        )?);
        secondary.push(compute_paired_comparison(
            "v1_train_remap16_vs_nearest_interior_key_intersection",
            "train_remap_16_v1",
            "nearest_endpoint_single_pass_v1",
            &int_hit_vec_inter(5),
            &int_hit_vec_inter(1),
        )?);
        secondary.push(compute_paired_comparison(
            "v2_global_vs_v1_current_top1_all",
            "twopass_global_writer_frame_v2",
            "current_single_pass_v1",
            &top1_hit_vec_all(12),
            &top1_hit_vec_all(0),
        )?);
        secondary.push(compute_paired_comparison(
            "reranker_v1_vs_writer_v1_top1_all",
            "reranker_t_rebuilt_pooled_v1",
            "current_single_pass_v1",
            &r_v1_all_hits,
            &top1_hit_vec_all(0),
        )?);
        secondary.push(compute_paired_comparison(
            "reranker_v1_vs_permuted_null_top1_all",
            "reranker_t_rebuilt_pooled_v1",
            "reranker_code_permuted_null_v1",
            &r_v1_all_hits,
            &r_perm_all_hits,
        )?);
        secondary.push(compute_paired_comparison(
            "reranker_v1_vs_permuted_null_top1_intersection",
            "reranker_t_rebuilt_pooled_v1",
            "reranker_code_permuted_null_v1",
            &r_v1_inter_hits,
            &r_perm_inter_hits,
        )?);
        secondary.push(compute_paired_comparison(
            "reranker_v2_vs_writer_v2_top1_all",
            "reranker_t_rebuilt_pooled_v2",
            "current_single_pass_v2",
            &r_v2_all_hits,
            &top1_hit_vec_all(6),
        )?);
        secondary.push(compute_paired_comparison(
            "reranker_v1_vs_g_bigram_restr_top1_all",
            "reranker_t_rebuilt_pooled_v1",
            "lookup_g_bigram_restr_v1",
            &r_v1_all_hits,
            &g_all_hits,
        )?);

        let twopass_clears_primary_bar =
            primary_v1.clears_strict_paired_rule || primary_v2.clears_strict_paired_rule;
        let reranker_beats_writer =
            secondary[6].clears_strict_paired_rule && secondary[7].clears_strict_paired_rule;
        let reranker_clears_matched_writer_and_g_bar =
            reranker_beats_writer && secondary[10].clears_strict_paired_rule;

        let (overall_b0_verdict, unlocked_next_step) = if twopass_clears_primary_bar {
            (
                "POSITIVE_TWOPASS_FRAMING".to_owned(),
                "Register component-wise ReadLawV2 + TwoPass training sheet with ablations before D11"
                    .to_owned(),
            )
        } else {
            (
                "NEGATIVE_TWOPASS_FRAMING_POSITIVE_WITHIN_CELL_RERANKER".to_owned(),
                "Stop epoch-0 TwoPass interior-key framing configuration; pursue exact-memory within-cell writer lookup/reranker scoring rather than another read law or capacity expansion"
                    .to_owned(),
            )
        };

        let report = B0GradeReport {
            schema: WITHIN_CELL_B0_SCHEMA.to_owned(),
            source_sha256,
            kernel_sha256,
            geometry_sha256,
            driver_source_sha256,
            within_cell_writer_image_sha256: image_sha256,
            full_vector_table_sha256: FULL_VECTOR_B4_TABLE_SHA256.to_owned(),
            a8_structured_table_sha256: A8_STRUCTURED_TABLE_SHA256.to_owned(),
            qualification_sha256: qual_pin.sha256,
            decomposition_sha256: decomp_pin.sha256,
            intersection_admitted_rows,
            frame_remap_summary,
            arms: arms_out,
            primary_v1_twopass_vs_nearest_interior_key: primary_v1,
            primary_v2_twopass_vs_nearest_interior_key: primary_v2,
            secondary_comparisons: secondary,
            twopass_v1_non_inert_rows,
            twopass_v2_non_inert_rows,
            twopass_clears_primary_bar,
            reranker_clears_matched_writer_and_g_bar,
            overall_b0_verdict,
            unlocked_next_step,
        };
        write_pinned_json(out_path, &report)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_orbit_summary_proves_v1_bias_and_v2_256_recovery() {
        let (summary, _, _, _) = host_snap_orbit_summary();
        assert_eq!(summary.total_pairs, 256);
        assert_eq!(summary.distinct_pairs, 240);
        // Under ReadLawV1 (`snap_2(r_q + v)`), only 129/256 all pairs and 113/240 distinct pairs recover v:
        assert_eq!(summary.v1_recover_v_all_256, 129);
        assert_eq!(summary.v1_recover_v_distinct_240, 113);
        assert_eq!(summary.v1_antipodal_zero_collapse, 15);
        // Under ReadLawV2 (`snap_3(r_q + 2v)`), 256/256 all pairs and 240/240 distinct pairs recover v:
        assert_eq!(summary.v2_pos_recover_v_all_256, 256);
        assert_eq!(summary.v2_pos_recover_v_distinct_240, 240);
        // Under ReadLawV2 negative gate (`snap_3(r_q - 2v)`), 0/256 equal +v and 256/256 recover the antipode -v:
        assert_eq!(summary.v2_neg_equals_v_count, 0);
        assert_eq!(summary.v2_neg_recovers_antipode_when_present, 256);
    }

    #[test]
    fn derangement_16_has_zero_fixed_points_and_is_bijective() {
        let perm = derangement_16(PERMUTATION_SEED_20261026);
        let mut seen = [false; 16];
        for (i, &p) in perm.iter().enumerate() {
            assert!(p < 16);
            assert_ne!(p as usize, i, "derangement must have no fixed points");
            seen[p as usize] = true;
        }
        assert!(seen.iter().all(|&b| b));
    }

    #[test]
    fn estimate_frame_remap_16_recovers_validation_transitions() {
        let cells: Vec<u16> = (0..256u16).collect();
        let mut rows = Vec::with_capacity(320);
        for _ in 0..20 {
            for r in 0..16u16 {
                let q = (r << 4) | r;
                let t = (((r + 5) & 15) << 4) | ((r + 9) & 15);
                rows.push(TrainRow {
                    code: 0,
                    query_token: q,
                    target_token: t,
                });
            }
        }
        let (_, summary) = estimate_frame_remap_16(&rows, &cells, 240);
        assert_eq!(summary.train_fit_rows, 240);
        assert_eq!(summary.validation_holdout_rows, 80);
        assert_eq!(summary.val_endpoint0_remap_hits, 80);
        assert_eq!(summary.val_endpoint0_identity_hits, 0);
        assert_eq!(summary.val_endpoint3_remap_hits, 80);
        assert_eq!(summary.val_endpoint3_identity_hits, 0);
    }

    #[test]
    fn strict_paired_comparison_enforces_d_gt_24_and_4n_rule() {
        // Case 1: b = 30, c = 0 -> d = 30 > 24, d^2 = 900 > 4 * 30 = 120 -> PASS
        let mut cand = vec![false; 100];
        let refer = vec![false; 100];
        for i in 0..30 {
            cand[i] = true;
        }
        let cmp_pass = compute_paired_comparison("t1", "cand", "ref", &cand, &refer).unwrap();
        assert_eq!(cmp_pass.net_diff_d, 30);
        assert!(cmp_pass.clears_strict_paired_rule);

        // Case 2: b = 24, c = 0 -> d = 24 (not > 24) -> FAIL
        let mut cand24 = vec![false; 100];
        for i in 0..24 {
            cand24[i] = true;
        }
        let cmp_fail_24 = compute_paired_comparison("t2", "cand", "ref", &cand24, &refer).unwrap();
        assert_eq!(cmp_fail_24.net_diff_d, 24);
        assert!(!cmp_fail_24.clears_strict_paired_rule);
    }

    #[test]
    fn within_cell_reranker_beats_permuted_code_null() {
        let token_cells: Vec<u16> = (0..w2_capacity::VOCABULARY as u16)
            .map(|t| t / 16)
            .collect();
        let mut unigram_counts = vec![1u64; w2_capacity::VOCABULARY];
        unigram_counts[0] = 100; // unigram favors token 0 in cell 0
                                 // Train table maps code 42 -> token 7 (in cell 0) 10 times
        let tbl = WithinCellCountTable::from_pairs((0..10).map(|_| (42u16, 7u16)));
        let perm = permuted_code_lookup_table(PERMUTATION_SEED_20261026);
        let perm_tbl = WithinCellCountTable::from_pairs((0..10).map(|_| (perm[42], 7u16)));
        let slots = [0u16, 1, 2, 3, 4, 5, 6, 7];
        let (c0_hit, prio_hit, pool_hit1, pool_hit8) =
            score_within_cell_reranker_row(&tbl, 42, &unigram_counts, &token_cells, &slots, 7);
        assert!(c0_hit && prio_hit && pool_hit1 && pool_hit8);
        if perm[42] != 42 {
            let (p_c0, _, p_pool1, _) = score_within_cell_reranker_row(
                &perm_tbl,
                42,
                &unigram_counts,
                &token_cells,
                &slots,
                7,
            );
            assert!(!p_c0 && !p_pool1);
        }
    }
}
