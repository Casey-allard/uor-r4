//! Lawful integer E8 orbit-root normalization mechanism (`OrbitRootSnapNormV1`)
//! and finite operator controls on the generated `Cl(0,6)` 64-frame / 16-root
//! E8 orbit (`docs/roadmap/items/m1-normalization-mechanism.md`, issue #2842).
//!
//! Host functions in this module are finite algebraic and metadata controls
//! over the build-time generated `geometry.rs` tables (`ROOTS`, `AXES`,
//! `COMPOSE`, `INVERSE`, `GENERATORS`, `IDENTITY`). Model training and corpus
//! evaluation remain on the GPU (`CHARTER.md` §3.5, D24). No floating-point
//! types or literals (D32), no dense matrix multiplication (D25), and no
//! smooth squashing or exponential maps (D23) appear on this path.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

/// Number of orbit roots per 8D block in the discrete M1 carrier.
pub const ORBIT_ROOTS: usize = 16;

/// Number of signed-permutation frames in the generated `Cl(0,6)` group.
pub const GROUP_FRAMES: usize = 64;

/// Number of legal frame candidates per block (`0 = IDENTITY`, `1..=15 = GENERATORS`).
pub const FRAME_CANDIDATES: u8 = 16;

/// Maximum positive integer multiplicity `n` accepted by `snap_n` normalization.
pub const MAX_MULTIPLICITY: u32 = 64;

/// Canonical L1 norm of every 8D root in the 16-root E8 orbit (`|2| + |2| = 4`).
pub const CANONICAL_BLOCK_L1: u32 = 4;

/// Canonical squared Euclidean norm of every 8D root in the 16-root E8 orbit (`2^2 + 2^2 = 8`).
pub const CANONICAL_BLOCK_L2_SQ: u32 = 8;

/// Canonical infinity norm (maximum coordinate magnitude) of every 8D root (`2`).
pub const CANONICAL_BLOCK_LINFTY: u32 = 2;

/// Canonical number of non-zero coordinates in every 8D root (`2`).
pub const CANONICAL_BLOCK_SUPPORT: u32 = 2;

/// Canonical number of bits carried by a normalized 8D block (`log2(16) = 4`).
pub const BITS_PER_NORMALIZED_BLOCK: u32 = 4;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// Normalization mode for multi-stage residual and read comparisons (`#2842`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationMode {
    /// Multiplicity-matched `snap_n` projection onto the 16-root E8 orbit
    /// minimizing `(L1(x, n * root), Hamming(x, n * root), root_index)`.
    #[default]
    ScaleMatchedSnapV1,
    /// Un-scaled `snap_1` control (`n = 1`) regardless of the number of summed terms.
    UnitSnapControlV1,
    /// Un-normalized control: passes the raw integer residual sum `[i32; 8]`
    /// through without snapping back to the 16-root E8 orbit.
    UnnormalizedControlV1,
}

impl NormalizationMode {
    pub fn is_scale_matched_snap(&self) -> bool {
        *self == Self::ScaleMatchedSnapV1
    }

    /// Device spec word `s[12]` of the Flock/Relation read: `0` keeps
    /// `snap_n`, `1` is the unit `snap_1` control, `2` is the `C_raw` control
    /// whose output block is the unsnapped integer residual in ℤ⁸.
    pub fn device_word(self) -> Result<u32> {
        match self {
            Self::ScaleMatchedSnapV1 => Ok(0),
            Self::UnitSnapControlV1 => Ok(1),
            Self::UnnormalizedControlV1 => Ok(2),
        }
    }
}

/// Whether a stored snapshot JSON value carries the default normalization mode.
/// The key is omitted when default, so a missing key is the default; any other
/// value (including an unknown one) is not.
pub(crate) fn stored_mode_is_default(model: &serde_json::Value) -> bool {
    model
        .get("normalization_mode")
        .is_none_or(|value| *value == serde_json::json!(NormalizationMode::default()))
}

/// Exact per-block integer norm invariants measured on an 8D integer block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockNormInvariants {
    /// Exact L1 norm `sum_d |x_d|`.
    pub l1_norm: u32,
    /// Exact squared Euclidean norm `sum_d x_d^2`.
    pub l2_norm_sq: u32,
    /// Exact infinity norm `max_d |x_d|`.
    pub linfty_norm: u32,
    /// Number of non-zero coordinates (`sum_d [x_d != 0]`).
    pub support_size: u32,
    /// Whether the block equals one of the 16 canonical orbit roots in `ROOTS`.
    pub on_canonical_orbit: bool,
}

