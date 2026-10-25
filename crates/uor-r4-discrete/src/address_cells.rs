//! `address_structured_v1`: an injective codebook whose writer cell
//! `(root(block 0), root(block B-1))` sits near the cells of the tokens that
//! precede it in TRAIN (prep sheet
//! `m1-attention-address-structured-cells-DRAFT-2026-10-19.md`, Construction).
//!
//! Host data preparation only (D24): integer bigram counts and a cell placement.
//! No read, writer or admission quantity is computed here. The distance is the
//! writer's `address_distance` on two roots: L1 over the generated orbit-root
//! coordinates (`discrete.cu` `root_cost`), summed over the two cell blocks.
use crate::{Error, Result, Spec};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

/// Writer cells: 16 roots for block 0 times 16 roots for block B-1.
pub const CELLS: usize = 256;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// The writer's cell address metric (#2834), device spec word `s[15]`.
///
/// `OuterL1V1` is the two-block outer key every existing model uses. The other
/// two are the host oracles [`cell_envelope_l1`] and [`full_vector_cell_l1`] on
/// the device. Their sheet registers zero updates only, so update capabilities
/// refuse them. The default is omitted when serialized, so existing configs,
/// snapshots and lineages keep their bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressMetric {
    #[default]
    OuterL1V1,
    CellEnvelopeL1V1,
    FullVectorCellL1V1,
}

impl AddressMetric {
    pub fn is_outer(&self) -> bool {
        *self == Self::OuterL1V1
    }
    pub fn device_word(self) -> u32 {
        match self {
            Self::OuterL1V1 => 0,
            Self::CellEnvelopeL1V1 => 1,
            Self::FullVectorCellL1V1 => 2,
        }
    }
}

/// Which placement is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressCellsMode {
    /// Treatment: predecessors pull a target toward their cells (outer blocks 0 and B-1;
    /// interior blocks 1..B-1 set to within-cell rank `k`).
    StructuredV1,
    /// S8 packing control: identical greedy, frequency order and quota, but the
    /// predecessor identities are relabelled by a seeded permutation.
    ShuffledV1 { seed: u64 },
    /// Stage-1 full-vector placement (`m1-address-key-full-vector`, #2834):
    /// same outer-cell greedy placement as `StructuredV1`, plus predecessor-pulled
    /// interior-block placement `(block 1, block 2)` with per-cell unique interior
    /// root pairs so all `B` blocks carry address geometry at `distinct_vectors == V`.
    FullVectorV1,
    /// Shuffled packing control for `FullVectorV1`: predecessor identities are
    /// relabelled by a seeded permutation before both outer and interior pulls.
    FullVectorShuffledV1 { seed: u64 },
}
impl AddressCellsMode {
    /// The table file's `mode` string.
    pub fn label(&self) -> String {
        match self {
            Self::StructuredV1 => "address_structured_v1".to_owned(),
            Self::ShuffledV1 { seed } => format!("address_shuffled_v1_seed_{seed}"),
            Self::FullVectorV1 => "address_full_vector_v1".to_owned(),
            Self::FullVectorShuffledV1 { seed } => {
                format!("address_full_vector_shuffled_v1_seed_{seed}")
            }
        }
    }
}

