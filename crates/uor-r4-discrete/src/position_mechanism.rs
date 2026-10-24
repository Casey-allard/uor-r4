//! Exact prefix-holonomy position mechanism (`PositionHolonomyV1`) and finite
//! operator controls on the generated `Cl(0,6)` 64-frame / 16-root E8 orbit
//! (`docs/roadmap/items/m1-position-mechanism.md`, issue #2843).
//!
//! Host functions in this module are finite algebraic and metadata controls
//! over the build-time generated `geometry.rs` tables (`COMPOSE`, `INVERSE`,
//! `GENERATORS`, `IDENTITY`, `AXES`, `ROOTS`). Model training and corpus
//! evaluation remain on the GPU (`CHARTER.md` §3.5, D24). No floating-point
//! types or literals (D32), no dense matrix multiplication (D25), and no
//! smooth gates or normalized exponentials (D23) appear on this path.
use crate::metadata::MAX_CONTEXT;
use crate::{Error, ReadSpec, Result, Spec};
use serde::{Deserialize, Serialize};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

/// Number of orbit roots per 8D block in the discrete M1 carrier.
pub const ORBIT_ROOTS: usize = 16;

/// Number of signed-permutation frames in the generated `Cl(0,6)` group.
pub const GROUP_FRAMES: usize = 64;

/// Number of legal step candidates per `(block, root)` slot: candidate `0` is
/// `IDENTITY`, and candidates `1..=15` are the 15 `Cl(0,6)` bivector generators.
pub const STEP_CANDIDATES: u8 = 16;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// Position transport mode for causal read comparisons (`#2843`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionMode {
    /// Causal prefix scan `F_{p,b} = COMPOSE[u_{p,b} * 64 + F_{p-1,b}]` with
    /// relative holonomy `H_{t <- s, b} = COMPOSE[F_{t,b} * 64 + INVERSE[F_{s,b}]]`.
    PrefixHolonomyV1,
    /// Parameter-matched endpoint-only control: uses the identical `16 * B`
    /// step table, but forms `COMPOSE[u_{t,b} * 64 + INVERSE[u_{s,b}]]` from
    /// the endpoints `(s, t)` alone, carrying zero intervening path order.
    EndpointControlV1,
    /// Identity transport control (`IDENTITY` at every position and interval).
    IdentityControlV1,
}

impl PositionMode {
    /// Optional device position word `s[14]`; zero is reserved for legacy BFS.
    pub fn device_word(self) -> u32 {
        match self {
            Self::PrefixHolonomyV1 => 1,
            Self::EndpointControlV1 => 2,
            Self::IdentityControlV1 => 3,
        }
    }
}

/// Opt-in position configuration; absence preserves the legacy read exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionConfig {
    pub mode: PositionMode,
    pub steps: PositionStepTable,
}

impl PositionConfig {
    pub fn new(mode: PositionMode, steps: PositionStepTable) -> Self {
        Self { mode, steps }
    }

    pub fn validate(&self, spec: Spec, read_spec: ReadSpec) -> Result<()> {
        read_spec.validate(spec)?;
        if read_spec == ReadSpec::Route {
            return Err(invalid("position transport needs a Flock or Relation read"));
        }
        self.steps.validate()?;
        if self.steps.blocks() != spec.blocks {
            return Err(invalid(
                "position step table blocks differ from the model spec",
            ));
        }
        Ok(())
    }
}

/// Whole-number step table assigning one of 16 legal `Cl(0,6)` move candidates
/// (`0..16`) to each `(block, orbit_root)` pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionStepTable {
    blocks: u32,
    candidates: Vec<u8>,
}

impl PositionStepTable {
    /// Construct and validate a step table for `blocks` 8D carrier blocks.
    pub fn new(blocks: u32, candidates: Vec<u8>) -> Result<Self> {
        let table = Self { blocks, candidates };
        table.validate()?;
        Ok(table)
    }