impl BlockNormInvariants {
    /// Measure the exact integer invariants of an 8D block `block`.
    pub fn measure(block: &[i32; 8]) -> Self {
        let mut l1_norm = 0u32;
        let mut l2_norm_sq = 0u32;
        let mut linfty_norm = 0u32;
        let mut support_size = 0u32;
        for &coord in block {
            let mag = coord.unsigned_abs();
            l1_norm = l1_norm.saturating_add(mag);
            l2_norm_sq = l2_norm_sq.saturating_add(mag.saturating_mul(mag));
            if mag > linfty_norm {
                linfty_norm = mag;
            }
            if coord != 0 {
                support_size += 1;
            }
        }
        let on_canonical_orbit = (0..ORBIT_ROOTS).any(|r| {
            geometry::ROOTS[r]
                .iter()
                .zip(block.iter())
                .all(|(&rc, &bc)| i32::from(rc) == bc)
        });
        Self {
            l1_norm,
            l2_norm_sq,
            linfty_norm,
            support_size,
            on_canonical_orbit,
        }
    }

    /// True iff all four canonical orbit invariants hold (`L1 = 4`, `L2^2 = 8`,
    /// `L_infty = 2`, `support = 2`, and the vector is one of the 16 orbit roots).
    pub fn is_canonical_unit_root(&self) -> bool {
        self.l1_norm == CANONICAL_BLOCK_L1
            && self.l2_norm_sq == CANONICAL_BLOCK_L2_SQ
            && self.linfty_norm == CANONICAL_BLOCK_LINFTY
            && self.support_size == CANONICAL_BLOCK_SUPPORT
            && self.on_canonical_orbit
    }
}

/// Diagnostic record for one `snap_n` root projection of an 8D block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapProjection {
    /// Selected orbit root index in `0..16`.
    pub root_index: u8,
    /// Aligned orbit root index in `0..16` after optional `Cl(0,6)` frame alignment.
    pub aligned_root_index: u8,
    /// Exact L1 distance `sum_d |x_d - n * rho_{r, d}|` at the selected root.
    pub l1_distance: u32,
    /// Exact coordinate Hamming distance `sum_d [x_d != n * rho_{r, d}]` at the selected root.
    pub hamming_distance: u32,
    /// Normalized 8D output block in `{-2, 0, 2}^8`.
    pub normalized_block: [i32; 8],
}

/// Optional per-block `Cl(0,6)` frame alignment table (`B` whole-number candidates in `0..16`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormFrameTable {
    blocks: u32,
    candidates: Vec<u8>,
}

impl NormFrameTable {
    /// Construct and validate a per-block frame alignment table for `blocks` carrier blocks.
    pub fn new(blocks: u32, candidates: Vec<u8>) -> Result<Self> {
        if !(2..=16).contains(&blocks) {
            return Err(invalid(
                "normalization frame table requires blocks in 2..=16 to match Spec",
            ));
        }
        if candidates.len() != blocks as usize {
            return Err(invalid(format!(
                "normalization frame table requires {blocks} entries, got {}",
                candidates.len()
            )));
        }
        if candidates.iter().any(|&c| c >= FRAME_CANDIDATES) {
            return Err(invalid(
                "normalization frame candidate must be in 0..16 (0 = identity, 1..=15 = generators)",
            ));
        }
        Ok(Self { blocks, candidates })
    }

    /// Construct the all-identity normalization frame table for `blocks` carrier blocks.
    pub fn identity(blocks: u32) -> Result<Self> {
        if !(2..=16).contains(&blocks) {
            return Err(invalid(
                "normalization frame table requires blocks in 2..=16 to match Spec",
            ));
        }
        Ok(Self {
            blocks,
            candidates: vec![0u8; blocks as usize],
        })
    }

    /// Number of 8D carrier blocks covered by this table.
    pub fn blocks(&self) -> u32 {
        self.blocks
    }

    /// Whole-number candidate slice (`u8` values in `0..16`) for D11 export.
    pub fn export_words(&self) -> &[u8] {
        &self.candidates
    }

    /// Information capacity in bits across all `blocks` normalized blocks (`4 * blocks`).
    pub fn information_bits(&self) -> u32 {
        self.blocks * BITS_PER_NORMALIZED_BLOCK
    }

    /// Resolve the `Cl(0,6)` frame action (`0..64`) for `block`.
    pub fn block_frame(&self, block: usize) -> Result<u8> {
        if block >= self.blocks as usize {
            return Err(invalid("normalization block index out of range"));
        }
        let candidate = self.candidates[block];
        Ok(if candidate == 0 {
            geometry::IDENTITY
        } else {
            geometry::GENERATORS[usize::from(candidate - 1)]
        })
    }
}

/// Return the canonical 8D coordinates `[i32; 8]` of orbit root `root` (`0..16`).
pub fn orbit_root_coords(root: u8) -> Result<[i32; 8]> {
    if usize::from(root) >= ORBIT_ROOTS {
        return Err(invalid("orbit root index must be in 0..16"));
    }
    let src = geometry::ROOTS[usize::from(root)];
    let mut out = [0i32; 8];
    for (d, slot) in out.iter_mut().enumerate() {
        *slot = i32::from(src[d]);
    }
    Ok(out)
}

/// Identify which of the 16 canonical orbit roots matches `block`, or `None` if off-orbit.
pub fn identify_orbit_root(block: &[i32; 8]) -> Option<u8> {
    for r in 0..ORBIT_ROOTS {
        if geometry::ROOTS[r]
            .iter()
            .zip(block.iter())
            .all(|(&rc, &bc)| i32::from(rc) == bc)
        {
            return Some(r as u8);
        }
    }
    None
}