/// TRAIN bigram counts at eligible target positions. `by_target[t]` lists
/// `(s, count(s -> t))` with `s` ascending; `frequency[t]` counts positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigramCounts {
    pub by_target: Vec<Vec<(u32, u64)>>,
    pub frequency: Vec<u64>,
}
impl BigramCounts {
    /// Count `tokens[p-1] -> tokens[p]` for every eligible target position `p`.
    /// The eligibility rule guarantees `p-1` is in the same document.
    pub fn count(tokens: &[u32], eligible: &[usize], vocabulary: u32) -> Result<Self> {
        let v = vocabulary as usize;
        let mut pairs = std::collections::BTreeMap::<(u32, u32), u64>::new();
        let mut frequency = vec![0u64; v];
        for &p in eligible {
            if p == 0 || p >= tokens.len() {
                return Err(invalid("eligible target position has no predecessor"));
            }
            let (s, t) = (tokens[p - 1], tokens[p]);
            if s as usize >= v || t as usize >= v {
                return Err(invalid("token outside the vocabulary"));
            }
            *pairs.entry((t, s)).or_default() += 1;
            frequency[t as usize] += 1;
        }
        let mut by_target = vec![Vec::new(); v];
        for ((t, s), c) in pairs {
            by_target[t as usize].push((s, c));
        }
        Ok(Self {
            by_target,
            frequency,
        })
    }
    /// Canonical serialization for the counts pin: little-endian
    /// `vocabulary`, then per target `frequency, n, (s, count)*n`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = (self.frequency.len() as u64).to_le_bytes().to_vec();
        for (t, edges) in self.by_target.iter().enumerate() {
            out.extend_from_slice(&self.frequency[t].to_le_bytes());
            out.extend_from_slice(&(edges.len() as u64).to_le_bytes());
            for &(s, c) in edges {
                out.extend_from_slice(&u64::from(s).to_le_bytes());
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        out
    }
}

fn root_l1(a: usize, b: usize) -> u64 {
    geometry::ROOTS[a]
        .iter()
        .zip(geometry::ROOTS[b].iter())
        .map(|(x, y)| u64::from((i32::from(*x) - i32::from(*y)).unsigned_abs()))
        .sum()
}
/// Writer address distance between two cells (L1 over both cell blocks' roots).
pub fn cell_l1(a: usize, b: usize) -> u64 {
    root_l1(a / 16, b / 16) + root_l1(a % 16, b % 16)
}

/// Exact integer L1 distance between an 8-int block and orbit root `root` (`0..16`).
pub fn block_l1(q_block: &[i32], root: usize) -> u64 {
    q_block
        .iter()
        .zip(geometry::ROOTS[root].iter())
        .map(|(&x, &y)| u64::from(x.abs_diff(i32::from(y))))
        .sum()
}

/// Endpoint-only 2-block cell address distance (`discrete.cu` `address_distance` at line 274):
/// compares only `block 0` to `cell / 16` and `block B-1` to `cell % 16`.
/// For any snapped root query vector `q`, this takes at most 5 distinct values `{0, 4, 8, 12, 16}`.
/// Host reference oracle and finite-control helper (not a CPU evaluation path; CHARTER §3.5 / D24).
pub fn cell_outer_l1(query: &[i32], blocks: usize, cell: usize) -> Result<u64> {
    if blocks < 2 || query.len() != blocks * 8 || cell >= CELLS {
        return Err(invalid(
            "cell_outer_l1 requires blocks >= 2, query.len() == blocks * 8, and cell in 0..256",
        ));
    }
    Ok(block_l1(&query[0..8], cell / 16)
        + block_l1(&query[(blocks - 1) * 8..blocks * 8], cell % 16))
}

/// Full-vector L1 distance across all `B` blocks (`discrete.cu` `token_distance` at line 277).
/// Host reference oracle and finite-control helper (not a CPU evaluation path; CHARTER §3.5 / D24).
pub fn token_full_l1(query: &[i32], token_vector: &[i32]) -> Result<u64> {
    if query.is_empty() || !query.len().is_multiple_of(8) || query.len() != token_vector.len() {
        return Err(invalid(
            "token_full_l1 requires equal non-empty 8-int-block vectors",
        ));
    }
    Ok(query
        .iter()
        .zip(token_vector.iter())
        .map(|(&x, &y)| u64::from(x.abs_diff(y)))
        .sum())
}

/// Pure unweighted cell-envelope full-vector distance (`m1-address-key-full-vector`, #2834):
/// minimum full-vector `B`-block L1 distance over all tokens currently occupying `cell`.
/// Uses only integer subtraction, absolute value, addition, and minimum.
/// Host reference oracle and finite-control helper (not a CPU evaluation path; CHARTER §3.5 / D24).
pub fn cell_envelope_l1(
    query: &[i32],
    table: &[i32],
    cells: &[u16],
    spec: Spec,
    cell: usize,
) -> Result<u64> {
    let v = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    let width = blocks * 8;
    if blocks < 2
        || v == 0
        || query.len() != width
        || table.len() != v * width
        || cells.len() != v
        || cell >= CELLS
    {
        return Err(invalid(
            "cell_envelope_l1 input dimensions do not match Spec",
        ));
    }
    let mut best: Option<u64> = None;
    for (t, &c) in cells.iter().enumerate() {
        if usize::from(c) == cell {
            let d = token_full_l1(query, &table[t * width..(t + 1) * width])?;
            best = Some(best.map_or(d, |cur| cur.min(d)));
        }
    }
    best.ok_or_else(|| invalid("cell has no tokens in table"))
}

/// Two-scale full-vector cell address metric (`m1-address-key-full-vector`, #2834):
/// combines the exact `B`-block cell-envelope minimum distance `d_env(q, cell)` with the
/// total `B`-block L1 distance summed over the cell's tokens `d_sum(q, cell)`:
/// `full_vector_cell_l1(q, cell) = (d_env(q, cell) << 8) + d_sum(q, cell)`.
/// Because every root L1 step is a multiple of 4 (`4 * 256 = 1024 > 16 * 32 = 512`),
/// `d_env` is strictly lexicographically primary and `d_sum` breaks within-shell ties
/// geometrically across all `B` blocks (invariant to token-ID ordering inside the cell)
/// before `cell_precedes` falls back to `cell < other_cell`.
/// Host reference oracle and finite-control helper (not a CPU evaluation path; CHARTER §3.5 / D24).
pub fn full_vector_cell_l1(
    query: &[i32],
    table: &[i32],
    cells: &[u16],
    spec: Spec,
    cell: usize,
) -> Result<u64> {
    let v = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    let width = blocks * 8;
    if blocks < 2
        || v == 0
        || query.len() != width
        || table.len() != v * width
        || cells.len() != v
        || cell >= CELLS
    {
        return Err(invalid(
            "full_vector_cell_l1 input dimensions do not match Spec",
        ));
    }
    let mut best: Option<u64> = None;
    let mut total: u64 = 0;
    for (t, &c) in cells.iter().enumerate() {
        if usize::from(c) == cell {
            let d = token_full_l1(query, &table[t * width..(t + 1) * width])?;
            total += d;
            best = Some(best.map_or(d, |cur| cur.min(d)));
        }
    }
    match best {
        Some(d_env) => Ok((d_env << 8) + total),
        None => Err(invalid("cell has no tokens in table")),
    }
}

/// Deterministic starting interior root pair `(a1, a2)` in `0..256` for outer cell `(r0, r3)`
/// when a token has zero placed-predecessor mass; per-cell uniqueness is enforced by `used_interior`.
fn latin_interior_anchor(cell: usize) -> usize {
    let r0 = cell / 16;
    let r3 = cell % 16;
    let a1 = r0 ^ r3;
    let a2 = r0 ^ ((r3.wrapping_mul(3) + 5) & 15);
    a1 * 16 + a2
}

/// Admit the top `k` cells out of `0..256` ordered by `(distances[cell], cell)`,
/// matching `discrete.cu` `cell_precedes(distance, cell, other_distance, other_cell)`.
/// Host reference oracle and finite-control helper (not a CPU evaluation path; CHARTER §3.5 / D24).
pub fn admit_cells_by_distances(distances: &[u64; CELLS], k: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..CELLS).collect();
    order.sort_by_key(|&c| (distances[c], c));
    order.truncate(k.min(CELLS));
    order
}

