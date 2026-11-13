//! The address table of a served stack (claude-366).
//!
//! A float trunk trained with `--address-memory` learns one table of rows a layer. After the
//! layer's Hamilton block, each position adds one row for every 8-block of its residual. The
//! block names its row: by the nearest of E8's 240 roots to the block (the `e8` arm), or by a
//! fixed hash of the position's input token, the layer and the block (the `token` arm). This
//! module is that read in integers, on the residual stream's own 2^-16 grid, and the file the
//! float trunk's rows travel in from its checkpoint to the exporter.
//!
//! The cell of a block follows the trainer's rule (`uor-r4-float-train`, `e8_address.rs`) step
//! for step, ties included. With `m1 >= ... >= m8` the block's magnitudes and `S` their sum, the
//! best root with two entries of +-1 scores `m1 + m2` and the best root of eight halves scores
//! `S/2 - odd * m8`, where `odd` says the block has an odd number of minus signs. The first wins
//! a tie. The trainer compares in 32-bit floats; the same compare here is `2 (m1 + m2) >= S -
//! 2 odd m8` in whole numbers. On the same block the two agree wherever the trainer's 32-bit
//! sum of the eight magnitudes is itself exact, which holds while they sum to under 2^24 steps
//! (256 in the stream's units); past that the trainer's sum rounds, and a block within that
//! rounding of a tie can be named differently here. The trainer's stream is also not held to
//! this reader's steps, so a block near a tie can differ for that reason too. The row check on
//! a trained model measures what both cost.
//!
//! The module is shared by both builds of this crate: the float trainer's snapshot tool writes
//! the rows file with it, and the served stack (`stack::address`) reads the table with it.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Coordinates of one address block: E8's dimension.
pub const ADDRESS_BLOCK: usize = 8;
/// Cells a block chooses among: E8's 240 roots.
pub const ADDRESS_CELLS: usize = 240;
/// The roots with two entries of +-1 come first; the 128 roots of eight halves follow them.
const INTEGER_CELLS: u32 = 112;
/// Exponent of a stored row value: the residual stream's.
pub const ADDRESS_ROW_EXP: i32 = -16;
/// A stored row value's bound in steps: the trainer holds every row inside it.
pub const ADDRESS_ROW_STEPS: i16 = 32_767;
/// Widest trunk an address table serves; a position's rows then sum inside 32 bits.
pub(crate) const MAX_WIDTH: usize = 4_096;
/// The first bytes of a rows file.
const ROWS_MAGIC: &[u8; 8] = b"UORADR01";
/// Bytes of a rows file before its values: the magic, then five 32-bit words (arm, the seed's
/// high and low halves, the width, the layer count).
const ROWS_HEADER: usize = 28;

/// How a block names its row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressArm {
    /// The nearest of E8's 240 roots to the block.
    E8,
    /// A fixed hash of the seed, the layer, the block and the position's input token.
    Token,
}

impl AddressArm {
    /// The arm's code in a rows file and in the trainer's checkpoint.
    fn code(self) -> u32 {
        match self {
            Self::E8 => 0,
            Self::Token => 3,
        }
    }

    fn from_code(code: u32) -> Result<Self, String> {
        match code {
            0 => Ok(Self::E8),
            3 => Ok(Self::Token),
            1 | 2 => Err(format!("address arm {code} (sign or collapse) has no served form")),
            _ => Err(format!("an address table of unknown arm {code}")),
        }
    }

    /// `e8` or `token`, the trainer's names.
    pub fn name(self) -> &'static str {
        match self {
            Self::E8 => "e8",
            Self::Token => "token",
        }
    }
}

/// The header record of a stack that serves an address table, under
/// [`super::STACK_FBS_ADDRESS_SCHEMA`]. Layer `l`'s rows are the table `address.l{l}`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StackAddress {
    pub arm: AddressArm,
    /// Coordinates a block: 8.
    pub block: usize,
    /// Cells a block: 240.
    pub cells: usize,
    /// Layers that hold a table: every layer of the stack.
    pub layers: usize,
    /// Exponent of a row value: -16.
    pub row_exp: i32,
    /// The key of the token arm's hash, high half then low half. The `e8` arm does not read it.
    pub seed: [u32; 2],
}

