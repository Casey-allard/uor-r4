//! Independent CPU Route, Flock and Relation reads for four-table discrete
//! artifacts, plus a read-only CPU path over the 19-table M1 trunk.

use uor_r4_core::spiralcore_operator::{cl06_finite_composition_table, Cl06FiniteCompositionTable};
use uor_r4_discrete::normalization_mechanism::{
    identify_orbit_root, orbit_root_coords, NormFrameTable, NormalizationMode,
};
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::training::{family_cursors, BlockCursor, Cursor, SchedulePolicy};
use uor_r4_discrete::trunk::{
    build_cell_tokens, compute_recurrence_state, evaluate_joint_pointer_full_writer,
    export_trunk_d11, import_trunk_d11, pack_u32_key, RecurrenceMode, SparseCountTable,
    TrunkTables, TrunkTrainRow, BLOCK_WIDTH, CELL_COUNT, READ_WIDTH, TRUNK_BLOCKS,
};
use uor_r4_discrete::w2_capacity::decode_read_vector;

use crate::{CpuTrainError, VOCABULARY};

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

const BFS: [u8; 256] = [
    0, 16, 7, 42, 3, 8, 9, 27, 5, 24, 4, 47, 2, 12, 1, 34, 16, 0, 34, 1, 12, 2, 47, 4, 24, 5, 27,
    9, 8, 3, 42, 7, 34, 7, 0, 26, 11, 9, 14, 10, 6, 4, 5, 2, 47, 1, 3, 16, 1, 42, 3, 0, 15, 4, 5,
    8, 13, 9, 2, 10, 27, 7, 16, 48, 12, 3, 1, 7, 0, 5, 6, 9, 2, 41, 13, 4, 35, 16, 15, 11, 2, 8,
    47, 27, 24, 0, 7, 1, 3, 12, 42, 34, 16, 5, 4, 9, 47, 9, 2, 10, 4, 34, 0, 3, 11, 1, 26, 16, 7,
    6, 5, 14, 4, 27, 5, 2, 13, 42, 48, 0, 7, 15, 16, 3, 1, 9, 8, 10, 35, 5, 4, 9, 41, 12, 1, 15, 0,
    16, 7, 11, 3, 2, 13, 6, 5, 35, 6, 13, 2, 3, 11, 7, 16, 0, 15, 1, 12, 41, 9, 4, 27, 4, 10, 8, 9,
    1, 3, 16, 15, 7, 0, 48, 42, 13, 2, 5, 9, 47, 14, 5, 6, 7, 16, 26, 1, 11, 3, 0, 34, 4, 10, 2, 8,
    2, 9, 4, 5, 16, 34, 42, 12, 3, 1, 7, 0, 24, 27, 47, 3, 12, 11, 15, 16, 35, 4, 13, 41, 2, 9, 6,
    5, 0, 7, 1, 42, 1, 48, 16, 7, 27, 10, 2, 9, 13, 8, 5, 4, 15, 0, 3, 7, 34, 16, 3, 1, 47, 2, 5,
    4, 6, 10, 14, 9, 11, 26, 0,
];

const FLOCK_SNAPSHOT_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";
const FLOCK_OPERATOR_KAPPA: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
const FLOCK_COMPOSITION_KAPPA: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";
const RELATION_EDGE_TYPES: usize = 5;
const DISCRETE_SCHEMA: &str = "uor-r4.lut-discrete/1";
const DISCRETE_TRUNK_SCHEMA: &str = "uor-r4.lut-discrete-trunk/1";

const BRANCH: [[u8; 3]; 16] = [
    [0, 14, 3],
    [1, 3, 14],
    [2, 13, 4],
    [3, 0, 1],
    [4, 2, 15],
    [5, 7, 10],
    [6, 9, 8],
    [7, 12, 5],
    [8, 6, 11],
    [9, 11, 6],
    [10, 5, 12],
    [11, 8, 9],
    [12, 10, 7],
    [13, 15, 2],
    [14, 1, 0],
    [15, 4, 13],
];

#[derive(Clone)]
pub struct CpuDiscreteRoute {
    context: usize,
    blocks: usize,
    stages: usize,
    token_blocks: Vec<Vec<[i32; 8]>>,
    routes: Vec<Vec<u32>>,
    composition: Cl06FiniteCompositionTable,
}

#[derive(Clone)]
struct BinderNode {
    children: Option<[usize; 3]>,
    atom: Option<usize>,
    level: usize,
    first: usize,
    last: usize,
}

#[derive(Clone)]
struct RowPlan {
    nodes: Vec<BinderNode>,
    frontier: Vec<usize>,
}

impl CpuDiscreteRoute {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_trunk(bytes, false)
    }

    fn from_trunk_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_trunk(bytes, true)
    }

    fn from_bytes_with_trunk(bytes: &[u8], trunk: bool) -> Result<Self> {
        let header = header(bytes)?;
        let expected_schema = if trunk {
            DISCRETE_TRUNK_SCHEMA
        } else {
            DISCRETE_SCHEMA
        };
        if header["schema"].as_str() != Some(expected_schema) {
            return Err(invalid(format!(
                "Route CPU read requires {expected_schema}"
            )));
        }
        let shape = &header["shape"];
        if shape["read_mode"].as_str() != Some("route")
            || shape["read_initialization"].as_str() != Some("zero_gates_v1")
        {
            return Err(invalid("Route CPU read refuses non-Route initialization"));
        }
        let vocab = usize::try_from(shape["vocabulary"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("vocabulary"))?;
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("blocks"))?;
        let context = usize::try_from(shape["context"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("context"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("stages"))?;
        let kept_cells = usize::try_from(shape["kept_cells"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("kept_cells"))?;
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
            || !(1..=16).contains(&kept_cells)
            || kept_cells > vocab
            || shape["read_kept"].as_u64() != Some(0)
            || shape["read_sweeps"].as_u64() != Some(0)
            || expected_stages(context) != Some(stages)
        {
            return Err(invalid("Route shape differs"));
        }
        let tables = header["tables"]
            .as_array()
            .ok_or_else(|| invalid("Route artifact tables differ"))?;
        let expected_tables = if trunk { 19 } else { 4 };
        if tables.len() != expected_tables {
            return Err(invalid(format!(
                "Route artifact requires {expected_tables} tables"
            )));
        }
        let table = section_i32(bytes, &header, "discrete.table", vocab * blocks * 8, "i32")?;
        let cells = section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
        let routes = section_u32(bytes, &header, "discrete.routes", blocks * stages, "u32")?;
        let gates = section_i32(bytes, &header, "discrete.gates", blocks * stages, "i32")?;
        if gates.iter().any(|gate| *gate != 0) {
            return Err(invalid("Route initialization requires zero gates"));
        }
        if routes.iter().any(|&action| action >= 64) {
            return Err(invalid("Route action outside 0..64"));
        }
        let mut token_blocks = Vec::with_capacity(vocab);
        for token in 0..vocab {
            let mut values = Vec::with_capacity(blocks);
            for block in 0..blocks {
                let start = (token * blocks + block) * 8;
                let coords: [i32; 8] = table[start..start + 8]
                    .try_into()
                    .map_err(|_| invalid("Route codebook width"))?;
                if root_id(&coords).is_none() {
                    return Err(invalid("Route codebook block outside orbit"));
                }
                values.push(coords);
            }
            token_blocks.push(values);
        }
        for (token, (blocks, &cell)) in token_blocks.iter().zip(&cells).enumerate() {
            let first = root_id(&blocks[0]).unwrap() as u32;
            let last = root_id(blocks.last().unwrap()).unwrap() as u32;
            if cell != first * 16 + last {
                return Err(invalid(format!("Route cell for token {token} differs")));
            }
        }
        validate_layout(bytes, &header)?;
        let routes = routes
            .chunks_exact(stages)
            .map(|row| row.to_vec())
            .collect();
        Self::from_parts(context, token_blocks, routes)
    }

    pub fn from_parts(
        context: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<Vec<u32>>,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = routes.first().map_or(0, Vec::len);
        if !(2..=8192).contains(&token_blocks.len())
            || !(2..=16).contains(&blocks)
            || expected_stages(context) != Some(stages)
            || routes.len() != blocks
            || routes
                .iter()
                .any(|row| row.len() != stages || row.iter().any(|&value| value >= 64))
            || token_blocks.iter().any(|token| {
                token.len() != blocks || token.iter().any(|block| root_id(block).is_none())
            })
        {
            return Err(invalid("Route parts differ"));
        }
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        Ok(Self {
            context,
            blocks,
            stages,
            token_blocks,
            routes,
            composition,
        })
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        if prefix.is_empty() || prefix.len() > self.context {
            return Err(invalid("Route prefix outside context"));
        }
        if prefix
            .iter()
            .any(|&token| token as usize >= self.token_blocks.len())
        {
            return Err(invalid("Route prefix token outside vocabulary"));
        }
        let plan = build_row_plan(prefix.len())?;
        let last = prefix[prefix.len() - 1] as usize;
        let mut out = Vec::with_capacity(self.blocks * 8);
        for block in 0..self.blocks {
            let query = &self.token_blocks[last][block];
            let mut node = 0usize;
            let mut root = 0usize;
            let mut total = 0usize;
            let mut resolved = None;
            for stage in 0..self.stages {
                let q = self.act(self.routes[block][stage] as usize, query)?;
                let count = if stage == 0 { plan.frontier.len() } else { 3 };
                let mut best = 0usize;
                let mut best_distance = u32::MAX;
                let mut best_hamming = u32::MAX;
                let mut goal = 0usize;
                #[allow(clippy::needless_range_loop)]
                for port in 0..count {
                    let r = if stage == 0 {
                        port
                    } else {
                        usize::from(BRANCH[root][port])
                    };
                    let (distance, hamming) = root_cost(&q, r);
                    if distance < best_distance
                        || (distance == best_distance && hamming < best_hamming)
                    {
                        best = port;
                        best_distance = distance;
                        best_hamming = hamming;
                        goal = r;
                    }
                }
                total = usize::from(
                    self.composition
                        .compose_index(BFS[(root << 4) + goal], total as u8)
                        .ok_or_else(|| invalid("Route composition failed"))?,
                );
                root = goal;
                let next = if stage == 0 {
                    plan.frontier[best]
                } else {
                    plan.nodes[node]
                        .children
                        .ok_or_else(|| invalid("Route stepped past atom"))?[best]
                };
                if let Some(atom) = plan.nodes[next].atom {
                    let carrier = &self.token_blocks[prefix[atom] as usize][block];
                    resolved = Some(self.act(total, carrier)?);
                    break;
                }
                node = next;
            }
            out.extend_from_slice(
                &resolved.ok_or_else(|| invalid("Route did not reach an atom leaf"))?,
            );
        }
        Ok(out)
    }

    fn act(&self, action: usize, input: &[i32; 8]) -> Result<[i32; 8]> {
        let root: [i8; 8] = std::array::from_fn(|index| (input[index] / 2) as i8);
        let out = self
            .composition
            .state(action as u8)
            .ok_or_else(|| invalid("Route action state absent"))?
            .apply_root_key(root)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(out.map(|value| i32::from(value) * 2))
    }
}

#[derive(Clone)]
enum ActiveLeafMode {
    Flock,
    Relation {
        sweeps: usize,
        relation_gates: Vec<[i32; RELATION_EDGE_TYPES]>,
    },
}

#[derive(Clone, Copy)]
enum ActiveLeafKind {
    Flock,
    Relation,
}

impl ActiveLeafKind {
    fn label(self) -> &'static str {
        match self {
            Self::Flock => "Flock",
            Self::Relation => "Relation",
        }
    }

    fn mode_name(self) -> &'static str {
        match self {
            Self::Flock => "flock",
            Self::Relation => "relation",
        }
    }
}

#[derive(Clone)]
struct ActiveLeafModel {
    context: usize,
    blocks: usize,
    stages: usize,
    kept: usize,
    token_blocks: Vec<Vec<[i32; 8]>>,
    routes: Vec<u32>,
    level_gates: Vec<Vec<i32>>,
    mode: ActiveLeafMode,
    composition: Cl06FiniteCompositionTable,
}

#[derive(Clone)]
pub struct CpuDiscreteFlock(ActiveLeafModel);

#[derive(Clone)]
pub struct CpuDiscreteRelation(ActiveLeafModel);

impl ActiveLeafModel {
    fn from_bytes(bytes: &[u8], kind: ActiveLeafKind) -> Result<Self> {
        Self::from_bytes_with_trunk(bytes, kind, false)
    }

    fn from_trunk_bytes(bytes: &[u8], kind: ActiveLeafKind) -> Result<Self> {
        Self::from_bytes_with_trunk(bytes, kind, true)
    }

    fn from_bytes_with_trunk(bytes: &[u8], kind: ActiveLeafKind, trunk: bool) -> Result<Self> {
        let label = kind.label();
        let header = header(bytes)?;
        let expected_schema = if trunk {
            DISCRETE_TRUNK_SCHEMA
        } else {
            DISCRETE_SCHEMA
        };
        if header["schema"].as_str() != Some(expected_schema) {
            return Err(invalid(format!(
                "{label} CPU read requires {expected_schema}"
            )));
        }
        let shape = &header["shape"];
        if shape["read_mode"].as_str() != Some(kind.mode_name())
            || shape["read_initialization"].as_str() != Some("active_leaf_v1")
        {
            return Err(invalid(format!(
                "{label} CPU read refuses non-{label} initialization"
            )));
        }
        let vocab = usize::try_from(shape["vocabulary"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("vocabulary"))?;
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("blocks"))?;
        let context = usize::try_from(shape["context"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("context"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("stages"))?;
        let kept_cells = usize::try_from(shape["kept_cells"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("kept_cells"))?;
        let kept = usize::try_from(shape["read_kept"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("read_kept"))?;
        let sweeps = usize::try_from(shape["read_sweeps"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("read_sweeps"))?;
        let sweeps_valid = match kind {
            ActiveLeafKind::Flock => sweeps == 0,
            ActiveLeafKind::Relation => (1..=5).contains(&sweeps),
        };
        if !(2..=8192).contains(&vocab)
            || !(2..=16).contains(&blocks)
            || !(1..=243).contains(&context)
            || !(1..=16).contains(&kept_cells)
            || kept_cells > vocab
            || !(1..=8).contains(&kept)
            || !sweeps_valid
            || expected_stages(context) != Some(stages)
            || !matches!(
                shape["objective_policy"].as_str(),
                Some("nearest_address_margin_v1" | "occupied_cell_rank_hinge_v1")
            )
        {
            return Err(invalid(format!("{label} shape differs")));
        }
        if header["matrices"]
            .as_array()
            .is_none_or(|matrices| !matrices.is_empty())
        {
            return Err(invalid(format!(
                "{label} artifact requires an empty matrices array"
            )));
        }
        let source = &header["source"];
        if source["snapshot_schema"].as_str() != Some(FLOCK_SNAPSHOT_SCHEMA)
            || source["operator_kappa"].as_str() != Some(FLOCK_OPERATOR_KAPPA)
            || source["composition_kappa"].as_str() != Some(FLOCK_COMPOSITION_KAPPA)
        {
            return Err(invalid(format!("{label} artifact source geometry differs")));
        }
        validate_header_padding(bytes)?;
        let tables = header["tables"]
            .as_array()
            .ok_or_else(|| invalid(format!("{label} artifact tables differ")))?;
        let expected_tables = if trunk { 19 } else { 4 };
        if tables.len() != expected_tables {
            return Err(invalid(format!(
                "{label} artifact requires {expected_tables} tables"
            )));
        }
        let table = section_i32(bytes, &header, "discrete.table", vocab * blocks * 8, "i32")?;
        let cells = section_u32(bytes, &header, "discrete.cells", vocab, "u32")?;
        let routes = section_u32(bytes, &header, "discrete.routes", blocks * stages, "u32")?;
        let base_gate_count = blocks * stages;
        let gate_count = base_gate_count
            + match kind {
                ActiveLeafKind::Flock => 0,
                ActiveLeafKind::Relation => RELATION_EDGE_TYPES * stages,
            };
        let gates = section_i32(bytes, &header, "discrete.gates", gate_count, "i32")?;
        if routes.iter().any(|&action| action >= 64) {
            return Err(invalid(format!("{label} route action outside 0..64")));
        }
        if gates.iter().any(|&gate| !(-1..=1).contains(&gate)) {
            return Err(invalid(format!("{label} gate trit outside -1..=1")));
        }
        let token_blocks = decode_token_blocks(&table, &cells, vocab, blocks, label)?;
        validate_layout(bytes, &header)?;
        let level_gates = gates[..base_gate_count]
            .chunks_exact(stages)
            .map(<[i32]>::to_vec)
            .collect();
        let mode = match kind {
            ActiveLeafKind::Flock => ActiveLeafMode::Flock,
            ActiveLeafKind::Relation => {
                let relation_gates = (0..stages)
                    .map(|stage| {
                        let start = base_gate_count + stage * RELATION_EDGE_TYPES;
                        gates[start..start + RELATION_EDGE_TYPES]
                            .try_into()
                            .expect("the relation gate slice has five entries")
                    })
                    .collect();
                ActiveLeafMode::Relation {
                    sweeps,
                    relation_gates,
                }
            }
        };
        Self::from_parts_with_routes(
            label,
            context,
            kept,
            token_blocks,
            routes,
            level_gates,
            mode,
        )
    }

    fn from_parts(
        label: &str,
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
        mode: ActiveLeafMode,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = level_gates.first().map_or(0, Vec::len);
        Self::from_parts_with_routes(
            label,
            context,
            kept,
            token_blocks,
            vec![0; blocks * stages],
            level_gates,
            mode,
        )
    }

    fn from_parts_with_routes(
        label: &str,
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<u32>,
        level_gates: Vec<Vec<i32>>,
        mode: ActiveLeafMode,
    ) -> Result<Self> {
        let blocks = token_blocks.first().map_or(0, Vec::len);
        let stages = level_gates.first().map_or(0, Vec::len);
        let mode_valid = match &mode {
            ActiveLeafMode::Flock => true,
            ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            } => {
                (1..=5).contains(sweeps)
                    && relation_gates.len() == stages
                    && relation_gates
                        .iter()
                        .all(|row| row.iter().all(|gate| (-1..=1).contains(gate)))
            }
        };
        if !(2..=8192).contains(&token_blocks.len())
            || !(2..=16).contains(&blocks)
            || !(1..=8).contains(&kept)
            || expected_stages(context) != Some(stages)
            || level_gates.len() != blocks
            || level_gates
                .iter()
                .any(|row| row.len() != stages || row.iter().any(|gate| !(-1..=1).contains(gate)))
            || routes.len() != blocks * stages
            || routes.iter().any(|&action| action >= 64)
            || token_blocks.iter().any(|token| {
                token.len() != blocks || token.iter().any(|block| root_id(block).is_none())
            })
            || !mode_valid
        {
            return Err(invalid(format!("{label} parts differ")));
        }
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        Ok(Self {
            context,
            blocks,
            stages,
            kept,
            token_blocks,
            routes,
            level_gates,
            mode,
            composition,
        })
    }

    fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        let label = match self.mode {
            ActiveLeafMode::Flock => "Flock",
            ActiveLeafMode::Relation { .. } => "Relation",
        };
        if prefix.is_empty() || prefix.len() > self.context {
            return Err(invalid(format!("{label} prefix outside context")));
        }
        if prefix
            .iter()
            .any(|&token| token as usize >= self.token_blocks.len())
        {
            return Err(invalid(format!("{label} prefix token outside vocabulary")));
        }
        let plan = build_row_plan(prefix.len())?;
        let length = prefix.len();
        let last_token = prefix[length - 1] as usize;
        let mut out = Vec::with_capacity(self.blocks * 8);
        for block in 0..self.blocks {
            let frame_block = if block == 0 { self.blocks - 1 } else { 0 };
            let query_frame = &self.token_blocks[last_token][frame_block];
            let query_frame_root = root_id(query_frame)
                .ok_or_else(|| invalid("active-leaf query frame outside orbit"))?;
            let mut value = self.token_blocks[last_token][block];
            let mut terms = 1usize;

            let mut frames = vec![0usize; plan.nodes.len()];
            let mut payload = vec![[0i32; 8]; plan.nodes.len()];
            for (node_index, node) in plan.nodes.iter().enumerate() {
                if let Some(atom) = node.atom {
                    let token = prefix[atom] as usize;
                    frames[node_index] = root_id(&self.token_blocks[token][frame_block])
                        .ok_or_else(|| invalid("active-leaf atom frame outside orbit"))?;
                    payload[node_index] = self.token_blocks[token][block];
                } else {
                    let children = node
                        .children
                        .ok_or_else(|| invalid("active-leaf non-atom missing children"))?;
                    frames[node_index] = frames[children[0]];
                    let mut sum = [0i32; 8];
                    for &child in &children {
                        let action = usize::from(BFS[(frames[child] << 4) + frames[children[0]]]);
                        let carried = self.act(action, &payload[child])?;
                        for dimension in 0..8 {
                            sum[dimension] += carried[dimension];
                        }
                    }
                    payload[node_index] = orbit_root_coords(snap_root(&sum, 3)? as u8)
                        .map_err(|error| invalid(error.to_string()))?;
                }
            }

            let mut relation_scores = vec![0i32; plan.nodes.len()];
            if let ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            } = &self.mode
            {
                let mut previous = (0..plan.nodes.len())
                    .map(|node| {
                        8i32 - i32::try_from(root_cost(query_frame, frames[node]).0)
                            .expect("root distance fits i32")
                    })
                    .collect::<Vec<_>>();
                for _ in 0..*sweeps {
                    for node in 0..plan.nodes.len() {
                        if plan.nodes[node].last >= length {
                            continue;
                        }
                        let level = plan.nodes[node].level;
                        for (edge, &gate) in relation_gates[level].iter().enumerate() {
                            if gate == 0 {
                                continue;
                            }
                            let mut sum = 0i32;
                            match edge {
                                0 => {
                                    if let Some(children) = plan.nodes[node].children {
                                        for child in children {
                                            if plan.nodes[child].last < length {
                                                sum += previous[child];
                                            }
                                        }
                                    }
                                    for (parent, &previous_parent) in previous
                                        .iter()
                                        .enumerate()
                                        .take(plan.nodes.len())
                                        .skip(node + 1)
                                    {
                                        if plan.nodes[parent].last < length
                                            && plan.nodes[parent]
                                                .children
                                                .is_some_and(|children| children.contains(&node))
                                        {
                                            sum += previous_parent;
                                        }
                                    }
                                }
                                1 => {
                                    for parent in (node + 1)..plan.nodes.len() {
                                        let Some(children) = plan.nodes[parent].children else {
                                            continue;
                                        };
                                        if plan.nodes[parent].last >= length {
                                            continue;
                                        }
                                        if node == children[0]
                                            && plan.nodes[children[1]].last < length
                                        {
                                            sum += previous[children[1]];
                                        } else if node == children[1] {
                                            if plan.nodes[children[0]].last < length {
                                                sum += previous[children[0]];
                                            }
                                            if plan.nodes[children[2]].last < length {
                                                sum += previous[children[2]];
                                            }
                                        } else if node == children[2]
                                            && plan.nodes[children[1]].last < length
                                        {
                                            sum += previous[children[1]];
                                        }
                                    }
                                }
                                2 => {
                                    for (sibling, &previous_sibling) in
                                        previous.iter().enumerate().take(node)
                                    {
                                        if plan.nodes[sibling].last < length
                                            && plan.nodes[sibling].level == level
                                            && plan.nodes[sibling].last + 1
                                                == plan.nodes[node].first
                                        {
                                            sum += previous_sibling;
                                        }
                                    }
                                }
                                3 => {
                                    if level == 0 {
                                        let token = prefix[plan.nodes[node].last];
                                        for peer in 0..node {
                                            if plan.nodes[peer].last < length
                                                && plan.nodes[peer].level == 0
                                                && prefix[plan.nodes[peer].last] == token
                                            {
                                                sum = previous[peer];
                                            }
                                        }
                                    }
                                }
                                4 => {
                                    for peer in 0..node {
                                        if plan.nodes[peer].last < length
                                            && plan.nodes[peer].level == level
                                            && frames[peer] == frames[node]
                                        {
                                            sum = previous[peer];
                                        }
                                    }
                                }
                                _ => unreachable!("the edge loop has five entries"),
                            }
                            relation_scores[node] += if gate > 0 { sum } else { -sum };
                        }
                    }
                    previous.copy_from_slice(&relation_scores);
                }
            }

            for level in 0..self.stages {
                let gate = self.level_gates[block][level];
                if gate == 0 {
                    continue;
                }
                let mut top = [0usize; 8];
                let mut key_relation = [0i32; 8];
                let mut key_distance = [u32::MAX; 8];
                let mut key_hamming = [u32::MAX; 8];
                let mut found = 0usize;
                for (node, candidate) in plan.nodes.iter().enumerate() {
                    if candidate.level != level || candidate.last + 1 >= length {
                        continue;
                    }
                    let (distance, hamming) = root_cost(query_frame, frames[node]);
                    let relation = relation_scores[node];
                    let mut at = found;
                    while at > 0 {
                        let previous = top[at - 1];
                        let better = relation > key_relation[at - 1]
                            || (relation == key_relation[at - 1]
                                && (distance < key_distance[at - 1]
                                    || (distance == key_distance[at - 1]
                                        && (hamming < key_hamming[at - 1]
                                            || (hamming == key_hamming[at - 1]
                                                && (plan.nodes[node].last
                                                    > plan.nodes[previous].last
                                                    || (plan.nodes[node].last
                                                        == plan.nodes[previous].last
                                                        && node < previous)))))));
                        if !better {
                            break;
                        }
                        if at < self.kept {
                            top[at] = previous;
                            key_relation[at] = key_relation[at - 1];
                            key_distance[at] = key_distance[at - 1];
                            key_hamming[at] = key_hamming[at - 1];
                        }
                        at -= 1;
                    }
                    if at < self.kept {
                        top[at] = node;
                        key_relation[at] = relation;
                        key_distance[at] = distance;
                        key_hamming[at] = hamming;
                        found += usize::from(found < self.kept);
                    }
                }
                if found == 0 {
                    continue;
                }
                let mut sum = [0i32; 8];
                for &node in &top[..found] {
                    let action = usize::from(BFS[(frames[node] << 4) + query_frame_root]);
                    let carried = self.act(action, &payload[node])?;
                    for dimension in 0..8 {
                        sum[dimension] += carried[dimension];
                    }
                }
                let consensus = snap_root(&sum, found)?;
                let root = orbit_root_coords(consensus as u8)
                    .map_err(|error| invalid(error.to_string()))?;
                for dimension in 0..8 {
                    value[dimension] += if gate > 0 {
                        root[dimension]
                    } else {
                        -root[dimension]
                    };
                }
                terms += 1;
            }
            let output_root = snap_root(&value, terms)?;
            out.extend_from_slice(
                &orbit_root_coords(output_root as u8)
                    .map_err(|error| invalid(error.to_string()))?,
            );
        }
        Ok(out)
    }

    fn act(&self, action: usize, input: &[i32; 8]) -> Result<[i32; 8]> {
        let root: [i8; 8] = std::array::from_fn(|index| (input[index] / 2) as i8);
        let out = self
            .composition
            .state(action as u8)
            .ok_or_else(|| invalid("active-leaf action state absent"))?
            .apply_root_key(root)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(out.map(|value| i32::from(value) * 2))
    }
}

impl CpuDiscreteFlock {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Flock).map(Self)
    }

    pub fn from_parts(
        context: usize,
        kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
    ) -> Result<Self> {
        ActiveLeafModel::from_parts(
            "Flock",
            context,
            kept,
            token_blocks,
            level_gates,
            ActiveLeafMode::Flock,
        )
        .map(Self)
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        self.0.read(prefix)
    }
}

