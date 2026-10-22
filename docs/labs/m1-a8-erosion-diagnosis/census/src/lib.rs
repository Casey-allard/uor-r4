//! Host-only placement census for the A8 dose arms (m1-a8-erosion-diagnosis).
//!
//! Rebuilds each token's address cell from a checkpoint table with the
//! kernel's `token_cell` rule (`discrete.cu:286-290`), with an explicit ABSENT.
//! A placement census is not admission: nothing here recomputes reach.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use uor_r4_discrete::normalization_mechanism::{orbit_root_coords, ORBIT_ROOTS};

/// Words per block.
pub const WORDS: usize = 8;
/// Blocks per token in the A8 dose tables.
pub const BLOCKS: usize = 4;

/// The 16 orbit roots (`geometry::ROOTS`) as i32 words, the kernel's comparison type.
pub fn roots() -> [[i32; WORDS]; ORBIT_ROOTS] {
    let mut out = [[0i32; WORDS]; ORBIT_ROOTS];
    for (r, slot) in out.iter_mut().enumerate() {
        *slot = orbit_root_coords(r as u8).expect("root index below 16");
    }
    out
}

/// Index of the root equal to `block`, or `None` if it is off the 16-root orbit.
pub fn root_of(block: &[i32]) -> Option<u8> {
    roots()
        .iter()
        .position(|r| r.as_slice() == block)
        .map(|r| r as u8)
}

/// Cell of token `t` in a table of `blocks` blocks: `root(block 0) * 16 + root(block B-1)`;
/// `None` (ABSENT) if either block is not a root.
pub fn token_cell(table: &[i32], blocks: usize, t: usize) -> Option<u8> {
    let width = blocks * WORDS;
    let base = t * width;
    let first = root_of(&table[base..base + WORDS])?;
    let last_at = base + (blocks - 1) * WORDS;
    let last = root_of(&table[last_at..last_at + WORDS])?;
    Some(first * 16 + last)
}

/// Every token's cell in a B = 4 table.
pub fn cells(table: &[i32]) -> Vec<Option<u8>> {
    (0..table.len() / (BLOCKS * WORDS))
        .map(|t| token_cell(table, BLOCKS, t))
        .collect()
}

/// Slots (8-word block indices) whose words differ between two equal-length tables.
pub fn changed_slots(a: &[i32], b: &[i32]) -> BTreeSet<u32> {
    assert_eq!(a.len(), b.len(), "tables differ in length");
    a.chunks_exact(WORDS)
        .zip(b.chunks_exact(WORDS))
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(slot, _)| slot as u32)
        .collect()
}

/// A `vector_block` slot's (token, block) with B = 4.
pub fn slot_token_block(slot: u32) -> (u32, u32) {
    (slot / BLOCKS as u32, slot % BLOCKS as u32)
}