impl StackAddress {
    /// The 64-bit seed.
    pub fn seed(&self) -> u64 {
        (u64::from(self.seed[0]) << 32) | u64::from(self.seed[1])
    }
}

/// The snapshot JSON's key for the address table's note (claude-366). A snapshot taken from an
/// address checkpoint carries the note, and the rows themselves travel in the rows file written
/// beside it. `FbsTrunkSnapshot` refuses unknown fields, so an exporter that does not know the
/// key refuses the snapshot instead of serving the model without its rows.
pub const FBS_ADDRESS_KEY: &str = "address_rows";

/// What a snapshot says of the rows file written beside it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FbsAddressNote {
    /// The table's record, as the artifact's header will carry it.
    pub record: StackAddress,
    /// The rows' width.
    pub width: usize,
    /// SHA-256 of the rows file, in hex.
    pub sha256: String,
}

impl FbsAddressNote {
    /// The table the rows file `bytes` holds, refused unless it is the file this note names:
    /// the same bytes by SHA-256, and the same record and width.
    pub fn rows(&self, bytes: &[u8]) -> Result<AddressImage, String> {
        let sum = hex::encode(Sha256::digest(bytes));
        if sum != self.sha256 {
            return Err(format!("the rows file's SHA-256 is {sum}; the snapshot names {}", self.sha256));
        }
        let image = AddressImage::from_bytes(bytes)?;
        if image.record() != self.record || image.width() != self.width {
            return Err("the rows file is not the table the snapshot describes".to_owned());
        }
        Ok(image)
    }
}

/// A stack's address table: what the artifact holds, and what a rows file holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressImage {
    arm: AddressArm,
    seed: u64,
    width: usize,
    /// A layer: `(width / 8) * 240` rows of `width` values at 2^-16; block `b`'s cell `c` is row
    /// `b * 240 + c`.
    rows: Vec<Vec<i16>>,
}

impl AddressImage {
    /// The table, refused unless every layer has `(width / 8) * 240` rows of `width` values, all
    /// inside the trainer's bound.
    pub fn checked(arm: AddressArm, seed: u64, width: usize, rows: Vec<Vec<i16>>) -> Result<Self, String> {
        if width == 0 || width % ADDRESS_BLOCK != 0 || width > MAX_WIDTH {
            return Err(format!("an address table needs a width in whole blocks of 8 up to {MAX_WIDTH}, not {width}"));
        }
        if rows.is_empty() {
            return Err("an address table with no layer".to_owned());
        }
        let values = width / ADDRESS_BLOCK * ADDRESS_CELLS * width;
        for (layer, table) in rows.iter().enumerate() {
            if table.len() != values {
                return Err(format!("address layer {layer} holds {} values, not {values}", table.len()));
            }
            if table.iter().any(|v| *v < -ADDRESS_ROW_STEPS) {
                return Err(format!("address layer {layer} holds a value under the trainer's bound"));
            }
        }
        Ok(Self { arm, seed, width, rows })
    }

    /// The address arm.
    pub fn arm(&self) -> AddressArm {
        self.arm
    }

    /// The token arm's hash key.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Coordinates of the residual the rows are added to.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Layers that hold a table.
    pub fn layers(&self) -> usize {
        self.rows.len()
    }

    /// Layer `layer`'s rows.
    pub fn layer(&self, layer: usize) -> &[i16] {
        &self.rows[layer]
    }

    /// The header record for this table.
    pub fn record(&self) -> StackAddress {
        StackAddress {
            arm: self.arm,
            block: ADDRESS_BLOCK,
            cells: ADDRESS_CELLS,
            layers: self.rows.len(),
            row_exp: ADDRESS_ROW_EXP,
            seed: [(self.seed >> 32) as u32, self.seed as u32],
        }
    }

