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

/// Which placement is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressCellsMode {
    /// Treatment: predecessors pull a target toward their cells.
    StructuredV1,
    /// S8 packing control: identical greedy, frequency order and quota, but the
    /// predecessor identities are relabelled by a seeded permutation.
    ShuffledV1 { seed: u64 },
}
impl AddressCellsMode {
    /// The table file's `mode` string.
    pub fn label(&self) -> String {
        match self {
            Self::StructuredV1 => "address_structured_v1".to_owned(),
            Self::ShuffledV1 { seed } => format!("address_shuffled_v1_seed_{seed}"),
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
        AddressCellsMode::StructuredV1 => (0..v as u32).collect(),
        AddressCellsMode::ShuffledV1 { seed } => permutation(v, seed),
    };
    let mut order: Vec<usize> = (0..v).collect();
    order.sort_by(|&a, &b| {
        counts.frequency[b]
            .cmp(&counts.frequency[a])
            .then(a.cmp(&b))
    });
    let mut cell_of: Vec<Option<usize>> = vec![None; v];
    let mut load = [0usize; CELLS];
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
    }
    let mut table = Vec::with_capacity(v * blocks * 8);
    let mut cells = Vec::with_capacity(v);
    let mut rank = [0usize; CELLS];
    for slot in &cell_of {
        let cell = slot.ok_or_else(|| invalid("token left unplaced"))?;
        let k = rank[cell];
        rank[cell] += 1;
        for b in 0..blocks {
            let root = match b {
                0 => cell / 16,
                _ if b == blocks - 1 => cell % 16,
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
}