/// Deterministic seeded permutation of `0..n` (splitmix64 Fisher-Yates).
fn permutation(n: usize, seed: u64) -> Vec<u32> {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let mut p: Vec<u32> = (0..n as u32).collect();
    for i in (1..n).rev() {
        p.swap(i, (next() % (i as u64 + 1)) as usize);
    }
    p
}

/// Placement result: the cell per token and the flat integer table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressCells {
    pub mode: String,
    pub cells: Vec<u16>,
    pub table: Vec<i32>,
    pub distinct_vectors: usize,
    pub occupied_cells: usize,
}

/// Greedy placement (sheet Construction steps 3-4). `fallback_cells[t]` is the
/// token's PrimeZeta cell, used when it has zero placed-predecessor mass.
pub fn build(
    spec: Spec,
    counts: &BigramCounts,
    fallback_cells: &[u16],
    mode: AddressCellsMode,
) -> Result<AddressCells> {
    let v = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    if blocks < 2 || v == 0 || !v.is_multiple_of(CELLS) || v / CELLS > 16 {
        return Err(invalid(
            "address cells need blocks >= 2 and vocabulary = 256 x quota, quota <= 16",
        ));
    }
    if counts.by_target.len() != v || counts.frequency.len() != v || fallback_cells.len() != v {
        return Err(invalid(
            "counts or fallback cells differ from the vocabulary",
        ));
    }
    let quota = v / CELLS;
    let relabel: Vec<u32> = match mode {
        AddressCellsMode::StructuredV1 | AddressCellsMode::FullVectorV1 => (0..v as u32).collect(),
        AddressCellsMode::ShuffledV1 { seed } | AddressCellsMode::FullVectorShuffledV1 { seed } => {
            permutation(v, seed)
        }
    };
    let use_full_vector_interior = matches!(
        mode,
        AddressCellsMode::FullVectorV1 | AddressCellsMode::FullVectorShuffledV1 { .. }
    );
    let mut order: Vec<usize> = (0..v).collect();
    order.sort_by(|&a, &b| {
        counts.frequency[b]
            .cmp(&counts.frequency[a])
            .then(a.cmp(&b))
    });
    let mut cell_of: Vec<Option<usize>> = vec![None; v];
    let mut interior_of: Vec<Option<usize>> = vec![None; v];
    let mut used_interior = [[false; CELLS]; CELLS];
    let mut load = [0usize; CELLS];
    let mut interior_load = [0usize; CELLS];
    for &t in &order {
        let placed: Vec<(usize, u64)> = counts.by_target[t]
            .iter()
            .filter_map(|&(s, c)| {
                let s = *relabel.get(s as usize)? as usize;
                cell_of[s].map(|cell| (cell, c))
            })
            .filter(|&(_, c)| c > 0)
            .collect();
        let best = if placed.is_empty() {
            let home = usize::from(fallback_cells[t]);
            if home >= CELLS {
                return Err(invalid("fallback cell outside 0..256"));
            }
            (0..CELLS)
                .filter(|&c| load[c] < quota)
                .min_by_key(|&c| (cell_l1(home, c), c))
        } else {
            (0..CELLS).filter(|&c| load[c] < quota).min_by_key(|&c| {
                let cost: u128 = placed
                    .iter()
                    .map(|&(sc, n)| u128::from(n) * u128::from(cell_l1(sc, c)))
                    .sum();
                (cost, c)
            })
        };
        let cell = best.ok_or_else(|| invalid("no cell has quota left"))?;
        load[cell] += 1;
        cell_of[t] = Some(cell);
        if use_full_vector_interior && blocks >= 3 {
            let anchor = latin_interior_anchor(cell);
            let placed_interior: Vec<(usize, u64)> = counts.by_target[t]
                .iter()
                .filter_map(|&(s, c)| {
                    let s = *relabel.get(s as usize)? as usize;
                    interior_of[s].map(|ic| (ic, c))
                })
                .filter(|&(_, c)| c > 0)
                .collect();
            let candidate_limit = if blocks == 3 { 16 } else { CELLS };
            let interior_quota = if blocks == 3 { v.div_ceil(16) } else { quota };
            let has_quota_candidate = (0..candidate_limit)
                .any(|ic| !used_interior[cell][ic] && interior_load[ic] < interior_quota);
            let candidates = (0..candidate_limit).filter(|&ic| {
                !used_interior[cell][ic]
                    && (!has_quota_candidate || interior_load[ic] < interior_quota)
            });
            let best_interior = if placed_interior.is_empty() {
                candidates.min_by_key(|&ic| {
                    let d = if blocks == 3 {
                        root_l1(anchor / 16, ic)
                    } else {
                        cell_l1(anchor, ic)
                    };
                    (d, ic)
                })
            } else {
                candidates.min_by_key(|&ic| {
                    let cost: u128 = placed_interior
                        .iter()
                        .map(|&(sic, n)| {
                            let d = if blocks == 3 {
                                root_l1(sic, ic)
                            } else {
                                cell_l1(sic, ic)
                            };
                            u128::from(n) * u128::from(d)
                        })
                        .sum();
                    let anchor_d = if blocks == 3 {
                        root_l1(anchor / 16, ic)
                    } else {
                        cell_l1(anchor, ic)
                    };
                    (cost, anchor_d, ic)
                })
            };
            let ic = best_interior.ok_or_else(|| invalid("no interior root pair left in cell"))?;
            used_interior[cell][ic] = true;
            interior_load[ic] += 1;
            interior_of[t] = Some(ic);
        }
    }
    let mut table = Vec::with_capacity(v * blocks * 8);
    let mut cells = Vec::with_capacity(v);
    let mut rank = [0usize; CELLS];
    for (t, slot) in cell_of.iter().enumerate() {
        let cell = slot.ok_or_else(|| invalid("token left unplaced"))?;
        let k = rank[cell];
        rank[cell] += 1;
        for b in 0..blocks {
            let root = match b {
                0 => cell / 16,
                _ if b == blocks - 1 => cell % 16,
                1 if use_full_vector_interior && blocks == 3 => {
                    interior_of[t].ok_or_else(|| invalid("token interior root left unplaced"))?
                }
                1 if use_full_vector_interior && blocks >= 4 => {
                    interior_of[t].ok_or_else(|| invalid("token interior pair left unplaced"))? / 16
                }
                2 if use_full_vector_interior && blocks >= 4 => {
                    interior_of[t].ok_or_else(|| invalid("token interior pair left unplaced"))? % 16
                }
                _ if use_full_vector_interior && blocks > 4 => {
                    let ic = interior_of[t]
                        .ok_or_else(|| invalid("token interior pair left unplaced"))?;
                    ((ic / 16) ^ (ic % 16) ^ b) & 15
                }
                _ => k,
            };
            table.extend(geometry::ROOTS[root].iter().map(|&x| i32::from(x)));
        }
        cells.push(cell as u16);
    }
    let distinct_vectors = crate::training::CodebookTable::count_distinct(&table, spec.blocks);
    let occupied_cells = load.iter().filter(|&&n| n > 0).count();
    if distinct_vectors != v {
        return Err(invalid("address-cells table is not injective"));
    }
    if load.iter().any(|&n| n != quota) {
        return Err(invalid("address-cells occupancy differs from the quota"));
    }
    Ok(AddressCells {
        mode: mode.label(),
        cells,
        table,
        distinct_vectors,
        occupied_cells,
    })
}