    /// The rows file: the magic, the arm, the seed, the width and the layer count as 32-bit
    /// little-endian words, then every layer's values as 16-bit little-endian, in order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let values: usize = self.rows.iter().map(Vec::len).sum();
        let mut out = Vec::with_capacity(ROWS_HEADER + 2 * values);
        out.extend_from_slice(ROWS_MAGIC);
        let words = [self.arm.code(), (self.seed >> 32) as u32, self.seed as u32, self.width as u32, self.rows.len() as u32];
        for word in words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        for table in &self.rows {
            for value in table {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        out
    }

    /// The table a rows file holds. A file of another length than its header names is refused.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < ROWS_HEADER || &bytes[..8] != ROWS_MAGIC {
            return Err("not an address rows file".to_owned());
        }
        let word = |i: usize| {
            let at = 8 + 4 * i;
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        let arm = AddressArm::from_code(word(0))?;
        let seed = (u64::from(word(1)) << 32) | u64::from(word(2));
        let (width, layers) = (word(3) as usize, word(4) as usize);
        if width == 0 || width % ADDRESS_BLOCK != 0 || width > MAX_WIDTH || layers == 0 || layers > 64 {
            return Err(format!("an address rows file of width {width} and {layers} layers"));
        }
        // The size is worked out with checked steps: on a 32-bit build (`serving-wasm`) a header
        // inside the bounds above can still name more bytes than an index holds.
        let values = width / ADDRESS_BLOCK * ADDRESS_CELLS * width;
        let named = values
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_mul(layers))
            .and_then(|bytes| bytes.checked_add(ROWS_HEADER))
            .ok_or_else(|| format!("an address rows file of width {width} and {layers} layers is past this build's reach"))?;
        if bytes.len() != named {
            return Err(format!("an address rows file of {} bytes, not the {named} its header names", bytes.len()));
        }
        let rows = bytes[ROWS_HEADER..]
            .chunks_exact(2 * values)
            .map(|table| table.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect())
            .collect();
        Self::checked(arm, seed, width, rows)
    }

    /// SHA-256 of the rows file, in hex: what a snapshot records of the rows it was taken with.
    pub fn sha256(&self) -> String {
        hex::encode(Sha256::digest(self.to_bytes()))
    }

    /// The note a snapshot carries for this table's rows file.
    pub fn note(&self) -> FbsAddressNote {
        FbsAddressNote {
            record: self.record(),
            width: self.width,
            sha256: self.sha256(),
        }
    }

    /// The served read at `layer`: every block of the residual `x` names a cell, from `x` as it
    /// stands before any row is added, and the position's rows are then added to `x`, held
    /// inside 32 bits as the residual's own rounding holds it. `token` is the position's input
    /// token. `sum` is scratch of at least `width` values. `layer` is one of the table's layers
    /// and `x` holds at least `width` values; the stack's reader checks both when it loads the
    /// table, and the read panics on a caller that breaks either.
    pub fn read(&self, layer: usize, token: u32, x: &mut [i32], sum: &mut [i32]) {
        let width = self.width;
        let table = &self.rows[layer];
        let x = &mut x[..width];
        let sum = &mut sum[..width];
        sum.fill(0);
        for (block, coords) in x.chunks_exact(ADDRESS_BLOCK).enumerate() {
            let cell = match self.arm {
                AddressArm::E8 => e8_cell(coords),
                AddressArm::Token => token_cell(self.seed, layer, block, token),
            } as usize;
            let start = (block * ADDRESS_CELLS + cell) * width;
            for (s, r) in sum.iter_mut().zip(&table[start..start + width]) {
                *s += i32::from(*r);
            }
        }
        for (x, s) in x.iter_mut().zip(sum.iter()) {
            *x = x.saturating_add(*s);
        }
    }
}