    /// Validate constructor-built and deserialized tables identically.
    pub fn validate(&self) -> Result<()> {
        if !(2..=16).contains(&self.blocks) {
            return Err(invalid(
                "position step table requires blocks in 2..=16 to match Spec",
            ));
        }
        let expected = (self.blocks as usize) * ORBIT_ROOTS;
        if self.candidates.len() != expected {
            return Err(invalid(format!(
                "position step table requires {expected} entries (blocks * 16), got {}",
                self.candidates.len()
            )));
        }
        if self.candidates.iter().any(|&c| c >= STEP_CANDIDATES) {
            return Err(invalid(
                "position step candidate must be in 0..16 (0 = identity, 1..=15 = generators)",
            ));
        }
        Ok(())
    }

    /// Construct the all-identity step table for `blocks` carrier blocks.
    pub fn identity(blocks: u32) -> Result<Self> {
        if !(2..=16).contains(&blocks) {
            return Err(invalid(
                "position step table requires blocks in 2..=16 to match Spec",
            ));
        }
        Ok(Self {
            blocks,
            candidates: vec![0u8; (blocks as usize) * ORBIT_ROOTS],
        })
    }

    /// Number of 8D carrier blocks covered by this table.
    pub fn blocks(&self) -> u32 {
        self.blocks
    }

    /// Total number of discrete parameters (`16 * blocks`).
    pub fn parameter_count(&self) -> usize {
        self.candidates.len()
    }

    /// Whole-number candidate slice (`u8` values in `0..16`) for D11 export.
    pub fn export_words(&self) -> &[u8] {
        &self.candidates
    }

    /// Resolve the `Cl(0,6)` frame action (`0..64`) for `(block, root)`.
    pub fn step_action(&self, block: usize, root: u8) -> Result<u8> {
        if block >= self.blocks as usize {
            return Err(invalid("position block index out of range"));
        }
        if usize::from(root) >= ORBIT_ROOTS {
            return Err(invalid("orbit root index must be in 0..16"));
        }
        let candidate = self.candidates[block * ORBIT_ROOTS + usize::from(root)];
        Ok(candidate_to_frame(candidate))
    }
}

/// Map a legal candidate `0..16` to its exact `Cl(0,6)` frame index `0..64`.
pub fn candidate_to_frame(candidate: u8) -> u8 {
    if candidate == 0 {
        geometry::IDENTITY
    } else {
        geometry::GENERATORS[usize::from(candidate - 1)]
    }
}

/// Exact group multiplication `a ∘ b` in the 64-element `Cl(0,6)` table.
pub fn compose_frames(a: u8, b: u8) -> Result<u8> {
    if usize::from(a) >= GROUP_FRAMES || usize::from(b) >= GROUP_FRAMES {
        return Err(invalid("frame index must be in 0..64"));
    }
    Ok(geometry::COMPOSE[usize::from(a) * GROUP_FRAMES + usize::from(b)])
}

/// Exact group inverse `a⁻¹` in the 64-element `Cl(0,6)` table.
pub fn invert_frame(a: u8) -> Result<u8> {
    if usize::from(a) >= GROUP_FRAMES {
        return Err(invalid("frame index must be in 0..64"));
    }
    Ok(geometry::INVERSE[usize::from(a)])
}

/// Compute cumulative causal prefix frames `F_{0,b}, ..., F_{L-1,b}` in `0..64`
/// for one block across a window of orbit roots (`1..=243`).
pub fn prefix_frames(
    table: &PositionStepTable,
    block: usize,
    window_roots: &[u8],
) -> Result<Vec<u8>> {
    if window_roots.is_empty() || window_roots.len() > MAX_CONTEXT {
        return Err(invalid("position window length must be in 1..=243"));
    }
    let mut frames = Vec::with_capacity(window_roots.len());
    let mut acc = geometry::IDENTITY;
    for &root in window_roots {
        let step = table.step_action(block, root)?;
        acc = compose_frames(step, acc)?;
        frames.push(acc);
    }
    Ok(frames)
}