impl CpuDiscreteRelation {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Relation).map(Self)
    }

    pub fn from_parts(
        context: usize,
        kept: usize,
        sweeps: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        level_gates: Vec<Vec<i32>>,
        relation_gates: Vec<[i32; RELATION_EDGE_TYPES]>,
    ) -> Result<Self> {
        ActiveLeafModel::from_parts(
            "Relation",
            context,
            kept,
            token_blocks,
            level_gates,
            ActiveLeafMode::Relation {
                sweeps,
                relation_gates,
            },
        )
        .map(Self)
    }

    pub fn read(&self, prefix: &[u32]) -> Result<Vec<i32>> {
        self.0.read(prefix)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CpuDiscreteProposalMode {
    Route,
    Flock,
    Relation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDiscreteProposalKind {
    Route,
    Table,
    Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuDiscreteProposal {
    pub lane: usize,
    pub current_loss: u64,
    pub selected_loss: u64,
    pub epoch: u64,
    pub sensitive: bool,
}

#[derive(Clone)]
pub struct CpuDiscreteProposalChooser {
    mode: CpuDiscreteProposalMode,
    blocks: usize,
    stages: usize,
    routes: Vec<u32>,
    token_blocks: Vec<Vec<[i32; 8]>>,
    gate_count: usize,
    move_actions: [u8; 16],
    composition: Cl06FiniteCompositionTable,
}

impl CpuDiscreteProposalChooser {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        let shape = &header["shape"];
        let blocks = usize::try_from(shape["blocks"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("proposal chooser blocks"))?;
        let stages = usize::try_from(shape["stages"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("proposal chooser stages"))?;
        let gate_count = blocks
            .checked_mul(stages)
            .ok_or_else(|| invalid("proposal chooser gate count overflows"))?;
        let (mode, routes, token_blocks, gate_count) = match shape["read_mode"].as_str() {
            Some("route") => {
                let route = CpuDiscreteRoute::from_bytes(bytes)?;
                (
                    CpuDiscreteProposalMode::Route,
                    route.routes.into_iter().flatten().collect(),
                    route.token_blocks,
                    gate_count,
                )
            }
            Some("flock") => {
                let flock = ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Flock)?;
                (
                    CpuDiscreteProposalMode::Flock,
                    flock.routes,
                    flock.token_blocks,
                    gate_count,
                )
            }
            Some("relation") => {
                let relation = ActiveLeafModel::from_bytes(bytes, ActiveLeafKind::Relation)?;
                (
                    CpuDiscreteProposalMode::Relation,
                    relation.routes,
                    relation.token_blocks,
                    gate_count + RELATION_EDGE_TYPES * stages,
                )
            }
            _ => return Err(invalid("proposal chooser read mode differs")),
        };
        let composition =
            cl06_finite_composition_table().map_err(|error| invalid(error.to_string()))?;
        let mut move_actions = [0u8; 16];
        for (candidate, action) in move_actions.iter_mut().enumerate() {
            *action = NormFrameTable::new(16, vec![candidate as u8; 16])
                .map_err(|error| invalid(error.to_string()))?
                .block_frame(0)
                .map_err(|error| invalid(error.to_string()))?;
        }
        Ok(Self {
            mode,
            blocks,
            stages,
            routes,
            token_blocks,
            gate_count,
            move_actions,
            composition,
        })
    }

    pub fn choose(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        losses: &[u64; 16],
        epoch: u64,
    ) -> Result<CpuDiscreteProposal> {
        match kind {
            CpuDiscreteProposalKind::Gate if self.mode == CpuDiscreteProposalMode::Route => {
                return Err(invalid("Gate proposal requires Flock or Relation mode"));
            }
            CpuDiscreteProposalKind::Route if self.mode != CpuDiscreteProposalMode::Route => {
                return Err(invalid("Route proposal requires Route mode"));
            }
            _ => {}
        }
        match kind {
            CpuDiscreteProposalKind::Gate if index >= self.gate_count => {
                return Err(invalid("Gate proposal index outside gate inventory"));
            }
            CpuDiscreteProposalKind::Route if index >= self.blocks * self.stages => {
                return Err(invalid("Route proposal index outside map inventory"));
            }
            CpuDiscreteProposalKind::Table if index >= self.token_blocks.len() * self.blocks => {
                return Err(invalid("Table proposal index outside codebook inventory"));
            }
            _ => {}
        }

        let active = if kind == CpuDiscreteProposalKind::Gate {
            3
        } else {
            16
        };
        let sensitive = losses[1..active].iter().any(|&loss| loss != losses[0]);
        let mut best = 0usize;
        if kind == CpuDiscreteProposalKind::Gate {
            for lane in 1..3 {
                if losses[lane] < losses[best] {
                    best = lane;
                }
            }
        } else {
            for lane in 1..16 {
                let tie = best != 0
                    && losses[lane] == losses[best]
                    && self.resulting_port(kind, index, lane)?
                        < self.resulting_port(kind, index, best)?;
                if losses[lane] < losses[best] || tie {
                    best = lane;
                }
            }
        }
        Ok(CpuDiscreteProposal {
            lane: best,
            current_loss: losses[0],
            selected_loss: losses[best],
            epoch,
            sensitive,
        })
    }

    fn resulting_port(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        candidate: usize,
    ) -> Result<u32> {
        let action = self.move_actions[candidate];
        match kind {
            CpuDiscreteProposalKind::Route => {
                let route = *self
                    .routes
                    .get(index)
                    .ok_or_else(|| invalid("Route proposal index absent"))?;
                self.composition
                    .compose_index(action, route as u8)
                    .map(u32::from)
                    .ok_or_else(|| invalid("Route proposal composition failed"))
            }
            CpuDiscreteProposalKind::Table => {
                let token = index / self.blocks;
                let block = index % self.blocks;
                let point = self
                    .token_blocks
                    .get(token)
                    .and_then(|token| token.get(block))
                    .ok_or_else(|| invalid("Table proposal index absent"))?;
                let root: [i8; 8] = std::array::from_fn(|dimension| (point[dimension] / 2) as i8);
                let moved = self
                    .composition
                    .state(action)
                    .ok_or_else(|| invalid("Table proposal action state absent"))?
                    .apply_root_key(root)
                    .map_err(|error| invalid(error.to_string()))?;
                let moved = moved.map(|value| i32::from(value) * 2);
                root_id(&moved)
                    .map(|root| root as u32)
                    .ok_or_else(|| invalid("Table proposal moved outside orbit"))
            }
            CpuDiscreteProposalKind::Gate => {
                Err(invalid("Gate proposals do not have resulting ports"))
            }
        }
    }

    fn bvgk_in_range(&self, kind: CpuDiscreteProposalKind, index: usize) -> bool {
        match kind {
            CpuDiscreteProposalKind::Route => index < self.blocks * self.stages,
            CpuDiscreteProposalKind::Table => index < self.token_blocks.len() * self.blocks,
            CpuDiscreteProposalKind::Gate => index < self.gate_count,
        }
    }

    fn from_model(model: &CpuDiscreteProposalModel) -> Result<Self> {
        let (mode, blocks, stages, routes, token_blocks, gate_count) = match model {
            CpuDiscreteProposalModel::Route(route) => (
                CpuDiscreteProposalMode::Route,
                route.blocks,
                route.stages,
                route.routes.iter().flatten().copied().collect(),
                route.token_blocks.clone(),
                route.blocks * route.stages,
            ),
            CpuDiscreteProposalModel::Active(active) => {
                let (mode, relation_gates) = match &active.mode {
                    ActiveLeafMode::Flock => (CpuDiscreteProposalMode::Flock, 0),
                    ActiveLeafMode::Relation { .. } => (
                        CpuDiscreteProposalMode::Relation,
                        RELATION_EDGE_TYPES * active.stages,
                    ),
                };
                (
                    mode,
                    active.blocks,
                    active.stages,
                    active.routes.clone(),
                    active.token_blocks.clone(),
                    active.blocks * active.stages + relation_gates,
                )
            }
        };
        let mut move_actions = [0u8; 16];
        for (candidate, action) in move_actions.iter_mut().enumerate() {
            *action = NormFrameTable::new(16, vec![candidate as u8; 16])
                .map_err(|error| invalid(error.to_string()))?
                .block_frame(0)
                .map_err(|error| invalid(error.to_string()))?;
        }
        Ok(Self {
            mode,
            blocks,
            stages,
            routes,
            token_blocks,
            gate_count,
            move_actions,
            composition: cl06_finite_composition_table()
                .map_err(|error| invalid(error.to_string()))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CpuDiscreteProposalSample {
    pub prefix: Vec<u32>,
    pub target: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteProposalScoring {
    pub losses: [u64; 16],
    pub eligible_rows: usize,
    pub proposal: CpuDiscreteProposal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteBlockScoring {
    pub records: Vec<CpuBvgkRecord>,
    pub joint_loss: Option<u64>,
    pub eligible_rows: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuDiscretePanelReading {
    pub rows: u64,
    pub target_admitted: u64,
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub margin_loss_sum: u64,
    pub target_rank_sum: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuDiscretePanelExportReading {
    pub rows: u64,
    pub query_coordinates_checked: u64,
    pub query_coordinate_mismatches: u64,
    pub admitted_cell_mismatches: u64,
    pub top8_mismatches: u64,
    pub token_mismatches: u64,
}

impl CpuDiscretePanelExportReading {
    pub fn is_exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.admitted_cell_mismatches == 0
            && self.top8_mismatches == 0
            && self.token_mismatches == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteBlock {
    pub kind: CpuDiscreteProposalKind,
    pub indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteScoreError {
    pub status: u32,
    pub message: String,
}

impl std::fmt::Display for CpuDiscreteScoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "status {}: {}", self.status, self.message)
    }
}

impl std::error::Error for CpuDiscreteScoreError {}

#[derive(Clone)]
enum CpuDiscreteProposalModel {
    Route(CpuDiscreteRoute),
    Active(ActiveLeafModel),
}

#[derive(Clone)]
pub struct CpuDiscreteProposalScorer {
    chooser: CpuDiscreteProposalChooser,
    model: CpuDiscreteProposalModel,
    kept_cells: usize,
    objective: CpuDiscreteObjective,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CpuDiscreteObjective {
    NearestAddressMargin,
    OccupiedCellRankHinge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CpuWriterLoss {
    loss: u64,
    token_rivals: usize,
}

impl CpuDiscreteProposalScorer {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        let shape = &header["shape"];
        if header["matrices"]
            .as_array()
            .is_none_or(|matrices| !matrices.is_empty())
        {
            return Err(invalid("proposal scorer requires an empty matrices array"));
        }
        let source = &header["source"];
        if source["snapshot_schema"].as_str() != Some(FLOCK_SNAPSHOT_SCHEMA)
            || source["operator_kappa"].as_str() != Some(FLOCK_OPERATOR_KAPPA)
            || source["composition_kappa"].as_str() != Some(FLOCK_COMPOSITION_KAPPA)
        {
            return Err(invalid("proposal scorer source geometry differs"));
        }
        if !matches!(
            shape["objective_policy"].as_str(),
            Some("nearest_address_margin_v1" | "occupied_cell_rank_hinge_v1")
        ) {
            return Err(invalid("proposal scorer objective policy differs"));
        }
        validate_header_padding(bytes)?;
        let model = match shape["read_mode"].as_str() {
            Some("route") => CpuDiscreteProposalModel::Route(CpuDiscreteRoute::from_bytes(bytes)?),
            Some("flock") => CpuDiscreteProposalModel::Active(ActiveLeafModel::from_bytes(
                bytes,
                ActiveLeafKind::Flock,
            )?),
            Some("relation") => CpuDiscreteProposalModel::Active(ActiveLeafModel::from_bytes(
                bytes,
                ActiveLeafKind::Relation,
            )?),
            _ => return Err(invalid("proposal scorer read mode differs")),
        };
        let kept_cells = usize::try_from(shape["kept_cells"].as_u64().unwrap_or(0))
            .map_err(|_| invalid("proposal scorer kept_cells"))?;
        let objective = match shape["objective_policy"].as_str() {
            Some("nearest_address_margin_v1") => CpuDiscreteObjective::NearestAddressMargin,
            Some("occupied_cell_rank_hinge_v1") => CpuDiscreteObjective::OccupiedCellRankHinge,
            _ => return Err(invalid("proposal scorer objective policy differs")),
        };
        Ok(Self {
            chooser: CpuDiscreteProposalChooser::from_bytes(bytes)?,
            model,
            kept_cells,
            objective,
        })
    }

    pub fn score(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        samples: &[CpuDiscreteProposalSample],
        epoch: u64,
    ) -> std::result::Result<CpuDiscreteProposalScoring, CpuDiscreteScoreError> {
        self.validate_proposal(kind, index)
            .map_err(|error| score_error(self.proposal_refusal_status(kind), error.to_string()))?;
        if samples.is_empty() {
            return Err(score_error(8, "proposal scorer requires at least one row"));
        }

        let mut eligible = Vec::new();
        for sample in samples {
            let query = self
                .read(&self.model, &sample.prefix)
                .map_err(|error| score_error(2, error.to_string()))?;
            let table = self.token_blocks(&self.model);
            let loss = writer_loss(
                &query,
                table,
                sample.target,
                self.kept_cells,
                self.objective,
            )
            .map_err(writer_score_error)?;
            if loss.token_rivals != 0 {
                eligible.push((sample, loss.loss));
            }
        }
        if eligible.is_empty() {
            return Err(score_error(
                8,
                "proposal baseline admitted no eligible rows",
            ));
        }

        let active_lanes = if kind == CpuDiscreteProposalKind::Gate {
            3
        } else {
            16
        };
        let mut losses = [u64::MAX; 16];
        for (candidate, lane_loss) in losses.iter_mut().enumerate().take(active_lanes) {
            let model = if candidate == 0 {
                self.model.clone()
            } else {
                self.virtual_model(kind, index, candidate)
                    .map_err(|error| score_error(4, error.to_string()))?
            };
            let mut total = 0u64;
            let mut sentinel = false;
            for (sample, baseline_loss) in &eligible {
                let cost = if candidate == 0 {
                    Some(*baseline_loss)
                } else {
                    let query = self
                        .read(&model, &sample.prefix)
                        .map_err(|error| score_error(2, error.to_string()))?;
                    let loss = writer_loss(
                        &query,
                        self.token_blocks(&model),
                        sample.target,
                        self.kept_cells,
                        self.objective,
                    )
                    .map_err(writer_score_error)?;
                    (loss.token_rivals != 0 && loss.loss != u64::MAX).then_some(loss.loss)
                };
                match accumulate_lane(total, cost)? {
                    Some(next) => total = next,
                    None => {
                        sentinel = true;
                        break;
                    }
                }
            }
            *lane_loss = if sentinel { u64::MAX } else { total };
        }

        let proposal = self
            .chooser
            .choose(kind, index, &losses, epoch)
            .map_err(|error| score_error(4, error.to_string()))?;
        Ok(CpuDiscreteProposalScoring {
            losses,
            eligible_rows: eligible.len(),
            proposal,
        })
    }

    pub fn score_block(
        &self,
        kind: CpuDiscreteProposalKind,
        indices: &[usize],
        samples: &[CpuDiscreteProposalSample],
        epoch: u64,
    ) -> std::result::Result<CpuDiscreteBlockScoring, CpuDiscreteScoreError> {
        if indices.is_empty() {
            return Err(score_error(8, "proposal block contains no coordinates"));
        }
        let mut records = Vec::with_capacity(indices.len());
        let mut eligible_rows = 0usize;
        for &index in indices {
            match self.score(kind, index, samples, epoch) {
                Ok(scoring) => {
                    eligible_rows = eligible_rows.max(scoring.eligible_rows);
                    records.push(CpuBvgkRecord {
                        kind,
                        index,
                        proposal: scoring.proposal,
                        eligible: scoring.eligible_rows != 0,
                        status: 0,
                    });
                }
                Err(error) if error.status == 8 => {}
                Err(error) => return Err(error),
            }
        }
        if records.is_empty() {
            return Ok(CpuDiscreteBlockScoring {
                records,
                joint_loss: None,
                eligible_rows,
            });
        }
        let mut model = self.model.clone();
        for record in &records {
            self.apply_virtual_move(&mut model, kind, record.index, record.proposal.lane)
                .map_err(|error| score_error(4, error.to_string()))?;
        }
        let mut baseline_eligible = Vec::new();
        for sample in samples {
            let query = self
                .read(&self.model, &sample.prefix)
                .map_err(|error| score_error(2, error.to_string()))?;
            let loss = writer_loss(
                &query,
                self.token_blocks(&self.model),
                sample.target,
                self.kept_cells,
                self.objective,
            )
            .map_err(writer_score_error)?;
            if loss.token_rivals != 0 {
                baseline_eligible.push(sample);
            }
        }
        if baseline_eligible.is_empty() {
            return Ok(CpuDiscreteBlockScoring {
                records,
                joint_loss: None,
                eligible_rows,
            });
        }
        let mut joint_loss = 0u64;
        for sample in baseline_eligible {
            let query = self
                .read(&model, &sample.prefix)
                .map_err(|error| score_error(2, error.to_string()))?;
            let loss = writer_loss(
                &query,
                self.token_blocks(&model),
                sample.target,
                self.kept_cells,
                self.objective,
            )
            .map_err(writer_score_error)?;
            if !accumulate_joint_loss(&mut joint_loss, loss)? {
                return Ok(CpuDiscreteBlockScoring {
                    records,
                    joint_loss: None,
                    eligible_rows,
                });
            }
        }
        Ok(CpuDiscreteBlockScoring {
            records,
            joint_loss: Some(joint_loss),
            eligible_rows,
        })
    }

    /// Evaluate the current exact writer over causal rows. This is a
    /// development reading, not a language-qualification result.
    pub fn evaluate_samples(
        &self,
        samples: &[CpuDiscreteProposalSample],
    ) -> Result<CpuDiscretePanelReading> {
        if samples.is_empty() {
            return Err(invalid("discrete panel evaluation requires rows"));
        }
        let mut reading = CpuDiscretePanelReading {
            rows: 0,
            target_admitted: 0,
            top1_correct: 0,
            top8_correct: 0,
            margin_loss_sum: 0,
            target_rank_sum: 0,
        };
        for sample in samples {
            let query = self.read(&self.model, &sample.prefix)?;
            let observation = writer_observation(
                &query,
                self.token_blocks(&self.model),
                sample.target,
                self.kept_cells,
                self.objective,
            )?;
            reading.rows = reading
                .rows
                .checked_add(1)
                .ok_or_else(|| invalid("discrete panel row count overflows"))?;
            reading.target_admitted = reading
                .target_admitted
                .checked_add(u64::from(observation.target_admitted))
                .ok_or_else(|| invalid("discrete panel admission count overflows"))?;
            reading.top1_correct = reading
                .top1_correct
                .checked_add(u64::from(observation.top8[0] == sample.target))
                .ok_or_else(|| invalid("discrete panel top-1 count overflows"))?;
            reading.top8_correct = reading
                .top8_correct
                .checked_add(u64::from(observation.top8.contains(&sample.target)))
                .ok_or_else(|| invalid("discrete panel top-8 count overflows"))?;
            reading.margin_loss_sum = reading
                .margin_loss_sum
                .checked_add(observation.loss)
                .ok_or_else(|| invalid("discrete panel loss sum overflows"))?;
            reading.target_rank_sum = reading
                .target_rank_sum
                .checked_add(
                    u64::try_from(observation.target_rank)
                        .map_err(|_| invalid("discrete panel target rank overflows"))?,
                )
                .ok_or_else(|| invalid("discrete panel rank sum overflows"))?;
        }
        Ok(reading)
    }

    pub fn query(&self, sample: &CpuDiscreteProposalSample) -> Result<Vec<i32>> {
        self.read(&self.model, &sample.prefix)
    }

    pub fn top8(&self, sample: &CpuDiscreteProposalSample) -> Result<[u32; 8]> {
        let query = self.query(sample)?;
        writer_observation(
            &query,
            self.token_blocks(&self.model),
            sample.target,
            self.kept_cells,
            self.objective,
        )
        .map(|reading| reading.top8)
    }

    pub fn export_parity(
        &self,
        artifact: &[u8],
        samples: &[CpuDiscreteProposalSample],
    ) -> Result<CpuDiscretePanelExportReading> {
        if samples.is_empty() {
            return Err(invalid("discrete export parity requires rows"));
        }
        let model = uor_r4_integer::discrete_d11::IntegerDiscreteModel::from_bytes(artifact)
            .map_err(|error| invalid(format!("discrete serving artifact: {error}")))?;
        let mut reading = CpuDiscretePanelExportReading {
            rows: 0,
            query_coordinates_checked: 0,
            query_coordinate_mismatches: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
        };
        for sample in samples {
            let query = self.query(sample)?;
            let observation = writer_observation(
                &query,
                self.token_blocks(&self.model),
                sample.target,
                self.kept_cells,
                self.objective,
            )?;
            let served = model
                .evaluate_prefix(&sample.prefix)
                .map_err(|error| invalid(format!("discrete serving row: {error}")))?;
            reading.rows = reading
                .rows
                .checked_add(1)
                .ok_or_else(|| invalid("discrete export row count overflows"))?;
            reading.query_coordinates_checked = reading
                .query_coordinates_checked
                .checked_add(
                    u64::try_from(query.len())
                        .map_err(|_| invalid("discrete query length overflows"))?,
                )
                .ok_or_else(|| invalid("discrete query count overflows"))?;
            reading.query_coordinate_mismatches = reading
                .query_coordinate_mismatches
                .checked_add(
                    u64::try_from(
                        query
                            .iter()
                            .zip(&served.read_vector)
                            .filter(|(left, right)| left != right)
                            .count()
                            + query.len().abs_diff(served.read_vector.len()),
                    )
                    .unwrap_or(u64::MAX),
                )
                .ok_or_else(|| invalid("discrete query mismatch count overflows"))?;
            let cpu_cells = admitted_cells(&query, self.token_blocks(&self.model), self.kept_cells);
            reading.admitted_cell_mismatches = reading
                .admitted_cell_mismatches
                .checked_add(u64::from(cpu_cells != served.admitted_cells))
                .ok_or_else(|| invalid("discrete admitted-cell mismatch count overflows"))?;
            let mut served_top8 = [u32::MAX; 8];
            for (slot, &token) in served.top8.iter().take(8).enumerate() {
                served_top8[slot] = token;
            }
            reading.top8_mismatches = reading
                .top8_mismatches
                .checked_add(u64::from(observation.top8 != served_top8))
                .ok_or_else(|| invalid("discrete top-8 mismatch count overflows"))?;
            reading.token_mismatches = reading
                .token_mismatches
                .checked_add(u64::from(observation.top8[0] != served.prediction))
                .ok_or_else(|| invalid("discrete token mismatch count overflows"))?;
        }
        Ok(reading)
    }

    fn validate_proposal(&self, kind: CpuDiscreteProposalKind, index: usize) -> Result<()> {
        match (&self.model, kind) {
            (CpuDiscreteProposalModel::Route(_), CpuDiscreteProposalKind::Gate) => {
                Err(invalid("Gate proposal requires Flock or Relation mode"))
            }
            (CpuDiscreteProposalModel::Active(_), CpuDiscreteProposalKind::Route) => {
                Err(invalid("Route proposal requires Route mode"))
            }
            (CpuDiscreteProposalModel::Route(model), CpuDiscreteProposalKind::Route)
                if index >= model.blocks * model.stages =>
            {
                Err(invalid("Route proposal index outside map inventory"))
            }
            (_, CpuDiscreteProposalKind::Gate) if index >= self.chooser.gate_count => {
                Err(invalid("Gate proposal index outside gate inventory"))
            }
            (_, CpuDiscreteProposalKind::Table)
                if index >= self.chooser.token_blocks.len() * self.chooser.blocks =>
            {
                Err(invalid("Table proposal index outside codebook inventory"))
            }
            _ => Ok(()),
        }
    }

    fn proposal_refusal_status(&self, kind: CpuDiscreteProposalKind) -> u32 {
        match (&self.model, kind) {
            (CpuDiscreteProposalModel::Route(_), CpuDiscreteProposalKind::Gate)
            | (CpuDiscreteProposalModel::Active(_), CpuDiscreteProposalKind::Route) => 9,
            _ => 4,
        }
    }

    fn virtual_model(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        candidate: usize,
    ) -> Result<CpuDiscreteProposalModel> {
        let mut model = self.model.clone();
        self.apply_virtual_move(&mut model, kind, index, candidate)?;
        Ok(model)
    }

    fn apply_virtual_move(
        &self,
        model: &mut CpuDiscreteProposalModel,
        kind: CpuDiscreteProposalKind,
        index: usize,
        candidate: usize,
    ) -> Result<()> {
        let action = self
            .chooser
            .move_actions
            .get(candidate)
            .copied()
            .ok_or_else(|| invalid("proposal lane outside active inventory"))?;
        match (model, kind) {
            (CpuDiscreteProposalModel::Route(route), CpuDiscreteProposalKind::Route) => {
                let block = index / route.stages;
                let stage = index % route.stages;
                let current = route.routes[block][stage];
                route.routes[block][stage] = u32::from(
                    route
                        .composition
                        .compose_index(action, current as u8)
                        .ok_or_else(|| invalid("Route proposal composition failed"))?,
                );
            }
            (CpuDiscreteProposalModel::Active(active), CpuDiscreteProposalKind::Gate) => {
                let base_gates = active.blocks * active.stages;
                if index < base_gates {
                    let block = index / active.stages;
                    let level = index % active.stages;
                    active.level_gates[block][level] =
                        sibling_gate(active.level_gates[block][level], candidate)?;
                } else {
                    let tail = index - base_gates;
                    let level = tail / RELATION_EDGE_TYPES;
                    let edge = tail % RELATION_EDGE_TYPES;
                    let ActiveLeafMode::Relation { relation_gates, .. } = &mut active.mode else {
                        return Err(invalid("Gate proposal relation tail differs"));
                    };
                    let current = relation_gates
                        .get(level)
                        .ok_or_else(|| invalid("Gate proposal level outside inventory"))?[edge];
                    relation_gates[level][edge] = sibling_gate(current, candidate)?;
                }
            }
            (model, CpuDiscreteProposalKind::Table) => {
                let blocks = model.blocks();
                let token = index / blocks;
                let block = index % blocks;
                let table = model.token_blocks_mut();
                let point = table
                    .get_mut(token)
                    .and_then(|token| token.get_mut(block))
                    .ok_or_else(|| invalid("Table proposal index outside codebook"))?;
                *point = move_root(point, action, &self.chooser.composition)?;
            }
            _ => return Err(invalid("proposal kind differs from read mode")),
        }
        Ok(())
    }

    fn read(&self, model: &CpuDiscreteProposalModel, prefix: &[u32]) -> Result<Vec<i32>> {
        match model {
            CpuDiscreteProposalModel::Route(route) => route.read(prefix),
            CpuDiscreteProposalModel::Active(active) => active.read(prefix),
        }
    }

    fn token_blocks<'a>(&self, model: &'a CpuDiscreteProposalModel) -> &'a [Vec<[i32; 8]>] {
        match model {
            CpuDiscreteProposalModel::Route(route) => &route.token_blocks,
            CpuDiscreteProposalModel::Active(active) => &active.token_blocks,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteTrunkDecision {
    pub query: [i32; READ_WIDTH],
    pub admitted_cells: Vec<u32>,
    pub top8: [u32; 8],
    pub token: u32,
    pub read_code: u16,
    pub recurrence_code: u16,
    pub pointer_successor: Option<u16>,
}

/// Read-only independent CPU execution of a trained 19-table M1 trunk. The
/// base read is the same CPU Route/Flock/Relation implementation used by the
/// trainer; the recurrence, pointer selection, Hamilton rotation and
/// post-rotation writer are the pure-integer implementations in
/// `uor-r4-discrete::trunk`.
#[derive(Clone)]
pub struct CpuDiscreteTrunkReader {
    model: CpuDiscreteProposalModel,
    snapshot: Snapshot,
    kept_cells: usize,
    tables: uor_r4_discrete::trunk::TrunkTables,
    step_table: uor_r4_discrete::position_mechanism::PositionStepTable,
    norm_frames: NormFrameTable,
    codebook_roots: Vec<[u8; TRUNK_BLOCKS]>,
    codebook_coords: Vec<[i32; READ_WIDTH]>,
    token_cells: Vec<u32>,
    cell_tokens: [Vec<u16>; CELL_COUNT],
}

impl CpuDiscreteTrunkReader {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let parsed = header(bytes)?;
        if parsed["schema"].as_str() != Some(DISCRETE_TRUNK_SCHEMA)
            || parsed["shape"]["blocks"].as_u64() != Some(TRUNK_BLOCKS as u64)
            || parsed["matrices"]
                .as_array()
                .is_none_or(|matrices| !matrices.is_empty())
        {
            return Err(invalid(
                "CPU trunk reader requires the 19-table four-block M1 schema",
            ));
        }
        let kept_cells = usize::try_from(
            parsed["shape"]["kept_cells"]
                .as_u64()
                .ok_or_else(|| invalid("CPU trunk kept_cells differs"))?,
        )
        .map_err(|_| invalid("CPU trunk kept_cells"))?;
        if kept_cells != 8 {
            return Err(invalid(
                "CPU trunk reader requires the M1 writer's eight kept cells",
            ));
        }
        let model = match parsed["shape"]["read_mode"].as_str() {
            Some("route") => {
                CpuDiscreteProposalModel::Route(CpuDiscreteRoute::from_trunk_bytes(bytes)?)
            }
            Some("flock") => CpuDiscreteProposalModel::Active(ActiveLeafModel::from_trunk_bytes(
                bytes,
                ActiveLeafKind::Flock,
            )?),
            Some("relation") => CpuDiscreteProposalModel::Active(
                ActiveLeafModel::from_trunk_bytes(bytes, ActiveLeafKind::Relation)?,
            ),
            _ => return Err(invalid("CPU trunk read mode differs")),
        };
        let token_blocks = match &model {
            CpuDiscreteProposalModel::Route(model) => &model.token_blocks,
            CpuDiscreteProposalModel::Active(model) => &model.token_blocks,
        };
        let mut codebook_roots = Vec::with_capacity(token_blocks.len());
        let mut codebook_coords = Vec::with_capacity(token_blocks.len());
        let mut token_cells = Vec::with_capacity(token_blocks.len());
        for (token, blocks) in token_blocks.iter().enumerate() {
            let mut roots = [0u8; TRUNK_BLOCKS];
            let mut coords = [0i32; READ_WIDTH];
            for block in 0..TRUNK_BLOCKS {
                roots[block] = root_id(&blocks[block])
                    .and_then(|root| u8::try_from(root).ok())
                    .ok_or_else(|| invalid(format!("CPU trunk token {token} is off orbit")))?;
                coords[block * BLOCK_WIDTH..(block + 1) * BLOCK_WIDTH]
                    .copy_from_slice(&blocks[block]);
            }
            token_cells.push(u16::from(roots[0]) * 16 + u16::from(roots[TRUNK_BLOCKS - 1]));
            codebook_roots.push(roots);
            codebook_coords.push(coords);
        }
        let cell_tokens = build_cell_tokens(&token_cells)
            .map_err(|error| invalid(format!("CPU trunk writer cells: {error}")))?;
        let (snapshot, step_table, norm_frames, tables) = import_trunk_d11(bytes)
            .map_err(|error| invalid(format!("CPU trunk tables: {error}")))?;
        Ok(Self {
            model,
            snapshot,
            kept_cells,
            tables,
            step_table,
            norm_frames,
            codebook_roots,
            codebook_coords,
            token_cells: token_cells.into_iter().map(u32::from).collect(),
            cell_tokens,
        })
    }

    pub fn decide(&self, prefix: &[u32]) -> Result<CpuDiscreteTrunkDecision> {
        let read = self.read(&self.model, prefix)?;
        let q_read: [i32; READ_WIDTH] = read
            .as_slice()
            .try_into()
            .map_err(|_| invalid("CPU trunk base read width differs"))?;
        let mut read_roots = [0u8; TRUNK_BLOCKS];
        for (block, root) in read_roots.iter_mut().enumerate() {
            let coords: [i32; BLOCK_WIDTH] = q_read[block * BLOCK_WIDTH..(block + 1) * BLOCK_WIDTH]
                .try_into()
                .map_err(|_| invalid("CPU trunk base read block width differs"))?;
            *root = u8::try_from(
                root_id(&coords).ok_or_else(|| invalid("CPU trunk base read left the orbit"))?,
            )
            .map_err(|_| invalid("CPU trunk base read root exceeds u8"))?;
        }
        let read_code = read_roots
            .iter()
            .enumerate()
            .fold(0u16, |code, (block, root)| {
                code | (u16::from(*root) << (block * 4))
            });
        let window = prefix
            .iter()
            .map(|token| {
                u16::try_from(*token).map_err(|_| invalid("CPU trunk prefix token exceeds u16"))
            })
            .collect::<Result<Vec<_>>>()?;
        let (_, recurrence_code) = compute_recurrence_state(
            &window,
            &self.codebook_roots,
            &self.step_table,
            &self.norm_frames,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            0,
        )
        .map_err(|error| invalid(format!("CPU trunk recurrence: {error}")))?;
        let (_, pointer_successor, query, _, top8, admitted) = evaluate_joint_pointer_full_writer(
            &q_read,
            &window,
            read_code,
            recurrence_code,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            &self.tables,
            &self.codebook_roots,
            &self.codebook_coords,
            &self.cell_tokens,
        )
        .map_err(|error| invalid(format!("CPU trunk decision: {error}")))?;
        let admitted_cells = admitted
            .iter()
            .take(self.kept_cells)
            .filter(|&&cell| usize::from(cell) < CELL_COUNT)
            .map(|&cell| u32::from(cell))
            .collect::<Vec<_>>();
        if admitted_cells.len() != self.kept_cells {
            return Err(invalid("CPU trunk writer admitted fewer than eight cells"));
        }
        Ok(CpuDiscreteTrunkDecision {
            query,
            admitted_cells,
            top8,
            token: top8[0],
            read_code,
            recurrence_code,
            pointer_successor,
        })
    }

    pub fn observe_target(
        &self,
        prefix: &[u32],
        target: u32,
    ) -> Result<(CpuDiscreteTrunkDecision, bool, bool, bool)> {
        let decision = self.decide(prefix)?;
        let cell = self
            .token_cells
            .get(target as usize)
            .ok_or_else(|| invalid("CPU trunk target token exceeds vocabulary"))?;
        Ok((
            decision.clone(),
            decision.admitted_cells.contains(cell),
            decision.token == target,
            decision.top8.contains(&target),
        ))
    }

    pub fn training_row(
        &self,
        prefix: &[u32],
        target: u32,
        row_seed: u64,
    ) -> Result<TrunkTrainRow> {
        if target as usize >= self.codebook_roots.len() {
            return Err(invalid("CPU trunk target token exceeds vocabulary"));
        }
        if prefix.is_empty()
            || prefix.len()
                > usize::try_from(self.snapshot.spec.context)
                    .map_err(|_| invalid("CPU trunk context exceeds usize"))?
        {
            return Err(invalid("CPU trunk training prefix is outside context"));
        }
        let read = self.read(&self.model, prefix)?;
        let q_read: [i32; READ_WIDTH] = read
            .as_slice()
            .try_into()
            .map_err(|_| invalid("CPU trunk training read width differs"))?;
        let read_code = decode_read_vector(&q_read)
            .map_err(|error| invalid(format!("CPU trunk training read code: {error}")))?;
        let window = prefix
            .iter()
            .map(|token| {
                u16::try_from(*token).map_err(|_| invalid("CPU trunk prefix token exceeds u16"))
            })
            .collect::<Result<Vec<_>>>()?;
        let query_token = *window
            .last()
            .ok_or_else(|| invalid("CPU trunk training prefix is empty"))?;
        let target_token =
            u16::try_from(target).map_err(|_| invalid("CPU trunk target exceeds u16"))?;
        let (_, rec_code) = compute_recurrence_state(
            &window,
            &self.codebook_roots,
            &self.step_table,
            &self.norm_frames,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            row_seed,
        )
        .map_err(|error| invalid(format!("CPU trunk training recurrence: {error}")))?;
        let (_, rec_shuf_code) = compute_recurrence_state(
            &window,
            &self.codebook_roots,
            &self.step_table,
            &self.norm_frames,
            RecurrenceMode::ShuffledOrderControlV1,
            NormalizationMode::ScaleMatchedSnapV1,
            row_seed,
        )
        .map_err(|error| invalid(format!("CPU trunk shuffled training recurrence: {error}")))?;
        let (_, rec_unit_code) = compute_recurrence_state(
            &window,
            &self.codebook_roots,
            &self.step_table,
            &self.norm_frames,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::UnitSnapControlV1,
            row_seed,
        )
        .map_err(|error| invalid(format!("CPU trunk unit training recurrence: {error}")))?;
        Ok(TrunkTrainRow {
            read_code,
            rec_code,
            rec_shuf_code,
            rec_unit_code,
            query_token,
            target_token,
        })
    }

    pub fn add_training_rows(&self, rows: &[TrunkTrainRow]) -> Result<Vec<u8>> {
        if rows.is_empty() {
            return Err(invalid("CPU trunk count training requires rows"));
        }
        let mut tables = self.tables.clone();
        for row in rows {
            add_trunk_count_row(&mut tables, row)?;
        }
        let artifact =
            export_trunk_d11(&self.snapshot, &self.step_table, &self.norm_frames, &tables)
                .map_err(|error| invalid(format!("CPU trunk count export: {error}")))?;
        let (snapshot, step_table, norm_frames, decoded) = import_trunk_d11(&artifact)
            .map_err(|error| invalid(format!("CPU trunk count re-import: {error}")))?;
        let snapshot_equal = serde_json::to_vec(&snapshot)
            .map_err(|error| invalid(format!("CPU trunk snapshot encoding: {error}")))?
            == serde_json::to_vec(&self.snapshot)
                .map_err(|error| invalid(format!("CPU trunk source encoding: {error}")))?;
        if !snapshot_equal
            || step_table != self.step_table
            || norm_frames != self.norm_frames
            || decoded != tables
        {
            return Err(invalid("CPU trunk count artifact round-trip differs"));
        }
        Ok(artifact)
    }

    pub fn add_training_rows_consuming(mut self, rows: &[TrunkTrainRow]) -> Result<Vec<u8>> {
        if rows.is_empty() {
            return Err(invalid("CPU trunk count training requires rows"));
        }
        for row in rows {
            add_trunk_count_row(&mut self.tables, row)?;
        }
        export_trunk_d11(
            &self.snapshot,
            &self.step_table,
            &self.norm_frames,
            &self.tables,
        )
        .map_err(|error| invalid(format!("CPU trunk count export: {error}")))
    }

    pub fn export_parity(
        &self,
        artifact: &[u8],
        prefixes: &[Vec<u32>],
    ) -> Result<CpuDiscretePanelExportReading> {
        if prefixes.is_empty() {
            return Err(invalid("CPU trunk export parity requires rows"));
        }
        let served = uor_r4_integer::discrete_d11::IntegerDiscreteModel::from_bytes(artifact)
            .map_err(|error| invalid(format!("discrete serving artifact: {error}")))?;
        let mut reading = CpuDiscretePanelExportReading {
            rows: 0,
            query_coordinates_checked: 0,
            query_coordinate_mismatches: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
        };
        for prefix in prefixes {
            let cpu = self.decide(prefix)?;
            let served = served
                .evaluate_prefix(prefix)
                .map_err(|error| invalid(format!("discrete serving row: {error}")))?;
            reading.rows = reading
                .rows
                .checked_add(1)
                .ok_or_else(|| invalid("CPU trunk parity row count overflows"))?;
            reading.query_coordinates_checked = reading
                .query_coordinates_checked
                .checked_add(READ_WIDTH as u64)
                .ok_or_else(|| invalid("CPU trunk query count overflows"))?;
            reading.query_coordinate_mismatches = reading
                .query_coordinate_mismatches
                .checked_add(
                    cpu.query
                        .iter()
                        .zip(&served.read_vector)
                        .filter(|(left, right)| left != right)
                        .count() as u64,
                )
                .ok_or_else(|| invalid("CPU trunk query mismatch count overflows"))?;
            reading.admitted_cell_mismatches = reading
                .admitted_cell_mismatches
                .checked_add(u64::from(cpu.admitted_cells != served.admitted_cells))
                .ok_or_else(|| invalid("CPU trunk cell mismatch count overflows"))?;
            let mut served_top8 = [u32::MAX; 8];
            for (slot, &token) in served.top8.iter().take(8).enumerate() {
                served_top8[slot] = token;
            }
            reading.top8_mismatches = reading
                .top8_mismatches
                .checked_add(u64::from(cpu.top8 != served_top8))
                .ok_or_else(|| invalid("CPU trunk top-8 mismatch count overflows"))?;
            reading.token_mismatches = reading
                .token_mismatches
                .checked_add(u64::from(cpu.token != served.prediction))
                .ok_or_else(|| invalid("CPU trunk token mismatch count overflows"))?;
        }
        Ok(reading)
    }

    fn read(&self, model: &CpuDiscreteProposalModel, prefix: &[u32]) -> Result<Vec<i32>> {
        match model {
            CpuDiscreteProposalModel::Route(route) => route.read(prefix),
            CpuDiscreteProposalModel::Active(active) => active.read(prefix),
        }
    }
}

fn add_trunk_count_row(tables: &mut TrunkTables, row: &TrunkTrainRow) -> Result<()> {
    let target = usize::from(row.target_token);
    let query = u32::from(row.query_token);
    if target >= tables.unigram_counts.len() || target >= VOCABULARY || query >= VOCABULARY as u32 {
        return Err(invalid("CPU trunk count row token exceeds vocabulary"));
    }
    tables.unigram_counts[target] = tables.unigram_counts[target]
        .checked_add(1)
        .ok_or_else(|| invalid("CPU trunk unigram count overflows"))?;
    add_sparse_count(
        &mut tables.read_table,
        u32::from(row.read_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.rec_table,
        u32::from(row.rec_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.read_rec_table,
        pack_u32_key(row.read_code, row.rec_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.rec_shuf_table,
        u32::from(row.rec_shuf_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.read_rec_shuf_table,
        pack_u32_key(row.read_code, row.rec_shuf_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.rec_unit_table,
        u32::from(row.rec_unit_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.read_rec_unit_table,
        pack_u32_key(row.read_code, row.rec_unit_code),
        row.target_token,
    )?;
    add_sparse_count(&mut tables.query_table, query, row.target_token)?;
    add_sparse_count(
        &mut tables.query_read_table,
        pack_u32_key(row.query_token, row.read_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.query_rec_table,
        pack_u32_key(row.query_token, row.rec_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.query_rec_shuf_table,
        pack_u32_key(row.query_token, row.rec_shuf_code),
        row.target_token,
    )?;
    add_sparse_count(
        &mut tables.query_rec_unit_table,
        pack_u32_key(row.query_token, row.rec_unit_code),
        row.target_token,
    )?;
    let mut best_token = 0u16;
    let mut best_count = 0u32;
    for (token, &count) in tables.unigram_counts.iter().enumerate() {
        if count > best_count {
            best_count = count;
            best_token =
                u16::try_from(token).map_err(|_| invalid("CPU trunk unigram token exceeds u16"))?;
        }
    }
    tables.unigram_best_global = best_token;
    Ok(())
}

fn add_sparse_count(table: &mut SparseCountTable, key: u32, token: u16) -> Result<()> {
    if table.get(key, token) == u32::MAX || table.total() == u64::MAX {
        return Err(invalid("CPU trunk sparse count overflows"));
    }
    table.add(key, token);
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuDiscreteApplyError {
    pub status: u32,
    pub message: String,
}

impl std::fmt::Display for CpuDiscreteApplyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "status {}: {}", self.status, self.message)
    }
}

impl std::error::Error for CpuDiscreteApplyError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuDiscreteApplyReceipt {
    pub kind: CpuDiscreteProposalKind,
    pub index: usize,
    pub lane: usize,
    pub loss_before: u64,
    pub loss_after: u64,
    pub epoch_before: u64,
    pub epoch_after: u64,
    pub applied: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuDiscreteSourceIdentity {
    snapshot_schema: String,
    snapshot_sha256: String,
    operator_kappa: String,
    composition_kappa: String,
    kernel_sha256: String,
    source_sha256: String,
    geometry_sha256: String,
    read_mode: String,
    read_initialization: String,
    context: usize,
    vocabulary: usize,
    blocks: usize,
    stages: usize,
    kept_cells: usize,
    read_kept: usize,
    read_sweeps: usize,
    objective_policy: String,
    seed: u32,
}

impl CpuDiscreteSourceIdentity {
    fn from_header(header: &serde_json::Value) -> Result<Self> {
        let source = &header["source"];
        let shape = &header["shape"];
        let required = |name: &str| {
            source[name]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid(format!("proposal source {name} differs")))
        };
        let shape_string = |name: &str| {
            shape[name]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid(format!("proposal shape {name} differs")))
        };
        let shape_usize = |name: &str| {
            usize::try_from(shape[name].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid(format!("proposal shape {name} differs")))
        };
        let identity = Self {
            snapshot_schema: required("snapshot_schema")?,
            snapshot_sha256: required("snapshot_sha256")?,
            operator_kappa: required("operator_kappa")?,
            composition_kappa: required("composition_kappa")?,
            kernel_sha256: required("kernel_sha256")?,
            source_sha256: required("source_sha256")?,
            geometry_sha256: required("geometry_sha256")?,
            read_mode: shape_string("read_mode")?,
            read_initialization: shape_string("read_initialization")?,
            context: shape_usize("context")?,
            vocabulary: shape_usize("vocabulary")?,
            blocks: shape_usize("blocks")?,
            stages: shape_usize("stages")?,
            kept_cells: shape_usize("kept_cells")?,
            read_kept: shape_usize("read_kept")?,
            read_sweeps: shape_usize("read_sweeps")?,
            objective_policy: shape_string("objective_policy")?,
            seed: u32::try_from(shape["seed"].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid("proposal source seed differs"))?,
        };
        for hash in [
            &identity.snapshot_sha256,
            &identity.kernel_sha256,
            &identity.source_sha256,
            &identity.geometry_sha256,
        ] {
            uor_r4_discrete::corpus::validate_sha256(hash)
                .map_err(|error| invalid(error.to_string()))?;
        }
        Ok(identity)
    }

    fn validate_model(&self, model: &CpuDiscreteModelState) -> Result<()> {
        let (
            read_mode,
            read_initialization,
            context,
            vocabulary,
            blocks,
            stages,
            kept_cells,
            read_kept,
            read_sweeps,
            objective_policy,
        ) = match model {
            CpuDiscreteModelState::Route {
                context,
                kept_cells,
                objective,
                token_blocks,
                ..
            } => (
                "route",
                "zero_gates_v1",
                *context,
                token_blocks.len(),
                token_blocks.first().map_or(0, Vec::len),
                expected_stages(*context).unwrap_or(0),
                *kept_cells,
                0,
                0,
                objective_name(*objective),
            ),
            CpuDiscreteModelState::Flock {
                context,
                kept_cells,
                objective,
                read_kept,
                token_blocks,
                level_gates,
                ..
            } => (
                "flock",
                "active_leaf_v1",
                *context,
                token_blocks.len(),
                token_blocks.first().map_or(0, Vec::len),
                level_gates.first().map_or(0, Vec::len),
                *kept_cells,
                *read_kept,
                0,
                objective_name(*objective),
            ),
            CpuDiscreteModelState::Relation {
                context,
                kept_cells,
                objective,
                read_kept,
                sweeps,
                token_blocks,
                level_gates,
                ..
            } => (
                "relation",
                "active_leaf_v1",
                *context,
                token_blocks.len(),
                token_blocks.first().map_or(0, Vec::len),
                level_gates.first().map_or(0, Vec::len),
                *kept_cells,
                *read_kept,
                *sweeps,
                objective_name(*objective),
            ),
        };
        if (
            self.read_mode.as_str(),
            self.read_initialization.as_str(),
            self.context,
            self.vocabulary,
            self.blocks,
            self.stages,
            self.kept_cells,
            self.read_kept,
            self.read_sweeps,
            self.objective_policy.as_str(),
        ) != (
            read_mode,
            read_initialization,
            context,
            vocabulary,
            blocks,
            stages,
            kept_cells,
            read_kept,
            read_sweeps,
            objective_policy,
        ) {
            return Err(invalid(
                "CPU checkpoint model shape or objective differs from its source",
            ));
        }
        Ok(())
    }
}

pub struct CpuDiscreteProposalApplier {
    scorer: CpuDiscreteProposalScorer,
    epoch: u64,
    source_artifact_sha256: String,
    source_identity: CpuDiscreteSourceIdentity,
    base_artifact: Vec<u8>,
}

impl CpuDiscreteProposalApplier {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = header(bytes)?;
        let epoch = header["shape"]["epoch"]
            .as_u64()
            .ok_or_else(|| invalid("proposal apply epoch differs"))?;
        Ok(Self {
            scorer: CpuDiscreteProposalScorer::from_bytes(bytes)?,
            epoch,
            source_artifact_sha256: uor_r4_discrete::corpus::sha256(bytes),
            source_identity: CpuDiscreteSourceIdentity::from_header(&header)?,
            base_artifact: bytes.to_vec(),
        })
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn apply(
        &mut self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        proposal: CpuDiscreteProposal,
    ) -> std::result::Result<CpuDiscreteApplyReceipt, CpuDiscreteApplyError> {
        if proposal.epoch != self.epoch {
            return Err(apply_error(3, "proposal epoch differs"));
        }
        self.scorer
            .validate_proposal(kind, index)
            .map_err(|error| {
                apply_error(self.scorer.proposal_refusal_status(kind), error.to_string())
            })?;
        let active = if kind == CpuDiscreteProposalKind::Gate {
            3
        } else {
            16
        };
        if proposal.lane >= active {
            return Err(apply_error(4, "proposal lane outside active inventory"));
        }
        let epoch_before = self.epoch;
        if proposal.lane == 0 {
            return Ok(CpuDiscreteApplyReceipt {
                kind,
                index,
                lane: 0,
                loss_before: proposal.current_loss,
                loss_after: proposal.current_loss,
                epoch_before,
                epoch_after: epoch_before,
                applied: false,
            });
        }
        if proposal.selected_loss >= proposal.current_loss || self.epoch == u64::MAX {
            return Err(apply_error(5, "proposal is not a strict-improving move"));
        }
        let model = self
            .scorer
            .virtual_model(kind, index, proposal.lane)
            .map_err(|error| apply_error(4, error.to_string()))?;
        self.scorer.model = model;
        match kind {
            CpuDiscreteProposalKind::Route => {
                let CpuDiscreteProposalModel::Route(route) = &self.scorer.model else {
                    return Err(apply_error(4, "applied Route state changed mode"));
                };
                self.scorer.chooser.routes = route.routes.iter().flatten().copied().collect();
            }
            CpuDiscreteProposalKind::Table => {
                let token_blocks = self.scorer.token_blocks(&self.scorer.model).to_vec();
                self.scorer.chooser.token_blocks = token_blocks;
            }
            CpuDiscreteProposalKind::Gate => {}
        }
        self.epoch += 1;
        Ok(CpuDiscreteApplyReceipt {
            kind,
            index,
            lane: proposal.lane,
            loss_before: proposal.current_loss,
            loss_after: proposal.selected_loss,
            epoch_before,
            epoch_after: self.epoch,
            applied: true,
        })
    }

    pub fn decide_bvgk(
        &self,
        kind: CpuDiscreteProposalKind,
        records: &[CpuBvgkRecord],
        joint_loss: Option<u64>,
    ) -> std::result::Result<CpuBvgkDecision, CpuDiscreteApplyError> {
        let mut members = vec![false; records.len()];
        let mut count = 0usize;
        let mut best: Option<usize> = None;
        let active = match kind {
            CpuDiscreteProposalKind::Gate => 3,
            CpuDiscreteProposalKind::Route | CpuDiscreteProposalKind::Table => 16,
        };
        for (ordinal, record) in records.iter().enumerate() {
            let valid = record.status == 0
                && record.eligible
                && record.proposal.lane < active
                && record.proposal.lane != 0
                && record.proposal.selected_loss < record.proposal.current_loss
                && self.scorer.chooser.bvgk_in_range(kind, record.index);
            members[ordinal] = valid;
            if valid {
                count += 1;
                if best.is_none_or(|ordinal| {
                    record.proposal.selected_loss < records[ordinal].proposal.selected_loss
                }) {
                    best = Some(ordinal);
                }
            }
        }
        if count >= 2 && joint_loss.is_none() {
            return Err(apply_error(17, "BVG-K joint loss is absent"));
        }
        let joint =
            count >= 2 && joint_loss < best.map(|ordinal| records[ordinal].proposal.selected_loss);
        let commit = records
            .iter()
            .enumerate()
            .map(|(ordinal, _)| {
                count != 0 && (joint && members[ordinal] || !joint && Some(ordinal) == best)
            })
            .collect::<Vec<_>>();
        Ok(CpuBvgkDecision {
            members,
            count,
            best,
            commit,
            joint,
            joint_loss,
            epoch: self.epoch,
        })
    }

    pub fn apply_bvgk(
        &mut self,
        kind: CpuDiscreteProposalKind,
        records: &[CpuBvgkRecord],
        decision: &CpuBvgkDecision,
    ) -> std::result::Result<CpuBvgkBatchReceipt, CpuDiscreteApplyError> {
        if records.len() != decision.commit.len() || records.len() != decision.members.len() {
            return Err(apply_error(4, "BVG-K record and decision lengths differ"));
        }
        if decision.epoch != self.epoch {
            return Err(apply_error(3, "BVG-K decision epoch differs"));
        }
        let mut committed: Vec<usize> = Vec::new();
        for (ordinal, record) in records.iter().enumerate() {
            if !decision.commit[ordinal] {
                continue;
            }
            if record.kind != kind {
                return Err(apply_error(14, "BVG-K member kind differs"));
            }
            if record.proposal.epoch != self.epoch {
                return Err(apply_error(3, "BVG-K member epoch differs"));
            }
            self.scorer
                .validate_proposal(kind, record.index)
                .map_err(|error| {
                    apply_error(self.scorer.proposal_refusal_status(kind), error.to_string())
                })?;
            let active = if kind == CpuDiscreteProposalKind::Gate {
                3
            } else {
                16
            };
            if record.proposal.lane >= active {
                return Err(apply_error(4, "BVG-K member lane outside active inventory"));
            }
            if record.proposal.lane == 0
                || record.proposal.selected_loss >= record.proposal.current_loss
                || self.epoch == u64::MAX
            {
                return Err(apply_error(5, "BVG-K member is not improving"));
            }
            if committed
                .iter()
                .any(|&prior| records[prior].index == record.index)
            {
                return Err(apply_error(15, "BVG-K members share one index"));
            }
            committed.push(ordinal);
        }
        if committed.is_empty() {
            return Err(apply_error(16, "BVG-K commit set is empty"));
        }
        let mut model = self.scorer.model.clone();
        for &ordinal in &committed {
            let record = &records[ordinal];
            self.scorer
                .apply_virtual_move(&mut model, kind, record.index, record.proposal.lane)
                .map_err(|error| apply_error(4, error.to_string()))?;
        }
        let refreshed_routes = match kind {
            CpuDiscreteProposalKind::Route => {
                let CpuDiscreteProposalModel::Route(route) = &model else {
                    return Err(apply_error(4, "BVG-K Route state changed mode"));
                };
                Some(route.routes.iter().flatten().copied().collect())
            }
            CpuDiscreteProposalKind::Table | CpuDiscreteProposalKind::Gate => None,
        };
        let refreshed_token_blocks = match kind {
            CpuDiscreteProposalKind::Table => Some(self.scorer.token_blocks(&model).to_vec()),
            CpuDiscreteProposalKind::Route | CpuDiscreteProposalKind::Gate => None,
        };
        self.scorer.model = model;
        match kind {
            CpuDiscreteProposalKind::Route => {
                self.scorer.chooser.routes = refreshed_routes.expect("Route refresh is present");
            }
            CpuDiscreteProposalKind::Table => {
                self.scorer.chooser.token_blocks =
                    refreshed_token_blocks.expect("Table refresh is present");
            }
            CpuDiscreteProposalKind::Gate => {}
        }
        let epoch_before = self.epoch;
        self.epoch += 1;
        Ok(CpuBvgkBatchReceipt {
            committed,
            joint: decision.joint,
            epoch_before,
            epoch_after: self.epoch,
        })
    }

    /// Rebuild the four-table `UORLUT01` artifact from the current exact
    /// model state. The header's epoch advances with the model; every section
    /// length and span remains fixed.
    pub fn artifact_bytes(&self) -> Result<Vec<u8>> {
        if self.base_artifact.is_empty() {
            return Err(invalid(
                "CPU checkpoint was resumed without its source artifact",
            ));
        }
        let mut artifact_header = header(&self.base_artifact)?;
        artifact_header["shape"]["epoch"] = serde_json::json!(self.epoch);
        let tables = artifact_header["tables"]
            .as_array()
            .ok_or_else(|| invalid("discrete artifact tables differ"))?
            .clone();
        let model = CpuDiscreteModelState::from_scorer(&self.scorer);
        let table = flattened_token_table(&self.scorer);
        let cells = token_cells(&self.scorer);
        let (routes, gates) = flattened_read_state(&self.scorer)?;
        let mut data = Vec::<u8>::new();
        for entry in tables {
            let name = entry["name"]
                .as_str()
                .ok_or_else(|| invalid("discrete artifact table name differs"))?;
            let offset = usize::try_from(
                entry["span"]["offset"]
                    .as_u64()
                    .ok_or_else(|| invalid("discrete artifact span offset differs"))?,
            )
            .map_err(|_| invalid("discrete artifact span offset exceeds usize"))?;
            if offset < data.len() {
                return Err(invalid(
                    "discrete artifact writeback requires contiguous table spans",
                ));
            }
            data.resize(offset, 0);
            let section_start = data.len();
            match name {
                "discrete.table" => append_i32(&mut data, &table),
                "discrete.cells" => append_u32(&mut data, &cells),
                "discrete.routes" => append_u32(&mut data, &routes),
                "discrete.gates" => append_i32(&mut data, &gates),
                _ => {
                    return Err(invalid(
                        "CPU discrete writeback supports only the four base tables",
                    ))
                }
            }
            let section_bytes = data.len() - section_start;
            if entry["span"]["bytes"].as_u64() != Some(section_bytes as u64) {
                return Err(invalid("discrete artifact writeback table length differs"));
            }
        }
        let header_bytes =
            serde_json::to_vec(&artifact_header).map_err(|error| invalid(error.to_string()))?;
        let unaligned = 16usize
            .checked_add(header_bytes.len())
            .ok_or_else(|| invalid("discrete artifact header length overflows"))?;
        let data_start = unaligned
            .checked_add(63)
            .ok_or_else(|| invalid("discrete artifact alignment overflows"))?
            / 64
            * 64;
        let mut artifact = vec![0u8; data_start + data.len()];
        artifact[..8].copy_from_slice(b"UORLUT01");
        artifact[8..16].copy_from_slice(&(header_bytes.len() as u64).to_le_bytes());
        artifact[16..16 + header_bytes.len()].copy_from_slice(&header_bytes);
        artifact[data_start..].copy_from_slice(&data);
        let parsed_header = header(&artifact)?;
        validate_layout(&artifact, &parsed_header)?;
        let reparsed = CpuDiscreteProposalScorer::from_bytes(&artifact)?;
        if canonical_json_sha256(&model)?
            != canonical_json_sha256(&CpuDiscreteModelState::from_scorer(&reparsed))?
        {
            return Err(invalid(
                "CPU discrete artifact writeback changed the model state",
            ));
        }
        Ok(artifact)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuBvgkRecord {
    pub kind: CpuDiscreteProposalKind,
    pub index: usize,
    pub proposal: CpuDiscreteProposal,
    pub eligible: bool,
    pub status: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuBvgkDecision {
    pub members: Vec<bool>,
    pub count: usize,
    pub best: Option<usize>,
    pub commit: Vec<bool>,
    pub joint: bool,
    pub joint_loss: Option<u64>,
    pub epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuBvgkBatchReceipt {
    pub committed: Vec<usize>,
    pub joint: bool,
    pub epoch_before: u64,
    pub epoch_after: u64,
}

pub const CPU_DISCRETE_CHECKPOINT_SCHEMA: &str = "uor-r4.cpu-discrete-checkpoint/1";
const CPU_DISCRETE_LINEAGE_SCHEMA: &str = "uor-r4.cpu-discrete-lineage/1";
const CPU_SUBSTRATE: &str = "cpu-development-research";
const CPU_QUALIFICATION: &str = "gpu-required";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuDiscreteSchedule {
    policy: SchedulePolicy,
    coordinate_block: u32,
    batch_rows: u32,
    eligible_targets: u64,
}

impl CpuDiscreteSchedule {
    pub fn new(
        policy: SchedulePolicy,
        coordinate_block: u32,
        batch_rows: u32,
        eligible_targets: u64,
    ) -> Result<Self> {
        let schedule = Self {
            policy,
            coordinate_block,
            batch_rows,
            eligible_targets,
        };
        schedule.validate()?;
        Ok(schedule)
    }

    fn validate(&self) -> Result<()> {
        if self.policy.carrier_mode().is_some() {
            return Err(invalid(
                "CPU checkpoint does not yet represent CUDA carrier target selection",
            ));
        }
        if !(1..=64).contains(&self.coordinate_block)
            || !(1..=64).contains(&self.batch_rows)
            || self.eligible_targets == 0
        {
            return Err(invalid(
                "CPU checkpoint schedule requires coordinate block 1..64, batch rows 1..64 and targets > 0",
            ));
        }
        Ok(())
    }

    fn validate_for_model(&self, model: &CpuDiscreteProposalModel) -> Result<()> {
        self.validate()?;
        if matches!(model, CpuDiscreteProposalModel::Active(_))
            && self.policy != SchedulePolicy::InterleavedReadEvery8V1
        {
            return Err(invalid(
                "active-leaf CPU checkpoint requires the interleaved schedule",
            ));
        }
        Ok(())
    }

    pub fn policy(&self) -> SchedulePolicy {
        self.policy
    }

    pub fn coordinate_block(&self) -> u32 {
        self.coordinate_block
    }

    pub fn batch_rows(&self) -> u32 {
        self.batch_rows
    }

    pub fn eligible_targets(&self) -> u64 {
        self.eligible_targets
    }

    pub fn initial_cursor(&self) -> Cursor {
        Cursor {
            block: (self.coordinate_block > 1).then_some(BlockCursor {
                o_vec: 0,
                o_read: 0,
                committed_moves: 0,
            }),
            ..Cursor::initial()
        }
    }

    fn batches(&self) -> u64 {
        self.eligible_targets.div_ceil(u64::from(self.batch_rows))
    }

    fn dimensions(&self, model: &CpuDiscreteProposalModel) -> (u64, u64) {
        let vectors = (model.token_blocks().len() * model.blocks()) as u64;
        let reads = match model {
            CpuDiscreteProposalModel::Route(route) => (route.blocks * route.stages) as u64,
            CpuDiscreteProposalModel::Active(active) => {
                let relation = match active.mode {
                    ActiveLeafMode::Flock => 0,
                    ActiveLeafMode::Relation { .. } => RELATION_EDGE_TYPES * active.stages,
                };
                (active.blocks * active.stages + relation) as u64
            }
        };
        (vectors, reads)
    }

    fn cycle(&self, model: &CpuDiscreteProposalModel) -> u64 {
        let (vectors, reads) = self.dimensions(model);
        match self.policy {
            SchedulePolicy::CanonicalVectorThenReadV1 => vectors + reads,
            SchedulePolicy::InterleavedReadEvery8V1 => {
                let base_gates = match model {
                    CpuDiscreteProposalModel::Route(route) => route.blocks * route.stages,
                    CpuDiscreteProposalModel::Active(active) => active.blocks * active.stages,
                };
                vectors + base_gates as u64
            }
            SchedulePolicy::CarrierCreditV1
            | SchedulePolicy::CarrierUniformTargetV1
            | SchedulePolicy::CarrierUniformRowV1 => unreachable!("carrier schedules are refused"),
        }
    }

    fn is_read_batch(
        &self,
        completed_batches: u64,
        model: &CpuDiscreteProposalModel,
    ) -> Result<bool> {
        match self.policy {
            SchedulePolicy::InterleavedReadEvery8V1 => Ok(completed_batches % 8 == 7),
            SchedulePolicy::CanonicalVectorThenReadV1 => {
                let (vectors, reads) = self.dimensions(model);
                let k = u64::from(self.coordinate_block);
                Ok(
                    completed_batches % (vectors.div_ceil(k) + reads.div_ceil(k))
                        >= vectors.div_ceil(k),
                )
            }
            SchedulePolicy::CarrierCreditV1
            | SchedulePolicy::CarrierUniformTargetV1
            | SchedulePolicy::CarrierUniformRowV1 => Err(invalid("carrier schedules are refused")),
        }
    }

    fn validate_cursor(&self, cursor: &Cursor, model: &CpuDiscreteProposalModel) -> Result<()> {
        self.validate_for_model(model)?;
        let batches = self.batches();
        let cycle = self.cycle(model);
        let expected_next_target = (cursor.completed_batches % batches)
            .checked_mul(u64::from(self.batch_rows))
            .ok_or_else(|| invalid("checkpoint target cursor overflow"))?;
        if cursor.accepted_updates > cursor.completed_batches
            || cursor.corpus_pass != cursor.completed_batches / batches
            || cursor.next_target != expected_next_target
            || cursor.next_coordinate != (cursor.completed_batches % cycle) as u32
        {
            return Err(invalid(
                "checkpoint cursor differs from corpus/schedule cycle",
            ));
        }
        let k = self.coordinate_block;
        match (&cursor.block, k > 1) {
            (None, false) => Ok(()),
            (Some(_), false) => Err(invalid("coordinate block 1 cursor carries block fields")),
            (None, true) => Err(invalid("coordinate block > 1 cursor requires block fields")),
            (Some(block), true) => {
                let (vectors, reads) = self.dimensions(model);
                let expected =
                    family_cursors(cursor.completed_batches, k, self.policy, vectors, reads)
                        .map_err(|error| invalid(error.to_string()))?;
                if (block.o_vec, block.o_read) != expected {
                    return Err(invalid(
                        "checkpoint o_vec/o_read differ from the block schedule",
                    ));
                }
                let max_moves = cursor
                    .accepted_updates
                    .checked_mul(u64::from(k))
                    .ok_or_else(|| invalid("checkpoint accepted-update bound overflow"))?;
                if block.committed_moves < cursor.accepted_updates
                    || block.committed_moves > max_moves
                {
                    return Err(invalid(
                        "checkpoint committed_moves outside accepted-update bounds",
                    ));
                }
                Ok(())
            }
        }
    }

    fn block_length(&self, cursor: &Cursor, model: &CpuDiscreteProposalModel) -> Result<u64> {
        if self.coordinate_block == 1 {
            return Ok(1);
        }
        let (vectors, reads) = self.dimensions(model);
        let (o_vec, o_read) = family_cursors(
            cursor.completed_batches,
            self.coordinate_block,
            self.policy,
            vectors,
            reads,
        )
        .map_err(|error| invalid(error.to_string()))?;
        let (ordinal, size) = if self.is_read_batch(cursor.completed_batches, model)? {
            (o_read, reads)
        } else {
            (o_vec, vectors)
        };
        let start = ordinal % size;
        Ok(size.min(start + u64::from(self.coordinate_block)) - start)
    }

    fn advance(
        &self,
        cursor: &Cursor,
        model: &CpuDiscreteProposalModel,
        committed: bool,
        moves: u64,
    ) -> Result<Cursor> {
        self.validate_cursor(cursor, model)?;
        let length = self.block_length(cursor, model)?;
        if committed != (moves > 0) || moves > length {
            return Err(invalid(
                "batch commit needs 1..=block-length moves, and no commit none",
            ));
        }
        let completed_batches = cursor
            .completed_batches
            .checked_add(1)
            .ok_or_else(|| invalid("batch cursor overflow"))?;
        let accepted_updates = cursor
            .accepted_updates
            .checked_add(u64::from(committed))
            .ok_or_else(|| invalid("update cursor overflow"))?;
        let block = if self.coordinate_block == 1 {
            None
        } else {
            let (vectors, reads) = self.dimensions(model);
            let (o_vec, o_read) = family_cursors(
                completed_batches,
                self.coordinate_block,
                self.policy,
                vectors,
                reads,
            )
            .map_err(|error| invalid(error.to_string()))?;
            Some(BlockCursor {
                o_vec,
                o_read,
                committed_moves: cursor
                    .block
                    .ok_or_else(|| invalid("checkpoint block fields are absent"))?
                    .committed_moves
                    .checked_add(moves)
                    .ok_or_else(|| invalid("move cursor overflow"))?,
            })
        };
        Ok(Cursor {
            completed_batches,
            accepted_updates,
            corpus_pass: completed_batches / self.batches(),
            next_target: (completed_batches % self.batches())
                .checked_mul(u64::from(self.batch_rows))
                .ok_or_else(|| invalid("target cursor overflow"))?,
            next_coordinate: (completed_batches % self.cycle(model)) as u32,
            block,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum CpuDiscreteModelState {
    Route {
        context: usize,
        kept_cells: usize,
        objective: CpuDiscreteObjective,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<Vec<u32>>,
    },
    Flock {
        context: usize,
        kept_cells: usize,
        objective: CpuDiscreteObjective,
        read_kept: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<u32>,
        level_gates: Vec<Vec<i32>>,
    },
    Relation {
        context: usize,
        kept_cells: usize,
        objective: CpuDiscreteObjective,
        read_kept: usize,
        sweeps: usize,
        token_blocks: Vec<Vec<[i32; 8]>>,
        routes: Vec<u32>,
        level_gates: Vec<Vec<i32>>,
        relation_gates: Vec<[i32; RELATION_EDGE_TYPES]>,
    },
}

fn objective_name(objective: CpuDiscreteObjective) -> &'static str {
    match objective {
        CpuDiscreteObjective::NearestAddressMargin => "nearest_address_margin_v1",
        CpuDiscreteObjective::OccupiedCellRankHinge => "occupied_cell_rank_hinge_v1",
    }
}

impl CpuDiscreteModelState {
    fn from_scorer(scorer: &CpuDiscreteProposalScorer) -> Self {
        match &scorer.model {
            CpuDiscreteProposalModel::Route(route) => Self::Route {
                context: route.context,
                kept_cells: scorer.kept_cells,
                objective: scorer.objective,
                token_blocks: route.token_blocks.clone(),
                routes: route.routes.clone(),
            },
            CpuDiscreteProposalModel::Active(active) => match &active.mode {
                ActiveLeafMode::Flock => Self::Flock {
                    context: active.context,
                    kept_cells: scorer.kept_cells,
                    objective: scorer.objective,
                    read_kept: active.kept,
                    token_blocks: active.token_blocks.clone(),
                    routes: active.routes.clone(),
                    level_gates: active.level_gates.clone(),
                },
                ActiveLeafMode::Relation {
                    sweeps,
                    relation_gates,
                } => Self::Relation {
                    context: active.context,
                    kept_cells: scorer.kept_cells,
                    objective: scorer.objective,
                    read_kept: active.kept,
                    sweeps: *sweeps,
                    token_blocks: active.token_blocks.clone(),
                    routes: active.routes.clone(),
                    level_gates: active.level_gates.clone(),
                    relation_gates: relation_gates.clone(),
                },
            },
        }
    }

    fn restore(&self) -> Result<(CpuDiscreteProposalModel, usize, CpuDiscreteObjective)> {
        match self {
            Self::Route {
                context,
                kept_cells,
                objective,
                token_blocks,
                routes,
            } => Ok((
                CpuDiscreteProposalModel::Route(CpuDiscreteRoute::from_parts(
                    *context,
                    token_blocks.clone(),
                    routes.clone(),
                )?),
                *kept_cells,
                *objective,
            )),
            Self::Flock {
                context,
                kept_cells,
                objective,
                read_kept,
                token_blocks,
                routes,
                level_gates,
            } => Ok((
                CpuDiscreteProposalModel::Active(ActiveLeafModel::from_parts_with_routes(
                    "Flock",
                    *context,
                    *read_kept,
                    token_blocks.clone(),
                    routes.clone(),
                    level_gates.clone(),
                    ActiveLeafMode::Flock,
                )?),
                *kept_cells,
                *objective,
            )),
            Self::Relation {
                context,
                kept_cells,
                objective,
                read_kept,
                sweeps,
                token_blocks,
                routes,
                level_gates,
                relation_gates,
            } => Ok((
                CpuDiscreteProposalModel::Active(ActiveLeafModel::from_parts_with_routes(
                    "Relation",
                    *context,
                    *read_kept,
                    token_blocks.clone(),
                    routes.clone(),
                    level_gates.clone(),
                    ActiveLeafMode::Relation {
                        sweeps: *sweeps,
                        relation_gates: relation_gates.clone(),
                    },
                )?),
                *kept_cells,
                *objective,
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuDiscreteChooserState {
    mode: CpuDiscreteProposalMode,
    blocks: usize,
    stages: usize,
    routes: Vec<u32>,
    token_blocks: Vec<Vec<[i32; 8]>>,
    gate_count: usize,
}

impl CpuDiscreteChooserState {
    fn from_chooser(chooser: &CpuDiscreteProposalChooser) -> Self {
        Self {
            mode: chooser.mode,
            blocks: chooser.blocks,
            stages: chooser.stages,
            routes: chooser.routes.clone(),
            token_blocks: chooser.token_blocks.clone(),
            gate_count: chooser.gate_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuDiscreteLineage {
    schema: String,
    source_artifact_sha256: String,
    #[serde(default)]
    source_epoch: u64,
    source_identity: CpuDiscreteSourceIdentity,
    schedule: CpuDiscreteSchedule,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuDiscreteCheckpoint {
    pub schema: String,
    pub substrate: String,
    pub qualification: String,
    pub source_artifact_sha256: String,
    pub lineage_sha256: String,
    pub cursor: Cursor,
    pub model_epoch: u64,
    pub model_sha256: String,
    pub chooser_sha256: String,
    lineage: CpuDiscreteLineage,
    model: CpuDiscreteModelState,
    chooser: CpuDiscreteChooserState,
}

impl CpuDiscreteCheckpoint {
    pub fn schedule(&self) -> &CpuDiscreteSchedule {
        &self.lineage.schedule
    }
}

pub struct CpuDiscreteCheckpointDriver {
    applier: CpuDiscreteProposalApplier,
    lineage: CpuDiscreteLineage,
    cursor: Cursor,
}

impl CpuDiscreteCheckpointDriver {
    pub fn from_artifact(bytes: &[u8], schedule: CpuDiscreteSchedule) -> Result<Self> {
        let applier = CpuDiscreteProposalApplier::from_bytes(bytes)?;
        schedule.validate_for_model(&applier.scorer.model)?;
        let lineage = CpuDiscreteLineage {
            schema: CPU_DISCRETE_LINEAGE_SCHEMA.into(),
            source_artifact_sha256: applier.source_artifact_sha256.clone(),
            source_epoch: applier.epoch,
            source_identity: applier.source_identity.clone(),
            schedule,
        };
        let cursor = lineage.schedule.initial_cursor();
        lineage
            .schedule
            .validate_cursor(&cursor, &applier.scorer.model)?;
        Ok(Self {
            applier,
            lineage,
            cursor,
        })
    }

    pub fn applier(&self) -> &CpuDiscreteProposalApplier {
        &self.applier
    }

    pub fn applier_mut(&mut self) -> &mut CpuDiscreteProposalApplier {
        &mut self.applier
    }

    pub fn score(
        &self,
        kind: CpuDiscreteProposalKind,
        index: usize,
        samples: &[CpuDiscreteProposalSample],
    ) -> std::result::Result<CpuDiscreteProposalScoring, CpuDiscreteScoreError> {
        self.applier
            .scorer
            .score(kind, index, samples, self.applier.epoch)
    }

    pub fn score_block(
        &self,
        kind: CpuDiscreteProposalKind,
        indices: &[usize],
        samples: &[CpuDiscreteProposalSample],
    ) -> std::result::Result<CpuDiscreteBlockScoring, CpuDiscreteScoreError> {
        self.applier
            .scorer
            .score_block(kind, indices, samples, self.applier.epoch)
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn schedule(&self) -> &CpuDiscreteSchedule {
        &self.lineage.schedule
    }

    pub fn advance_after_batch(&mut self, committed: bool, moves: u64) -> Result<()> {
        self.lineage
            .schedule
            .validate_cursor(&self.cursor, &self.applier.scorer.model)?;
        let next = self.lineage.schedule.advance(
            &self.cursor,
            &self.applier.scorer.model,
            committed,
            moves,
        )?;
        let expected_next_epoch = self
            .lineage
            .source_epoch
            .checked_add(next.accepted_updates)
            .ok_or_else(|| invalid("CPU model epoch overflows"))?;
        if self.applier.epoch != expected_next_epoch {
            return Err(invalid(
                "CPU model epoch did not advance with the committed batch",
            ));
        }
        self.cursor = next;
        Ok(())
    }

    pub fn checkpoint(&self) -> Result<CpuDiscreteCheckpoint> {
        self.lineage
            .schedule
            .validate_cursor(&self.cursor, &self.applier.scorer.model)?;
        let expected_epoch = self
            .lineage
            .source_epoch
            .checked_add(self.cursor.accepted_updates)
            .ok_or_else(|| invalid("CPU source epoch overflows"))?;
        if self.applier.epoch != expected_epoch {
            return Err(invalid(
                "CPU model epoch differs from source epoch plus accepted updates",
            ));
        }
        let model = CpuDiscreteModelState::from_scorer(&self.applier.scorer);
        self.lineage.source_identity.validate_model(&model)?;
        let chooser = CpuDiscreteChooserState::from_chooser(&self.applier.scorer.chooser);
        let lineage_sha256 = canonical_json_sha256(&self.lineage)?;
        Ok(CpuDiscreteCheckpoint {
            schema: CPU_DISCRETE_CHECKPOINT_SCHEMA.into(),
            substrate: CPU_SUBSTRATE.into(),
            qualification: CPU_QUALIFICATION.into(),
            source_artifact_sha256: self.lineage.source_artifact_sha256.clone(),
            lineage_sha256,
            cursor: self.cursor,
            model_epoch: self.applier.epoch,
            model_sha256: canonical_json_sha256(&model)?,
            chooser_sha256: canonical_json_sha256(&chooser)?,
            lineage: self.lineage.clone(),
            model,
            chooser,
        })
    }

    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.checkpoint()?).map_err(|error| invalid(error.to_string()))
    }

    pub fn resume(bytes: &[u8], expected_source_artifact_sha256: &str) -> Result<Self> {
        Self::resume_inner(bytes, expected_source_artifact_sha256, None)
    }

    /// Resume a checkpoint with its authenticated source artifact attached so
    /// the driver can emit an updated four-table artifact.
    pub fn resume_with_artifact(
        bytes: &[u8],
        expected_source_artifact_sha256: &str,
        source_artifact: &[u8],
    ) -> Result<Self> {
        let actual = uor_r4_discrete::corpus::sha256(source_artifact);
        if actual != expected_source_artifact_sha256 {
            return Err(invalid("CPU resume source artifact sha256 differs"));
        }
        Self::resume_inner(
            bytes,
            expected_source_artifact_sha256,
            Some(source_artifact),
        )
    }

    fn resume_inner(
        bytes: &[u8],
        expected_source_artifact_sha256: &str,
        source_artifact: Option<&[u8]>,
    ) -> Result<Self> {
        let checkpoint: CpuDiscreteCheckpoint =
            serde_json::from_slice(bytes).map_err(|error| invalid(error.to_string()))?;
        if checkpoint.schema != CPU_DISCRETE_CHECKPOINT_SCHEMA
            || checkpoint.substrate != CPU_SUBSTRATE
            || checkpoint.qualification != CPU_QUALIFICATION
            || checkpoint.lineage.schema != CPU_DISCRETE_LINEAGE_SCHEMA
        {
            return Err(invalid("CPU checkpoint schema or substrate differs"));
        }
        for hash in [
            expected_source_artifact_sha256,
            &checkpoint.source_artifact_sha256,
            &checkpoint.lineage.source_artifact_sha256,
            &checkpoint.lineage_sha256,
            &checkpoint.model_sha256,
            &checkpoint.chooser_sha256,
        ] {
            uor_r4_discrete::corpus::validate_sha256(hash)
                .map_err(|error| invalid(error.to_string()))?;
        }
        if checkpoint.source_artifact_sha256 != expected_source_artifact_sha256
            || checkpoint.lineage.source_artifact_sha256 != expected_source_artifact_sha256
        {
            return Err(invalid("CPU checkpoint source artifact differs"));
        }
        if canonical_json_sha256(&checkpoint.lineage)? != checkpoint.lineage_sha256
            || canonical_json_sha256(&checkpoint.model)? != checkpoint.model_sha256
            || canonical_json_sha256(&checkpoint.chooser)? != checkpoint.chooser_sha256
        {
            return Err(invalid("CPU checkpoint identity hash differs"));
        }
        checkpoint.lineage.schedule.validate()?;
        let source = &checkpoint.lineage.source_identity;
        if source.snapshot_schema != FLOCK_SNAPSHOT_SCHEMA
            || source.operator_kappa != FLOCK_OPERATOR_KAPPA
            || source.composition_kappa != FLOCK_COMPOSITION_KAPPA
        {
            return Err(invalid("CPU checkpoint source geometry differs"));
        }
        for hash in [
            &source.snapshot_sha256,
            &source.kernel_sha256,
            &source.source_sha256,
            &source.geometry_sha256,
        ] {
            uor_r4_discrete::corpus::validate_sha256(hash)
                .map_err(|error| invalid(error.to_string()))?;
        }
        let (model, kept_cells, objective) = checkpoint.model.restore()?;
        source.validate_model(&checkpoint.model)?;
        let chooser = CpuDiscreteProposalChooser::from_model(&model)?;
        if CpuDiscreteChooserState::from_chooser(&chooser) != checkpoint.chooser {
            return Err(invalid(
                "CPU checkpoint chooser state differs from its model",
            ));
        }
        checkpoint
            .lineage
            .schedule
            .validate_cursor(&checkpoint.cursor, &model)?;
        let expected_epoch = checkpoint
            .lineage
            .source_epoch
            .checked_add(checkpoint.cursor.accepted_updates)
            .ok_or_else(|| invalid("CPU checkpoint source epoch overflows"))?;
        if checkpoint.model_epoch != expected_epoch {
            return Err(invalid(
                "CPU checkpoint model epoch differs from source epoch plus accepted updates",
            ));
        }
        let base_artifact = if let Some(source_artifact) = source_artifact {
            let source_header = header(source_artifact)?;
            if source_header["shape"]["epoch"].as_u64() != Some(checkpoint.lineage.source_epoch) {
                return Err(invalid("CPU resume source epoch differs from checkpoint"));
            }
            source_artifact.to_vec()
        } else {
            Vec::new()
        };
        let applier = CpuDiscreteProposalApplier {
            scorer: CpuDiscreteProposalScorer {
                chooser,
                model,
                kept_cells,
                objective,
            },
            epoch: checkpoint.model_epoch,
            source_artifact_sha256: checkpoint.source_artifact_sha256,
            source_identity: checkpoint.lineage.source_identity.clone(),
            base_artifact,
        };
        Ok(Self {
            applier,
            lineage: checkpoint.lineage,
            cursor: checkpoint.cursor,
        })
    }

    pub fn artifact_bytes(&self) -> Result<Vec<u8>> {
        self.applier.artifact_bytes()
    }

    pub fn next_block(&self) -> Result<CpuDiscreteBlock> {
        let model = &self.applier.scorer.model;
        self.lineage.schedule.validate_cursor(&self.cursor, model)?;
        let length = self.lineage.schedule.block_length(&self.cursor, model)?;
        let length =
            usize::try_from(length).map_err(|_| invalid("CPU block length exceeds usize"))?;
        let (kind, start, size) = if self
            .lineage
            .schedule
            .is_read_batch(self.cursor.completed_batches, model)?
        {
            let (_, o_read) = family_cursors(
                self.cursor.completed_batches,
                self.lineage.schedule.coordinate_block,
                self.lineage.schedule.policy,
                self.lineage.schedule.dimensions(model).0,
                self.lineage.schedule.dimensions(model).1,
            )
            .map_err(|error| invalid(error.to_string()))?;
            let kind = match model {
                CpuDiscreteProposalModel::Route(_) => CpuDiscreteProposalKind::Route,
                CpuDiscreteProposalModel::Active(_) => CpuDiscreteProposalKind::Gate,
            };
            (kind, o_read, self.lineage.schedule.dimensions(model).1)
        } else {
            let (o_vec, _) = family_cursors(
                self.cursor.completed_batches,
                self.lineage.schedule.coordinate_block,
                self.lineage.schedule.policy,
                self.lineage.schedule.dimensions(model).0,
                self.lineage.schedule.dimensions(model).1,
            )
            .map_err(|error| invalid(error.to_string()))?;
            (
                CpuDiscreteProposalKind::Table,
                o_vec,
                self.lineage.schedule.dimensions(model).0,
            )
        };
        let start =
            usize::try_from(start % size).map_err(|_| invalid("CPU block start exceeds usize"))?;
        Ok(CpuDiscreteBlock {
            kind,
            indices: (start..start + length).collect(),
        })
    }
}

fn canonical_json_sha256<T: serde::Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|error| invalid(error.to_string()))?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn apply_error(status: u32, message: impl Into<String>) -> CpuDiscreteApplyError {
    CpuDiscreteApplyError {
        status,
        message: message.into(),
    }
}

fn score_error(status: u32, message: impl Into<String>) -> CpuDiscreteScoreError {
    CpuDiscreteScoreError {
        status,
        message: message.into(),
    }
}

fn writer_score_error(error: CpuTrainError) -> CpuDiscreteScoreError {
    let message = error.to_string();
    let status = if message.contains("overflow") { 12 } else { 8 };
    score_error(status, message)
}

fn accumulate_lane(
    total: u64,
    cost: Option<u64>,
) -> std::result::Result<Option<u64>, CpuDiscreteScoreError> {
    let Some(cost) = cost else {
        return Ok(None);
    };
    total
        .checked_add(cost)
        .map(Some)
        .ok_or_else(|| score_error(12, "proposal lane loss sum overflows"))
}

impl CpuDiscreteProposalModel {
    fn blocks(&self) -> usize {
        match self {
            Self::Route(route) => route.blocks,
            Self::Active(active) => active.blocks,
        }
    }

    fn token_blocks(&self) -> &[Vec<[i32; 8]>] {
        match self {
            Self::Route(route) => &route.token_blocks,
            Self::Active(active) => &active.token_blocks,
        }
    }

    fn token_blocks_mut(&mut self) -> &mut Vec<Vec<[i32; 8]>> {
        match self {
            Self::Route(route) => &mut route.token_blocks,
            Self::Active(active) => &mut active.token_blocks,
        }
    }
}

fn sibling_gate(current: i32, candidate: usize) -> Result<i32> {
    const ORDER: [i32; 3] = [0, -1, 1];
    if candidate == 0 {
        return Ok(current);
    }
    ORDER
        .into_iter()
        .filter(|gate| *gate != current)
        .nth(candidate - 1)
        .ok_or_else(|| invalid("Gate proposal candidate outside active inventory"))
}

fn move_root(
    point: &[i32; 8],
    action: u8,
    composition: &Cl06FiniteCompositionTable,
) -> Result<[i32; 8]> {
    let root: [i8; 8] = std::array::from_fn(|dimension| (point[dimension] / 2) as i8);
    let moved = composition
        .state(action)
        .ok_or_else(|| invalid("proposal move action absent"))?
        .apply_root_key(root)
        .map_err(|error| invalid(error.to_string()))?;
    Ok(moved.map(|value| i32::from(value) * 2))
}

fn writer_loss(
    query: &[i32],
    token_blocks: &[Vec<[i32; 8]>],
    target: u32,
    kept_cells: usize,
    objective: CpuDiscreteObjective,
) -> Result<CpuWriterLoss> {
    let blocks = token_blocks
        .first()
        .map(Vec::len)
        .ok_or_else(|| invalid("proposal writer codebook is empty"))?;
    let width = blocks
        .checked_mul(8)
        .ok_or_else(|| invalid("proposal writer width overflows"))?;
    if query.len() != width || target as usize >= token_blocks.len() {
        return Err(invalid("proposal writer row shape differs"));
    }
    let mut token_cells = Vec::with_capacity(token_blocks.len());
    let mut occupied = [false; 256];
    for token in token_blocks {
        if token.len() != blocks {
            return Err(invalid("proposal writer block count differs"));
        }
        let first = identify_orbit_root(&token[0])
            .ok_or_else(|| invalid("proposal writer first block off orbit"))?;
        let last = identify_orbit_root(token.last().unwrap())
            .ok_or_else(|| invalid("proposal writer last block off orbit"))?;
        let cell = usize::from(first) * 16 + usize::from(last);
        occupied[cell] = true;
        token_cells.push(cell);
    }
    let target_cell = token_cells[target as usize];
    let mut used = [false; 256];
    let mut admitted = Vec::with_capacity(kept_cells);
    for _ in 0..kept_cells {
        let mut best = None;
        for cell in 0..256 {
            if occupied[cell] && !used[cell] {
                let cost = writer_cell_distance(query, blocks, cell);
                if best.is_none_or(|(best_cost, best_cell)| {
                    cost < best_cost || (cost == best_cost && cell < best_cell)
                }) {
                    best = Some((cost, cell));
                }
            }
        }
        let Some((_, cell)) = best else {
            break;
        };
        used[cell] = true;
        admitted.push(cell);
    }
    let mut best_token = None;
    let mut rival_distance = None;
    let mut rival_address = None;
    let mut token_rivals = 0usize;
    for &cell in &admitted {
        let address = writer_cell_distance(query, blocks, cell);
        if cell != target_cell {
            rival_address = Some(rival_address.map_or(address, |value: u32| value.min(address)));
        }
        for (token, &token_cell) in token_cells.iter().enumerate() {
            if token_cell != cell {
                continue;
            }
            let distance = writer_token_distance(query, blocks, &token_blocks[token]);
            if best_token.is_none_or(|(best_distance, best_id)| {
                distance < best_distance || (distance == best_distance && token < best_id)
            }) {
                best_token = Some((distance, token));
            }
            if token != target as usize {
                token_rivals += 1;
                rival_distance =
                    Some(rival_distance.map_or(distance, |value: u32| value.min(distance)));
            }
        }
    }
    if best_token.is_none() {
        return Err(invalid("proposal writer admitted no token"));
    }
    let width_u64 = u64::try_from(width).map_err(|_| invalid("proposal width overflow"))?;
    let target_distance = writer_token_distance(query, blocks, &token_blocks[target as usize]);
    let target_address = writer_cell_distance(query, blocks, target_cell);
    let token_hinge = rival_distance.map_or(0u64, |rival| {
        let threshold = width_u64 + u64::from(target_distance);
        threshold.saturating_sub(u64::from(rival))
    });
    let address_hinge = rival_address.map_or(0u64, |rival| {
        8u64.saturating_add(u64::from(target_address))
            .saturating_sub(u64::from(rival))
    });
    let rank_hinge = if matches!(objective, CpuDiscreteObjective::OccupiedCellRankHinge) {
        let rank = 1
            + (0..256)
                .filter(|&cell| occupied[cell])
                .filter(|&cell| {
                    let distance = writer_cell_distance(query, blocks, cell);
                    distance < target_address || (distance == target_address && cell < target_cell)
                })
                .count();
        u64::try_from(rank.saturating_sub(kept_cells))
            .map_err(|_| invalid("proposal rank overflow"))?
    } else {
        0
    };
    let weighted_token = token_hinge
        .checked_mul(8)
        .ok_or_else(|| invalid("proposal token loss overflows"))?;
    let weighted_address = address_hinge
        .checked_mul(width_u64)
        .ok_or_else(|| invalid("proposal address loss overflows"))?;
    let weighted_rank = rank_hinge
        .checked_mul(width_u64)
        .ok_or_else(|| invalid("proposal rank loss overflows"))?;
    let loss = match objective {
        CpuDiscreteObjective::NearestAddressMargin => weighted_token
            .checked_add(weighted_address)
            .ok_or_else(|| invalid("proposal nearest loss overflows"))?,
        CpuDiscreteObjective::OccupiedCellRankHinge => weighted_token
            .checked_add(weighted_rank)
            .ok_or_else(|| invalid("proposal rank loss overflows"))?,
    };
    Ok(CpuWriterLoss { loss, token_rivals })
}

fn accumulate_joint_loss(
    total: &mut u64,
    loss: CpuWriterLoss,
) -> std::result::Result<bool, CpuDiscreteScoreError> {
    if loss.token_rivals == 0 || loss.loss == u64::MAX {
        return Ok(false);
    }
    *total = total
        .checked_add(loss.loss)
        .ok_or_else(|| score_error(9, "BVG-K joint loss sum overflows"))?;
    Ok(true)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CpuWriterObservation {
    loss: u64,
    top8: [u32; 8],
    target_admitted: bool,
    target_rank: usize,
}

fn writer_observation(
    query: &[i32],
    token_blocks: &[Vec<[i32; 8]>],
    target: u32,
    kept_cells: usize,
    objective: CpuDiscreteObjective,
) -> Result<CpuWriterObservation> {
    let loss = writer_loss(query, token_blocks, target, kept_cells, objective)?;
    let blocks = token_blocks
        .first()
        .map(Vec::len)
        .ok_or_else(|| invalid("proposal writer codebook is empty"))?;
    if query.len() != blocks * 8 || target as usize >= token_blocks.len() {
        return Err(invalid("proposal writer row shape differs"));
    }
    let mut token_cells = Vec::with_capacity(token_blocks.len());
    let mut occupied = [false; 256];
    for token in token_blocks {
        let first = identify_orbit_root(&token[0])
            .ok_or_else(|| invalid("proposal writer first block off orbit"))?;
        let last = identify_orbit_root(token.last().unwrap())
            .ok_or_else(|| invalid("proposal writer last block off orbit"))?;
        let cell = usize::from(first) * 16 + usize::from(last);
        occupied[cell] = true;
        token_cells.push(cell);
    }
    let mut used = [false; 256];
    let mut admitted = Vec::with_capacity(kept_cells);
    for _ in 0..kept_cells {
        let mut best = None;
        for cell in 0..256 {
            if occupied[cell] && !used[cell] {
                let cost = writer_cell_distance(query, blocks, cell);
                if best.is_none_or(|(best_cost, best_cell)| {
                    cost < best_cost || (cost == best_cost && cell < best_cell)
                }) {
                    best = Some((cost, cell));
                }
            }
        }
        let Some((_, cell)) = best else {
            break;
        };
        used[cell] = true;
        admitted.push(cell);
    }
    let target_admitted = admitted.contains(&token_cells[target as usize]);
    let mut ranked = Vec::new();
    for (token, &cell) in token_cells.iter().enumerate() {
        if admitted.contains(&cell) {
            ranked.push((
                writer_token_distance(query, blocks, &token_blocks[token]),
                token as u32,
            ));
        }
    }
    ranked.sort_unstable();
    let mut top8 = [u32::MAX; 8];
    for (slot, &(_, token)) in ranked.iter().take(8).enumerate() {
        top8[slot] = token;
    }
    let target_rank = ranked
        .iter()
        .position(|&(_, token)| token == target)
        .map_or(ranked.len(), |rank| rank + 1);
    Ok(CpuWriterObservation {
        loss: loss.loss,
        top8,
        target_admitted,
        target_rank,
    })
}

fn admitted_cells(query: &[i32], token_blocks: &[Vec<[i32; 8]>], kept_cells: usize) -> Vec<u32> {
    let blocks = token_blocks.first().map_or(0, Vec::len);
    let mut occupied = [false; 256];
    for token in token_blocks {
        let first = identify_orbit_root(&token[0]).expect("model token blocks were validated");
        let last =
            identify_orbit_root(token.last().unwrap()).expect("model token blocks were validated");
        occupied[usize::from(first) * 16 + usize::from(last)] = true;
    }
    let mut candidates = (0..256)
        .filter(|&cell| occupied[cell])
        .map(|cell| (writer_cell_distance(query, blocks, cell), cell))
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates
        .into_iter()
        .take(kept_cells)
        .map(|(_, cell)| cell as u32)
        .collect()
}

fn flattened_token_table(scorer: &CpuDiscreteProposalScorer) -> Vec<i32> {
    scorer
        .model
        .token_blocks()
        .iter()
        .flat_map(|token| token.iter().flat_map(|block| block.iter().copied()))
        .collect()
}

fn token_cells(scorer: &CpuDiscreteProposalScorer) -> Vec<u32> {
    scorer
        .model
        .token_blocks()
        .iter()
        .map(|token| {
            let first = identify_orbit_root(&token[0]).expect("model token blocks were validated");
            let last = identify_orbit_root(token.last().unwrap())
                .expect("model token blocks were validated");
            u32::from(first) * 16 + u32::from(last)
        })
        .collect()
}

fn flattened_read_state(scorer: &CpuDiscreteProposalScorer) -> Result<(Vec<u32>, Vec<i32>)> {
    match &scorer.model {
        CpuDiscreteProposalModel::Route(route) => Ok((
            route.routes.iter().flatten().copied().collect(),
            vec![0; route.blocks * route.stages],
        )),
        CpuDiscreteProposalModel::Active(active) => {
            let mut gates = active
                .level_gates
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>();
            if let ActiveLeafMode::Relation { relation_gates, .. } = &active.mode {
                gates.extend(relation_gates.iter().flatten().copied());
            }
            Ok((active.routes.clone(), gates))
        }
    }
}

fn append_i32(output: &mut Vec<u8>, values: &[i32]) {
    output.extend(values.iter().flat_map(|value| value.to_le_bytes()));
}

fn append_u32(output: &mut Vec<u8>, values: &[u32]) {
    output.extend(values.iter().flat_map(|value| value.to_le_bytes()));
}

fn writer_cell_distance(query: &[i32], blocks: usize, cell: usize) -> u32 {
    let first: &[i32; 8] = query[..8]
        .try_into()
        .expect("writer query width was checked");
    let last: &[i32; 8] = query[(blocks - 1) * 8..blocks * 8]
        .try_into()
        .expect("writer query width was checked");
    root_cost(first, cell / 16).0 + root_cost(last, cell % 16).0
}

fn writer_token_distance(query: &[i32], blocks: usize, token: &[[i32; 8]]) -> u32 {
    (0..blocks)
        .flat_map(|block| query[block * 8..(block + 1) * 8].iter().zip(token[block]))
        .map(|(&left, right)| left.abs_diff(right))
        .fold(0u32, u32::saturating_add)
}

fn root_id(block: &[i32; 8]) -> Option<usize> {
    (0..16u8)
        .find(|&root| orbit_root_coords(root).ok().as_ref() == Some(block))
        .map(usize::from)
}

fn decode_token_blocks(
    table: &[i32],
    cells: &[u32],
    vocab: usize,
    blocks: usize,
    mode: &str,
) -> Result<Vec<Vec<[i32; 8]>>> {
    let mut token_blocks = Vec::with_capacity(vocab);
    for token in 0..vocab {
        let mut values = Vec::with_capacity(blocks);
        for block in 0..blocks {
            let start = (token * blocks + block) * 8;
            let coords: [i32; 8] = table[start..start + 8]
                .try_into()
                .map_err(|_| invalid(format!("{mode} codebook width")))?;
            if root_id(&coords).is_none() {
                return Err(invalid(format!("{mode} codebook block outside orbit")));
            }
            values.push(coords);
        }
        token_blocks.push(values);
    }
    for (token, (blocks, &cell)) in token_blocks.iter().zip(cells).enumerate() {
        let first = root_id(&blocks[0]).expect("decoded codebook block is an orbit root") as u32;
        let last = root_id(blocks.last().unwrap()).expect("decoded codebook block is an orbit root")
            as u32;
        if cell != first * 16 + last {
            return Err(invalid(format!("{mode} cell for token {token} differs")));
        }
    }
    Ok(token_blocks)
}

fn expected_stages(context: usize) -> Option<usize> {
    if !(1..=243).contains(&context) {
        return None;
    }
    let mut stages = 1usize;
    let mut span = 3usize;
    while span <= context {
        stages += 1;
        span = span.saturating_mul(3);
    }
    Some(stages)
}

fn root_cost(block: &[i32; 8], root: usize) -> (u32, u32) {
    let target = orbit_root_coords(root as u8).expect("root index is 0..16");
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for (left, right) in block.iter().zip(target) {
        distance += left.abs_diff(right);
        hamming += u32::from(*left != right);
    }
    (distance, hamming)
}

fn snap_root(block: &[i32; 8], multiplicity: usize) -> Result<usize> {
    let mut best = 0usize;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for root in 0..16 {
        let target = orbit_root_coords(root as u8)
            .map_err(|error| invalid(format!("Flock snap root: {error}")))?;
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for (value, target) in block.iter().zip(target) {
            let scaled = target
                .checked_mul(i32::try_from(multiplicity).unwrap_or(i32::MAX))
                .ok_or_else(|| invalid("Flock snap multiplicity overflows"))?;
            distance = distance.saturating_add(value.abs_diff(scaled));
            hamming += u32::from(*value != scaled);
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = root;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    Ok(best)
}

fn build_row_plan(length: usize) -> Result<RowPlan> {
    if length == 0 || length > 243 {
        return Err(invalid("Route plan length outside context"));
    }
    let mut nodes = Vec::new();
    let mut frontier = Vec::new();
    for atom in 0..length {
        frontier.push(nodes.len());
        nodes.push(BinderNode {
            children: None,
            atom: Some(atom),
            level: 0,
            first: atom,
            last: atom,
        });
        loop {
            let n = frontier.len();
            if n < 3 {
                break;
            }
            let children = [frontier[n - 3], frontier[n - 2], frontier[n - 1]];
            let level = nodes[children[0]].level;
            if children.iter().any(|&child| nodes[child].level != level) {
                break;
            }
            let parent = nodes.len();
            let first = nodes[children[0]].first;
            let last = nodes[children[2]].last;
            nodes.push(BinderNode {
                children: Some(children),
                atom: None,
                level: level + 1,
                first,
                last,
            });
            frontier.truncate(n - 3);
            frontier.push(parent);
        }
    }
    Ok(RowPlan { nodes, frontier })
}

fn header(bytes: &[u8]) -> Result<serde_json::Value> {
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err(invalid("Route artifact magic differs"));
    }
    let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let end = 16usize
        .checked_add(usize::try_from(len).map_err(|_| invalid("Route header length"))?)
        .ok_or_else(|| invalid("Route header end overflows"))?;
    serde_json::from_slice(
        bytes
            .get(16..end)
            .ok_or_else(|| invalid("Route header truncated"))?,
    )
    .map_err(|error| invalid(format!("Route header: {error}")))
}

fn section<'a>(
    bytes: &'a [u8],
    header: &serde_json::Value,
    name: &str,
    expected_len: usize,
    expected_kind: &str,
) -> Result<&'a [u8]> {
    let mut matches = header["tables"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entry| entry["name"].as_str() == Some(name));
    let entry = matches
        .next()
        .ok_or_else(|| invalid(format!("missing {name}")))?;
    if matches.next().is_some() {
        return Err(invalid(format!("duplicate {name}")));
    }
    if entry["len"].as_u64() != Some(expected_len as u64) {
        return Err(invalid(format!("{name} length differs")));
    }
    if entry["kind"].as_str() != Some(expected_kind) {
        return Err(invalid(format!("{name} kind differs")));
    }
    let width = if expected_kind == "i16" { 2 } else { 4 };
    if entry["span"]["bytes"].as_u64() != Some((expected_len * width) as u64) {
        return Err(invalid(format!("{name} span byte length differs")));
    }
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let offset = usize::try_from(entry["span"]["offset"].as_u64().unwrap_or(u64::MAX))
        .map_err(|_| invalid("section offset"))?;
    if offset % 64 != 0 {
        return Err(invalid(format!("{name} is not 64-byte aligned")));
    }
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Route header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Route header end overflows"))?;
    let data_start = header_end
        .checked_add(63)
        .ok_or_else(|| invalid("data start overflow"))?
        / 64
        * 64;
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| invalid("Route section start overflows"))?;
    let len = usize::try_from(entry["span"]["bytes"].as_u64().unwrap_or(0))
        .map_err(|_| invalid("section bytes"))?;
    let end = start
        .checked_add(len)
        .ok_or_else(|| invalid(format!("{name} end overflows")))?;
    bytes
        .get(start..end)
        .ok_or_else(|| invalid(format!("{name} outside artifact")))
}

fn validate_layout(bytes: &[u8], header: &serde_json::Value) -> Result<()> {
    let mut spans: Vec<(usize, usize, &str)> = header["tables"]
        .as_array()
        .ok_or_else(|| invalid("Route tables differ"))?
        .iter()
        .map(|entry| {
            let offset = usize::try_from(entry["span"]["offset"].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid("layout offset"))?;
            let len = usize::try_from(entry["span"]["bytes"].as_u64().unwrap_or(u64::MAX))
                .map_err(|_| invalid("layout bytes"))?;
            let end = offset
                .checked_add(len)
                .ok_or_else(|| invalid("layout end overflow"))?;
            Ok((offset, end, entry["name"].as_str().unwrap_or("")))
        })
        .collect::<Result<_>>()?;
    spans.sort_unstable_by_key(|span| span.0);
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Route header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Route header end overflows"))?;
    let data_start = header_end
        .checked_add(63)
        .ok_or_else(|| invalid("data start overflow"))?
        / 64
        * 64;
    let data = bytes
        .get(data_start..)
        .ok_or_else(|| invalid("Route data start outside artifact"))?;
    let mut cursor = 0usize;
    for (start, end, name) in spans {
        if start < cursor || end > data.len() || data[cursor..start].iter().any(|byte| *byte != 0) {
            return Err(invalid(format!(
                "{name} layout overlaps or has nonzero padding"
            )));
        }
        cursor = end;
    }
    if data[cursor..].iter().any(|byte| *byte != 0) {
        return Err(invalid("Route artifact has nonzero trailing padding"));
    }
    Ok(())
}

fn validate_header_padding(bytes: &[u8]) -> Result<()> {
    let header_len = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| invalid("Flock header length differs"))?,
    );
    let unaligned = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("Flock header length exceeds usize"))?,
        )
        .ok_or_else(|| invalid("Flock header end overflows"))?;
    let data_start = unaligned
        .checked_add(63)
        .ok_or_else(|| invalid("Flock data start overflow"))?
        / 64
        * 64;
    let padding = bytes
        .get(unaligned..data_start)
        .ok_or_else(|| invalid("Flock header padding is truncated"))?;
    if padding.iter().any(|byte| *byte != 0) {
        return Err(invalid("Flock header padding is nonzero"));
    }
    Ok(())
}

fn section_i32(
    bytes: &[u8],
    header: &serde_json::Value,
    name: &str,
    len: usize,
    kind: &str,
) -> Result<Vec<i32>> {
    Ok(section(bytes, header, name, len, kind)?
        .chunks_exact(4)
        .map(|v| i32::from_le_bytes(v.try_into().unwrap()))
        .collect())
}

fn section_u32(
    bytes: &[u8],
    header: &serde_json::Value,
    name: &str,
    len: usize,
    kind: &str,
) -> Result<Vec<u32>> {
    Ok(section(bytes, header, name, len, kind)?
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VOCABULARY;
    use serde_json::{json, Value};
    use uor_r4_discrete::metadata::BinderPlan;
    use uor_r4_discrete::snapshot::Snapshot;
    use uor_r4_discrete::trunk::{
        build_trunk_norm_frames, build_trunk_step_table, export_trunk_d11, TrunkTables,
        TrunkTrainRow, KEPT_CELLS_DEFAULT, TRUNK_BLOCKS,
    };
    use uor_r4_discrete::{
        ObjectivePolicy, ReadInitialization, ReadSpec, Spec, SCHEMA as DISCRETE_SNAPSHOT_SCHEMA,
    };
    use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

    fn synthetic_trunk_artifact() -> Vec<u8> {
        let mut table = Vec::with_capacity(VOCABULARY * READ_WIDTH);
        for token in 0..VOCABULARY {
            for block in 0..TRUNK_BLOCKS {
                let root = ((token >> (block * 4)) & 15) as u8;
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
        }
        let spec = Spec {
            vocabulary: VOCABULARY as u32,
            blocks: TRUNK_BLOCKS as u32,
            context: 6,
            kept_cells: KEPT_CELLS_DEFAULT as u32,
        };
        let stages = BinderPlan::new(spec.context as usize).unwrap().stages() as u32;
        let read_spec = ReadSpec::Relation { kept: 8, sweeps: 1 };
        let gate_len = read_spec.gate_count(spec.blocks, stages).unwrap() as usize;
        let snapshot = Snapshot {
            schema: DISCRETE_SNAPSHOT_SCHEMA.to_owned(),
            spec,
            read_spec,
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            read_initialization: ReadInitialization::ActiveLeafV1,
            normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
            position: None,
            address_metric: Default::default(),
            seed: 90,
            epoch: 420,
            table,
            routes: vec![u32::from(BFS[0]); (spec.blocks * stages) as usize],
            gates: vec![0; gate_len],
            operator_kappa: FLOCK_OPERATOR_KAPPA.to_owned(),
            composition_kappa: FLOCK_COMPOSITION_KAPPA.to_owned(),
            kernel_sha256: "11".repeat(32),
            source_sha256: "22".repeat(32),
            geometry_sha256: "33".repeat(32),
        };
        let steps = build_trunk_step_table(spec.blocks, 0).unwrap();
        let norms = build_trunk_norm_frames(spec.blocks, 0).unwrap();
        let rows = vec![
            TrunkTrainRow {
                read_code: 1,
                rec_code: 2,
                rec_shuf_code: 2,
                rec_unit_code: 2,
                query_token: 1,
                target_token: 2,
            },
            TrunkTrainRow {
                read_code: 3,
                rec_code: 4,
                rec_shuf_code: 4,
                rec_unit_code: 4,
                query_token: 2,
                target_token: 3,
            },
        ];
        export_trunk_d11(
            &snapshot,
            &steps,
            &norms,
            &TrunkTables::build(&rows).unwrap(),
        )
        .unwrap()
    }

    fn table_payload(artifact: &[u8], table_index: usize) -> (usize, usize) {
        let header = header(artifact).unwrap();
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16 + header_len).div_ceil(64) * 64;
        let table = &header["tables"][table_index];
        (
            data_start + table["span"]["offset"].as_u64().unwrap() as usize,
            table["span"]["bytes"].as_u64().unwrap() as usize,
        )
    }

    #[test]
    fn cpu_trunk_matches_integer_serving_and_training_still_refuses_it() {
        let artifact = synthetic_trunk_artifact();
        let cpu = CpuDiscreteTrunkReader::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [
            &[1u32][..],
            &[1, 1][..],
            &[1, 2, 1][..],
            &[1, 2, 1, 3, 1][..],
        ] {
            let decision = cpu.decide(prefix).unwrap();
            let served = oracle.evaluate_prefix(prefix).unwrap();
            let mut top8 = [u32::MAX; 8];
            for (slot, &token) in served.top8.iter().take(8).enumerate() {
                top8[slot] = token;
            }
            assert_eq!(decision.query, served.read_vector.as_slice());
            assert_eq!(decision.admitted_cells, served.admitted_cells);
            assert_eq!(decision.top8, top8);
            assert_eq!(decision.token, served.prediction);
        }
        assert!(CpuDiscreteRelation::from_bytes(&artifact).is_err());
        assert!(CpuDiscreteProposalScorer::from_bytes(&artifact).is_err());
    }

    #[test]
    fn cpu_trunk_derives_training_rows_and_round_trips_all_count_tables() {
        let artifact = synthetic_trunk_artifact();
        let cpu = CpuDiscreteTrunkReader::from_bytes(&artifact).unwrap();
        let prefix = [1u32, 2, 1];
        let decision = cpu.decide(&prefix).unwrap();
        let row = cpu.training_row(&prefix, 2, 17).unwrap();
        assert_eq!(row.read_code, decision.read_code);
        assert_eq!(row.rec_code, decision.recurrence_code);
        assert_eq!(row.query_token, 1);
        assert_eq!(row.target_token, 2);
        assert!(cpu.training_row(&prefix, VOCABULARY as u32, 17).is_err());

        let (_, _, _, before) = import_trunk_d11(&artifact).unwrap();
        let updated = cpu.add_training_rows(&[row]).unwrap();
        let (snapshot, steps, norms, after) = import_trunk_d11(&updated).unwrap();
        let (source_snapshot, source_steps, source_norms, _) = import_trunk_d11(&artifact).unwrap();
        assert_eq!(
            serde_json::to_vec(&snapshot).unwrap(),
            serde_json::to_vec(&source_snapshot).unwrap()
        );
        assert_eq!(steps, source_steps);
        assert_eq!(norms, source_norms);
        assert_eq!(after.unigram_counts[2], before.unigram_counts[2] + 1);
        for (before, after) in [
            (&before.read_table, &after.read_table),
            (&before.rec_table, &after.rec_table),
            (&before.read_rec_table, &after.read_rec_table),
            (&before.rec_shuf_table, &after.rec_shuf_table),
            (&before.read_rec_shuf_table, &after.read_rec_shuf_table),
            (&before.rec_unit_table, &after.rec_unit_table),
            (&before.read_rec_unit_table, &after.read_rec_unit_table),
            (&before.query_table, &after.query_table),
            (&before.query_read_table, &after.query_read_table),
            (&before.query_rec_table, &after.query_rec_table),
            (&before.query_rec_shuf_table, &after.query_rec_shuf_table),
            (&before.query_rec_unit_table, &after.query_rec_unit_table),
        ] {
            assert_eq!(after.total(), before.total() + 1);
        }
        assert_eq!(cpu.add_training_rows(&[row]).unwrap(), updated);
    }

    #[test]
    fn cpu_trunk_parallel_rows_and_decisions_preserve_serial_results() {
        use rayon::prelude::*;

        let artifact = synthetic_trunk_artifact();
        let reader = CpuDiscreteTrunkReader::from_bytes(&artifact).unwrap();
        let served = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        let prefixes = vec![
            vec![1u32],
            vec![1, 2],
            vec![1, 2, 1],
            vec![1, 2, 1, 3],
            vec![2, 1, 2],
            vec![3, 1, 2],
        ];
        let targets = [2u32, 1, 2, 3, 1, 2];
        let expected_rows = prefixes
            .iter()
            .zip(targets)
            .enumerate()
            .map(|(index, (prefix, target))| {
                reader
                    .training_row(prefix, target, 17 + index as u64)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let expected_artifact = reader
            .clone()
            .add_training_rows_consuming(&expected_rows)
            .unwrap();
        let expected_decisions = prefixes
            .iter()
            .map(|prefix| reader.decide(prefix).unwrap())
            .collect::<Vec<_>>();

        for workers in [1usize, 8, 16, 32] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap();
            pool.install(|| {
                let rows = prefixes
                    .par_iter()
                    .zip(targets.par_iter())
                    .enumerate()
                    .map(|(index, (prefix, target))| {
                        reader
                            .training_row(prefix, *target, 17 + index as u64)
                            .unwrap()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(rows, expected_rows, "{workers} workers");
                assert_eq!(
                    reader.clone().add_training_rows_consuming(&rows).unwrap(),
                    expected_artifact,
                    "{workers} workers"
                );

                let decisions = prefixes
                    .par_iter()
                    .map(|prefix| reader.decide(prefix).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(decisions, expected_decisions, "{workers} workers");
                let parity = prefixes
                    .par_iter()
                    .map(|prefix| {
                        let cpu = reader.decide(prefix).unwrap();
                        let served = served.evaluate_prefix(prefix).unwrap();
                        let mut top8 = [u32::MAX; 8];
                        for (slot, &token) in served.top8.iter().take(8).enumerate() {
                            top8[slot] = token;
                        }
                        assert_eq!(cpu.query, served.read_vector.as_slice());
                        assert_eq!(cpu.admitted_cells, served.admitted_cells);
                        assert_eq!(cpu.top8, top8);
                        assert_eq!(cpu.token, served.prediction);
                    })
                    .collect::<Vec<_>>();
                assert_eq!(parity.len(), prefixes.len(), "{workers} workers");
            });
        }
    }

    #[test]
    fn cpu_trunk_count_training_keeps_the_smaller_unigram_on_a_tie() {
        let mut tables = TrunkTables::build(&[TrunkTrainRow {
            read_code: 3,
            rec_code: 4,
            rec_shuf_code: 5,
            rec_unit_code: 6,
            query_token: 7,
            target_token: 8,
        }])
        .unwrap();
        assert_eq!(tables.unigram_best_global, 8);
        add_trunk_count_row(
            &mut tables,
            &TrunkTrainRow {
                read_code: 9,
                rec_code: 10,
                rec_shuf_code: 11,
                rec_unit_code: 12,
                query_token: 13,
                target_token: 7,
            },
        )
        .unwrap();
        assert_eq!(tables.unigram_counts[7], 1);
        assert_eq!(tables.unigram_counts[8], 1);
        assert_eq!(tables.unigram_best_global, 7);
    }

    #[test]
    fn cpu_trunk_refuses_shape_table_and_recurrence_corruption() {
        let artifact = synthetic_trunk_artifact();
        for cut in [artifact.len() - 1, artifact.len() / 2, 15] {
            assert!(CpuDiscreteTrunkReader::from_bytes(&artifact[..cut]).is_err());
        }

        let mut bad_step = artifact.clone();
        let (step_start, _) = table_payload(&bad_step, 4);
        bad_step[step_start..step_start + 4].copy_from_slice(&16u32.to_le_bytes());
        assert!(CpuDiscreteTrunkReader::from_bytes(&bad_step).is_err());

        let mut bad_query = artifact;
        let (query_start, _) = table_payload(&bad_query, 14);
        bad_query[query_start + 4..query_start + 8]
            .copy_from_slice(&((VOCABULARY + 1) as u32).to_le_bytes());
        assert!(CpuDiscreteTrunkReader::from_bytes(&bad_query).is_err());
    }

    fn identity_route() -> CpuDiscreteRoute {
        let root = orbit_root_coords(0).unwrap();
        CpuDiscreteRoute {
            context: 3,
            blocks: 2,
            stages: 2,
            token_blocks: vec![vec![root; 2]; 3],
            routes: vec![vec![0; 2]; 2],
            composition: cl06_finite_composition_table().unwrap(),
        }
    }

    #[test]
    fn identity_route_is_deterministic_across_context_lengths() {
        let route = identity_route();
        let expected = orbit_root_coords(0).unwrap();
        let one = route.read(&[0]).unwrap();
        let three = route.read(&[0, 1, 2]).unwrap();
        assert_eq!(
            one,
            expected.into_iter().chain(expected).collect::<Vec<_>>()
        );
        assert_eq!(one, three);
        assert_eq!(
            route.read(&[]).unwrap_err().to_string(),
            "Route prefix outside context"
        );
        assert!(route.read(&[3]).is_err());
    }

    #[test]
    fn malformed_artifacts_are_refused() {
        assert!(CpuDiscreteRoute::from_bytes(b"bad").is_err());
        assert!(root_id(&[1; 8]).is_none());
    }

    #[test]
    fn serialized_nonzero_route_matches_integer_discrete_model() {
        let artifact = serialized_route();
        let cpu = CpuDiscreteRoute::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [&[0u32][..], &[0, 1][..], &[0, 1, 2][..], &[1, 2, 0][..]] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
    }

    #[test]
    fn cpu_discrete_writeback_round_trips_serving_and_advances_epoch() {
        let artifact = serialized_route();
        let samples = vec![CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2],
            target: 0,
        }];
        let mut applier = CpuDiscreteProposalApplier::from_bytes(&artifact).unwrap();
        let source_epoch = applier.epoch;
        let receipt = applier
            .apply(
                CpuDiscreteProposalKind::Table,
                0,
                CpuDiscreteProposal {
                    lane: 1,
                    current_loss: 10,
                    selected_loss: 9,
                    epoch: source_epoch,
                    sensitive: true,
                },
            )
            .unwrap();
        assert!(receipt.applied);
        let rewritten = applier.artifact_bytes().unwrap();
        let scorer = CpuDiscreteProposalScorer::from_bytes(&rewritten).unwrap();
        let parity = scorer.export_parity(&rewritten, &samples).unwrap();
        assert!(parity.is_exact(), "{parity:?}");
        assert!(scorer.export_parity(&rewritten, &[]).is_err());
        let header = header(&rewritten).unwrap();
        assert_eq!(header["shape"]["epoch"], json!(source_epoch + 1));
        let serving = IntegerDiscreteModel::from_bytes(&rewritten).unwrap();
        assert_eq!(serving.shape().epoch, source_epoch + 1);
    }

    #[test]
    fn cpu_discrete_checkpoint_preserves_nonzero_source_epoch() {
        let artifact = set_route_epoch(serialized_route(), 420);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap();
        let mut driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        assert_eq!(driver.applier().epoch(), 420);
        assert_eq!(driver.cursor().accepted_updates, 0);
        driver.advance_after_batch(false, 0).unwrap();
        let checkpoint = driver.checkpoint_bytes().unwrap();
        let source = driver.checkpoint().unwrap().source_artifact_sha256;
        let resumed =
            CpuDiscreteCheckpointDriver::resume_with_artifact(&checkpoint, &source, &artifact)
                .unwrap();
        assert_eq!(resumed.applier().epoch(), 420);
        assert_eq!(resumed.cursor().completed_batches, 1);
        assert_eq!(
            resumed.artifact_bytes().unwrap(),
            driver.artifact_bytes().unwrap()
        );
    }

    fn set_route_epoch(artifact: Vec<u8>, epoch: u64) -> Vec<u8> {
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16usize + header_len).div_ceil(64) * 64;
        let mut value: Value = serde_json::from_slice(&artifact[16..16 + header_len]).unwrap();
        value["shape"]["epoch"] = json!(epoch);
        let header = serde_json::to_vec(&value).unwrap();
        let new_data_start = (16usize + header.len()).div_ceil(64) * 64;
        let mut output = vec![0u8; new_data_start + artifact.len() - data_start];
        output[..8].copy_from_slice(b"UORLUT01");
        output[8..16].copy_from_slice(&(header.len() as u64).to_le_bytes());
        output[16..16 + header.len()].copy_from_slice(&header);
        output[new_data_start..].copy_from_slice(&artifact[data_start..]);
        output
    }

    #[test]
    fn serialized_flock_matches_integer_discrete_model() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 0, 1, 0],
            2,
        );
        let cpu = CpuDiscreteFlock::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [
            &[0u32][..],
            &[0, 1][..],
            &[0, 1, 2, 3, 4, 1][..],
            &[4, 3, 2, 1, 0][..],
        ] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
    }

    #[test]
    fn flock_zero_gate_fallback_and_gate_trits_are_distinct() {
        let roots = [[1, 0], [0, 0], [0, 0]];
        let mut zero_fixture = DiscreteFixture::flock();
        zero_fixture.roots = roots;
        let mut positive_fixture = DiscreteFixture::flock();
        positive_fixture.roots = roots;
        positive_fixture.gates = vec![1, 0, 1, 0];
        let mut negative_fixture = DiscreteFixture::flock();
        negative_fixture.roots = roots;
        negative_fixture.gates = vec![-1, 0, -1, 0];
        let zero_artifact = serialized_discrete(zero_fixture);
        let positive_artifact = serialized_discrete(positive_fixture);
        let negative_artifact = serialized_discrete(negative_fixture);
        let zero = CpuDiscreteFlock::from_bytes(&zero_artifact).unwrap();
        let positive = CpuDiscreteFlock::from_bytes(&positive_artifact).unwrap();
        let negative = CpuDiscreteFlock::from_bytes(&negative_artifact).unwrap();
        let zero_oracle = IntegerDiscreteModel::from_bytes(&zero_artifact).unwrap();
        let positive_oracle = IntegerDiscreteModel::from_bytes(&positive_artifact).unwrap();
        let negative_oracle = IntegerDiscreteModel::from_bytes(&negative_artifact).unwrap();
        let mut changed = false;
        for prefix in [&[0u32, 1][..], &[0, 1, 2][..], &[2, 1, 0][..]] {
            let zero_read = zero.read(prefix).unwrap();
            let positive_read = positive.read(prefix).unwrap();
            let negative_read = negative.read(prefix).unwrap();
            assert_eq!(
                zero_read,
                zero_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            assert_eq!(
                positive_read,
                positive_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            assert_eq!(
                negative_read,
                negative_oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
            let last = prefix[prefix.len() - 1] as usize;
            assert_eq!(
                zero_read,
                roots[last]
                    .iter()
                    .flat_map(|&root| orbit_root_coords(root).unwrap())
                    .collect::<Vec<_>>()
            );
            changed |= positive_read != zero_read;
        }
        assert!(changed);
    }

    #[test]
    fn serialized_multilevel_flock_matches_integer_discrete_model() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let artifact = serialized_flock_fixture(9, &roots, &[1, 1, -1, 1, -1, 1], 3);
        let cpu = CpuDiscreteFlock::from_bytes(&artifact).unwrap();
        let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
        for prefix in [
            &[0u32][..],
            &[0, 1][..],
            &[0, 1, 2, 3, 4][..],
            &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
            &[7, 6, 5, 4, 3, 2, 1, 0, 5][..],
        ] {
            assert_eq!(
                cpu.read(prefix).unwrap(),
                oracle.evaluate_prefix(prefix).unwrap().read_vector
            );
        }
        let active = cpu.read(&[0, 1, 2, 3, 4, 5, 6, 7, 0]).unwrap();
        let zero = CpuDiscreteFlock::from_bytes(&serialized_flock_fixture(9, &roots, &[0; 6], 3))
            .unwrap()
            .read(&[0, 1, 2, 3, 4, 5, 6, 7, 0])
            .unwrap();
        assert_ne!(active, zero);
    }

    #[test]
    fn flock_plan_retains_spans_across_an_unequal_frontier() {
        let plan = build_row_plan(5).unwrap();
        assert_eq!(
            plan.frontier
                .iter()
                .map(|&node| plan.nodes[node].level)
                .collect::<Vec<_>>(),
            [1, 0, 0]
        );
        let parent = plan.frontier[0];
        assert_eq!((plan.nodes[parent].first, plan.nodes[parent].last), (0, 2));
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_flock_read_benchmark() {
        use std::time::Instant;

        const VOCAB: usize = 4_096;
        const BLOCKS: usize = 4;
        const CONTEXT: usize = 192;
        const PREFIX: usize = 64;
        const KEPT: usize = 3;
        const ROWS: usize = 1_024;

        let token_blocks = (0..VOCAB)
            .map(|token| {
                (0..BLOCKS)
                    .map(|block| orbit_root_coords(((token + block * 7) & 15) as u8).unwrap())
                    .collect()
            })
            .collect();
        let stages = expected_stages(CONTEXT).unwrap();
        let gates = (0..BLOCKS)
            .map(|block| {
                (0..stages)
                    .map(|level| ((block + level) % 3) as i32 - 1)
                    .collect()
            })
            .collect();
        let flock = CpuDiscreteFlock::from_parts(CONTEXT, KEPT, token_blocks, gates).unwrap();
        let prefix = (0..PREFIX as u32)
            .map(|index| (index * 997 + 17) % VOCAB as u32)
            .collect::<Vec<_>>();
        for _ in 0..4 {
            flock.read(&prefix).unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for _ in 0..ROWS {
            for value in flock.read(&prefix).unwrap() {
                checksum = checksum.rotate_left(7).wrapping_add(value as i64 as u64);
            }
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=flock vocab={VOCAB} blocks={BLOCKS} stages={stages} kept={KEPT} prefix={PREFIX} rows={ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn malformed_flock_artifacts_are_refused() {
        let mut route = DiscreteFixture::flock();
        route.mode = "route";
        route.initialization = "zero_gates_v1";
        route.kept = 0;
        route.gates.fill(0);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(route)).is_err());

        let mut relation = DiscreteFixture::flock();
        relation.mode = "relation";
        relation.sweeps = 1;
        relation.gates = vec![0; 14];
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(relation)).is_err());

        for kept in [0, 9] {
            let mut fixture = DiscreteFixture::flock();
            fixture.kept = kept;
            assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(fixture)).is_err());
        }

        let mut sweeps = DiscreteFixture::flock();
        sweeps.sweeps = 1;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(sweeps)).is_err());

        let mut initialization = DiscreteFixture::flock();
        initialization.initialization = "zero_gates_v1";
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(initialization)).is_err());

        let mut bad_gate_count = DiscreteFixture::flock();
        bad_gate_count.gates.pop();
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_gate_count)).is_err());

        let mut bad_gate = DiscreteFixture::flock();
        bad_gate.gates[0] = 2;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_gate)).is_err());

        let mut bad_map = DiscreteFixture::flock();
        bad_map.routes[0] = 64;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_map)).is_err());

        let mut non_orbit = DiscreteFixture::flock();
        non_orbit.table = Some(vec![1; 3 * 2 * 8]);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(non_orbit)).is_err());

        let mut bad_cell = DiscreteFixture::flock();
        bad_cell.cells = Some(vec![0, 0xff, 0]);
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(bad_cell)).is_err());

        let mut short_span = DiscreteFixture::flock();
        short_span.short_span = true;
        assert!(CpuDiscreteFlock::from_bytes(&serialized_discrete(short_span)).is_err());

        let artifact = serialized_discrete(DiscreteFixture::flock());
        let wrong_kind = rewrite_header(artifact.clone(), |header| {
            header["tables"][0]["kind"] = json!("u32");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_kind).is_err());

        let misaligned = rewrite_header(artifact.clone(), |header| {
            header["tables"][0]["span"]["offset"] = json!(1);
        });
        assert!(CpuDiscreteFlock::from_bytes(&misaligned).is_err());

        let duplicate = rewrite_header(artifact.clone(), |header| {
            header["tables"][1]["name"] = json!("discrete.table");
        });
        assert!(CpuDiscreteFlock::from_bytes(&duplicate).is_err());

        let nonempty_matrices = rewrite_header(artifact.clone(), |header| {
            header["matrices"] = json!([{"name": "unexpected"}]);
        });
        assert!(CpuDiscreteFlock::from_bytes(&nonempty_matrices).is_err());

        let wrong_snapshot = rewrite_header(artifact.clone(), |header| {
            header["source"]["snapshot_schema"] = json!("uor-r4.wrong/1");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_snapshot).is_err());

        let wrong_operator = rewrite_header(artifact.clone(), |header| {
            header["source"]["operator_kappa"] = json!("blake3:wrong");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_operator).is_err());

        let wrong_composition = rewrite_header(artifact.clone(), |header| {
            header["source"]["composition_kappa"] = json!("blake3:wrong");
        });
        assert!(CpuDiscreteFlock::from_bytes(&wrong_composition).is_err());

        let unknown_objective = rewrite_header(artifact.clone(), |header| {
            header["shape"]["objective_policy"] = json!("unknown");
        });
        assert!(CpuDiscreteFlock::from_bytes(&unknown_objective).is_err());

        let missing_objective = rewrite_header(artifact.clone(), |header| {
            header["shape"]
                .as_object_mut()
                .unwrap()
                .remove("objective_policy");
        });
        assert!(CpuDiscreteFlock::from_bytes(&missing_objective).is_err());

        let mut nonzero_header_padding = artifact.clone();
        let header_len =
            u64::from_le_bytes(nonzero_header_padding[8..16].try_into().unwrap()) as usize;
        let unaligned = 16 + header_len;
        let data_start = unaligned.div_ceil(64) * 64;
        assert!(unaligned < data_start);
        nonzero_header_padding[unaligned] = 1;
        assert!(CpuDiscreteFlock::from_bytes(&nonzero_header_padding).is_err());

        let mut nonzero_padding = artifact;
        nonzero_padding.push(1);
        assert!(CpuDiscreteFlock::from_bytes(&nonzero_padding).is_err());

        let reader =
            CpuDiscreteFlock::from_bytes(&serialized_discrete(DiscreteFixture::flock())).unwrap();
        assert!(reader.read(&[]).is_err());
        assert!(reader.read(&[3]).is_err());
        assert!(reader.read(&[0, 1, 2, 0]).is_err());
    }

    #[test]
    fn serialized_relation_matches_host_across_sweeps_and_levels() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let relation_gates = [[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1], [1, 1, -1, -1, 1]];
        for sweeps in [1, 2, 5] {
            let artifact =
                serialized_relation_fixture(9, &roots, &level_gates, &relation_gates, 3, sweeps);
            let cpu = CpuDiscreteRelation::from_bytes(&artifact).unwrap();
            let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
            for prefix in [
                &[0u32][..],
                &[0, 1][..],
                &[0, 1, 2, 3, 4][..],
                &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
                &[7, 6, 5, 4, 3, 2, 1, 0, 5][..],
            ] {
                assert_eq!(
                    cpu.read(prefix).unwrap(),
                    oracle.evaluate_prefix(prefix).unwrap().read_vector,
                    "sweeps={sweeps} prefix={prefix:?}"
                );
            }
        }
    }

    #[test]
    fn each_relation_edge_kind_matches_host() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let prefix = [0u32, 1, 2, 3, 4, 5, 6, 7, 0];
        let zero_artifact = serialized_relation_fixture(
            9,
            &roots,
            &level_gates,
            &[[0; RELATION_EDGE_TYPES]; 3],
            2,
            2,
        );
        let zero_read = CpuDiscreteRelation::from_bytes(&zero_artifact)
            .unwrap()
            .read(&prefix)
            .unwrap();
        let mut changed = false;
        for edge in 0..RELATION_EDGE_TYPES {
            let mut relation_gates = [[0i32; RELATION_EDGE_TYPES]; 3];
            relation_gates[0][edge] = 1;
            let artifact =
                serialized_relation_fixture(9, &roots, &level_gates, &relation_gates, 2, 2);
            let cpu = CpuDiscreteRelation::from_bytes(&artifact).unwrap();
            let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
            let read = cpu.read(&prefix).unwrap();
            assert_eq!(
                read,
                oracle.evaluate_prefix(&prefix).unwrap().read_vector,
                "edge={edge}"
            );
            changed |= read != zero_read;
        }
        assert!(changed);
    }

    #[test]
    fn zero_relation_gates_equal_the_corresponding_flock_read() {
        let roots = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [8, 9],
            [10, 11],
            [12, 13],
            [14, 15],
        ];
        let level_gates = [1, 1, -1, 1, -1, 1];
        let relation = serialized_relation_fixture(
            9,
            &roots,
            &level_gates,
            &[[0; RELATION_EDGE_TYPES]; 3],
            3,
            2,
        );
        let flock = serialized_flock_fixture(9, &roots, &level_gates, 3);
        let relation = CpuDiscreteRelation::from_bytes(&relation).unwrap();
        let flock = CpuDiscreteFlock::from_bytes(&flock).unwrap();
        for prefix in [
            &[0u32, 1][..],
            &[0, 1, 2, 3, 4][..],
            &[0, 1, 2, 3, 4, 5, 6, 7, 0][..],
        ] {
            assert_eq!(relation.read(prefix).unwrap(), flock.read(prefix).unwrap());
        }
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_relation_read_benchmark() {
        use std::time::Instant;

        const VOCAB: usize = 4_096;
        const BLOCKS: usize = 4;
        const CONTEXT: usize = 192;
        const PREFIX: usize = 64;
        const KEPT: usize = 3;
        const SWEEPS: usize = 2;
        const ROWS: usize = 1_024;

        let token_blocks = (0..VOCAB)
            .map(|token| {
                (0..BLOCKS)
                    .map(|block| orbit_root_coords(((token + block * 7) & 15) as u8).unwrap())
                    .collect()
            })
            .collect();
        let stages = expected_stages(CONTEXT).unwrap();
        let level_gates = (0..BLOCKS)
            .map(|block| {
                (0..stages)
                    .map(|level| ((block + level) % 3) as i32 - 1)
                    .collect()
            })
            .collect();
        let relation_gates = (0..stages)
            .map(|level| std::array::from_fn(|edge| ((level + edge) % 3) as i32 - 1))
            .collect();
        let relation = CpuDiscreteRelation::from_parts(
            CONTEXT,
            KEPT,
            SWEEPS,
            token_blocks,
            level_gates,
            relation_gates,
        )
        .unwrap();
        let prefix = (0..PREFIX as u32)
            .map(|index| (index * 997 + 17) % VOCAB as u32)
            .collect::<Vec<_>>();
        for _ in 0..4 {
            relation.read(&prefix).unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for _ in 0..ROWS {
            for value in relation.read(&prefix).unwrap() {
                checksum = checksum.rotate_left(7).wrapping_add(value as i64 as u64);
            }
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=relation vocab={VOCAB} blocks={BLOCKS} stages={stages} kept={KEPT} sweeps={SWEEPS} prefix={PREFIX} rows={ROWS} elapsed_ms={:.3} rows_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn non_relation_and_malformed_relation_artifacts_are_refused() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let level_gates = [0, 1, 0, 1];
        let relation_gates = [[0; RELATION_EDGE_TYPES]; 2];
        let artifact = serialized_relation_fixture(6, &roots, &level_gates, &relation_gates, 2, 2);

        for mode in ["flock", "route"] {
            let wrong_mode = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_mode"] = json!(mode);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_mode).is_err());
        }
        for sweeps in [0, 6] {
            let wrong_sweeps = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_sweeps"] = json!(sweeps);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_sweeps).is_err());
        }
        for kept in [0, 9] {
            let wrong_kept = rewrite_header(artifact.clone(), |header| {
                header["shape"]["read_kept"] = json!(kept);
            });
            assert!(CpuDiscreteRelation::from_bytes(&wrong_kept).is_err());
        }
        let wrong_initialization = rewrite_header(artifact.clone(), |header| {
            header["shape"]["read_initialization"] = json!("zero_gates_v1");
        });
        assert!(CpuDiscreteRelation::from_bytes(&wrong_initialization).is_err());

        let mut bad_level_gate = level_gates;
        bad_level_gate[0] = 2;
        assert!(
            CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
                6,
                &roots,
                &bad_level_gate,
                &relation_gates,
                2,
                2
            ))
            .is_err()
        );

        let mut bad_relation_gates = relation_gates;
        bad_relation_gates[0][0] = 2;
        assert!(
            CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
                6,
                &roots,
                &level_gates,
                &bad_relation_gates,
                2,
                2
            ))
            .is_err()
        );

        let short_gate_table = rewrite_header(artifact.clone(), |header| {
            header["tables"][3]["len"] = json!(header["tables"][3]["len"].as_u64().unwrap() - 1);
            header["tables"][3]["span"]["bytes"] =
                json!(header["tables"][3]["span"]["bytes"].as_u64().unwrap() - 4);
        });
        assert!(CpuDiscreteRelation::from_bytes(&short_gate_table).is_err());

        let mut bad_map = artifact.clone();
        set_table_u32(&mut bad_map, "discrete.routes", 0, 64);
        assert!(CpuDiscreteRelation::from_bytes(&bad_map).is_err());

        let relation = CpuDiscreteRelation::from_bytes(&serialized_relation_fixture(
            6,
            &roots,
            &level_gates,
            &relation_gates,
            2,
            2,
        ))
        .unwrap();
        assert!(relation.read(&[]).is_err());
        assert!(relation.read(&[5]).is_err());
        assert!(relation.read(&[0, 1, 2, 3, 4, 0, 1]).is_err());

        assert!(CpuDiscreteRelation::from_bytes(&serialized_flock_fixture(
            6,
            &roots,
            &level_gates,
            2
        ))
        .is_err());
    }

    #[test]
    fn gate_chooser_uses_strict_improvement_and_ignores_inactive_lanes() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            2,
        );
        let chooser = CpuDiscreteProposalChooser::from_bytes(&artifact).unwrap();
        let mut improved = [12u64; 16];
        improved[1] = 10;
        improved[2] = 9;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &improved, 7)
            .unwrap();
        assert_eq!(selected.lane, 2);
        assert!(selected.sensitive);

        let mut tied = [11u64; 16];
        tied[3] = 0;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &tied, 7)
            .unwrap();
        assert_eq!(selected.lane, 0);
        assert!(!selected.sensitive);

        let mut all_worse = [10u64; 16];
        all_worse[1] = 11;
        all_worse[2] = 12;
        let selected = chooser
            .choose(CpuDiscreteProposalKind::Gate, 0, &all_worse, 8)
            .unwrap();
        assert_eq!(selected.lane, 0);
        assert!(selected.sensitive);
        assert_eq!(selected.epoch, 8);
    }

    #[test]
    fn route_and_table_ties_choose_the_lower_resulting_port() {
        const ROUTE_PORTS: [u32; 15] = [16, 27, 34, 41, 47, 2, 3, 4, 5, 21, 22, 23, 24, 25, 26];
        const TABLE_PORTS: [u32; 15] = [0, 10, 2, 5, 6, 5, 13, 7, 9, 11, 0, 2, 8, 7, 4];
        let composition = cl06_finite_composition_table().unwrap();
        let oracle_actions = std::array::from_fn::<_, 16, _>(|candidate| {
            NormFrameTable::new(16, vec![candidate as u8; 16])
                .unwrap()
                .block_frame(0)
                .unwrap()
        });
        let independent_route_ports = std::array::from_fn::<_, 15, _>(|lane| {
            u32::from(
                composition
                    .compose_index(oracle_actions[lane + 1], 1)
                    .unwrap(),
            )
        });
        assert_eq!(independent_route_ports, ROUTE_PORTS);
        let point = orbit_root_coords(3).unwrap();
        let independent_table_ports = std::array::from_fn::<_, 15, _>(|lane| {
            uor_r4_discrete::normalization_mechanism::identify_orbit_root(
                &uor_r4_discrete::normalization_mechanism::transform_block_by_frame(
                    oracle_actions[lane + 1],
                    &point,
                )
                .unwrap(),
            )
            .unwrap() as u32
        });
        assert_eq!(independent_table_ports, TABLE_PORTS);

        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let route = CpuDiscreteProposalChooser::from_bytes(&route_artifact).unwrap();
        let observed_route_ports = std::array::from_fn::<_, 15, _>(|lane| {
            route
                .resulting_port(CpuDiscreteProposalKind::Route, 0, lane + 1)
                .unwrap()
        });
        assert_eq!(observed_route_ports, ROUTE_PORTS);
        let mut losses = [1u64; 16];
        losses[0] = 2;
        let route_expected = 1 + ROUTE_PORTS
            .iter()
            .enumerate()
            .min_by_key(|&(lane, &port)| (port, lane))
            .unwrap()
            .0;
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &losses, 3)
                .unwrap()
                .lane,
            route_expected
        );
        let tied_to_current = [1u64; 16];
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &tied_to_current, 3)
                .unwrap()
                .lane,
            0
        );
        let mut lower_loss = [1u64; 16];
        lower_loss[7] = 0;
        assert_eq!(
            route
                .choose(CpuDiscreteProposalKind::Route, 0, &lower_loss, 3)
                .unwrap()
                .lane,
            7
        );

        let table_artifact = serialized_flock_fixture(
            6,
            &[[3, 2], [1, 0], [7, 4], [9, 8], [5, 6]],
            &[1, 0, 1, 0],
            2,
        );
        let table = CpuDiscreteProposalChooser::from_bytes(&table_artifact).unwrap();
        let observed_table_ports = std::array::from_fn::<_, 15, _>(|lane| {
            table
                .resulting_port(CpuDiscreteProposalKind::Table, 0, lane + 1)
                .unwrap()
        });
        assert_eq!(observed_table_ports, TABLE_PORTS);
        let table_expected = 1 + TABLE_PORTS
            .iter()
            .enumerate()
            .min_by_key(|&(lane, &port)| (port, lane))
            .unwrap()
            .0;
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &losses, 3)
                .unwrap()
                .lane,
            table_expected
        );
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &tied_to_current, 3)
                .unwrap()
                .lane,
            0
        );
        assert_eq!(
            table
                .choose(CpuDiscreteProposalKind::Table, 0, &lower_loss, 3)
                .unwrap()
                .lane,
            7
        );
    }

    #[test]
    fn proposal_chooser_refuses_invalid_modes_and_inventory() {
        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let route = CpuDiscreteProposalChooser::from_bytes(&route_artifact).unwrap();
        let losses = [0u64; 16];
        assert!(route
            .choose(CpuDiscreteProposalKind::Gate, 0, &losses, 0)
            .is_err());
        assert!(route
            .choose(CpuDiscreteProposalKind::Route, 4, &losses, 0)
            .is_err());
        assert!(route
            .choose(CpuDiscreteProposalKind::Table, 6, &losses, 0)
            .is_err());

        let relation_artifact = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[0; RELATION_EDGE_TYPES]; 2],
            2,
            1,
        );
        let relation = CpuDiscreteProposalChooser::from_bytes(&relation_artifact).unwrap();
        assert!(relation
            .choose(CpuDiscreteProposalKind::Route, 0, &losses, 0)
            .is_err());
        assert!(relation
            .choose(CpuDiscreteProposalKind::Gate, 14, &losses, 0)
            .is_err());
        assert!(CpuDiscreteProposalChooser::from_bytes(b"bad").is_err());
    }

    #[test]
    fn proposal_writer_baseline_matches_host_for_both_objectives() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let nearest = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let rank = rewrite_header(nearest.clone(), |header| {
            header["shape"]["objective_policy"] = json!("occupied_cell_rank_hinge_v1");
        });
        let sample = CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2, 3, 4, 1],
            target: 2,
        };
        for artifact in [nearest, rank] {
            let scorer = CpuDiscreteProposalScorer::from_bytes(&artifact).unwrap();
            let scoring = scorer
                .score(
                    CpuDiscreteProposalKind::Gate,
                    0,
                    std::slice::from_ref(&sample),
                    1,
                )
                .unwrap();
            let oracle = IntegerDiscreteModel::from_bytes(&artifact).unwrap();
            assert_eq!(
                scoring.losses[0],
                oracle
                    .observe_target(&sample.prefix, sample.target)
                    .unwrap()
                    .selected_loss
            );
            assert_eq!(scoring.eligible_rows, 1);
        }
    }

    #[test]
    fn block_scorer_returns_each_coordinate_and_an_exact_joint_loss() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let scorer = CpuDiscreteProposalScorer::from_bytes(&artifact).unwrap();
        let before = scorer.model.token_blocks().to_vec();
        let sample = CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2, 3, 4, 1],
            target: 2,
        };
        let scoring = scorer
            .score_block(
                CpuDiscreteProposalKind::Table,
                &[0, 1],
                std::slice::from_ref(&sample),
                1,
            )
            .unwrap();
        assert_eq!(scoring.eligible_rows, 1);
        assert_eq!(
            scoring
                .records
                .iter()
                .map(|record| record.index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert!(scoring.joint_loss.is_some());
        assert_eq!(scorer.model.token_blocks(), before);
    }

    #[test]
    fn joint_loss_rejects_a_row_that_loses_baseline_eligibility() {
        let mut total = 7u64;
        assert!(!accumulate_joint_loss(
            &mut total,
            CpuWriterLoss {
                loss: 11,
                token_rivals: 0,
            }
        )
        .unwrap());
        assert_eq!(total, 7);
        assert!(!accumulate_joint_loss(
            &mut total,
            CpuWriterLoss {
                loss: u64::MAX,
                token_rivals: 1,
            }
        )
        .unwrap());
        assert_eq!(total, 7);
        assert!(accumulate_joint_loss(
            &mut total,
            CpuWriterLoss {
                loss: 5,
                token_rivals: 1,
            }
        )
        .unwrap());
        assert_eq!(total, 12);
    }

    #[test]
    fn proposal_virtual_moves_match_serialized_host_states() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let sample = CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2, 3, 4, 1],
            target: 2,
        };

        let gate_artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let gate_scorer = CpuDiscreteProposalScorer::from_bytes(&gate_artifact).unwrap();
        let gate_scoring = gate_scorer
            .score(
                CpuDiscreteProposalKind::Gate,
                0,
                std::slice::from_ref(&sample),
                1,
            )
            .unwrap();
        let gate_mutated = serialized_flock_fixture(6, &roots, &[0, 0, 1, 0], 2);
        assert_eq!(
            gate_scoring.losses[1],
            IntegerDiscreteModel::from_bytes(&gate_mutated)
                .unwrap()
                .observe_target(&sample.prefix, sample.target)
                .unwrap()
                .selected_loss
        );
        let rank_gate = rewrite_header(gate_artifact, |header| {
            header["shape"]["objective_policy"] = json!("occupied_cell_rank_hinge_v1");
        });
        let rank_gate_scorer = CpuDiscreteProposalScorer::from_bytes(&rank_gate).unwrap();
        let rank_gate_scoring = rank_gate_scorer
            .score(
                CpuDiscreteProposalKind::Gate,
                0,
                std::slice::from_ref(&sample),
                1,
            )
            .unwrap();
        let rank_gate_mutated = rewrite_header(gate_mutated, |header| {
            header["shape"]["objective_policy"] = json!("occupied_cell_rank_hinge_v1");
        });
        assert_eq!(
            rank_gate_scoring.losses[1],
            IntegerDiscreteModel::from_bytes(&rank_gate_mutated)
                .unwrap()
                .observe_target(&sample.prefix, sample.target)
                .unwrap()
                .selected_loss
        );

        let table_artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let table_scorer = CpuDiscreteProposalScorer::from_bytes(&table_artifact).unwrap();
        let table_scoring = table_scorer
            .score(
                CpuDiscreteProposalKind::Table,
                0,
                std::slice::from_ref(&sample),
                1,
            )
            .unwrap();
        let mut moved_roots = roots;
        moved_roots[0][0] = identify_orbit_root(
            &move_root(
                &orbit_root_coords(roots[0][0]).unwrap(),
                table_scorer.chooser.move_actions[1],
                &table_scorer.chooser.composition,
            )
            .unwrap(),
        )
        .unwrap();
        let table_mutated = serialized_flock_fixture(6, &moved_roots, &[1, 0, 1, 0], 2);
        assert_eq!(
            table_scoring.losses[1],
            IntegerDiscreteModel::from_bytes(&table_mutated)
                .unwrap()
                .observe_target(&sample.prefix, sample.target)
                .unwrap()
                .selected_loss
        );

        let route_sample = CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2],
            target: 2,
        };
        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let route_scorer = CpuDiscreteProposalScorer::from_bytes(&route_artifact).unwrap();
        let route_scoring = route_scorer
            .score(
                CpuDiscreteProposalKind::Route,
                0,
                std::slice::from_ref(&route_sample),
                1,
            )
            .unwrap();
        let moved_route = route_scorer
            .chooser
            .composition
            .compose_index(route_scorer.chooser.move_actions[1], 1)
            .unwrap();
        let route_mutated = serialized_route_with(
            "route",
            vec![u32::from(moved_route), 2, 3, 4],
            0,
            0,
            2,
            None,
            false,
        );
        assert_eq!(
            route_scoring.losses[1],
            IntegerDiscreteModel::from_bytes(&route_mutated)
                .unwrap()
                .observe_target(&route_sample.prefix, route_sample.target)
                .unwrap()
                .selected_loss
        );
    }

    #[test]
    fn proposal_scorer_refuses_all_ineligible_rows_and_invalid_requests() {
        let mut fixture = DiscreteFixture::flock();
        fixture.kept = 1;
        let artifact = rewrite_header(serialized_discrete(fixture), |header| {
            header["shape"]["kept_cells"] = json!(1);
        });
        let scorer = CpuDiscreteProposalScorer::from_bytes(&artifact).unwrap();
        let ineligible = CpuDiscreteProposalSample {
            prefix: vec![2],
            target: 2,
        };
        assert_eq!(
            scorer
                .score(
                    CpuDiscreteProposalKind::Gate,
                    0,
                    std::slice::from_ref(&ineligible),
                    1
                )
                .unwrap_err()
                .status,
            8
        );
        assert_eq!(
            scorer
                .score(CpuDiscreteProposalKind::Gate, 0, &[], 1)
                .unwrap_err()
                .status,
            8
        );
        assert_eq!(
            scorer
                .score(
                    CpuDiscreteProposalKind::Route,
                    0,
                    std::slice::from_ref(&ineligible),
                    1
                )
                .unwrap_err()
                .status,
            9
        );
        assert_eq!(
            scorer
                .score(
                    CpuDiscreteProposalKind::Gate,
                    4,
                    std::slice::from_ref(&ineligible),
                    1
                )
                .unwrap_err()
                .status,
            4
        );
        assert_eq!(
            scorer
                .score(CpuDiscreteProposalKind::Route, 0, &[], 1)
                .unwrap_err()
                .status,
            9
        );
        assert_eq!(
            scorer
                .score(CpuDiscreteProposalKind::Gate, 4, &[], 1)
                .unwrap_err()
                .status,
            4
        );
        let route = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let bad_route_header = rewrite_header(route, |header| {
            header["matrices"] = json!([{"name": "unexpected"}]);
        });
        assert!(CpuDiscreteProposalScorer::from_bytes(&bad_route_header).is_err());
    }

    #[test]
    fn proposal_lane_sentinel_and_checked_overflow_are_explicit() {
        assert_eq!(accumulate_lane(7, None).unwrap(), None);
        assert_eq!(accumulate_lane(7, Some(5)).unwrap(), Some(12));
        assert_eq!(
            accumulate_lane(u64::MAX - 1, Some(2)).unwrap_err().status,
            12
        );
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_proposal_scoring_benchmark() {
        use std::time::Instant;

        const ROWS: usize = 512;
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let scorer = CpuDiscreteProposalScorer::from_bytes(&artifact).unwrap();
        let samples = [CpuDiscreteProposalSample {
            prefix: vec![0, 1, 2, 3, 4, 1],
            target: 2,
        }];
        for _ in 0..4 {
            scorer
                .score(CpuDiscreteProposalKind::Gate, 0, &samples, 1)
                .unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for row in 0..ROWS {
            let scoring = scorer
                .score(CpuDiscreteProposalKind::Gate, 0, &samples, row as u64)
                .unwrap();
            for loss in scoring.losses {
                checksum = checksum.rotate_left(7).wrapping_add(loss);
            }
            checksum = checksum.wrapping_add(scoring.proposal.lane as u64);
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=proposal-scoring rows={ROWS} samples={} elapsed_ms={:.3} scores_per_second={:.3} checksum={checksum:016x}",
            samples.len(),
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn exact_gate_route_and_table_applies_mutate_only_the_coordinate() {
        let roots = [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]];
        let gate_artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let mut gate = CpuDiscreteProposalApplier::from_bytes(&gate_artifact).unwrap();
        let before_gate = match &gate.scorer.model {
            CpuDiscreteProposalModel::Active(active) => active.level_gates[0][0],
            _ => unreachable!(),
        };
        let receipt = gate
            .apply(
                CpuDiscreteProposalKind::Gate,
                0,
                CpuDiscreteProposal {
                    lane: 1,
                    current_loss: 2,
                    selected_loss: 1,
                    epoch: 0,
                    sensitive: true,
                },
            )
            .unwrap();
        assert!(receipt.applied);
        assert_eq!(receipt.epoch_after, 1);
        let after_gate = match &gate.scorer.model {
            CpuDiscreteProposalModel::Active(active) => active.level_gates[0][0],
            _ => unreachable!(),
        };
        assert_ne!(before_gate, after_gate);
        assert_eq!(after_gate, sibling_gate(before_gate, 1).unwrap());

        let table_artifact = serialized_flock_fixture(6, &roots, &[1, 0, 1, 0], 2);
        let mut table = CpuDiscreteProposalApplier::from_bytes(&table_artifact).unwrap();
        let before_table = match &table.scorer.model {
            CpuDiscreteProposalModel::Active(active) => active.token_blocks[0][0],
            _ => unreachable!(),
        };
        table
            .apply(
                CpuDiscreteProposalKind::Table,
                0,
                CpuDiscreteProposal {
                    lane: 1,
                    current_loss: 2,
                    selected_loss: 1,
                    epoch: 0,
                    sensitive: true,
                },
            )
            .unwrap();
        let after_table = match &table.scorer.model {
            CpuDiscreteProposalModel::Active(active) => active.token_blocks[0][0],
            _ => unreachable!(),
        };
        let expected_table = move_root(
            &before_table,
            table.scorer.chooser.move_actions[1],
            &table.scorer.chooser.composition,
        )
        .unwrap();
        assert_eq!(after_table, expected_table);
        assert_eq!(table.scorer.chooser.token_blocks[0][0], expected_table);
        let tie_losses = {
            let mut losses = [1u64; 16];
            losses[0] = 2;
            losses
        };
        let table_expected = (1..16)
            .min_by_key(|&lane| {
                (
                    table
                        .scorer
                        .chooser
                        .resulting_port(CpuDiscreteProposalKind::Table, 0, lane)
                        .unwrap(),
                    lane,
                )
            })
            .unwrap();
        assert_eq!(
            table
                .scorer
                .chooser
                .choose(CpuDiscreteProposalKind::Table, 0, &tie_losses, 1)
                .unwrap()
                .lane,
            table_expected
        );

        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let mut route = CpuDiscreteProposalApplier::from_bytes(&route_artifact).unwrap();
        route
            .apply(
                CpuDiscreteProposalKind::Route,
                0,
                CpuDiscreteProposal {
                    lane: 1,
                    current_loss: 2,
                    selected_loss: 1,
                    epoch: 0,
                    sensitive: true,
                },
            )
            .unwrap();
        let after_route = match &route.scorer.model {
            CpuDiscreteProposalModel::Route(route) => route.routes[0][0],
            _ => unreachable!(),
        };
        assert_eq!(after_route, 16);
        assert_eq!(route.scorer.chooser.routes[0], 16);
        let route_expected = (1..16)
            .min_by_key(|&lane| {
                (
                    route
                        .scorer
                        .chooser
                        .resulting_port(CpuDiscreteProposalKind::Route, 0, lane)
                        .unwrap(),
                    lane,
                )
            })
            .unwrap();
        assert_eq!(
            route
                .scorer
                .chooser
                .choose(CpuDiscreteProposalKind::Route, 0, &tie_losses, 1)
                .unwrap()
                .lane,
            route_expected
        );
    }

    #[test]
    fn apply_status_order_and_noop_match_cuda() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let mut applier = CpuDiscreteProposalApplier::from_bytes(&artifact).unwrap();
        let prefix = [0u32, 1, 2];
        let initial_read = applier.scorer.read(&applier.scorer.model, &prefix).unwrap();
        let proposal = |lane, current, selected, epoch| CpuDiscreteProposal {
            lane,
            current_loss: current,
            selected_loss: selected,
            epoch,
            sensitive: true,
        };
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Gate, 0, proposal(1, 2, 1, 1))
                .unwrap_err()
                .status,
            3
        );
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Route, 0, proposal(1, 2, 1, 0))
                .unwrap_err()
                .status,
            9
        );
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Gate, 4, proposal(1, 2, 1, 0))
                .unwrap_err()
                .status,
            4
        );
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Gate, 0, proposal(3, 2, 1, 0))
                .unwrap_err()
                .status,
            4
        );
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Gate, 0, proposal(1, 2, 2, 0))
                .unwrap_err()
                .status,
            5
        );
        let noop = applier
            .apply(CpuDiscreteProposalKind::Gate, 0, proposal(0, 2, 2, 0))
            .unwrap();
        assert!(!noop.applied);
        assert_eq!(applier.epoch(), 0);
        assert_eq!(
            applier.scorer.read(&applier.scorer.model, &prefix).unwrap(),
            initial_read
        );
        applier.epoch = u64::MAX;
        assert_eq!(
            applier
                .apply(
                    CpuDiscreteProposalKind::Gate,
                    0,
                    proposal(1, 2, 1, u64::MAX)
                )
                .unwrap_err()
                .status,
            5
        );
        assert_eq!(
            applier.scorer.read(&applier.scorer.model, &prefix).unwrap(),
            initial_read
        );
        let epoch_five = rewrite_header(artifact, |header| {
            header["shape"]["epoch"] = json!(5);
        });
        let mut applier = CpuDiscreteProposalApplier::from_bytes(&epoch_five).unwrap();
        assert_eq!(applier.epoch(), 5);
        assert_eq!(
            applier
                .apply(CpuDiscreteProposalKind::Gate, 0, proposal(1, 2, 1, 5))
                .unwrap()
                .epoch_after,
            6
        );
    }

    #[test]
    fn bvgk_joint_decision_and_batch_apply_are_exact() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let mut applier = CpuDiscreteProposalApplier::from_bytes(&artifact).unwrap();
        let record = |index, lane, current, selected| CpuBvgkRecord {
            kind: CpuDiscreteProposalKind::Route,
            index,
            proposal: CpuDiscreteProposal {
                lane,
                current_loss: current,
                selected_loss: selected,
                epoch: 0,
                sensitive: true,
            },
            eligible: true,
            status: 0,
        };
        let records = vec![record(0, 1, 10, 7), record(1, 2, 10, 5)];
        let single = applier
            .decide_bvgk(CpuDiscreteProposalKind::Route, &records, Some(7))
            .unwrap();
        assert_eq!(single.count, 2);
        assert!(!single.joint);
        assert_eq!(single.commit, [false, true]);
        let joint = applier
            .decide_bvgk(CpuDiscreteProposalKind::Route, &records, Some(4))
            .unwrap();
        assert!(joint.joint);
        assert_eq!(joint.commit, [true, true]);
        let receipt = applier
            .apply_bvgk(CpuDiscreteProposalKind::Route, &records, &joint)
            .unwrap();
        assert_eq!(receipt.committed, [0, 1]);
        assert_eq!(receipt.epoch_after, 1);
        let CpuDiscreteProposalModel::Route(route) = &applier.scorer.model else {
            unreachable!()
        };
        assert_ne!(route.routes[0][0], 1);
        assert_ne!(route.routes[0][1], 2);
        assert_eq!(applier.scorer.chooser.routes[..2], route.routes[0]);
        assert_eq!(
            applier
                .decide_bvgk(CpuDiscreteProposalKind::Route, &records, None)
                .unwrap_err()
                .status,
            17
        );
        let mut duplicate = vec![record(0, 1, 10, 7), record(0, 2, 10, 5)];
        for record in &mut duplicate {
            record.proposal.epoch = 1;
        }
        let duplicate_decision = applier
            .decide_bvgk(CpuDiscreteProposalKind::Route, &duplicate, Some(4))
            .unwrap();
        assert_eq!(
            applier
                .apply_bvgk(
                    CpuDiscreteProposalKind::Route,
                    &duplicate,
                    &duplicate_decision
                )
                .unwrap_err()
                .status,
            15
        );
    }

    #[test]
    fn bvgk_refusals_preserve_cuda_order_and_applier_state() {
        #[derive(Debug, PartialEq, Eq)]
        struct RouteState {
            epoch: u64,
            routes: Vec<Vec<u32>>,
            chooser_routes: Vec<u32>,
            token_blocks: Vec<Vec<[i32; 8]>>,
        }

        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let record = |kind, index, lane, current, selected, epoch| CpuBvgkRecord {
            kind,
            index,
            proposal: CpuDiscreteProposal {
                lane,
                current_loss: current,
                selected_loss: selected,
                epoch,
                sensitive: true,
            },
            eligible: true,
            status: 0,
        };
        let decision = |commit: Vec<bool>, epoch| {
            let count = commit.iter().filter(|&&committed| committed).count();
            CpuBvgkDecision {
                members: commit.clone(),
                count,
                best: commit.iter().position(|&committed| committed),
                commit,
                joint: count >= 2,
                joint_loss: None,
                epoch,
            }
        };
        let route_state = |applier: &CpuDiscreteProposalApplier| {
            let CpuDiscreteProposalModel::Route(route) = &applier.scorer.model else {
                unreachable!()
            };
            RouteState {
                epoch: applier.epoch,
                routes: route.routes.clone(),
                chooser_routes: applier.scorer.chooser.routes.clone(),
                token_blocks: route.token_blocks.clone(),
            }
        };
        let refusal = |kind, records: &[CpuBvgkRecord], decision: &CpuBvgkDecision, status| {
            let mut applier = CpuDiscreteProposalApplier::from_bytes(&artifact).unwrap();
            let before = route_state(&applier);
            assert_eq!(
                applier
                    .apply_bvgk(kind, records, decision)
                    .unwrap_err()
                    .status,
                status
            );
            assert_eq!(route_state(&applier), before);
        };

        let mut malformed_kind = record(CpuDiscreteProposalKind::Table, 0, 1, 2, 1, 0);
        let malformed = CpuDiscreteProposalApplier::from_bytes(&artifact)
            .unwrap()
            .decide_bvgk(CpuDiscreteProposalKind::Route, &[malformed_kind], None)
            .unwrap();
        assert_eq!(malformed.count, 1);
        assert_eq!(malformed.commit, [true]);
        refusal(
            CpuDiscreteProposalKind::Route,
            &[malformed_kind],
            &malformed,
            14,
        );
        malformed_kind.kind = CpuDiscreteProposalKind::Route;

        let stale_decision = CpuDiscreteProposalApplier::from_bytes(&artifact)
            .unwrap()
            .decide_bvgk(CpuDiscreteProposalKind::Route, &[malformed_kind], None)
            .unwrap();
        let mut stale = stale_decision.clone();
        stale.epoch = 1;
        refusal(CpuDiscreteProposalKind::Route, &[malformed_kind], &stale, 3);
        let mut stale_member = malformed_kind;
        stale_member.proposal.epoch = 1;
        refusal(
            CpuDiscreteProposalKind::Route,
            &[stale_member],
            &stale_decision,
            3,
        );

        let gate = record(CpuDiscreteProposalKind::Gate, 0, 1, 2, 1, 0);
        let mode = CpuDiscreteProposalApplier::from_bytes(&artifact)
            .unwrap()
            .decide_bvgk(CpuDiscreteProposalKind::Gate, &[gate], None)
            .unwrap();
        assert_eq!(mode.commit, [true]);
        refusal(CpuDiscreteProposalKind::Gate, &[gate], &mode, 9);

        let bad_lane = record(CpuDiscreteProposalKind::Route, 0, 16, 2, 1, 0);
        refusal(
            CpuDiscreteProposalKind::Route,
            &[bad_lane],
            &decision(vec![true], 0),
            4,
        );
        let bad_index = record(CpuDiscreteProposalKind::Route, 4, 1, 2, 1, 0);
        refusal(
            CpuDiscreteProposalKind::Route,
            &[bad_index],
            &decision(vec![true], 0),
            4,
        );
        let not_improving = record(CpuDiscreteProposalKind::Route, 0, 1, 2, 2, 0);
        refusal(
            CpuDiscreteProposalKind::Route,
            &[not_improving],
            &decision(vec![true], 0),
            5,
        );

        let gate_artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let mut gate_applier = CpuDiscreteProposalApplier::from_bytes(&gate_artifact).unwrap();
        let CpuDiscreteProposalModel::Active(gate_model) = &gate_applier.scorer.model else {
            unreachable!()
        };
        let gates_before = gate_model.level_gates.clone();
        let gate_tokens_before = gate_model.token_blocks.clone();
        let bad_gate = record(CpuDiscreteProposalKind::Gate, 0, 3, 2, 1, 0);
        assert_eq!(
            gate_applier
                .apply_bvgk(
                    CpuDiscreteProposalKind::Gate,
                    &[bad_gate],
                    &decision(vec![true], 0)
                )
                .unwrap_err()
                .status,
            4
        );
        let CpuDiscreteProposalModel::Active(gate_model) = &gate_applier.scorer.model else {
            unreachable!()
        };
        assert_eq!(gate_model.level_gates, gates_before);
        assert_eq!(gate_model.token_blocks, gate_tokens_before);
        assert_eq!(gate_applier.epoch(), 0);

        let first = record(CpuDiscreteProposalKind::Route, 0, 1, 2, 1, 0);
        let bad_later = record(CpuDiscreteProposalKind::Route, 1, 16, 2, 1, 0);
        refusal(
            CpuDiscreteProposalKind::Route,
            &[first, bad_later],
            &decision(vec![true, true], 0),
            4,
        );
        refusal(
            CpuDiscreteProposalKind::Route,
            &[first],
            &decision(vec![false], 0),
            16,
        );

        let mut exhausted = CpuDiscreteProposalApplier::from_bytes(&artifact).unwrap();
        exhausted.epoch = u64::MAX;
        let before = route_state(&exhausted);
        let max_record = record(CpuDiscreteProposalKind::Route, 0, 1, 2, 1, u64::MAX);
        assert_eq!(
            exhausted
                .apply_bvgk(
                    CpuDiscreteProposalKind::Route,
                    &[max_record],
                    &decision(vec![true], u64::MAX)
                )
                .unwrap_err()
                .status,
            5
        );
        assert_eq!(route_state(&exhausted), before);
    }

    #[test]
    fn cpu_discrete_checkpoints_round_trip_every_read_mode() {
        let route = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let flock = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let relation = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1]],
            2,
            2,
        );
        for (artifact, prefix, schedule) in [
            (
                route,
                vec![0, 1, 2],
                CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8)
                    .unwrap(),
            ),
            (
                flock,
                vec![0, 1, 2],
                CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 1, 2, 8).unwrap(),
            ),
            (
                relation,
                vec![0, 1, 2],
                CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 1, 2, 8).unwrap(),
            ),
        ] {
            let driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
            let source = driver.checkpoint().unwrap().source_artifact_sha256;
            let before = driver.checkpoint_bytes().unwrap();
            let resumed = CpuDiscreteCheckpointDriver::resume(&before, &source).unwrap();
            assert_eq!(resumed.checkpoint_bytes().unwrap(), before);
            assert_eq!(resumed.cursor(), driver.cursor());
            assert_eq!(
                resumed
                    .applier()
                    .scorer
                    .read(&resumed.applier().scorer.model, &prefix)
                    .unwrap(),
                driver
                    .applier()
                    .scorer
                    .read(&driver.applier().scorer.model, &prefix)
                    .unwrap()
            );
        }
    }

    #[test]
    fn cpu_discrete_checkpoint_restart_continues_identically() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 2, 2, 8).unwrap();
        let mut uninterrupted =
            CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        let record = |index, lane, epoch| CpuBvgkRecord {
            kind: CpuDiscreteProposalKind::Table,
            index,
            proposal: CpuDiscreteProposal {
                lane,
                current_loss: 10,
                selected_loss: 10 - lane as u64,
                epoch,
                sensitive: true,
            },
            eligible: true,
            status: 0,
        };
        let records = [record(0, 1, 0), record(1, 2, 0)];
        let decision = uninterrupted
            .applier()
            .decide_bvgk(CpuDiscreteProposalKind::Table, &records, Some(0))
            .unwrap();
        uninterrupted
            .applier_mut()
            .apply_bvgk(CpuDiscreteProposalKind::Table, &records, &decision)
            .unwrap();
        uninterrupted.advance_after_batch(true, 2).unwrap();

        let checkpoint = uninterrupted.checkpoint_bytes().unwrap();
        let source = uninterrupted.checkpoint().unwrap().source_artifact_sha256;
        let mut resumed = CpuDiscreteCheckpointDriver::resume(&checkpoint, &source).unwrap();
        assert_eq!(resumed.checkpoint_bytes().unwrap(), checkpoint);
        let prefix = [0u32, 1, 2];
        assert_eq!(
            resumed
                .applier()
                .scorer
                .read(&resumed.applier().scorer.model, &prefix)
                .unwrap(),
            uninterrupted
                .applier()
                .scorer
                .read(&uninterrupted.applier().scorer.model, &prefix)
                .unwrap()
        );

        let next = [record(2, 3, 1)];
        let left_decision = uninterrupted
            .applier()
            .decide_bvgk(CpuDiscreteProposalKind::Table, &next, None)
            .unwrap();
        let right_decision = resumed
            .applier()
            .decide_bvgk(CpuDiscreteProposalKind::Table, &next, None)
            .unwrap();
        assert_eq!(right_decision, left_decision);
        let left_receipt = uninterrupted
            .applier_mut()
            .apply_bvgk(CpuDiscreteProposalKind::Table, &next, &left_decision)
            .unwrap();
        let right_receipt = resumed
            .applier_mut()
            .apply_bvgk(CpuDiscreteProposalKind::Table, &next, &right_decision)
            .unwrap();
        assert_eq!(right_receipt, left_receipt);
        uninterrupted.advance_after_batch(true, 1).unwrap();
        resumed.advance_after_batch(true, 1).unwrap();
        assert_eq!(
            resumed.checkpoint_bytes().unwrap(),
            uninterrupted.checkpoint_bytes().unwrap()
        );
    }

    #[test]
    fn cpu_discrete_no_commit_advances_only_schedule_cursor() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap();
        let mut driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        let before = driver.checkpoint().unwrap();
        driver.advance_after_batch(false, 0).unwrap();
        let after = driver.checkpoint().unwrap();
        assert_eq!(driver.cursor().completed_batches, 1);
        assert_eq!(driver.cursor().accepted_updates, 0);
        assert_eq!(driver.cursor().next_target, 2);
        assert_eq!(driver.cursor().next_coordinate, 1);
        assert_eq!(after.model_epoch, 0);
        assert_eq!(after.model_sha256, before.model_sha256);
        assert_eq!(after.chooser_sha256, before.chooser_sha256);
        let resumed = CpuDiscreteCheckpointDriver::resume(
            &driver.checkpoint_bytes().unwrap(),
            &after.source_artifact_sha256,
        )
        .unwrap();
        assert_eq!(
            resumed.checkpoint_bytes().unwrap(),
            driver.checkpoint_bytes().unwrap()
        );
    }

    #[test]
    fn cpu_discrete_interleaved_schedule_crosses_eight_batches_into_a_gate_read() {
        let artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 1, 2, 16).unwrap();
        let mut driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        let mut kinds = Vec::new();
        for _ in 0..10 {
            kinds.push(driver.next_block().unwrap().kind);
            driver.advance_after_batch(false, 0).unwrap();
        }
        assert_eq!(
            kinds,
            [
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Gate,
                CpuDiscreteProposalKind::Table,
                CpuDiscreteProposalKind::Table,
            ]
        );
    }

    #[test]
    fn cpu_discrete_checkpoint_continues_typed_mutations_and_interleaved_blocks() {
        fn apply_batch(
            driver: &mut CpuDiscreteCheckpointDriver,
            kind: CpuDiscreteProposalKind,
            indices: &[usize],
            lane: usize,
        ) {
            let epoch = driver.applier().epoch();
            let records = indices
                .iter()
                .map(|&index| CpuBvgkRecord {
                    kind,
                    index,
                    proposal: CpuDiscreteProposal {
                        lane,
                        current_loss: 10,
                        selected_loss: 9,
                        epoch,
                        sensitive: true,
                    },
                    eligible: true,
                    status: 0,
                })
                .collect::<Vec<_>>();
            let decision = driver
                .applier()
                .decide_bvgk(kind, &records, Some(0))
                .unwrap();
            assert!(decision.commit.iter().all(|&committed| committed));
            let receipt = driver
                .applier_mut()
                .apply_bvgk(kind, &records, &decision)
                .unwrap();
            driver
                .advance_after_batch(true, receipt.committed.len() as u64)
                .unwrap();
        }

        fn assert_continuation(
            artifact: &[u8],
            schedule: CpuDiscreteSchedule,
            first_kind: CpuDiscreteProposalKind,
            first_indices: &[usize],
            second_kind: CpuDiscreteProposalKind,
            second_indices: &[usize],
        ) {
            let mut uninterrupted =
                CpuDiscreteCheckpointDriver::from_artifact(artifact, schedule).unwrap();
            apply_batch(&mut uninterrupted, first_kind, first_indices, 1);
            let checkpoint = uninterrupted.checkpoint_bytes().unwrap();
            let source = uninterrupted.checkpoint().unwrap().source_artifact_sha256;
            let mut resumed = CpuDiscreteCheckpointDriver::resume(&checkpoint, &source).unwrap();
            let prefix = [0u32, 1, 2];
            let read = |driver: &CpuDiscreteCheckpointDriver| {
                driver
                    .applier()
                    .scorer
                    .read(&driver.applier().scorer.model, &prefix)
                    .unwrap()
            };
            assert_eq!(read(&resumed), read(&uninterrupted));
            apply_batch(&mut resumed, second_kind, second_indices, 2);
            apply_batch(&mut uninterrupted, second_kind, second_indices, 2);
            assert_eq!(read(&resumed), read(&uninterrupted));
            assert_eq!(
                resumed.checkpoint_bytes().unwrap(),
                uninterrupted.checkpoint_bytes().unwrap()
            );
        }

        let route = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        assert_continuation(
            &route,
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap(),
            CpuDiscreteProposalKind::Route,
            &[0],
            CpuDiscreteProposalKind::Route,
            &[1],
        );
        assert_continuation(
            &route,
            CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 3, 2, 8).unwrap(),
            CpuDiscreteProposalKind::Table,
            &[0, 1, 2],
            CpuDiscreteProposalKind::Table,
            &[3, 4, 5],
        );

        let flock = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        assert_continuation(
            &flock,
            CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 1, 2, 8).unwrap(),
            CpuDiscreteProposalKind::Gate,
            &[0],
            CpuDiscreteProposalKind::Gate,
            &[1],
        );

        let relation = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1]],
            2,
            2,
        );
        let relation_tail = 2 * expected_stages(6).unwrap();
        assert_continuation(
            &relation,
            CpuDiscreteSchedule::new(SchedulePolicy::InterleavedReadEvery8V1, 1, 2, 8).unwrap(),
            CpuDiscreteProposalKind::Gate,
            &[relation_tail],
            CpuDiscreteProposalKind::Gate,
            &[relation_tail + 1],
        );
    }

    const CPU_CHECKPOINT_PROCESS_INPUT: &str = "UOR_CPU_DISCRETE_CHECKPOINT_INPUT";
    const CPU_CHECKPOINT_PROCESS_SOURCE: &str = "UOR_CPU_DISCRETE_CHECKPOINT_SOURCE";
    const CPU_CHECKPOINT_PROCESS_OUTPUT: &str = "UOR_CPU_DISCRETE_CHECKPOINT_OUTPUT";

    #[test]
    fn cpu_discrete_checkpoint_process_helper() {
        let Some(input) = std::env::var_os(CPU_CHECKPOINT_PROCESS_INPUT) else {
            return;
        };
        let source = std::env::var(CPU_CHECKPOINT_PROCESS_SOURCE).unwrap();
        let output = std::env::var_os(CPU_CHECKPOINT_PROCESS_OUTPUT).unwrap();
        let checkpoint = std::fs::read(input).unwrap();
        let driver = CpuDiscreteCheckpointDriver::resume(&checkpoint, &source).unwrap();
        let reading = driver
            .applier()
            .scorer
            .read(&driver.applier().scorer.model, &[0, 1, 2])
            .unwrap();
        let result = json!({
            "reading": reading,
            "checkpoint_sha256": uor_r4_discrete::corpus::sha256(
                &driver.checkpoint_bytes().unwrap()
            ),
        });
        std::fs::write(output, serde_json::to_vec(&result).unwrap()).unwrap();
    }

    #[test]
    fn cpu_discrete_checkpoint_survives_process_restart() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap();
        let driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        let checkpoint = driver.checkpoint_bytes().unwrap();
        let source = driver.checkpoint().unwrap().source_artifact_sha256;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let checkpoint_path = std::env::temp_dir().join(format!(
            "uor-r4-cpu-discrete-{}-{nonce}.checkpoint",
            std::process::id()
        ));
        let output_path = checkpoint_path.with_extension("result");
        std::fs::write(&checkpoint_path, &checkpoint).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("discrete_route::tests::cpu_discrete_checkpoint_process_helper")
            .arg("--nocapture")
            .env(CPU_CHECKPOINT_PROCESS_INPUT, &checkpoint_path)
            .env(CPU_CHECKPOINT_PROCESS_SOURCE, &source)
            .env(CPU_CHECKPOINT_PROCESS_OUTPUT, &output_path)
            .env("RUST_TEST_THREADS", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "checkpoint helper failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let child: Value = serde_json::from_slice(&std::fs::read(&output_path).unwrap()).unwrap();
        let resumed = CpuDiscreteCheckpointDriver::resume(&checkpoint, &source).unwrap();
        assert_eq!(
            child["reading"],
            json!(resumed
                .applier()
                .scorer
                .read(&resumed.applier().scorer.model, &[0, 1, 2])
                .unwrap())
        );
        assert_eq!(
            child["checkpoint_sha256"],
            json!(uor_r4_discrete::corpus::sha256(
                &resumed.checkpoint_bytes().unwrap()
            ))
        );
        std::fs::remove_file(checkpoint_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn cpu_discrete_k3_resume_continues_after_truncated_family_block() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 3, 2, 8).unwrap();
        let mut uninterrupted =
            CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule).unwrap();
        let apply = |driver: &mut CpuDiscreteCheckpointDriver,
                     kind: CpuDiscreteProposalKind,
                     indices: &[usize]| {
            let epoch = driver.applier().epoch();
            let records = indices
                .iter()
                .map(|&index| CpuBvgkRecord {
                    kind,
                    index,
                    proposal: CpuDiscreteProposal {
                        lane: 1,
                        current_loss: 2,
                        selected_loss: 1,
                        epoch,
                        sensitive: true,
                    },
                    eligible: true,
                    status: 0,
                })
                .collect::<Vec<_>>();
            let decision = driver
                .applier()
                .decide_bvgk(kind, &records, Some(0))
                .unwrap();
            driver
                .applier_mut()
                .apply_bvgk(kind, &records, &decision)
                .unwrap();
            driver
                .advance_after_batch(true, records.len() as u64)
                .unwrap();
        };
        apply(
            &mut uninterrupted,
            CpuDiscreteProposalKind::Table,
            &[0, 1, 2],
        );
        apply(
            &mut uninterrupted,
            CpuDiscreteProposalKind::Table,
            &[3, 4, 5],
        );
        apply(
            &mut uninterrupted,
            CpuDiscreteProposalKind::Route,
            &[0, 1, 2],
        );
        assert_eq!(uninterrupted.cursor().block.unwrap().o_read, 3);
        let model = &uninterrupted.applier().scorer.model;
        assert_eq!(
            uninterrupted
                .schedule()
                .block_length(uninterrupted.cursor(), model)
                .unwrap(),
            1
        );
        let checkpoint = uninterrupted.checkpoint_bytes().unwrap();
        let source = uninterrupted.checkpoint().unwrap().source_artifact_sha256;
        let mut resumed = CpuDiscreteCheckpointDriver::resume(&checkpoint, &source).unwrap();
        apply(&mut uninterrupted, CpuDiscreteProposalKind::Route, &[3]);
        apply(&mut resumed, CpuDiscreteProposalKind::Route, &[3]);
        assert_eq!(resumed.cursor().block.unwrap().o_read, 4);
        assert_eq!(
            resumed.checkpoint_bytes().unwrap(),
            uninterrupted.checkpoint_bytes().unwrap()
        );
    }

    #[test]
    fn cpu_discrete_checkpoint_refuses_cursor_and_identity_corruption() {
        let artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let schedule =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap();
        let driver =
            CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule.clone()).unwrap();
        let checkpoint = driver.checkpoint().unwrap();
        let source = checkpoint.source_artifact_sha256.clone();
        let bytes = driver.checkpoint_bytes().unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let refusal = |value: Value| {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert!(CpuDiscreteCheckpointDriver::resume(&bytes, &source).is_err());
        };
        for mutate in [
            |value: &mut Value| value["schema"] = json!("wrong"),
            |value: &mut Value| value["substrate"] = json!("gpu"),
            |value: &mut Value| value["qualification"] = json!("qualified"),
            |value: &mut Value| value["lineage_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["model_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["chooser_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["cursor"]["accepted_updates"] = json!(1),
            |value: &mut Value| value["cursor"]["next_target"] = json!(2),
            |value: &mut Value| value["cursor"]["next_coordinate"] = json!(1),
            |value: &mut Value| {
                value["cursor"]["block"] = json!({"o_vec":0,"o_read":0,"committed_moves":0});
            },
            |value: &mut Value| value["model_epoch"] = json!(1),
            |value: &mut Value| value["unexpected"] = json!(true),
            |value: &mut Value| {
                value["lineage"]["source_identity"]["snapshot_sha256"] = json!("zz")
            },
        ] {
            let mut bad = value.clone();
            mutate(&mut bad);
            refusal(bad);
        }
        for invalid_schedule in [
            CpuDiscreteSchedule {
                policy: SchedulePolicy::CanonicalVectorThenReadV1,
                coordinate_block: 0,
                batch_rows: 2,
                eligible_targets: 8,
            },
            CpuDiscreteSchedule {
                policy: SchedulePolicy::CanonicalVectorThenReadV1,
                coordinate_block: 1,
                batch_rows: 0,
                eligible_targets: 8,
            },
            CpuDiscreteSchedule {
                policy: SchedulePolicy::CanonicalVectorThenReadV1,
                coordinate_block: 1,
                batch_rows: 2,
                eligible_targets: 0,
            },
            CpuDiscreteSchedule {
                policy: SchedulePolicy::CarrierCreditV1,
                coordinate_block: 1,
                batch_rows: 2,
                eligible_targets: 8,
            },
        ] {
            let mut checkpoint = checkpoint.clone();
            checkpoint.lineage.schedule = invalid_schedule;
            checkpoint.lineage_sha256 = canonical_json_sha256(&checkpoint.lineage).unwrap();
            let forged = serde_json::to_vec(&checkpoint).unwrap();
            assert!(CpuDiscreteCheckpointDriver::resume(&forged, &source).is_err());
        }
        let source_mutations: [fn(&mut CpuDiscreteSourceIdentity); 7] = [
            |source| source.snapshot_schema = "wrong".into(),
            |source| source.snapshot_sha256 = "zz".into(),
            |source| source.operator_kappa = "wrong".into(),
            |source| source.read_mode = "relation".into(),
            |source| source.context += 1,
            |source| source.vocabulary += 1,
            |source| source.objective_policy = "occupied_cell_rank_hinge_v1".into(),
        ];
        for mutate in source_mutations {
            let mut forged = checkpoint.clone();
            mutate(&mut forged.lineage.source_identity);
            forged.lineage_sha256 = canonical_json_sha256(&forged.lineage).unwrap();
            assert!(CpuDiscreteCheckpointDriver::resume(
                &serde_json::to_vec(&forged).unwrap(),
                &source
            )
            .is_err());
        }
        let mut model_mismatch = checkpoint.clone();
        let CpuDiscreteModelState::Route { token_blocks, .. } = &mut model_mismatch.model else {
            unreachable!()
        };
        token_blocks[0][0] = [i32::MAX; 8];
        model_mismatch.model_sha256 = canonical_json_sha256(&model_mismatch.model).unwrap();
        assert!(CpuDiscreteCheckpointDriver::resume(
            &serde_json::to_vec(&model_mismatch).unwrap(),
            &source
        )
        .is_err());
        let mut chooser_mismatch = checkpoint.clone();
        chooser_mismatch.chooser.blocks += 1;
        chooser_mismatch.chooser_sha256 = canonical_json_sha256(&chooser_mismatch.chooser).unwrap();
        assert!(CpuDiscreteCheckpointDriver::resume(
            &serde_json::to_vec(&chooser_mismatch).unwrap(),
            &source
        )
        .is_err());
        assert!(CpuDiscreteCheckpointDriver::resume(&bytes, &"00".repeat(32)).is_err());
        let mut trailing = bytes;
        trailing.push(b'0');
        assert!(CpuDiscreteCheckpointDriver::resume(&trailing, &source).is_err());
        let mut truncated = driver.checkpoint_bytes().unwrap();
        truncated.pop();
        assert!(CpuDiscreteCheckpointDriver::resume(&truncated, &source).is_err());

        let k3 =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 3, 2, 8).unwrap();
        let k3_driver = CpuDiscreteCheckpointDriver::from_artifact(&artifact, k3).unwrap();
        let mut missing_block = k3_driver.checkpoint().unwrap();
        missing_block.cursor.block = None;
        let missing = serde_json::to_vec(&missing_block).unwrap();
        assert!(CpuDiscreteCheckpointDriver::resume(&missing, &source).is_err());
        let mut move_bound_overflow = k3_driver.checkpoint().unwrap();
        let accepted_updates = u64::MAX / 2;
        let (o_vec, o_read) = family_cursors(
            accepted_updates,
            3,
            SchedulePolicy::CanonicalVectorThenReadV1,
            6,
            4,
        )
        .unwrap();
        move_bound_overflow.cursor.completed_batches = accepted_updates;
        move_bound_overflow.cursor.accepted_updates = accepted_updates;
        move_bound_overflow.cursor.corpus_pass = accepted_updates / 4;
        move_bound_overflow.cursor.next_target = (accepted_updates % 4) * 2;
        move_bound_overflow.cursor.next_coordinate = (accepted_updates % 10) as u32;
        move_bound_overflow.cursor.block = Some(BlockCursor {
            o_vec,
            o_read,
            committed_moves: accepted_updates,
        });
        move_bound_overflow.model_epoch = accepted_updates;
        assert!(CpuDiscreteCheckpointDriver::resume(
            &serde_json::to_vec(&move_bound_overflow).unwrap(),
            &source
        )
        .is_err());

        let mut overflow_cursor = checkpoint.cursor;
        overflow_cursor.completed_batches = u64::MAX;
        overflow_cursor.accepted_updates = 0;
        overflow_cursor.corpus_pass = u64::MAX / 4;
        overflow_cursor.next_target = (u64::MAX % 4) * 2;
        overflow_cursor.next_coordinate = (u64::MAX % 10) as u32;
        let mut overflow_checkpoint = checkpoint;
        overflow_checkpoint.cursor = overflow_cursor;
        overflow_checkpoint.lineage_sha256 =
            canonical_json_sha256(&overflow_checkpoint.lineage).unwrap();
        let overflow_bytes = serde_json::to_vec(&overflow_checkpoint).unwrap();
        let mut overflow = CpuDiscreteCheckpointDriver::resume(&overflow_bytes, &source).unwrap();
        let before = overflow.checkpoint_bytes().unwrap();
        assert!(overflow.advance_after_batch(false, 0).is_err());
        assert_eq!(overflow.checkpoint_bytes().unwrap(), before);
    }

    #[test]
    fn cpu_discrete_checkpoint_refuses_carrier_schedule() {
        assert!(CpuDiscreteSchedule::new(SchedulePolicy::CarrierCreditV1, 1, 2, 8).is_err());
        let flock = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let canonical =
            CpuDiscreteSchedule::new(SchedulePolicy::CanonicalVectorThenReadV1, 1, 2, 8).unwrap();
        assert!(CpuDiscreteCheckpointDriver::from_artifact(&flock, canonical).is_err());
    }

    #[test]
    fn sequential_apply_refreshes_route_and_table_tie_ports() {
        let proposal = CpuDiscreteProposal {
            lane: 1,
            current_loss: 2,
            selected_loss: 1,
            epoch: 0,
            sensitive: true,
        };
        let route_artifact = serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false);
        let mut route = CpuDiscreteProposalApplier::from_bytes(&route_artifact).unwrap();
        route
            .apply(CpuDiscreteProposalKind::Route, 0, proposal)
            .unwrap();
        let route_losses = [2u64, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let expected_route = (1..16)
            .min_by_key(|&lane| {
                let port = route
                    .scorer
                    .chooser
                    .composition
                    .compose_index(
                        route.scorer.chooser.move_actions[lane],
                        route.scorer.chooser.routes[0] as u8,
                    )
                    .unwrap();
                (port, lane)
            })
            .unwrap();
        assert_eq!(
            route
                .scorer
                .chooser
                .choose(CpuDiscreteProposalKind::Route, 0, &route_losses, 1)
                .unwrap()
                .lane,
            expected_route
        );

        let table_artifact = serialized_flock_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[1, 0, 1, 0],
            2,
        );
        let mut table = CpuDiscreteProposalApplier::from_bytes(&table_artifact).unwrap();
        table
            .apply(CpuDiscreteProposalKind::Table, 0, proposal)
            .unwrap();
        let table_losses = [2u64, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let expected_table = (1..16)
            .min_by_key(|&lane| {
                let moved = move_root(
                    &table.scorer.chooser.token_blocks[0][0],
                    table.scorer.chooser.move_actions[lane],
                    &table.scorer.chooser.composition,
                )
                .unwrap();
                (identify_orbit_root(&moved).unwrap(), lane)
            })
            .unwrap();
        assert_eq!(
            table
                .scorer
                .chooser
                .choose(CpuDiscreteProposalKind::Table, 0, &table_losses, 1)
                .unwrap()
                .lane,
            expected_table
        );
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_proposal_chooser_benchmark() {
        use std::time::Instant;

        const ROWS: usize = 1_000_000;
        let artifact = serialized_relation_fixture(
            6,
            &[[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]],
            &[0, 1, 0, 1],
            &[[1, -1, 1, -1, 1], [-1, 1, -1, 1, -1]],
            2,
            2,
        );
        let chooser = CpuDiscreteProposalChooser::from_bytes(&artifact).unwrap();
        let losses = [12, 11, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for _ in 0..4 {
            chooser
                .choose(CpuDiscreteProposalKind::Gate, 0, &losses, 1)
                .unwrap();
        }
        let started = Instant::now();
        let mut checksum = 0u64;
        for row in 0..ROWS {
            let selected = chooser
                .choose(CpuDiscreteProposalKind::Gate, 0, &losses, row as u64)
                .unwrap();
            checksum = checksum
                .rotate_left(7)
                .wrapping_add(selected.lane as u64)
                .wrapping_add(selected.current_loss)
                .wrapping_add(selected.selected_loss)
                .wrapping_add(selected.epoch);
        }
        let seconds = started.elapsed().as_secs_f64();
        println!(
            "benchmark=proposal-chooser rows={ROWS} elapsed_ms={:.3} decisions_per_second={:.3} checksum={checksum:016x}",
            seconds * 1_000.0,
            ROWS as f64 / seconds
        );
    }

    #[test]
    fn non_route_and_malformed_route_artifacts_are_refused() {
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "relation",
            vec![1, 2, 3, 4],
            2,
            1,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "flock",
            vec![1, 2, 3, 4],
            2,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![64, 2, 3, 4],
            0,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3],
            0,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            2,
            Some(vec![1; 3 * 2 * 8]),
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            1,
            0,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            1,
            2,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            0,
            None,
            false
        ))
        .is_err());
        assert!(CpuDiscreteRoute::from_bytes(&serialized_route_with(
            "route",
            vec![1, 2, 3, 4],
            0,
            0,
            2,
            None,
            true
        ))
        .is_err());
    }

    fn serialized_route() -> Vec<u8> {
        serialized_route_with("route", vec![1, 2, 3, 4], 0, 0, 2, None, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn serialized_route_with(
        mode: &str,
        routes: Vec<u32>,
        read_kept: u32,
        read_sweeps: u32,
        kept_cells: u32,
        table_override: Option<Vec<i32>>,
        short_span: bool,
    ) -> Vec<u8> {
        let roots = [[0u8, 1], [2, 3], [4, 5]];
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        if let Some(table_override) = table_override {
            table = table_override;
        }
        let gates = vec![0i32; 4];
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let mut routes_span = append(&mut blob, &ints_u32(&routes));
        if short_span {
            routes_span["bytes"] = json!(1);
        }
        let gates_span = append(&mut blob, &ints_i32(&gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": 3, "blocks": 2, "context": 3, "kept_cells": kept_cells,
                "stages": 2, "read_mode": mode, "read_kept": read_kept,
                "read_sweeps": read_sweeps, "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "zero_gates_v1", "seed": 1, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    struct DiscreteFixture {
        mode: &'static str,
        initialization: &'static str,
        kept: u32,
        sweeps: u32,
        routes: Vec<u32>,
        gates: Vec<i32>,
        roots: [[u8; 2]; 3],
        table: Option<Vec<i32>>,
        cells: Option<Vec<u32>>,
        short_span: bool,
    }

    impl DiscreteFixture {
        fn flock() -> Self {
            Self {
                mode: "flock",
                initialization: "active_leaf_v1",
                kept: 2,
                sweeps: 0,
                routes: vec![1, 2, 3, 4],
                gates: vec![0; 4],
                roots: [[0, 1], [2, 3], [4, 5]],
                table: None,
                cells: None,
                short_span: false,
            }
        }
    }

    fn serialized_discrete(fixture: DiscreteFixture) -> Vec<u8> {
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in fixture.roots {
            for root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let table = fixture.table.unwrap_or(table);
        let cells = fixture.cells.unwrap_or(cells);
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&fixture.routes));
        let mut gates_span = append(&mut blob, &ints_i32(&fixture.gates));
        if fixture.short_span {
            gates_span["bytes"] = json!(1);
        }
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": 3, "blocks": 2, "context": 3, "kept_cells": 2,
                "stages": 2, "read_mode": fixture.mode,
                "read_kept": fixture.kept, "read_sweeps": fixture.sweeps,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": fixture.initialization,
                "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":fixture.routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":fixture.gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn rewrite_header(artifact: Vec<u8>, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16 + header_len).div_ceil(64) * 64;
        let mut header: Value = serde_json::from_slice(&artifact[16..16 + header_len]).unwrap();
        edit(&mut header);
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&artifact[data_start..]);
        out
    }

    fn set_table_u32(artifact: &mut [u8], name: &str, index: usize, value: u32) {
        let parsed = header(artifact).unwrap();
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let data_start = (16 + header_len).div_ceil(64) * 64;
        let table = parsed["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|table| table["name"].as_str() == Some(name))
            .unwrap();
        let offset = table["span"]["offset"].as_u64().unwrap() as usize;
        let start = data_start + offset + index * 4;
        artifact[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn serialized_flock_fixture(
        context: u32,
        roots: &[[u8; 2]],
        gates: &[i32],
        kept: u32,
    ) -> Vec<u8> {
        let stages = expected_stages(context as usize).unwrap() as u32;
        assert_eq!(gates.len(), 2 * stages as usize);
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for &root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let routes = vec![0u32; 2 * stages as usize];
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&routes));
        let gates_span = append(&mut blob, &ints_i32(gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": roots.len(), "blocks": 2, "context": context,
                "kept_cells": 2, "stages": stages, "read_mode": "flock",
                "read_kept": kept, "read_sweeps": 0,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "active_leaf_v1", "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn serialized_relation_fixture(
        context: u32,
        roots: &[[u8; 2]],
        level_gates: &[i32],
        relation_gates: &[[i32; RELATION_EDGE_TYPES]],
        kept: u32,
        sweeps: u32,
    ) -> Vec<u8> {
        let stages = expected_stages(context as usize).unwrap();
        assert_eq!(level_gates.len(), 2 * stages);
        assert_eq!(relation_gates.len(), stages);
        let stages = stages as u32;
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for blocks in roots {
            for &root in blocks {
                table.extend_from_slice(&orbit_root_coords(root).unwrap());
            }
            cells.push(u32::from(blocks[0] << 4 | blocks[1]));
        }
        let routes = vec![0u32; 2 * stages as usize];
        let mut gates = level_gates.to_vec();
        gates.extend(relation_gates.iter().flatten());
        let mut blob = Vec::new();
        let table_span = append(&mut blob, &ints_i32(&table));
        let cells_span = append(&mut blob, &ints_u32(&cells));
        let routes_span = append(&mut blob, &ints_u32(&routes));
        let gates_span = append(&mut blob, &ints_i32(&gates));
        let header = json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": {
                "vocabulary": roots.len(), "blocks": 2, "context": context,
                "kept_cells": 2, "stages": stages, "read_mode": "relation",
                "read_kept": kept, "read_sweeps": sweeps,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "active_leaf_v1", "seed": 7, "epoch": 0
            },
            "matrices": [],
            "tables": [
                {"name":"discrete.table","kind":"i32","len":table.len(),"span":table_span},
                {"name":"discrete.cells","kind":"u32","len":cells.len(),"span":cells_span},
                {"name":"discrete.routes","kind":"u32","len":routes.len(),"span":routes_span},
                {"name":"discrete.gates","kind":"i32","len":gates.len(),"span":gates_span}
            ],
            "source": {
                "snapshot_schema":"uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256":"00".repeat(32),
                "operator_kappa":"blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa":"blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256":"11".repeat(32),
                "source_sha256":"22".repeat(32),
                "geometry_sha256":"33".repeat(32)
            }
        });
        let header = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(&header);
        out.resize(out.len().div_ceil(64) * 64, 0);
        out.extend_from_slice(&blob);
        out
    }

    fn append(blob: &mut Vec<u8>, bytes: &[u8]) -> serde_json::Value {
        blob.resize(blob.len().div_ceil(64) * 64, 0);
        let offset = blob.len();
        blob.extend_from_slice(bytes);
        json!({"offset": offset, "bytes": bytes.len()})
    }

    fn ints_i32(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn ints_u32(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
}