/// Rotate an arbitrary 8D integer vector `x` by `Cl(0,6)` signed-axis action `frame` (`0..64`).
pub fn transform_block_by_frame(frame: u8, x: &[i32; 8]) -> Result<[i32; 8]> {
    if usize::from(frame) >= GROUP_FRAMES {
        return Err(invalid("frame index must be in 0..64"));
    }
    let mut out = [0i32; 8];
    let base = usize::from(frame) * 8;
    for (d, slot) in out.iter_mut().enumerate() {
        let axis = geometry::AXES[base + d];
        let v = x[usize::from(axis & 7)];
        *slot = if (axis & 8) != 0 { -v } else { v };
    }
    Ok(out)
}

/// Exact integer cost `(l1_distance, hamming_distance)` between `x` and `multiplicity * ROOTS[root]`,
/// built by repeated integer addition (`0` multiply instructions, matching `discrete.cu` `snap`).
pub fn root_scaled_cost(x: &[i32; 8], multiplicity: u32, root: u8) -> Result<(u32, u32)> {
    if multiplicity == 0 || multiplicity > MAX_MULTIPLICITY {
        return Err(invalid("multiplicity must be in 1..=64"));
    }
    if usize::from(root) >= ORBIT_ROOTS {
        return Err(invalid("orbit root index must be in 0..16"));
    }
    let r_coords = geometry::ROOTS[usize::from(root)];
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for d in 0..8 {
        let v = i32::from(r_coords[d]);
        let mut y = 0i32;
        for _ in 0..multiplicity {
            y = y.saturating_add(v);
        }
        distance = distance.saturating_add(x[d].saturating_sub(y).unsigned_abs());
        if x[d] != y {
            hamming += 1;
        }
    }
    Ok((distance, hamming))
}

/// Normalize one 8D integer block `x` with term multiplicity `multiplicity` (`1..=64`)
/// and `Cl(0,6)` frame alignment `frame` (`0..64`).
pub fn normalize_block_with_frame(
    x: &[i32; 8],
    multiplicity: u32,
    frame: u8,
) -> Result<SnapProjection> {
    if multiplicity == 0 || multiplicity > MAX_MULTIPLICITY {
        return Err(invalid("multiplicity must be in 1..=64"));
    }
    if usize::from(frame) >= GROUP_FRAMES {
        return Err(invalid("frame index must be in 0..64"));
    }
    let mut best_root = 0u8;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for r in 0..(ORBIT_ROOTS as u8) {
        let (distance, hamming) = root_scaled_cost(x, multiplicity, r)?;
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best_root = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    let base_coords = orbit_root_coords(best_root)?;
    let normalized_block = transform_block_by_frame(frame, &base_coords)?;
    let aligned_root_index = identify_orbit_root(&normalized_block)
        .ok_or_else(|| invalid("signed-axis action left the 16-root orbit"))?;
    Ok(SnapProjection {
        root_index: best_root,
        aligned_root_index,
        l1_distance: best_distance,
        hamming_distance: best_hamming,
        normalized_block,
    })
}

/// Normalize one 8D integer block `x` under the selected [`NormalizationMode`].
/// For `UnnormalizedControlV1`, returns `x` unchanged with `root_index = 0`.
pub fn normalize_block_mode(
    mode: NormalizationMode,
    x: &[i32; 8],
    multiplicity: u32,
    frame: u8,
) -> Result<[i32; 8]> {
    match mode {
        NormalizationMode::ScaleMatchedSnapV1 => {
            Ok(normalize_block_with_frame(x, multiplicity, frame)?.normalized_block)
        }
        NormalizationMode::UnitSnapControlV1 => {
            Ok(normalize_block_with_frame(x, 1, frame)?.normalized_block)
        }
        NormalizationMode::UnnormalizedControlV1 => {
            if multiplicity == 0 || multiplicity > MAX_MULTIPLICITY {
                return Err(invalid("multiplicity must be in 1..=64"));
            }
            transform_block_by_frame(frame, x)
        }
    }
}

/// Normalize a full multi-block carrier vector (`B` blocks of 8 integers) and export
/// both the normalized `[i32; 8]` blocks and their D11 4-bit root indices (`u8` in `0..16`).
pub fn normalize_carrier_vector(
    table: &NormFrameTable,
    blocks_in: &[[i32; 8]],
    multiplicity: u32,
) -> Result<(Vec<[i32; 8]>, Vec<u8>)> {
    if blocks_in.len() != table.blocks() as usize {
        return Err(invalid("input block count does not match NormFrameTable"));
    }
    let mut normalized = Vec::with_capacity(blocks_in.len());
    let mut exported_roots = Vec::with_capacity(blocks_in.len());
    for (b, block) in blocks_in.iter().enumerate() {
        let frame = table.block_frame(b)?;
        let proj = normalize_block_with_frame(block, multiplicity, frame)?;
        normalized.push(proj.normalized_block);
        exported_roots.push(proj.aligned_root_index);
    }
    Ok((normalized, exported_roots))
}

/// Endpoint address distance (`discrete.cu` `address_distance`) between a multi-block
/// vector `blocks` (`B >= 2`) and cell `cell` (`0..256`).
/// On normalized vectors (`OrbitRootSnapNormV1`), this distance is guaranteed to lie
/// in the exact 5-value set `{0, 4, 8, 12, 16}`.
pub fn endpoint_address_distance(blocks: &[[i32; 8]], cell: u8) -> Result<u32> {
    if blocks.len() < 2 {
        return Err(invalid(
            "endpoint address distance requires at least 2 blocks",
        ));
    }
    let first_root = cell / 16;
    let last_root = cell % 16;
    let (d_first, _) = root_scaled_cost(&blocks[0], 1, first_root)?;
    let (d_last, _) = root_scaled_cost(&blocks[blocks.len() - 1], 1, last_root)?;
    Ok(d_first + d_last)
}

/// Words in one Flock trace record header (`discrete.cu` `TRACE_ROW`).
pub const STRUCTURE_TRACE_HEADER_WORDS: usize = 4;
/// Words per model level in one Flock trace record (`discrete.cu` `TRACE_LEVEL`).
pub const STRUCTURE_TRACE_LEVEL_WORDS: usize = 28;
/// The registered endpoint `address_distance` set `{0, 4, 8, 12, 16}`.
pub const ENDPOINT_DISTANCE_SET: [u32; 5] = [0, 4, 8, 12, 16];

/// Structural-norm stability counts over traced stage outputs. Final outputs are
/// judged on the actual output block; level consensus roots on the root table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StructuralSummary {
    pub stage_outputs: u64,
    pub valid_root_outputs: u64,
    pub invalid_root_outputs: u64,
    pub norm_failures: u64,
    /// Counts for `ENDPOINT_DISTANCE_SET`, in that order, over every row x 256 cells.
    pub endpoint_distance_histogram: [u64; 5],
    pub endpoint_distance_off_set: u64,
    pub met: bool,
}