/// Serialised `uor-r4.codebook-table/1` file for a placed table (shared by the
/// exporter and its round-trip test).
pub fn table_file_bytes(spec: Spec, placed: &AddressCells) -> Result<Vec<u8>> {
    use crate::training::{CodebookTable, CODEBOOK_TABLE_SCHEMA};
    let table = CodebookTable {
        schema: CODEBOOK_TABLE_SCHEMA.into(),
        spec,
        mode: placed.mode.clone(),
        distinct_vectors: CodebookTable::count_distinct(&placed.table, spec.blocks),
        table_sha256: CodebookTable::words_sha256(&placed.table),
        table: placed.table.clone(),
    };
    serde_json::to_vec(&table).map_err(|e| invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> Spec {
        Spec {
            vocabulary: 512,
            blocks: 4,
            context: 8,
            kept_cells: 8,
        }
    }
    /// Chain-structured synthetic stream: token t is mostly preceded by t/2.
    fn fixture() -> (BigramCounts, Vec<u16>) {
        let mut tokens = Vec::new();
        for t in 1..512u32 {
            for _ in 0..(1 + (512 - t) / 64) {
                tokens.push(t / 2);
                tokens.push(t);
            }
        }
        let eligible: Vec<usize> = (1..tokens.len()).step_by(2).collect();
        let counts = BigramCounts::count(&tokens, &eligible, 512).expect("counts");
        let fallback = (0..512u32).map(|t| (t % 256) as u16).collect();
        (counts, fallback)
    }

    #[test]
    fn address_cells_injective_quota_and_roots() {
        let (counts, fallback) = fixture();
        let built =
            build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("build");
        assert_eq!(built.distinct_vectors, 512);
        assert_eq!(built.occupied_cells, 256);
        let mut load = [0usize; CELLS];
        for &c in &built.cells {
            load[usize::from(c)] += 1;
        }
        assert!(load.iter().all(|&n| n == 2));
        for block in built.table.chunks_exact(8) {
            assert!(geometry::ROOTS
                .iter()
                .any(|r| r.iter().zip(block).all(|(a, b)| i32::from(*a) == *b)));
        }
        for (t, w) in built.table.chunks_exact(32).enumerate() {
            let root = |b: usize| {
                geometry::ROOTS.iter().position(|r| {
                    r.iter()
                        .zip(&w[b * 8..b * 8 + 8])
                        .all(|(a, x)| i32::from(*a) == *x)
                })
            };
            assert_eq!(
                root(0).zip(root(3)).map(|(a, b)| a * 16 + b),
                Some(usize::from(built.cells[t]))
            );
        }
    }

    #[test]
    fn address_cells_deterministic() {
        let (counts, fallback) = fixture();
        let a = build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("a");
        let b = build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("b");
        assert_eq!(a, b);
        assert_eq!(counts.canonical_bytes(), fixture().0.canonical_bytes());
    }

    #[test]
    fn address_cells_zero_mass_uses_prime_zeta_cell_and_pull_reaches_distance_zero() {
        // Token 5 is placed first (highest frequency) with no placed predecessor:
        // zero mass, so it lands in its PrimeZeta cell, not the low-id cell 0.
        let mut counts = BigramCounts {
            by_target: vec![Vec::new(); 512],
            frequency: vec![0; 512],
        };
        counts.frequency[5] = 100;
        counts.frequency[6] = 50;
        counts.by_target[6] = vec![(5, 9)];
        let fallback: Vec<u16> = (0..512u32).map(|t| ((t * 7 + 3) % 256) as u16).collect();
        let built =
            build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("build");
        assert_eq!(built.cells[5], fallback[5]);
        // Token 6's only predecessor is 5, so it sits at distance 0 from cell(5).
        assert_ne!(fallback[6], fallback[5]);
        assert_eq!(built.cells[6], built.cells[5]);
    }

    #[test]
    fn address_cells_counts_use_the_immediate_predecessor() {
        // The scored target at position p has window tokens[start..p), so its
        // predecessor is tokens[p - 1], never tokens[p + 1].
        let tokens = [10u32, 20, 30, 40, 50];
        let counts = BigramCounts::count(&tokens, &[2, 3], 512).expect("count");
        assert_eq!(counts.by_target[30], vec![(20, 1)]);
        assert_eq!(counts.by_target[40], vec![(30, 1)]);
        assert!(counts.by_target[50].is_empty());
        assert_eq!((counts.frequency[30], counts.frequency[40]), (1, 1));
        assert!(BigramCounts::count(&tokens, &[0], 512).is_err());
    }

    #[test]
    fn address_cells_two_block_table_with_shared_cells_is_refused() {
        // With B = 2 every token in a cell would share one word: the injectivity
        // recount must refuse it rather than export a non-injective table.
        let two_block = Spec {
            vocabulary: 512,
            blocks: 2,
            context: 8,
            kept_cells: 8,
        };
        let (counts, fallback) = fixture();
        assert!(build(
            two_block,
            &counts,
            &fallback,
            AddressCellsMode::StructuredV1
        )
        .is_err());
    }

    #[test]
    fn address_cells_shuffled_keeps_quota_and_differs() {
        let (counts, fallback) = fixture();
        let s = build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("s");
        let p = build(
            spec(),
            &counts,
            &fallback,
            AddressCellsMode::ShuffledV1 { seed: 8 },
        )
        .expect("p");
        assert_eq!(p.mode, "address_shuffled_v1_seed_8");
        assert_eq!((p.distinct_vectors, p.occupied_cells), (512, 256));
        assert_ne!(s.cells, p.cells);
        let mean_pull = |b: &AddressCells| -> u64 {
            (1..512)
                .map(|t| cell_l1(usize::from(b.cells[t / 2]), usize::from(b.cells[t])))
                .sum()
        };
        assert!(mean_pull(&s) < mean_pull(&p));
    }

    #[test]
    fn address_cells_rejects_bad_vocabulary() {
        let (counts, fallback) = fixture();
        let bad = Spec {
            vocabulary: 500,
            ..spec()
        };
        assert!(build(bad, &counts, &fallback, AddressCellsMode::StructuredV1).is_err());
    }

    #[test]
    fn address_cells_table_file_loads_through_pinned_loader() {
        use crate::corpus::{sha256, FilePin};
        use crate::training::CodebookTable;
        let (counts, fallback) = fixture();
        let placed =
            build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("build");
        let bytes = table_file_bytes(spec(), &placed).expect("bytes");
        let path = std::path::PathBuf::from(env!("OUT_DIR")).join(format!(
            "address_cells_roundtrip_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("write");
        let pin = FilePin {
            path: path.clone(),
            sha256: sha256(&bytes),
        };
        let loaded = CodebookTable::load(&pin, spec());
        std::fs::remove_file(&path).ok();
        let loaded = loaded.expect("load");
        assert_eq!(loaded, placed.table);
        assert_eq!(
            CodebookTable::count_distinct(&loaded, spec().blocks),
            spec().vocabulary as usize
        );
        assert_eq!(placed.distinct_vectors, spec().vocabulary as usize);
    }

    #[test]
    fn full_vector_v1_preserves_injectivity_alias_ceiling_and_zero_tie_mass() {
        let (counts, fallback) = fixture();
        let built =
            build(spec(), &counts, &fallback, AddressCellsMode::FullVectorV1).expect("build");
        assert_eq!(built.mode, "address_full_vector_v1");
        assert_eq!(built.distinct_vectors, 512);
        assert_eq!(built.occupied_cells, 256);
        for block in built.table.chunks_exact(8) {
            assert!(geometry::ROOTS
                .iter()
                .any(|r| r.iter().zip(block).all(|(a, b)| i32::from(*a) == *b)));
        }
        let targets: Vec<u32> = (0..512u32).collect();
        let report = crate::coverage_census::census(
            &built.table,
            spec(),
            &targets,
            Some(&built.cells),
            Some(200),
        )
        .expect("census");
        assert_eq!(report["distinct_vectors"], 512);
        assert_eq!(report["dev_alias_ceiling_rows"], 512);
        assert_eq!(report["within_cell_tie_mass"]["tokens_in_alias_groups"], 0);
        assert_eq!(
            report["within_cell_tie_mass"]["dev_rows_targeting_non_representative"],
            0
        );
        assert_eq!(report["g_cap"]["verdict"], "ALIAS_CEILING_ABOVE_BAR");
    }

    #[test]
    fn full_vector_address_metric_expands_levels_and_lifts_fixture_admission() {
        let (counts, fallback) = fixture();
        let structured =
            build(spec(), &counts, &fallback, AddressCellsMode::StructuredV1).expect("structured");
        let full_vec =
            build(spec(), &counts, &fallback, AddressCellsMode::FullVectorV1).expect("full_vec");
        assert_eq!(structured.cells, full_vec.cells);

        let q0 = &full_vec.table[0..32];
        let mut outer_dists = [0u64; CELLS];
        let mut env_structured = [0u64; CELLS];
        let mut env_full = [0u64; CELLS];
        let mut two_scale_full = [0u64; CELLS];
        for c in 0..CELLS {
            outer_dists[c] = cell_outer_l1(q0, 4, c).expect("outer");
            env_structured[c] =
                cell_envelope_l1(q0, &structured.table, &structured.cells, spec(), c)
                    .expect("env_s");
            env_full[c] =
                cell_envelope_l1(q0, &full_vec.table, &full_vec.cells, spec(), c).expect("env_f");
            two_scale_full[c] =
                full_vector_cell_l1(q0, &full_vec.table, &full_vec.cells, spec(), c)
                    .expect("two_scale");
        }
        let distinct_outer: std::collections::BTreeSet<u64> = outer_dists.iter().copied().collect();
        let distinct_env_structured: std::collections::BTreeSet<u64> =
            env_structured.iter().copied().collect();
        let distinct_env_full: std::collections::BTreeSet<u64> = env_full.iter().copied().collect();
        let distinct_two_scale: std::collections::BTreeSet<u64> =
            two_scale_full.iter().copied().collect();
        // Outer 2-block metric has exactly 5 levels {0, 4, 8, 12, 16}.
        assert_eq!(
            distinct_outer,
            std::collections::BTreeSet::from([0, 4, 8, 12, 16])
        );
        // On StructuredV1, every cell shares the same 2 interior ranks k=0,1, so envelope
        // levels remain collapsed; FullVectorV1 expands both envelope and two-scale levels.
        assert!(distinct_env_full.len() > distinct_outer.len());
        assert!(distinct_env_full.len() > distinct_env_structured.len());
        assert!(distinct_two_scale.len() >= 15);

        // K = 8 and K = 16 boundary tie bucket size: in the 16-root SpiralCore orbit,
        // each root has (1, 2, 13) roots at distances (0, 4, 8), so the 2-block outer
        // metric has exact shell sizes (1, 4, 30, 52, 169) at distances (0, 4, 8, 12, 16).
        // Both K = 8 (8th cell) and K = 16 (16th cell) sit inside the 30-cell d = 8 shell!
        let adm_outer = admit_cells_by_distances(&outer_dists, 8);
        let boundary_outer_d = outer_dists[adm_outer[7]];
        assert_eq!(boundary_outer_d, 8);
        let outer_tie_bucket = outer_dists
            .iter()
            .filter(|&&d| d == boundary_outer_d)
            .count();
        assert_eq!(outer_tie_bucket, 30);

        let adm_full = admit_cells_by_distances(&two_scale_full, 8);
        let boundary_full_d = two_scale_full[adm_full[7]];
        let full_tie_bucket = two_scale_full
            .iter()
            .filter(|&&d| d == boundary_full_d)
            .count();
        assert!(full_tie_bucket < outer_tie_bucket);

        // Predecessor-query target cell admission at K = 8 across all 511 chain edges (t/2 -> t).
        let mut admitted_outer_k8 = 0usize;
        let mut admitted_env_k8 = 0usize;
        let mut admitted_full_k8 = 0usize;
        for t in 1..512usize {
            let s = t / 2;
            let q = &full_vec.table[s * 32..(s + 1) * 32];
            let target_cell = usize::from(full_vec.cells[t]);
            let mut d_out = [0u64; CELLS];
            let mut d_env = [0u64; CELLS];
            let mut d_fv = [0u64; CELLS];
            for c in 0..CELLS {
                d_out[c] = cell_outer_l1(q, 4, c).expect("d_out");
                d_env[c] = cell_envelope_l1(q, &full_vec.table, &full_vec.cells, spec(), c)
                    .expect("d_env");
                d_fv[c] = full_vector_cell_l1(q, &full_vec.table, &full_vec.cells, spec(), c)
                    .expect("d_fv");
            }
            if admit_cells_by_distances(&d_out, 8).contains(&target_cell) {
                admitted_outer_k8 += 1;
            }
            if admit_cells_by_distances(&d_env, 8).contains(&target_cell) {
                admitted_env_k8 += 1;
            }
            if admit_cells_by_distances(&d_fv, 8).contains(&target_cell) {
                admitted_full_k8 += 1;
            }
        }
        assert!(
            admitted_env_k8 > admitted_outer_k8,
            "expected cell_envelope_l1 ({admitted_env_k8}) > cell_outer_l1 ({admitted_outer_k8})"
        );
        assert_eq!(
            (
                distinct_outer.len(),
                distinct_env_structured.len(),
                distinct_env_full.len(),
                distinct_two_scale.len()
            ),
            (5, 5, 7, 15)
        );
        assert_eq!((outer_tie_bucket, full_tie_bucket), (30, 6));
        assert_eq!(
            (admitted_outer_k8, admitted_env_k8, admitted_full_k8),
            (186, 349, 446)
        );
    }

    #[test]
    fn full_vector_shuffled_control_preserves_quota_and_loses_interior_pull() {
        let (counts, fallback) = fixture();
        let fv = build(spec(), &counts, &fallback, AddressCellsMode::FullVectorV1).expect("fv");
        let fvs = build(
            spec(),
            &counts,
            &fallback,
            AddressCellsMode::FullVectorShuffledV1 { seed: 8 },
        )
        .expect("fvs");
        assert_eq!(fvs.mode, "address_full_vector_shuffled_v1_seed_8");
        assert_eq!((fvs.distinct_vectors, fvs.occupied_cells), (512, 256));
        assert_ne!(fv.table, fvs.table);
        let total_full_pull = |b: &AddressCells| -> u64 {
            (1..512usize)
                .map(|t| {
                    let s = t / 2;
                    token_full_l1(
                        &b.table[s * 32..(s + 1) * 32],
                        &b.table[t * 32..(t + 1) * 32],
                    )
                    .expect("full_l1")
                })
                .sum()
        };
        assert!(total_full_pull(&fv) < total_full_pull(&fvs));
    }
}