/// Compute the relative position transport `H_{to <- from, b}` in `0..64` for
/// `from_pos <= to_pos < window_roots.len()` under the selected [`PositionMode`].
pub fn relative_holonomy(
    mode: PositionMode,
    table: &PositionStepTable,
    block: usize,
    window_roots: &[u8],
    from_pos: usize,
    to_pos: usize,
) -> Result<u8> {
    if window_roots.is_empty() || window_roots.len() > MAX_CONTEXT {
        return Err(invalid("position window length must be in 1..=243"));
    }
    if from_pos > to_pos || to_pos >= window_roots.len() {
        return Err(invalid(
            "relative holonomy requires from_pos <= to_pos < window length",
        ));
    }
    for &r in window_roots {
        if usize::from(r) >= ORBIT_ROOTS {
            return Err(invalid("orbit root index must be in 0..16"));
        }
    }
    if block >= table.blocks() as usize {
        return Err(invalid("position block index out of range"));
    }
    match mode {
        PositionMode::IdentityControlV1 => Ok(geometry::IDENTITY),
        PositionMode::EndpointControlV1 => {
            if from_pos == to_pos {
                return Ok(geometry::IDENTITY);
            }
            let u_from = table.step_action(block, window_roots[from_pos])?;
            let u_to = table.step_action(block, window_roots[to_pos])?;
            let inv_from = invert_frame(u_from)?;
            compose_frames(u_to, inv_from)
        }
        PositionMode::PrefixHolonomyV1 => {
            let frames = prefix_frames(table, block, window_roots)?;
            let inv_from = invert_frame(frames[from_pos])?;
            compose_frames(frames[to_pos], inv_from)
        }
    }
}

/// Direct ordered product `u_{to} ∘ u_{to-1} ∘ ... ∘ u_{from+1}` along the
/// interval `(from_pos, to_pos]`, used as an exact reference control for
/// `F_{to} ∘ F_{from}⁻¹`.
pub fn direct_step_product(
    table: &PositionStepTable,
    block: usize,
    window_roots: &[u8],
    from_pos: usize,
    to_pos: usize,
) -> Result<u8> {
    if window_roots.is_empty() || window_roots.len() > MAX_CONTEXT {
        return Err(invalid("position window length must be in 1..=243"));
    }
    if from_pos > to_pos || to_pos >= window_roots.len() {
        return Err(invalid(
            "direct step product requires from_pos <= to_pos < window length",
        ));
    }
    let mut prod = geometry::IDENTITY;
    for &root in &window_roots[from_pos + 1..=to_pos] {
        let step = table.step_action(block, root)?;
        prod = compose_frames(step, prod)?;
    }
    Ok(prod)
}

/// Apply signed-coordinate action `frame` (`0..64`) to `root` (`0..16`),
/// returning both the resulting orbit root index (`0..16`) and its exact 8D
/// integer coordinates in `{-2, 0, 2}^8`.
pub fn apply_frame_to_root(frame: u8, root: u8) -> Result<(u8, [i8; 8])> {
    if usize::from(frame) >= GROUP_FRAMES {
        return Err(invalid("frame index must be in 0..64"));
    }
    if usize::from(root) >= ORBIT_ROOTS {
        return Err(invalid("orbit root index must be in 0..16"));
    }
    let src = geometry::ROOTS[usize::from(root)];
    let mut out = [0i8; 8];
    for (d, slot) in out.iter_mut().enumerate() {
        let axis = geometry::AXES[usize::from(frame) * 8 + d];
        let v = src[usize::from(axis & 7)];
        *slot = if (axis & 8) != 0 { -v } else { v };
    }
    let idx = geometry::ROOTS
        .iter()
        .position(|&candidate| candidate == out)
        .map(|i| i as u8)
        .ok_or_else(|| invalid("signed action left the 16-root anchor orbit"))?;
    Ok((idx, out))
}

