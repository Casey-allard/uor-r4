//! CPU development trainer components.
//!
//! D84 permits only this named module to run CPU model research. GPU results
//! remain the qualification authority. This first unit contains no training:
//! it provides an independent exact writer/admission decision core.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use uor_r4_discrete::normalization_mechanism::{identify_orbit_root, orbit_root_coords};

mod counterfactual;
mod discrete_route;
mod fbs;
mod hamming;
mod memory;
mod panel;
mod train;

pub use counterfactual::{
    CounterfactualBundle, CounterfactualCandidate, CounterfactualConfig, CounterfactualField,
    CounterfactualGroup, CounterfactualStep, CpuCounterfactualTrainer,
};
pub use discrete_route::{
    CpuBvgkBatchReceipt, CpuBvgkDecision, CpuBvgkRecord, CpuDiscreteApplyError,
    CpuDiscreteApplyReceipt, CpuDiscreteFlock, CpuDiscreteProposal, CpuDiscreteProposalApplier,
    CpuDiscreteProposalChooser, CpuDiscreteProposalKind, CpuDiscreteProposalMode,
    CpuDiscreteProposalSample, CpuDiscreteProposalScorer, CpuDiscreteProposalScoring,
    CpuDiscreteRelation, CpuDiscreteRoute, CpuDiscreteScoreError,
};
pub use fbs::{CpuFbsForward, CpuFbsHead, CpuFbsStep, FbsForwardError};
pub use panel::{
    panel_admission_gate, CpuPanelEvaluator, PanelDelta, PanelExportReading, PanelGateDecision,
    PanelReading,
};
pub use train::{
    CpuRankMarginTrainer, RankMarginConfig, RankMarginReading, RankMarginSample, RankMarginStep,
};

pub const VOCABULARY: usize = 4_096;
pub const TRUNK_BLOCKS: usize = 4;
pub const BLOCK_WIDTH: usize = 8;
pub const READ_WIDTH: usize = TRUNK_BLOCKS * BLOCK_WIDTH;
pub const CELL_COUNT: usize = 256;
pub const TOP_K: usize = 8;
pub const ABSENT_CELL: u16 = u16::MAX;

pub type Vector = [i32; READ_WIDTH];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuTrainError {
    Invalid(String),
}

impl std::fmt::Display for CpuTrainError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CpuTrainError {}

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

fn root_vector(root: u8) -> Result<[i32; BLOCK_WIDTH]> {
    orbit_root_coords(root).map_err(|error| invalid(format!("invalid orbit root {root}: {error}")))
}