/// The cell of the nearest E8 root to an 8-block, by the trainer's rule. Equal magnitudes take
/// their places in coordinate order, a zero counts as plus, and the root with two entries wins a
/// tie with the root of eight halves.
///
/// Cells 0 to 111 are the roots with two entries of +-1: with `lo < hi` their coordinates, the
/// pair's number is `lo (15 - lo) / 2 + hi - lo - 1` and the cell `4 pair + 2 n_lo + n_hi`, an
/// `n` being 1 for an entry of -1. Cells 112 to 239 are the roots of eight halves: `112` plus
/// bit `i` for a minus at coordinate `i < 7`; the eighth sign makes the count of minus signs even.
pub fn e8_cell(block: &[i32]) -> u32 {
    let mut magnitude = [0i64; ADDRESS_BLOCK];
    let mut negative = [false; ADDRESS_BLOCK];
    for ((m, n), v) in magnitude.iter_mut().zip(negative.iter_mut()).zip(block) {
        *m = i64::from(*v).abs();
        *n = *v < 0;
    }
    // Each coordinate's place by magnitude, 0 the largest: the coordinates above it, and the
    // equal ones before it.
    let (mut first, mut second, mut eighth) = (0usize, 0usize, 0usize);
    for i in 0..ADDRESS_BLOCK {
        let place = (0..ADDRESS_BLOCK)
            .filter(|&j| magnitude[j] > magnitude[i] || (j < i && magnitude[j] == magnitude[i]))
            .count();
        match place {
            0 => first = i,
            1 => second = i,
            7 => eighth = i,
            _ => {}
        }
    }
    let total: i64 = magnitude.iter().sum();
    let odd = negative.iter().filter(|n| **n).count() % 2 == 1;
    let two_entries = 2 * (magnitude[first] + magnitude[second]);
    let halves = total - if odd { 2 * magnitude[eighth] } else { 0 };
    if two_entries >= halves {
        let (lo, hi) = if first < second { (first, second) } else { (second, first) };
        let pair = lo * (15 - lo) / 2 + hi - lo - 1;
        return (4 * pair + 2 * usize::from(negative[lo]) + usize::from(negative[hi])) as u32;
    }
    let mut cell = INTEGER_CELLS;
    for (i, n) in negative.iter().enumerate().take(ADDRESS_BLOCK - 1) {
        if *n {
            cell += 1 << i;
        }
    }
    // An odd count flips the smallest coordinate's sign. The eighth coordinate has no bit: its
    // sign is the one the even count fixes.
    if odd && eighth + 1 < ADDRESS_BLOCK {
        if negative[eighth] {
            cell -= 1 << eighth;
        } else {
            cell += 1 << eighth;
        }
    }
    cell
}

/// The cell of `block` at `layer` for the input token `token` under the token arm: the
/// trainer's hash of the seed, the layer, the block and the token, into 240 cells.
pub fn token_cell(seed: u64, layer: usize, block: usize, token: u32) -> u32 {
    let key = seed ^ ((layer as u64) << 48) ^ ((block as u64) << 32) ^ u64::from(token);
    (mix(key) % ADDRESS_CELLS as u64) as u32
}