/// Deterministically shuffle prefix positions `0..window_roots.len()-1`
/// (splitmix64 Fisher-Yates) while keeping the query position `L - 1` fixed,
/// for the shuffled-context-order null control (`C_shuf`).
pub fn shuffle_prefix_roots(window_roots: &[u8], seed: u64) -> Result<Vec<u8>> {
    if window_roots.is_empty() || window_roots.len() > MAX_CONTEXT {
        return Err(invalid("position window length must be in 1..=243"));
    }
    for &r in window_roots {
        if usize::from(r) >= ORBIT_ROOTS {
            return Err(invalid("orbit root index must be in 0..16"));
        }
    }
    let mut out = window_roots.to_vec();
    let prefix_len = out.len().saturating_sub(1);
    let mut state = seed;
    let mut next_u64 = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    for i in (1..prefix_len).rev() {
        let j = (next_u64() % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterned_table(blocks: u32) -> PositionStepTable {
        let mut candidates = Vec::with_capacity((blocks as usize) * ORBIT_ROOTS);
        for b in 0..blocks as usize {
            for r in 0..ORBIT_ROOTS {
                candidates.push(((b * 3 + r) % 15 + 1) as u8);
            }
        }
        PositionStepTable::new(blocks, candidates).unwrap()
    }

    #[test]
    fn identity_table_leaves_all_frames_and_roots_unchanged() {
        let table = PositionStepTable::identity(4).unwrap();
        let window: Vec<u8> = (0..32).map(|i| (i % 16) as u8).collect();
        for block in 0..4 {
            let frames = prefix_frames(&table, block, &window).unwrap();
            assert!(frames.iter().all(|&f| f == geometry::IDENTITY));
            for from in 0..window.len() {
                for to in from..window.len() {
                    let h = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &table,
                        block,
                        &window,
                        from,
                        to,
                    )
                    .unwrap();
                    assert_eq!(h, geometry::IDENTITY);
                    let h_ctrl = relative_holonomy(
                        PositionMode::IdentityControlV1,
                        &table,
                        block,
                        &window,
                        from,
                        to,
                    )
                    .unwrap();
                    assert_eq!(h_ctrl, geometry::IDENTITY);
                }
            }
        }
        for r in 0..ORBIT_ROOTS as u8 {
            let (mapped, coords) = apply_frame_to_root(geometry::IDENTITY, r).unwrap();
            assert_eq!(mapped, r);
            assert_eq!(coords, geometry::ROOTS[usize::from(r)]);
        }
    }

    #[test]
    fn prefix_holonomy_matches_direct_composition_and_transitivity() {
        let table = patterned_table(4);
        let window: Vec<u8> = (0..MAX_CONTEXT)
            .map(|i| ((i * 7 + 3) % ORBIT_ROOTS) as u8)
            .collect();
        for block in 0..4 {
            let frames = prefix_frames(&table, block, &window).unwrap();
            assert_eq!(frames.len(), MAX_CONTEXT);
            for &s in &[0usize, 1, 5, 19, 63, 127, 242] {
                for &t in &[s, (s + 1).min(242), (s + 17).min(242), 242] {
                    let holonomy = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &table,
                        block,
                        &window,
                        s,
                        t,
                    )
                    .unwrap();
                    let direct = direct_step_product(&table, block, &window, s, t).unwrap();
                    assert_eq!(
                        holonomy, direct,
                        "F_t * F_s^-1 must equal direct product on ({s}, {t}]"
                    );
                    let mid = s + (t - s) / 2;
                    let h_sm = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &table,
                        block,
                        &window,
                        s,
                        mid,
                    )
                    .unwrap();
                    let h_mt = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &table,
                        block,
                        &window,
                        mid,
                        t,
                    )
                    .unwrap();
                    assert_eq!(
                        compose_frames(h_mt, h_sm).unwrap(),
                        holonomy,
                        "path holonomy must compose transitively across {s} -> {mid} -> {t}"
                    );
                }
            }
        }
    }

    #[test]
    fn states_and_coordinates_stay_within_exact_integer_bounds() {
        for frame in 0..GROUP_FRAMES as u8 {
            let inv = invert_frame(frame).unwrap();
            assert!((inv as usize) < GROUP_FRAMES);
            assert_eq!(compose_frames(frame, inv).unwrap(), geometry::IDENTITY);
            assert_eq!(compose_frames(inv, frame).unwrap(), geometry::IDENTITY);
            for root in 0..ORBIT_ROOTS as u8 {
                let (mapped, coords) = apply_frame_to_root(frame, root).unwrap();
                assert!((mapped as usize) < ORBIT_ROOTS);
                let l1: i32 = coords.iter().map(|&c| i32::from(c).abs()).sum();
                let l2_sq: i32 = coords.iter().map(|&c| i32::from(c) * i32::from(c)).sum();
                assert_eq!(l1, 4, "every orbit root has L1 norm 4");
                assert_eq!(l2_sq, 8, "every orbit root has squared norm 8");
                assert!(coords.iter().all(|&c| c == -2 || c == 0 || c == 2));
                let (roundtrip, _) = apply_frame_to_root(inv, mapped).unwrap();
                assert_eq!(roundtrip, root);
            }
        }
    }

    #[test]
    fn noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind() {
        let table = patterned_table(4);
        // Find two generator steps that do not commute in Cl(0,6) and map an
        // orbit root to distinct images when their order is swapped.
        let mut found_noncommuting = false;
        'search: for r1 in 0..ORBIT_ROOTS as u8 {
            for r2 in 0..ORBIT_ROOTS as u8 {
                let w_ab = [0u8, r1, r2, 3u8];
                let w_ba = [0u8, r2, r1, 3u8];
                let h_ab =
                    relative_holonomy(PositionMode::PrefixHolonomyV1, &table, 0, &w_ab, 0, 3)
                        .unwrap();
                let h_ba =
                    relative_holonomy(PositionMode::PrefixHolonomyV1, &table, 0, &w_ba, 0, 3)
                        .unwrap();
                let c_ab =
                    relative_holonomy(PositionMode::EndpointControlV1, &table, 0, &w_ab, 0, 3)
                        .unwrap();
                let c_ba =
                    relative_holonomy(PositionMode::EndpointControlV1, &table, 0, &w_ba, 0, 3)
                        .unwrap();
                assert_eq!(
                    c_ab, c_ba,
                    "endpoint control depends only on endpoints (0, 3) and must ignore interior swap"
                );
                if h_ab != h_ba {
                    let (root_ab, _) = apply_frame_to_root(h_ab, w_ab[0]).unwrap();
                    let (root_ba, _) = apply_frame_to_root(h_ba, w_ba[0]).unwrap();
                    if root_ab != root_ba {
                        found_noncommuting = true;
                        break 'search;
                    }
                }
            }
        }
        assert!(
            found_noncommuting,
            "non-commutative Cl(0,6) prefix holonomy must distinguish interior token order swaps"
        );

        let window = [1u8, 2, 5, 9, 14, 3, 7, 11, 4];
        let shuffled = shuffle_prefix_roots(&window, 42).unwrap();
        assert_eq!(shuffled.len(), window.len());
        assert_eq!(
            shuffled.last(),
            window.last(),
            "shuffle must preserve the query token at L - 1"
        );
        let mut sorted_a = window[..window.len() - 1].to_vec();
        let mut sorted_b = shuffled[..shuffled.len() - 1].to_vec();
        sorted_a.sort_unstable();
        sorted_b.sort_unstable();
        assert_eq!(
            sorted_a, sorted_b,
            "shuffle must preserve the prefix token multiset"
        );
    }

    #[test]
    fn cuda_legacy_ranking_preserves_raw_frame_cost() {
        let source = include_str!("discrete.cu");
        let helper = source
            .split("__device__ bool positioned_root_cost")
            .nth(1)
            .unwrap();
        assert!(
            helper.contains("if(s[14]==0)root_cost(query,g,root,distance,hamming);"),
            "legacy ranking must bypass transport and compare the raw root"
        );
        assert!(helper.contains("root_id(transported,g)"));
        assert_eq!(
            source
                .matches("if(!positioned_root_cost(s,query_position")
                .count(),
            2
        );
    }

    #[test]
    fn cuda_position_arms_use_payload_roots_and_span_last() {
        let source = include_str!("discrete.cu");
        assert!(source.contains("int query_payload[8];for(U d=0;d<8;++d)query_payload[d]=u[d];"));
        assert!(source.contains("int* query_position=s[14]==0?query_frame:query_payload;"));
        assert!(source.contains("row,p,b,s,kind,index,candidate,fv"));
        assert!(source.contains("row,last[node],b,s,kind,index,candidate,rv"));
        assert_eq!(
            source
                .matches(
                    "positioned_root_cost(s,query_position,g,transport[node],transport_root[node]"
                )
                .count(),
            2
        );
    }

    #[test]
    fn cuda_snapshot_validation_launch_has_one_position_argument() {
        let source = include_str!("device.rs");
        let body = source
            .split("pub fn from_snapshot")
            .nth(1)
            .unwrap()
            .split("fn gpu_spec")
            .next()
            .unwrap();
        assert!(!body.contains("&self.position_steps"));
        assert!(body.contains(
            ".arg(&spec)\n            .arg(&model.position_steps)\n            .arg(&mut status);"
        ));
        assert!(body.contains(".max(model.position_steps.len())"));
    }

    #[test]
    fn position_config_validates_device_words_and_shapes() {
        use crate::{ReadSpec, Spec};

        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        let relation = ReadSpec::Relation { kept: 8, sweeps: 1 };
        let config = PositionConfig::new(
            PositionMode::PrefixHolonomyV1,
            PositionStepTable::identity(4).unwrap(),
        );
        assert_eq!(config.mode.device_word(), 1);
        assert_eq!(PositionMode::EndpointControlV1.device_word(), 2);
        assert_eq!(PositionMode::IdentityControlV1.device_word(), 3);
        config.validate(spec, relation).unwrap();
        config.validate(spec, ReadSpec::Flock { kept: 8 }).unwrap();
        assert!(config.validate(spec, ReadSpec::Route).is_err());

        let wrong_blocks = PositionConfig::new(
            PositionMode::EndpointControlV1,
            PositionStepTable::identity(2).unwrap(),
        );
        assert!(wrong_blocks.validate(spec, relation).is_err());
    }

    #[test]
    fn deserialized_position_table_is_revalidated() {
        let mut candidates = vec![0u8; 64];
        candidates[0] = 16;
        let config: PositionConfig = serde_json::from_value(serde_json::json!({
            "mode": "prefix_holonomy_v1",
            "steps": {"blocks": 4, "candidates": candidates}
        }))
        .unwrap();
        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        assert!(config
            .validate(spec, ReadSpec::Relation { kept: 8, sweeps: 1 })
            .is_err());
    }

    #[test]
    fn table_validation_and_parameter_matched_serialization() {
        let holonomy_table = patterned_table(4);
        assert_eq!(holonomy_table.parameter_count(), 64);
        assert_eq!(holonomy_table.export_words().len(), 64);
        assert!(PositionStepTable::new(1, vec![0; 16]).is_err());
        assert!(PositionStepTable::new(17, vec![0; 17 * 16]).is_err());
        assert!(PositionStepTable::new(4, vec![0; 63]).is_err());
        let mut bad_candidate = vec![0u8; 64];
        bad_candidate[7] = 16;
        assert!(PositionStepTable::new(4, bad_candidate).is_err());
        assert!(prefix_frames(&holonomy_table, 0, &[]).is_err());
        assert!(prefix_frames(&holonomy_table, 0, &vec![0u8; MAX_CONTEXT + 1]).is_err());
        assert!(prefix_frames(&holonomy_table, 4, &[0u8; 4]).is_err());
        assert!(prefix_frames(&holonomy_table, 0, &[16u8]).is_err());
        assert!(relative_holonomy(
            PositionMode::PrefixHolonomyV1,
            &holonomy_table,
            0,
            &[0, 1, 2],
            2,
            1
        )
        .is_err());
    }
}