/// SHA-256 of the words as i32 little-endian bytes (`CodebookTable::words_sha256`).
pub fn words_sha256(table: &[i32]) -> String {
    let mut h = Sha256::new();
    for w in table {
        h.update(w.to_le_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Placement summary of one cell map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occupancy {
    pub occupied: usize,
    pub max: usize,
    pub absent: usize,
    /// occupancy value -> number of occupied cells holding that many tokens
    pub histogram: BTreeMap<usize, usize>,
}

pub fn occupancy(cells: &[Option<u8>]) -> Occupancy {
    let mut load = [0usize; 256];
    let mut absent = 0;
    for c in cells {
        match c {
            Some(c) => load[usize::from(*c)] += 1,
            None => absent += 1,
        }
    }
    let mut histogram = BTreeMap::new();
    for &n in load.iter().filter(|&&n| n > 0) {
        *histogram.entry(n).or_insert(0) += 1;
    }
    Occupancy {
        occupied: load.iter().filter(|&&n| n > 0).count(),
        max: load.iter().copied().max().unwrap_or(0),
        absent,
        histogram,
    }
}

/// Size of the symmetric difference between the changed-slot set and the
/// accepted `vector_block` indices of one interval: 0 means the join holds.
pub fn join_mismatches(changed: &BTreeSet<u32>, accepted_vector: &BTreeSet<u32>) -> usize {
    changed.symmetric_difference(accepted_vector).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A B = 4 table whose tokens use the given root ids per block.
    fn table_of(tokens: &[[usize; BLOCKS]]) -> Vec<i32> {
        let r = roots();
        tokens
            .iter()
            .flat_map(|t| t.iter().flat_map(|&b| r[b].to_vec()))
            .collect()
    }

    #[test]
    fn roots_are_sixteen_distinct_blocks() {
        let r = roots();
        let set: BTreeSet<_> = r.iter().collect();
        assert_eq!(set.len(), 16);
        for (i, block) in r.iter().enumerate() {
            assert_eq!(root_of(block), Some(i as u8));
        }
    }

    #[test]
    fn cell_is_first_block_times_16_plus_last_block() {
        let table = table_of(&[[3, 5, 7, 9], [15, 0, 0, 1], [0, 2, 2, 0]]);
        assert_eq!(token_cell(&table, BLOCKS, 0), Some(3 * 16 + 9));
        assert_eq!(token_cell(&table, BLOCKS, 1), Some(15 * 16 + 1));
        assert_eq!(token_cell(&table, BLOCKS, 2), Some(0));
        assert_eq!(cells(&table), vec![Some(57), Some(241), Some(0)]);
    }

    #[test]
    fn cell_ignores_middle_blocks_but_reads_last_block_for_any_b() {
        let table = table_of(&[[4, 11, 12, 6]]);
        // B = 2 reading of the same words: token 0 = blocks (4, 11).
        assert_eq!(token_cell(&table, 2, 0), Some(4 * 16 + 11));
        let mut off_middle = table.clone();
        off_middle[WORDS + 3] += 1; // block 1 off orbit
        assert_eq!(token_cell(&off_middle, BLOCKS, 0), Some(4 * 16 + 6));
    }

    #[test]
    fn absent_when_block0_or_last_block_is_off_orbit() {
        let base = table_of(&[[3, 5, 7, 9]]);
        let mut first = base.clone();
        first[0] = 7; // block 0 word 0
        assert_eq!(root_of(&first[..WORDS]), None);
        assert_eq!(token_cell(&first, BLOCKS, 0), None);
        let mut last = base.clone();
        last[3 * WORDS + 7] = -9; // block 3 word 7
        assert_eq!(token_cell(&last, BLOCKS, 0), None);
        let occ = occupancy(&cells(&last));
        assert_eq!((occ.occupied, occ.absent), (0, 1));
    }

    #[test]
    fn slot_maps_to_token_and_block() {
        assert_eq!(slot_token_block(0), (0, 0));
        assert_eq!(slot_token_block(7 * 4 + 2), (7, 2));
        assert_eq!(slot_token_block(4095 * 4 + 3), (4095, 3));
    }

    #[test]
    fn changed_slots_join_accepted_vector_indices() {
        let a = table_of(&[[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11]]);
        let mut b = a.clone();
        b[2 * WORDS] = b[2 * WORDS].wrapping_add(1); // slot 2 = token 0 block 2
        b[9 * WORDS + 5] = 42; // slot 9 = token 2 block 1
        let changed = changed_slots(&a, &b);
        assert_eq!(changed, BTreeSet::from([2, 9]));
        assert_eq!(join_mismatches(&changed, &BTreeSet::from([2, 9])), 0);
        assert_eq!(join_mismatches(&changed, &BTreeSet::from([2])), 1);
        assert_eq!(join_mismatches(&changed, &BTreeSet::from([2, 9, 11])), 1);
        assert!(changed_slots(&a, &a).is_empty());
    }

    #[test]
    fn occupancy_histogram_counts_cells_by_load() {
        let table = table_of(&[[1, 0, 0, 2], [1, 3, 3, 2], [1, 0, 0, 2], [5, 0, 0, 5]]);
        let occ = occupancy(&cells(&table));
        assert_eq!(occ.occupied, 2);
        assert_eq!(occ.max, 3);
        assert_eq!(occ.absent, 0);
        assert_eq!(occ.histogram, BTreeMap::from([(1, 1), (3, 1)]));
    }

    #[test]
    fn words_digest_matches_codebook_digest() {
        assert_eq!(
            words_sha256(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let table = table_of(&[[3, 5, 7, 9], [1, 2, 3, 4]]);
        assert_eq!(
            words_sha256(&table),
            uor_r4_discrete::training::CodebookTable::words_sha256(&table)
        );
    }
}