impl StructuralSummary {
    /// Add another batch's counts (checked) and recompute `met`.
    pub fn absorb(&mut self, other: &StructuralSummary) -> Result<()> {
        let add = |a: u64, b: u64| a.checked_add(b).ok_or_else(|| invalid("summary overflow"));
        self.stage_outputs = add(self.stage_outputs, other.stage_outputs)?;
        self.valid_root_outputs = add(self.valid_root_outputs, other.valid_root_outputs)?;
        self.invalid_root_outputs = add(self.invalid_root_outputs, other.invalid_root_outputs)?;
        self.norm_failures = add(self.norm_failures, other.norm_failures)?;
        for (a, &b) in self
            .endpoint_distance_histogram
            .iter_mut()
            .zip(&other.endpoint_distance_histogram)
        {
            *a = add(*a, b)?;
        }
        self.endpoint_distance_off_set = add(
            self.endpoint_distance_off_set,
            other.endpoint_distance_off_set,
        )?;
        self.met = self.computed_met();
        Ok(())
    }

    fn computed_met(&self) -> bool {
        self.invalid_root_outputs == 0
            && self.norm_failures == 0
            && self.endpoint_distance_off_set == 0
    }
}

/// A traced root id word as a root index, or `None` for `ABSENT` / out-of-table ids.
fn trace_root(word: u32) -> Option<u8> {
    u8::try_from(word)
        .ok()
        .filter(|&r| usize::from(r) < ORBIT_ROOTS)
}

fn canonical_norms(block: &[i32; 8]) -> bool {
    let inv = BlockNormInvariants::measure(block);
    inv.l1_norm == CANONICAL_BLOCK_L1
        && inv.l2_norm_sq == CANONICAL_BLOCK_L2_SQ
        && inv.linfty_norm == CANONICAL_BLOCK_LINFTY
}