pub fn saturating_abs_diff(left: i32, right: i32) -> u32 {
    left.saturating_sub(right).unsigned_abs()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterDecision {
    pub top8: [u32; TOP_K],
    pub admitted: [u16; CELL_COUNT / 16],
    pub admitted_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriterReading {
    pub top1: u32,
    pub top8: [u32; TOP_K],
    pub target_admitted: bool,
    pub target_top1: bool,
    pub target_top8: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WriterSummary {
    pub rows: u64,
    pub top1_hits: u64,
    pub top8_hits: u64,
    pub target_admitted: u64,
}

#[derive(Debug, Clone)]
pub struct CpuWriterCore {
    vectors: Vec<Vector>,
    token_cells: Vec<u16>,
    cell_tokens: [Vec<u16>; CELL_COUNT],
    roots: [[i32; BLOCK_WIDTH]; CELL_COUNT / 16],
    kept_cells: usize,
}

impl CpuWriterCore {
    pub fn from_codebook(table: &[i32], kept_cells: usize) -> Result<Self> {
        if kept_cells == 0 || kept_cells > CELL_COUNT / 16 {
            return Err(invalid("kept_cells must be in 1..=16"));
        }
        if table.len() != VOCABULARY * READ_WIDTH {
            return Err(invalid("codebook length must equal 4096 * 32"));
        }

        let roots = std::array::from_fn::<_, 16, _>(|root| {
            root_vector(root as u8).expect("the 16 canonical orbit roots are valid")
        });
        let mut vectors = Vec::with_capacity(VOCABULARY);
        let mut token_cells = Vec::with_capacity(VOCABULARY);
        let mut cell_tokens: [Vec<u16>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());

        for token in 0..VOCABULARY {
            let mut vector = [0i32; READ_WIDTH];
            vector.copy_from_slice(&table[token * READ_WIDTH..(token + 1) * READ_WIDTH]);

            let mut token_roots = [0u8; TRUNK_BLOCKS];
            for block in 0..TRUNK_BLOCKS {
                let coordinates: [i32; BLOCK_WIDTH] = vector
                    [block * BLOCK_WIDTH..(block + 1) * BLOCK_WIDTH]
                    .try_into()
                    .map_err(|_| invalid("codebook block width mismatch"))?;
                token_roots[block] = identify_orbit_root(&coordinates).ok_or_else(|| {
                    invalid(format!(
                        "token {token} block {block} is not on the 16-root orbit"
                    ))
                })?;
            }
            let cell = u16::from(token_roots[0]) * 16 + u16::from(token_roots[3]);
            token_cells.push(cell);
            cell_tokens[usize::from(cell)].push(token as u16);
            vectors.push(vector);
        }

        Ok(Self {
            vectors,
            token_cells,
            cell_tokens,
            roots,
            kept_cells,
        })
    }

    pub fn from_roots(roots: &[[u8; TRUNK_BLOCKS]], kept_cells: usize) -> Result<Self> {
        if roots.len() != VOCABULARY {
            return Err(invalid("roots length must equal 4096"));
        }
        let mut table = Vec::with_capacity(VOCABULARY * READ_WIDTH);
        for (token, token_roots) in roots.iter().enumerate() {
            for &root in token_roots {
                if usize::from(root) >= CELL_COUNT / 16 {
                    return Err(invalid(format!(
                        "token {token} orbit root {root} is outside 0..16"
                    )));
                }
                table.extend_from_slice(&root_vector(root)?);
            }
        }
        Self::from_codebook(&table, kept_cells)
    }

    pub fn kept_cells(&self) -> usize {
        self.kept_cells
    }

    pub fn token_vector(&self, token: usize) -> Option<&Vector> {
        self.vectors.get(token)
    }

    pub fn token_cell(&self, token: usize) -> Option<u16> {
        self.token_cells.get(token).copied()
    }

    pub fn decide(&self, query: &Vector) -> WriterDecision {
        let first = self.block_l1(query, 0, 0);
        let last = self.block_l1(query, TRUNK_BLOCKS - 1, 0);
        let mut first_costs = [0u32; CELL_COUNT / 16];
        let mut last_costs = [0u32; CELL_COUNT / 16];

        for root in 0..CELL_COUNT / 16 {
            first_costs[root] = self.block_l1(query, 0, root);
            last_costs[root] = self.block_l1(query, TRUNK_BLOCKS - 1, root);
        }
        debug_assert_eq!(first, first_costs[0]);
        debug_assert_eq!(last, last_costs[0]);

        let mut candidates: Vec<(u32, usize)> = (0..CELL_COUNT)
            .filter(|&cell| !self.cell_tokens[cell].is_empty())
            .map(|cell| {
                (
                    first_costs[cell / 16].saturating_add(last_costs[cell % 16]),
                    cell,
                )
            })
            .collect();
        candidates.sort_unstable();

        let mut admitted = [ABSENT_CELL; CELL_COUNT / 16];
        let mut admitted_count = 0usize;
        for &(_, cell) in candidates.iter().take(self.kept_cells) {
            admitted[admitted_count] = cell as u16;
            admitted_count += 1;
        }

        let mut ranked = Vec::new();
        for &cell in &admitted[..admitted_count] {
            for &token in &self.cell_tokens[usize::from(cell)] {
                ranked.push((
                    vector_l1(query, &self.vectors[usize::from(token)]),
                    u32::from(token),
                ));
            }
        }
        ranked.sort_unstable();
        ranked.truncate(TOP_K);
        while ranked.len() < TOP_K {
            ranked.push((u32::MAX, u32::MAX));
        }
        let top8 = std::array::from_fn(|index| ranked[index].1);

        WriterDecision {
            top8,
            admitted,
            admitted_count,
        }
    }

    pub fn read_target(&self, query: &Vector, target: u32) -> Result<WriterReading> {
        if target as usize >= VOCABULARY {
            return Err(invalid("target token must be in 0..4096"));
        }
        let decision = self.decide(query);
        let target_cell = self.token_cells[target as usize];
        let target_admitted = decision.admitted[..decision.admitted_count].contains(&target_cell);
        Ok(WriterReading {
            top1: decision.top8[0],
            top8: decision.top8,
            target_admitted,
            target_top1: decision.top8[0] == target,
            target_top8: decision.top8.contains(&target),
        })
    }

    pub fn summarize(&self, queries: &[Vector], targets: &[u32]) -> Result<WriterSummary> {
        if queries.len() != targets.len() {
            return Err(invalid("queries and targets must have equal length"));
        }
        let mut summary = WriterSummary::default();
        for (query, &target) in queries.iter().zip(targets) {
            let reading = self.read_target(query, target)?;
            summary.rows += 1;
            summary.top1_hits += u64::from(reading.target_top1);
            summary.top8_hits += u64::from(reading.target_top8);
            summary.target_admitted += u64::from(reading.target_admitted);
        }
        Ok(summary)
    }

    pub fn admitted_count(&self, decision: &WriterDecision) -> usize {
        decision.admitted[..decision.admitted_count]
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn block_l1(&self, query: &Vector, block: usize, root: usize) -> u32 {
        let start = block * BLOCK_WIDTH;
        query[start..start + BLOCK_WIDTH]
            .iter()
            .zip(self.roots[root].iter())
            .map(|(&left, &right)| saturating_abs_diff(left, right))
            .fold(0u32, u32::saturating_add)
    }
}

pub fn vector_l1(left: &Vector, right: &Vector) -> u32 {
    left.iter()
        .zip(right.iter())
        .map(|(&left, &right)| saturating_abs_diff(left, right))
        .fold(0u32, u32::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::trunk::{
        build_cell_tokens, writer_score_full_outer_l1, CELL_COUNT as ORACLE_CELLS,
    };

    fn roots() -> Vec<[u8; TRUNK_BLOCKS]> {
        (0..VOCABULARY)
            .map(|token| {
                [
                    (token & 15) as u8,
                    ((token >> 4) & 15) as u8,
                    ((token >> 8) & 15) as u8,
                    (((token >> 4) + (token >> 8)) & 15) as u8,
                ]
            })
            .collect()
    }

    fn oracle_inputs() -> (Vec<i32>, Vec<[i32; READ_WIDTH]>, [Vec<u16>; ORACLE_CELLS]) {
        let roots = roots();
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for token_roots in &roots {
            for &root in token_roots {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u16::from(token_roots[0]) * 16 + u16::from(token_roots[3]));
        }
        let vectors = table
            .chunks_exact(READ_WIDTH)
            .map(|chunk| chunk.try_into().unwrap())
            .collect();
        (table, vectors, build_cell_tokens(&cells).unwrap())
    }

    fn query_mix(row: usize) -> Vector {
        let roots = roots();
        let tokens = [
            row % VOCABULARY,
            (row * 17 + 3) % VOCABULARY,
            (row * 31 + 7) % VOCABULARY,
            (row * 47 + 11) % VOCABULARY,
        ];
        let mut query = [0i32; READ_WIDTH];
        for (block, token) in tokens.into_iter().enumerate() {
            let root = roots[token][block];
            query[block * BLOCK_WIDTH..(block + 1) * BLOCK_WIDTH]
                .copy_from_slice(&orbit_root_coords(root).unwrap());
        }
        query
    }

    #[test]
    fn exact_queries_return_their_token_and_admit_their_cell() {
        let core = CpuWriterCore::from_roots(&roots(), 8).unwrap();
        for token in [0, 1, 15, 16, 1027, 2777, 4095] {
            let query = *core.token_vector(token).unwrap();
            let reading = core.read_target(&query, token as u32).unwrap();
            assert_eq!(reading.top1, token as u32);
            assert!(reading.target_admitted);
            assert!(reading.target_top8);
            assert!(reading.top8.iter().all(|&candidate| candidate < 4_096));
            assert_eq!(
                reading.top8.iter().copied().collect::<BTreeSet<_>>().len(),
                TOP_K
            );
        }
    }

    #[test]
    fn matches_the_existing_host_writer_oracle() {
        let (table, vectors, cell_tokens) = oracle_inputs();
        let core = CpuWriterCore::from_codebook(&table, 8).unwrap();
        for row in 0..256 {
            let query = query_mix(row);
            let (oracle_top8, oracle_admitted) =
                writer_score_full_outer_l1(&query, &vectors, &cell_tokens, 8).unwrap();
            let decision = core.decide(&query);
            assert_eq!(decision.top8, oracle_top8, "row {row}");
            assert_eq!(decision.admitted, oracle_admitted, "row {row}");
        }
    }

    #[test]
    fn tied_distances_choose_the_lower_token_id() {
        let mut roots = roots();
        roots[9] = roots[1];
        let core = CpuWriterCore::from_roots(&roots, 8).unwrap();
        let query = *core.token_vector(1).unwrap();
        assert_eq!(core.decide(&query).top8[0], 1);
    }

    #[test]
    fn malformed_table_and_kept_counts_are_refused() {
        assert!(CpuWriterCore::from_codebook(&[0; 3], 8).is_err());
        assert!(CpuWriterCore::from_codebook(&[0; VOCABULARY * READ_WIDTH], 8).is_err());
        assert!(CpuWriterCore::from_roots(&roots(), 0).is_err());
        assert!(CpuWriterCore::from_roots(&roots(), 17).is_err());
        let mut invalid_root = roots();
        invalid_root[0][0] = 16;
        assert!(CpuWriterCore::from_roots(&invalid_root, 8).is_err());
    }

    #[test]
    fn admission_is_distinct_and_reports_all_requested_cells() {
        let core = CpuWriterCore::from_roots(&roots(), 8).unwrap();
        let decision = core.decide(&query_mix(11));
        assert_eq!(decision.admitted_count, 8);
        assert_eq!(core.admitted_count(&decision), 8);
        assert!(decision.admitted[..8]
            .iter()
            .all(|&cell| usize::from(cell) < CELL_COUNT));
    }

    #[test]
    fn admission_cost_ties_choose_lower_cell_and_skip_empty_cells() {
        let root0 = orbit_root_coords(0).unwrap();
        let root1 = orbit_root_coords(1).unwrap();
        let roots: Vec<[u8; TRUNK_BLOCKS]> = (0..VOCABULARY)
            .map(|token| {
                [
                    0,
                    ((token >> 4) & 15) as u8,
                    ((token >> 8) & 15) as u8,
                    u8::from(token >= VOCABULARY / 2),
                ]
            })
            .collect();
        let core = CpuWriterCore::from_roots(&roots, 8).unwrap();
        let mut query = [0i32; READ_WIDTH];
        query[..BLOCK_WIDTH].copy_from_slice(&root0);
        for coordinate in 0..BLOCK_WIDTH {
            query[(TRUNK_BLOCKS - 1) * BLOCK_WIDTH + coordinate] =
                (root0[coordinate] + root1[coordinate]) / 2;
        }
        let decision = core.decide(&query);
        assert_eq!(decision.admitted_count, 2);
        assert_eq!(&decision.admitted[..2], &[0, 1]);
        assert!(decision.admitted[2..]
            .iter()
            .all(|&cell| cell == ABSENT_CELL));
    }

    #[test]
    fn distance_sums_saturate_instead_of_overflowing() {
        assert_eq!(saturating_abs_diff(i32::MAX, i32::MIN), 2_147_483_647);
        assert_eq!(saturating_abs_diff(i32::MIN, i32::MAX), 2_147_483_648);
        assert_eq!(
            vector_l1(&[i32::MIN; READ_WIDTH], &[i32::MAX; READ_WIDTH]),
            u32::MAX
        );
    }

    #[test]
    fn target_reading_separates_admission_from_ranking() {
        let core = CpuWriterCore::from_roots(&roots(), 8).unwrap();
        let query = *core.token_vector(2777).unwrap();
        let correct = core.read_target(&query, 2777).unwrap();
        assert!(correct.target_admitted && correct.target_top1 && correct.target_top8);
        let other_query = *core.token_vector(17).unwrap();
        let other = core.read_target(&other_query, 2777).unwrap();
        assert_eq!(other.target_top8, other.top8.contains(&2777));
        assert_eq!(other.target_top1, other.top1 == 2777);
        assert!(core.read_target(&query, VOCABULARY as u32).is_err());
    }

    #[test]
    fn aggregate_readings_count_rank_and_admission_separately() {
        let core = CpuWriterCore::from_roots(&roots(), 8).unwrap();
        let queries = [
            *core.token_vector(11).unwrap(),
            *core.token_vector(2777).unwrap(),
        ];
        let exact = core.summarize(&queries, &[11, 2777]).unwrap();
        assert_eq!(
            exact,
            WriterSummary {
                rows: 2,
                top1_hits: 2,
                top8_hits: 2,
                target_admitted: 2,
            }
        );
        assert!(core.summarize(&queries, &[11]).is_err());
    }

    #[test]
    fn the_cpu_crate_is_not_a_default_workspace_member() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
        let contents = std::fs::read_to_string(manifest).unwrap();
        let defaults = contents
            .split_once("default-members = [")
            .and_then(|(_, tail)| tail.split_once(']'))
            .map(|(body, _)| body)
            .unwrap();
        assert!(!defaults.contains("uor-r4-cpu-train"));
        assert!(contents.contains("\"crates/uor-r4-cpu-train\""));
    }
}