/// splitmix64's step: a fixed scramble of 64 bits.
fn mix(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Blocks on the 2^-16 grid that sit on a wall or hold equal magnitudes, shared with the float
/// trainer's tests: its decode of each, in 32-bit floats, must name the cell [`e8_cell`] names.
pub const ADDRESS_SHARED_BLOCKS: [[i32; 8]; 12] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [65_536, 65_536, 65_536, 65_536, 0, 0, 0, 0],
    [196_608, -196_608, 196_608, 0, 0, 0, 0, 0],
    [327_680, 327_680, 327_680, 327_680, 327_680, 327_680, 327_680, -65_536],
    [-65_536, 327_680, 327_680, 327_680, 327_680, 327_680, 327_680, 327_680],
    [65_536, -327_680, 327_680, 327_680, 327_680, 327_680, 327_680, 327_680],
    [65_536, 39_322, 32_768, 3_277, 2_621, 1_966, 1_311, 655],
    [-1, 1, -1, 1, -1, 1, -1, 1],
    [131_072, 131_072, 65_536, 65_536, 65_536, 65_536, 0, 0],
    [-131_072, -131_072, -65_536, -65_536, -65_536, -65_536, -65_536, -65_536],
    [7, 0, 0, 0, 0, 0, 0, -7],
    [20_000, -20_000, 20_000, -20_000, 20_000, -20_000, 20_000, -20_001],
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Root `cell` with every entry doubled, so the halves are whole numbers.
    fn root(cell: usize) -> [i64; 8] {
        let mut r = [0i64; 8];
        if cell < INTEGER_CELLS as usize {
            let (pair, signs) = (cell / 4, cell % 4);
            let mut k = 0;
            for lo in 0..8 {
                for hi in lo + 1..8 {
                    if k == pair {
                        r[lo] = if signs & 2 != 0 { -2 } else { 2 };
                        r[hi] = if signs & 1 != 0 { -2 } else { 2 };
                    }
                    k += 1;
                }
            }
            return r;
        }
        let code = cell - INTEGER_CELLS as usize;
        let mut minus = 0;
        for (i, entry) in r.iter_mut().enumerate().take(7) {
            if (code >> i) & 1 == 1 {
                *entry = -1;
                minus += 1;
            } else {
                *entry = 1;
            }
        }
        r[7] = if minus % 2 == 1 { -1 } else { 1 };
        r
    }

    fn score(cell: usize, block: &[i32; 8]) -> i64 {
        root(cell).iter().zip(block).map(|(r, v)| r * i64::from(*v)).sum()
    }

    fn draw(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(*state)
    }

    #[test]
    fn the_240_cells_are_the_240_roots() {
        let mut seen = std::collections::HashSet::new();
        for cell in 0..ADDRESS_CELLS {
            let r = root(cell);
            // A root's doubled length squared is 8.
            assert_eq!(r.iter().map(|v| v * v).sum::<i64>(), 8, "cell {cell}");
            assert!(seen.insert(r), "cell {cell} repeats a root");
        }
    }

    #[test]
    fn the_cell_is_a_nearest_root_over_all_240() {
        let mut state = 366;
        let mut blocks: Vec<[i32; 8]> = ADDRESS_SHARED_BLOCKS.to_vec();
        for round in 0..4_000 {
            // Three scales: the whole 32-bit range, residual-sized values, and small values
            // where equal magnitudes and walls are common.
            let range = match round % 3 {
                0 => u64::from(u32::MAX),
                1 => 1 << 20,
                _ => 7,
            };
            let mut block = [0i32; 8];
            for v in &mut block {
                *v = ((draw(&mut state) % (range + 1)) as i64 - (range / 2) as i64) as i32;
            }
            blocks.push(block);
        }
        for block in &blocks {
            let cell = e8_cell(block) as usize;
            assert!(cell < ADDRESS_CELLS, "{block:?} names cell {cell}");
            let best = (0..ADDRESS_CELLS).map(|c| score(c, block)).max().unwrap();
            assert_eq!(score(cell, block), best, "{block:?} names cell {cell}, not a nearest root");
        }
    }

    #[test]
    fn a_tie_takes_the_trainers_side() {
        // A zero block: places in coordinate order, and the root with two entries wins the tie.
        assert_eq!(e8_cell(&[0; 8]), 0);
        // m1 + m2 equals S/2: the root on coordinates 0 and 1, not the root of eight halves.
        assert_eq!(e8_cell(&[1, 1, 1, 1, 0, 0, 0, 0]), 0);
        // One step past that wall, the root of eight halves, all plus.
        assert_eq!(e8_cell(&[1, 1, 1, 1, 1, 0, 0, 0]), 112);
        // Three equal magnitudes: coordinates 0 and 1 hold the first two places, and coordinate
        // 1's minus is the second entry's.
        assert_eq!(e8_cell(&[3, -3, 3, 0, 0, 0, 0, 0]), 1);
        // The same block with the equal magnitudes later: coordinates 5 and 6.
        assert_eq!(e8_cell(&[0, 0, 0, 0, 0, 3, -3, 3]), 4 * 25 + 1);
    }

    #[test]
    fn an_odd_count_flips_the_smallest_coordinate() {
        // The lone minus is the smallest coordinate, the eighth: it flips, and has no bit.
        assert_eq!(e8_cell(&[5, 5, 5, 5, 5, 5, 5, -1]), 112);
        // The lone minus is the smallest coordinate, the first: its bit is taken back.
        assert_eq!(e8_cell(&[-1, 5, 5, 5, 5, 5, 5, 5]), 112);
        // The smallest coordinate is plus and another holds the minus: both bits are set.
        assert_eq!(e8_cell(&[1, -5, 5, 5, 5, 5, 5, 5]), 112 + 1 + 2);
        // An even count flips nothing.
        assert_eq!(e8_cell(&[-5, -5, 5, 5, 5, 5, 5, 1]), 112 + 1 + 2);
        // Two coordinates tie for the smallest magnitude: equal magnitudes take their places in
        // coordinate order, so the later one is the eighth. It flips; the earlier keeps its sign.
        assert_eq!(e8_cell(&[-1, 1, 5, 5, 5, 5, 5, 5]), 112 + 1 + 2);
        assert_eq!(e8_cell(&[1, -1, 5, 5, 5, 5, 5, 5]), 112);
    }

    #[test]
    fn the_widest_values_do_not_overflow() {
        assert_eq!(e8_cell(&[i32::MIN; 8]), 112 + 127);
        assert_eq!(e8_cell(&[i32::MAX; 8]), 112);
        assert_eq!(e8_cell(&[i32::MIN, i32::MAX, 0, 0, 0, 0, 0, 0]), 2);
    }

    #[test]
    fn the_token_cell_is_the_trainers_hash() {
        // splitmix64's first value from a zero state.
        assert_eq!(mix(0), 0xE220_A839_7B1D_CDAF);
        let seed = 0x0123_4567_89AB_CDEF;
        let key = seed ^ (5u64 << 48) ^ (31u64 << 32) ^ 4_095;
        assert_eq!(u64::from(token_cell(seed, 5, 31, 4_095)), mix(key) % 240);
        // The seed, the layer, the block and the token each move the cell somewhere in a small
        // sweep: none of them is left out of the key.
        let base: Vec<u32> = (0..64).map(|t| token_cell(seed, 2, 3, t)).collect();
        assert!(base.iter().all(|c| (*c as usize) < ADDRESS_CELLS));
        assert_ne!(base, (0..64).map(|t| token_cell(seed + 1, 2, 3, t)).collect::<Vec<_>>());
        assert_ne!(base, (0..64).map(|t| token_cell(seed, 3, 3, t)).collect::<Vec<_>>());
        assert_ne!(base, (0..64).map(|t| token_cell(seed, 2, 4, t)).collect::<Vec<_>>());
        assert!(base.windows(2).any(|w| w[0] != w[1]));
    }

    /// A table of two layers at `width`, every value drawn from `seed` inside the bound.
    fn table(arm: AddressArm, width: usize, seed: u64) -> AddressImage {
        let mut state = seed;
        let values = width / ADDRESS_BLOCK * ADDRESS_CELLS * width;
        let rows = (0..2)
            .map(|_| (0..values).map(|_| (draw(&mut state) % 4_001) as i16 - 2_000).collect())
            .collect();
        AddressImage::checked(arm, 0xFEED_0000_0000_0366, width, rows).unwrap()
    }

    fn residual(width: usize, seed: u64) -> Vec<i32> {
        let mut state = seed;
        (0..width).map(|_| (draw(&mut state) % 400_001) as i32 - 200_000).collect()
    }

    #[test]
    fn the_read_adds_the_named_rows_and_no_other() {
        let width = 24;
        for arm in [AddressArm::E8, AddressArm::Token] {
            let image = table(arm, width, 11);
            for layer in 0..2 {
                let before = residual(width, 5 + layer as u64);
                let mut x = before.clone();
                let mut scratch = vec![-1; width];
                image.read(layer, 77, &mut x, &mut scratch);
                // Every block's cell from the residual before the read, then its row, by hand.
                let mut want = before.clone();
                for block in 0..width / ADDRESS_BLOCK {
                    let cell = match arm {
                        AddressArm::E8 => e8_cell(&before[block * 8..block * 8 + 8]),
                        AddressArm::Token => token_cell(image.seed(), layer, block, 77),
                    } as usize;
                    let start = (block * ADDRESS_CELLS + cell) * width;
                    for (w, r) in want.iter_mut().zip(&image.layer(layer)[start..start + width]) {
                        *w += i32::from(*r);
                    }
                }
                assert_eq!(x, want, "{arm:?} layer {layer}");
                assert_ne!(x, before, "{arm:?} layer {layer}: the rows moved nothing");
            }
        }
    }

    #[test]
    fn every_cell_is_named_before_any_row_is_added() {
        // Block 0's row pushes block 1 across a wall. Read in order, block 1 would name another
        // cell; the trainer names every cell first.
        let width = 16;
        let values = width / ADDRESS_BLOCK * ADDRESS_CELLS * width;
        let mut rows = vec![0i16; values];
        let before: Vec<i32> = [[9, 8, 0, 0, 0, 0, 0, 0], [9, 8, 0, 0, 0, 0, 0, 0]].concat();
        let cell0 = e8_cell(&before[..8]) as usize;
        let cell1 = e8_cell(&before[8..]) as usize;
        // Block 0's row adds 100 to block 1's third coordinate, which would then lead it.
        rows[cell0 * width + 10] = 100;
        // Block 1's rows: 1 at its cell before the push, 2 at its cell after.
        let mut pushed = before.clone();
        pushed[10] += 100;
        let moved = e8_cell(&pushed[8..]) as usize;
        assert_ne!(moved, cell1, "the push must change block 1's cell for this test to bite");
        rows[(ADDRESS_CELLS + cell1) * width] = 1;
        rows[(ADDRESS_CELLS + moved) * width] = 2;
        let image = AddressImage::checked(AddressArm::E8, 0, width, vec![rows]).unwrap();
        let mut x = before.clone();
        image.read(0, 0, &mut x, &mut vec![0; width]);
        assert_eq!(x[0] - before[0], 1, "block 1 read the row of its cell before the push");
    }

    #[test]
    fn the_read_holds_the_residual_inside_32_bits() {
        let width = 8;
        let rows = vec![ADDRESS_ROW_STEPS; ADDRESS_CELLS * width];
        let image = AddressImage::checked(AddressArm::Token, 1, width, vec![rows]).unwrap();
        let mut x = vec![i32::MAX - 5; width];
        image.read(0, 3, &mut x, &mut vec![0; width]);
        assert_eq!(x, vec![i32::MAX; width]);
    }

    #[test]
    fn a_rows_file_reads_back_and_a_cut_one_is_refused() {
        let image = table(AddressArm::Token, 16, 3);
        let bytes = image.to_bytes();
        assert_eq!(AddressImage::from_bytes(&bytes).unwrap(), image);
        assert_eq!(image.sha256().len(), 64);
        assert!(AddressImage::from_bytes(&bytes[..bytes.len() - 2]).is_err());
        let mut longer = bytes.clone();
        longer.extend_from_slice(&[0, 0]);
        assert!(AddressImage::from_bytes(&longer).is_err());
        let mut other = bytes.clone();
        other[0] ^= 1;
        assert!(AddressImage::from_bytes(&other).is_err());
        // The sign and collapse arms were trained as controls and have no served form.
        let mut sign = bytes.clone();
        sign[8] = 1;
        assert!(AddressImage::from_bytes(&sign).unwrap_err().contains("no served form"));
        // An arm code no trainer writes, no layers, and a width that is not whole blocks.
        let mut unknown = bytes.clone();
        unknown[8] = 7;
        assert!(AddressImage::from_bytes(&unknown).is_err());
        // Each file below is as long as its own header names, so only the header's rule refuses it.
        let mut none = bytes[..ROWS_HEADER].to_vec();
        none[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(AddressImage::from_bytes(&none).unwrap_err().contains("of width 16 and 0 layers"));
        let mut ragged = bytes[..ROWS_HEADER].to_vec();
        ragged[20..24].copy_from_slice(&12u32.to_le_bytes());
        ragged.resize(ROWS_HEADER + 2 * 2 * ADDRESS_CELLS * 12, 0);
        assert!(AddressImage::from_bytes(&ragged).unwrap_err().contains("of width 12 and 2 layers"));
    }

    #[test]
    fn a_table_of_the_wrong_size_or_past_the_bound_is_refused() {
        let good = vec![0i16; 2 * ADDRESS_CELLS * 16];
        assert!(AddressImage::checked(AddressArm::E8, 0, 16, vec![good.clone()]).is_ok());
        // A width that is not whole blocks, with a layer of the size that width would name: only
        // the width's rule refuses it.
        let ragged = vec![0i16; ADDRESS_CELLS * 12];
        assert!(AddressImage::checked(AddressArm::E8, 0, 12, vec![ragged]).unwrap_err().contains("whole blocks"));
        assert!(AddressImage::checked(AddressArm::E8, 0, 16, vec![good[1..].to_vec()]).is_err());
        assert!(AddressImage::checked(AddressArm::E8, 0, 16, Vec::new()).is_err());
        let mut low = good;
        low[5] = i16::MIN;
        assert!(AddressImage::checked(AddressArm::E8, 0, 16, vec![low]).is_err());
    }
}