/// Score the structural-norm bar term over one traced forward batch.
///
/// `trace` is the Flock region only (`rows * blocks * stride` words, record
/// layout `[frame, out_root, terms, payload]` then per level
/// `[gate+1, found, consensus, eligible, kept[8], actions[8], carried[8]]`).
/// `outputs` is the actual `[rows, blocks]` output block vectors. Stage outputs
/// are every final output plus every level consensus root with `found > 0`.
pub fn structural_summary(
    trace: &[u32],
    stride: usize,
    rows: usize,
    blocks: usize,
    stages: usize,
    outputs: &[[i32; 8]],
) -> Result<StructuralSummary> {
    let records = rows
        .checked_mul(blocks)
        .ok_or_else(|| invalid("trace record count overflow"))?;
    let needed = stages
        .checked_mul(STRUCTURE_TRACE_LEVEL_WORDS)
        .and_then(|w| w.checked_add(STRUCTURE_TRACE_HEADER_WORDS))
        .ok_or_else(|| invalid("trace stride overflow"))?;
    if blocks < 2
        || stride < needed
        || records.checked_mul(stride) != Some(trace.len())
        || outputs.len() != records
    {
        return Err(invalid(
            "structural summary trace/output shape differs from rows x blocks x stride",
        ));
    }
    let mut summary = StructuralSummary::default();
    let mut judge = |root: Option<u8>, block: Option<&[i32; 8]>| -> Result<()> {
        summary.stage_outputs += 1;
        let Some(root) = root else {
            summary.invalid_root_outputs += 1;
            return Ok(());
        };
        summary.valid_root_outputs += 1;
        let vector = match block {
            Some(b) => *b,
            None => orbit_root_coords(root)?,
        };
        if !canonical_norms(&vector) {
            summary.norm_failures += 1;
        }
        Ok(())
    };
    for (record, output) in trace.chunks_exact(stride).zip(outputs) {
        judge(trace_root(record[1]), Some(output))?;
        for level in 0..stages {
            let lv = &record[STRUCTURE_TRACE_HEADER_WORDS + level * STRUCTURE_TRACE_LEVEL_WORDS..];
            if lv[1] != 0 {
                judge(trace_root(lv[2]), None)?;
            }
        }
    }
    for row in outputs.chunks_exact(blocks) {
        for cell in 0..=u8::MAX {
            let d = endpoint_address_distance(row, cell)?;
            match ENDPOINT_DISTANCE_SET.iter().position(|&v| v == d) {
                Some(i) => summary.endpoint_distance_histogram[i] += 1,
                None => summary.endpoint_distance_off_set += 1,
            }
        }
    }
    summary.met = summary.computed_met();
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::ABSENT;

    /// One record: final root `out`, level 0 consensus `m0` (found 1), level 1 empty.
    fn record(out: u32, m0: u32) -> Vec<u32> {
        let mut r = vec![ABSENT; STRUCTURE_TRACE_HEADER_WORDS + 2 * STRUCTURE_TRACE_LEVEL_WORDS];
        r[1] = out;
        let l0 = STRUCTURE_TRACE_HEADER_WORDS;
        r[l0..l0 + 4].copy_from_slice(&[1, 1, m0, 1]);
        let l1 = l0 + STRUCTURE_TRACE_LEVEL_WORDS;
        r[l1..l1 + 4].copy_from_slice(&[1, 0, ABSENT, 0]);
        r
    }

    #[test]
    fn structural_summary_counts_valid_stage_outputs() -> Result<()> {
        let roots = [[0u8, 5], [9, 15]];
        let mut trace = Vec::new();
        let mut outputs = Vec::new();
        for row in roots {
            for r in row {
                trace.extend(record(u32::from(r), u32::from(15 - r)));
                outputs.push(orbit_root_coords(r)?);
            }
        }
        let stride = STRUCTURE_TRACE_HEADER_WORDS + 2 * STRUCTURE_TRACE_LEVEL_WORDS;
        let s = structural_summary(&trace, stride, 2, 2, 2, &outputs)?;
        assert_eq!(s.stage_outputs, 8);
        assert_eq!(s.valid_root_outputs, 8);
        assert_eq!(s.invalid_root_outputs, 0);
        assert_eq!(s.norm_failures, 0);
        assert_eq!(s.endpoint_distance_off_set, 0);
        // Histogram = per-row convolution of the per-endpoint root distance counts.
        let mut expected = [0u64; 5];
        for row in roots {
            let count = |root: u8| -> Result<[u64; 5]> {
                let mut c = [0u64; 5];
                for r in 0..(ORBIT_ROOTS as u8) {
                    let (d, _) = root_scaled_cost(&orbit_root_coords(root)?, 1, r)?;
                    c[usize::try_from(d / 4).map_err(|e| invalid(e.to_string()))?] += 1;
                }
                Ok(c)
            };
            let (first, last) = (count(row[0])?, count(row[1])?);
            for (a, &fa) in first.iter().enumerate() {
                for (b, &lb) in last.iter().enumerate() {
                    if fa * lb > 0 {
                        expected[a + b] += fa * lb;
                    }
                }
            }
        }
        assert_eq!(s.endpoint_distance_histogram, expected);
        assert_eq!(s.endpoint_distance_histogram.iter().sum::<u64>(), 512);
        assert_eq!(s.endpoint_distance_histogram[0], 2);
        assert!(s.met);
        Ok(())
    }

    #[test]
    fn structural_summary_flags_bad_outputs() -> Result<()> {
        let stride = STRUCTURE_TRACE_HEADER_WORDS + 2 * STRUCTURE_TRACE_LEVEL_WORDS;
        let mut trace = Vec::new();
        trace.extend(record(ABSENT, 1)); // ABSENT final root (C_raw-like)
        trace.extend(record(16, 2)); // out-of-table final root
        trace.extend(record(3, ABSENT)); // found>0 level with ABSENT consensus
        trace.extend(record(4, 4));
        let residual = [1, 1, 1, 0, 0, 0, 0, 0]; // L1 = 3: not a root block
        let outputs = vec![
            orbit_root_coords(0)?,
            orbit_root_coords(1)?,
            residual,
            orbit_root_coords(4)?,
        ];
        let s = structural_summary(&trace, stride, 2, 2, 2, &outputs)?;
        assert_eq!(s.stage_outputs, 8);
        assert_eq!(s.invalid_root_outputs, 3);
        assert_eq!(s.valid_root_outputs, 5);
        assert_eq!(s.norm_failures, 1);
        // Row 1 starts with the residual, so every one of its 256 cells is off-set
        // (odd distance); row 0 is two roots and stays in the set.
        assert_eq!(s.endpoint_distance_off_set, 256);
        assert_eq!(s.endpoint_distance_histogram.iter().sum::<u64>(), 256);
        assert!(!s.met);
        assert!(structural_summary(&trace, stride, 2, 2, 2, &outputs[..3]).is_err());
        assert!(structural_summary(&trace, stride - 1, 2, 2, 2, &outputs).is_err());
        Ok(())
    }

    #[test]
    fn unit_snap_word_default_is_scale_matched_zero() {
        assert_eq!(
            NormalizationMode::default(),
            NormalizationMode::ScaleMatchedSnapV1
        );
        assert!(NormalizationMode::default().is_scale_matched_snap());
        assert_eq!(NormalizationMode::default().device_word().ok(), Some(0));
    }

    #[test]
    fn unit_snap_word_control_is_one() {
        assert_eq!(
            NormalizationMode::UnitSnapControlV1.device_word().ok(),
            Some(1)
        );
        assert!(!NormalizationMode::UnitSnapControlV1.is_scale_matched_snap());
    }

    #[test]
    fn unit_snap_word_maps_unnormalized_control_to_raw() {
        assert_eq!(
            NormalizationMode::UnnormalizedControlV1.device_word().ok(),
            Some(2)
        );
    }

    #[test]
    fn unit_snap_word_serialized_names_unchanged() -> Result<()> {
        let modes = [
            NormalizationMode::ScaleMatchedSnapV1,
            NormalizationMode::UnitSnapControlV1,
            NormalizationMode::UnnormalizedControlV1,
        ];
        let text = serde_json::to_string(&modes).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(
            text,
            r#"["scale_matched_snap_v1","unit_snap_control_v1","unnormalized_control_v1"]"#
        );
        let back: Vec<NormalizationMode> =
            serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(back, modes);
        Ok(())
    }

    /// Control 1: Across arbitrary positive, negative, zero, and high-amplitude
    /// inputs `x` and multiplicities `n in 1..=16`, `OrbitRootSnapNormV1`
    /// outputs an exact orbit root with `L1 = 4`, `L2^2 = 8`, `L_infty = 2`,
    /// and `support = 2`.
    #[test]
    fn norm_stability_and_bounded_range() -> Result<()> {
        let samples: [[i32; 8]; 8] = [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [100, -100, 3, -4, 0, 1, -1, 2],
            [-50, -50, -50, -50, -50, -50, -50, -50],
            [7, 19, -23, 41, -3, 0, 12, -8],
            [1, 0, 0, 0, 0, 0, 0, 0],
            [2, 2, 2, 2, 2, 2, 2, 2],
            [-2, 0, 2, 0, -2, 0, 2, 0],
            [999, 998, -997, 0, 5, -5, 1, -1],
        ];
        for n in 1..=16 {
            for frame in [geometry::IDENTITY, geometry::GENERATORS[0], 63u8] {
                for sample in &samples {
                    let proj = normalize_block_with_frame(sample, n, frame)?;
                    assert!((proj.root_index as usize) < ORBIT_ROOTS);
                    assert!((proj.aligned_root_index as usize) < ORBIT_ROOTS);
                    let inv = BlockNormInvariants::measure(&proj.normalized_block);
                    assert!(inv.is_canonical_unit_root());
                    assert_eq!(inv.l1_norm, CANONICAL_BLOCK_L1);
                    assert_eq!(inv.l2_norm_sq, CANONICAL_BLOCK_L2_SQ);
                    assert_eq!(inv.linfty_norm, CANONICAL_BLOCK_LINFTY);
                    assert_eq!(inv.support_size, CANONICAL_BLOCK_SUPPORT);
                }
            }
        }
        Ok(())
    }

    /// Control 2: Idempotence / two-pass stability (`Norm_1(Norm_n(x)) == Norm_n(x)`).
    /// Every canonical root snaps to itself at `n = 1` with zero L1 and Hamming
    /// displacement, and applying normalization twice under identity frame is
    /// strictly idempotent on all inputs.
    #[test]
    fn idempotence_two_pass_stability() -> Result<()> {
        for r in 0..(ORBIT_ROOTS as u8) {
            let coords = orbit_root_coords(r)?;
            let proj = normalize_block_with_frame(&coords, 1, geometry::IDENTITY)?;
            assert_eq!(proj.root_index, r);
            assert_eq!(proj.aligned_root_index, r);
            assert_eq!(proj.l1_distance, 0);
            assert_eq!(proj.hamming_distance, 0);
            assert_eq!(proj.normalized_block, coords);
        }

        for seed in 0..64i32 {
            let arbitrary = [
                seed * 3 - 17,
                -seed * 5 + 11,
                seed * 7 - 29,
                -seed * 2 + 3,
                seed - 8,
                -seed * 4 + 19,
                seed * 6 - 5,
                -seed + 2,
            ];
            for n in [1u32, 3, 7, 12] {
                let first = normalize_block_with_frame(&arbitrary, n, geometry::IDENTITY)?;
                let second =
                    normalize_block_with_frame(&first.normalized_block, 1, geometry::IDENTITY)?;
                assert_eq!(first.normalized_block, second.normalized_block);
                assert_eq!(first.aligned_root_index, second.aligned_root_index);
                assert_eq!(second.l1_distance, 0);
                assert_eq!(second.hamming_distance, 0);
            }
        }
        Ok(())
    }

    /// Control 3: Scale equivariance on positive integer multiples of every orbit root,
    /// and strict separation between scale-matched `snap_n` and un-scaled `snap_1`
    /// when a dominant scaled root `n * rho_r` is perturbed by bounded off-support noise.
    #[test]
    fn scale_equivariance_on_orbit_multiples() -> Result<()> {
        for r in 0..(ORBIT_ROOTS as u8) {
            let base = orbit_root_coords(r)?;
            for n in 1..=16u32 {
                let mut scaled = [0i32; 8];
                for d in 0..8 {
                    scaled[d] = base[d] * (n as i32);
                }
                let matched = normalize_block_with_frame(&scaled, n, geometry::IDENTITY)?;
                assert_eq!(matched.root_index, r);
                assert_eq!(matched.l1_distance, 0);
                assert_eq!(matched.hamming_distance, 0);
                assert_eq!(matched.normalized_block, base);

                let (unscaled_d1, _) = root_scaled_cost(&scaled, 1, r)?;
                assert_eq!(unscaled_d1, 4 * (n - 1));
            }
        }

        // Find a pair of orbit roots `(0, r_orth)` with disjoint support and
        // form the 4-term sum `3 * rho_0 + 1 * rho_{r_orth}` (3 votes for root 0,
        // 1 vote for `r_orth`). Un-scaled `snap_1` ties on L1 distance (12 vs 12)
        // and picks the 1-vote minority root `r_orth` by Hamming distance (2 < 4),
        // whereas scale-matched `snap_4` (`n = 4`) strictly selects the 3-vote
        // majority root `0` (`L1 = 8 < 24`).
        let r0 = orbit_root_coords(0)?;
        let r_orth = (1..(ORBIT_ROOTS as u8))
            .find(|&r| {
                orbit_root_coords(r)
                    .map(|c| (0..8).all(|d| r0[d] == 0 || c[d] == 0))
                    .unwrap_or(false)
            })
            .ok_or_else(|| invalid("expected an orthogonal orbit root with disjoint support"))?;
        let c_orth = orbit_root_coords(r_orth)?;
        let mut mixed = [0i32; 8];
        for d in 0..8 {
            mixed[d] = 3 * r0[d] + c_orth[d];
        }
        let matched = normalize_block_with_frame(&mixed, 4, geometry::IDENTITY)?;
        let unscaled = normalize_block_with_frame(&mixed, 1, geometry::IDENTITY)?;
        assert_eq!(matched.root_index, 0);
        assert_eq!(matched.l1_distance, 8);
        assert_eq!(unscaled.root_index, r_orth);
        assert_eq!(unscaled.l1_distance, 12);
        Ok(())
    }

    /// Control 4: Deterministic tie-breaking on the zero vector and across all
    /// 120 pairs of distinct orbit roots (`r1 < r2`).
    #[test]
    fn deterministic_tie_breaking_on_zero_and_midpoints() -> Result<()> {
        let zero = [0i32; 8];
        for n in 1..=8u32 {
            let proj = normalize_block_with_frame(&zero, n, geometry::IDENTITY)?;
            assert_eq!(proj.root_index, 0);
            assert_eq!(proj.l1_distance, 4 * n);
            assert_eq!(proj.hamming_distance, 2);
        }

        for r1 in 0..(ORBIT_ROOTS as u8) {
            let c1 = orbit_root_coords(r1)?;
            for r2 in (r1 + 1)..(ORBIT_ROOTS as u8) {
                let c2 = orbit_root_coords(r2)?;
                let mut sum_ab = [0i32; 8];
                let mut sum_ba = [0i32; 8];
                for d in 0..8 {
                    sum_ab[d] = c1[d] + c2[d];
                    sum_ba[d] = c2[d] + c1[d];
                }
                let proj_ab = normalize_block_with_frame(&sum_ab, 2, geometry::IDENTITY)?;
                let proj_ba = normalize_block_with_frame(&sum_ba, 2, geometry::IDENTITY)?;
                assert_eq!(proj_ab, proj_ba);
                let cost_r1 = root_scaled_cost(&sum_ab, 2, r1)?;
                let cost_r2 = root_scaled_cost(&sum_ab, 2, r2)?;
                assert_eq!(cost_r1, cost_r2);
                assert!(proj_ab.root_index <= r1);
            }
        }
        Ok(())
    }

    /// Control 5: Exact equivariance under all 64 `Cl(0,6)` signed-axis frames
    /// and D11 whole-number export verification (`NormFrameTable`).
    #[test]
    fn cl06_frame_equivariance_across_all_64_actions() -> Result<()> {
        for frame in 0..(GROUP_FRAMES as u8) {
            for r in 0..(ORBIT_ROOTS as u8) {
                let coords = orbit_root_coords(r)?;
                for n in [1u32, 4, 8] {
                    let mut scaled = [0i32; 8];
                    for d in 0..8 {
                        scaled[d] = coords[d] * (n as i32);
                    }
                    let rotated_input = transform_block_by_frame(frame, &scaled)?;
                    let norm_of_rotated =
                        normalize_block_with_frame(&rotated_input, n, geometry::IDENTITY)?;
                    let rotated_of_norm = normalize_block_with_frame(&scaled, n, frame)?;
                    assert_eq!(
                        norm_of_rotated.normalized_block,
                        rotated_of_norm.normalized_block
                    );
                    assert_eq!(
                        norm_of_rotated.aligned_root_index,
                        rotated_of_norm.aligned_root_index
                    );
                }
            }
        }

        let table = NormFrameTable::new(4, vec![0, 1, 5, 15])?;
        assert_eq!(table.blocks(), 4);
        assert_eq!(table.export_words(), &[0, 1, 5, 15]);
        assert_eq!(table.information_bits(), 16);
        let in_blocks = [
            orbit_root_coords(0)?,
            orbit_root_coords(3)?,
            orbit_root_coords(7)?,
            orbit_root_coords(12)?,
        ];
        let (norm_blocks, exported_roots) = normalize_carrier_vector(&table, &in_blocks, 1)?;
        assert_eq!(norm_blocks.len(), 4);
        assert_eq!(exported_roots.len(), 4);
        for (b, block) in norm_blocks.iter().enumerate() {
            assert!(BlockNormInvariants::measure(block).is_canonical_unit_root());
            assert_eq!(identify_orbit_root(block), Some(exported_roots[b]));
        }
        Ok(())
    }

    /// Control 6: Multi-stage residual accumulation across `S = 16` stages.
    /// Without normalization (`UnnormalizedControlV1`), block L1 norm grows to
    /// `64` and endpoint address distance leaves `{0, 4, 8, 12, 16}`. With
    /// `ScaleMatchedSnapV1`, block L1 norm stays `4` and endpoint address
    /// distance stays in `{0, 4, 8, 12, 16}` at every stage.
    #[test]
    fn multistage_residual_accumulation_vs_unnormalized_drift() -> Result<()> {
        let stages = 16u32;
        let mut raw_blocks = [orbit_root_coords(0)?, orbit_root_coords(5)?];
        let mut snap_blocks = raw_blocks;

        for s in 1..=stages {
            let delta0 = orbit_root_coords(((s * 3) % 16) as u8)?;
            let delta1 = orbit_root_coords(5)?;
            for d in 0..8 {
                raw_blocks[0][d] += delta0[d];
                raw_blocks[1][d] += delta1[d];
                snap_blocks[0][d] += delta0[d];
                snap_blocks[1][d] += delta1[d];
            }
            snap_blocks[0] = normalize_block_mode(
                NormalizationMode::ScaleMatchedSnapV1,
                &snap_blocks[0],
                2,
                geometry::IDENTITY,
            )?;
            snap_blocks[1] = normalize_block_mode(
                NormalizationMode::ScaleMatchedSnapV1,
                &snap_blocks[1],
                2,
                geometry::IDENTITY,
            )?;

            for b in 0..2 {
                let inv = BlockNormInvariants::measure(&snap_blocks[b]);
                assert!(inv.is_canonical_unit_root());
            }
            for cell in [0u8, 5, 85, 255] {
                let d_snap = endpoint_address_distance(&snap_blocks, cell)?;
                assert!(matches!(d_snap, 0 | 4 | 8 | 12 | 16));
            }
        }

        let raw_inv1 = BlockNormInvariants::measure(&raw_blocks[1]);
        assert_eq!(raw_inv1.l1_norm, 4 * (stages + 1));
        assert_eq!(raw_inv1.linfty_norm, 2 * (stages + 1));
        assert!(!raw_inv1.on_canonical_orbit);
        let d_raw = endpoint_address_distance(&raw_blocks, 0)?;
        assert!(d_raw > 16);
        Ok(())
    }
    #[test]
    fn stored_mode_missing_key_is_default_and_other_modes_are_not() {
        use serde_json::json;
        assert!(stored_mode_is_default(&json!({})));
        assert!(stored_mode_is_default(
            &json!({"normalization_mode": "scale_matched_snap_v1"})
        ));
        assert!(!stored_mode_is_default(
            &json!({"normalization_mode": "unit_snap_control_v1"})
        ));
        assert!(!stored_mode_is_default(
            &json!({"normalization_mode": "unnormalized_control_v1"})
        ));
        assert!(!stored_mode_is_default(&json!({"normalization_mode": 7})));
    }
}
