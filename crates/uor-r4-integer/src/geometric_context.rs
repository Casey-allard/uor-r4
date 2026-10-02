//! Native geometric contextual state with a separate finite address readout.
//!
//! Up to two independent heads retain up to four signed H4 roots each. All
//! transitions select actions from token + OLD own + OLD next-neighbor tables,
//! then right-compose synchronously. Separate root and radius/presence readouts
//! use token + NEW own + NEW next-neighbor tables. Neighbor rings stay inside
//! each head. The readout is intentionally noninjective: emitted addresses are
//! not the complete recurrent state. In particular absence NEVER resets state.
//!
//! Root/action axes use stride128 with codes120..128 zero padded. Categories
//! use stride64: 0 is canonical absence, 1..=32 are present radius bins0..31;
//! categories33..64 are zero padded. All learned entries are signed Q24 i32
//! score values. Three-term sums widen to i64 before addition. These are not
//! four-bit linear weights. Artifact/source binding belongs to the enclosing
//! loader; group composition uses the independently admitted fixed H4 table.
//!
//! Admission allocates and prepartitions tables. State and step output use
//! fixed arrays of eight slots. Step/reset have no source floating arithmetic,
//! multiplication, division or allocation. Compiled instruction evidence and
//! whole-model serving qualification remain separate.

use std::fmt;

use crate::geometric_potential::{AddressLane, PotentialError};
use crate::h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT};

pub const FRACTIONAL_BITS: u32 = 24;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_HEADS: usize = 2;
pub const MAX_LANES_PER_HEAD: usize = 4;
pub const MAX_LANES: usize = 8;
pub const ROOT_STRIDE: usize = 128;
pub const CATEGORY_STRIDE: usize = 64;
pub const CATEGORY_COUNT: usize = 33;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ContextFamily {
    TokenTransition,
    SelfTransition,
    NeighborTransition,
    TokenRoot,
    SelfRoot,
    NeighborRoot,
    TokenCategory,
    SelfCategory,
    NeighborCategory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextError {
    InvalidVocabulary(usize),
    InvalidHeads(usize),
    InvalidLanes(usize),
    DimensionOverflow,
    TableLength {
        family: ContextFamily,
        expected: usize,
        actual: usize,
    },
    NonzeroPadding {
        family: ContextFamily,
        offset: usize,
    },
    MissingNeighbor(ContextFamily),
    NonzeroSingleLaneNeighbor {
        family: ContextFamily,
        offset: usize,
    },
    InvalidCode(u8),
    Address(PotentialError),
    TokenOutOfRange {
        token: usize,
        vocabulary: usize,
    },
    ShapeMismatch {
        state_heads: usize,
        state_lanes: usize,
        table_heads: usize,
        table_lanes: usize,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVocabulary(v) => write!(f, "native context vocabulary {v} is outside 1..={MAX_VOCAB}"),
            Self::InvalidHeads(h) => write!(f, "native context heads {h} are outside 1..={MAX_HEADS}"),
            Self::InvalidLanes(l) => write!(f, "native context lanes/head {l} are outside 1..={MAX_LANES_PER_HEAD}"),
            Self::DimensionOverflow => f.write_str("native context dimensions overflow"),
            Self::TableLength { family, expected, actual } => write!(f, "native context {family:?} needs {expected} entries, got {actual}"),
            Self::NonzeroPadding { family, offset } => write!(f, "native context {family:?} padding at {offset} is nonzero"),
            Self::MissingNeighbor(family) => write!(f, "native context needs {family:?} for multiple lanes/head"),
            Self::NonzeroSingleLaneNeighbor { family, offset } => write!(f, "single-lane native context {family:?} at {offset} is nonzero"),
            Self::InvalidCode(code) => write!(f, "native context code {code} is invalid"),
            Self::Address(error) => error.fmt(f),
            Self::TokenOutOfRange { token, vocabulary } => write!(f, "native context token {token} is outside 0..{vocabulary}"),
            Self::ShapeMismatch { state_heads, state_lanes, table_heads, table_lanes } => write!(f, "native context state {state_heads}x{state_lanes} differs from tables {table_heads}x{table_lanes}"),
        }
    }
}

impl std::error::Error for ContextError {}
pub type ContextResult<T> = Result<T, ContextError>;

/// Flattened head-major families. Token families are [V,H,L,stride]; other
/// families are [H,L,128,stride]. Neighbor families may be absent only for L1.
#[derive(Clone, Copy)]
pub struct ContextTableSlices<'a> {
    pub token_transition: &'a [i32],
    pub self_transition: &'a [i32],
    pub neighbor_transition: Option<&'a [i32]>,
    pub token_root: &'a [i32],
    pub self_root: &'a [i32],
    pub neighbor_root: Option<&'a [i32]>,
    pub token_category: &'a [i32],
    pub self_category: &'a [i32],
    pub neighbor_category: Option<&'a [i32]>,
}

/// Counts describe owned learned coefficients, excluding table metadata,
/// fixed group data and caller artifacts. Reads are logical inspections per
/// token, not measured cache traffic or energy.
///
/// Private descriptors use power-of-two strides for runtime indexing. On a
/// 64-bit target they occupy 64 bytes/token and 128 bytes/head-lane, compared
/// with the former 48 and 104 bytes. Thus the layout repair adds 16*vocabulary
/// + 24*(heads*lanes_per_head) bytes of descriptor storage (at most 65,728
/// bytes), excluded from stored_bytes. Coefficient/artifact bytes are unchanged;
/// allocator bookkeeping and alignment overhead are not included in this count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextTableStats {
    pub stored_entries: usize,
    pub stored_bytes: usize,
    pub transition_reads: usize,
    pub root_readout_reads: usize,
    pub category_readout_reads: usize,
    pub coefficient_reads: usize,
}

fn dimensions(heads: usize, lanes: usize) -> ContextResult<usize> {
    if !(1..=MAX_HEADS).contains(&heads) {
        return Err(ContextError::InvalidHeads(heads));
    }
    if !(1..=MAX_LANES_PER_HEAD).contains(&lanes) {
        return Err(ContextError::InvalidLanes(lanes));
    }
    heads
        .checked_mul(lanes)
        .ok_or(ContextError::DimensionOverflow)
}

fn validate_table(
    family: ContextFamily,
    values: &[i32],
    expected: usize,
    stride: usize,
    choices: usize,
    state_rows: bool,
) -> ContextResult<()> {
    if values.len() != expected {
        return Err(ContextError::TableLength {
            family,
            expected,
            actual: values.len(),
        });
    }
    for (row, values) in values.chunks_exact(stride).enumerate() {
        for (column, &value) in values.iter().enumerate() {
            if (column >= choices || (state_rows && row % ROOT_STRIDE >= ROOT_COUNT)) && value != 0
            {
                return Err(ContextError::NonzeroPadding {
                    family,
                    offset: row * stride + column,
                });
            }
        }
    }
    Ok(())
}

fn validate_neighbor(
    family: ContextFamily,
    values: Option<&[i32]>,
    expected: usize,
    stride: usize,
    choices: usize,
    lanes: usize,
) -> ContextResult<()> {
    match values {
        None if lanes > 1 => Err(ContextError::MissingNeighbor(family)),
        None => Ok(()),
        Some(values) => {
            validate_table(family, values, expected, stride, choices, true)?;
            if lanes == 1 {
                if let Some(offset) = values.iter().position(|&value| value != 0) {
                    return Err(ContextError::NonzeroSingleLaneNeighbor { family, offset });
                }
            }
            Ok(())
        }
    }
}

fn rows<const N: usize>(values: &[i32]) -> Box<[[i32; N]]> {
    values
        .chunks_exact(N)
        .map(|row| {
            let mut owned = [0; N];
            owned.copy_from_slice(row);
            owned
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

// Private admission metadata only: a 64-byte stride permits shift addressing
// instead of the observed M1 multiply for the former 48-byte descriptor.
#[repr(align(64))]
#[derive(Debug)]
struct TokenTables {
    transition: Box<[[i32; ROOT_STRIDE]]>,
    root: Box<[[i32; ROOT_STRIDE]]>,
    category: Box<[[i32; CATEGORY_STRIDE]]>,
}

#[derive(Debug)]
struct Factors {
    own: Box<[i32]>,
    neighbor: Option<Box<[i32]>>,
}

impl Factors {
    fn new(
        own: &[i32],
        neighbor: Option<&[i32]>,
        lane: usize,
        stride: usize,
        lanes: usize,
    ) -> Self {
        let count = ROOT_STRIDE * stride;
        let start = lane * count;
        Self {
            own: own[start..start + count].into(),
            neighbor: if lanes == 1 {
                None
            } else {
                neighbor.map(|values| values[start..start + count].into())
            },
        }
    }

    fn select(
        &self,
        token: &[i32],
        own: H4Code,
        neighbor: H4Code,
        shift: u32,
        choices: usize,
    ) -> usize {
        let start = usize::from(own.index()) << shift;
        let own = &self.own[start..start + choices];
        let start = usize::from(neighbor.index()) << shift;
        let neighbor = self
            .neighbor
            .as_ref()
            .map(|values| &values[start..start + choices]);
        let mut winner = 0;
        let mut best = i64::MIN;
        for choice in 0..choices {
            let mut score = i64::from(token[choice]) + i64::from(own[choice]);
            if let Some(neighbor) = neighbor {
                score += i64::from(neighbor[choice]);
            }
            if score > best {
                best = score;
                winner = choice;
            }
        }
        winner
    }
}

// The former 104-byte descriptor produced MADD during lane iteration. Padding
// this private metadata to a power-of-two stride changes no numerical table.
#[repr(align(128))]
#[derive(Debug)]
struct LaneTables {
    transition: Factors,
    root: Factors,
    category: Factors,
    neighbor_index: usize,
}

#[derive(Debug)]
pub struct NativeContextTables {
    tokens: Box<[TokenTables]>,
    lanes: Box<[LaneTables]>,
    heads: usize,
    lanes_per_head: usize,
    codes: [H4Code; ROOT_COUNT],
    absent: AddressLane,
    stats: ContextTableStats,
}

impl NativeContextTables {
    pub fn new(
        vocab: usize,
        heads: usize,
        lanes_per_head: usize,
        data: ContextTableSlices<'_>,
    ) -> ContextResult<Self> {
        if !(1..=MAX_VOCAB).contains(&vocab) {
            return Err(ContextError::InvalidVocabulary(vocab));
        }
        let total = dimensions(heads, lanes_per_head)?;
        let token_root_width = total
            .checked_mul(ROOT_STRIDE)
            .ok_or(ContextError::DimensionOverflow)?;
        let token_category_width = total
            .checked_mul(CATEGORY_STRIDE)
            .ok_or(ContextError::DimensionOverflow)?;
        let token_roots = vocab
            .checked_mul(token_root_width)
            .ok_or(ContextError::DimensionOverflow)?;
        let token_categories = vocab
            .checked_mul(token_category_width)
            .ok_or(ContextError::DimensionOverflow)?;
        let state_roots = total
            .checked_mul(ROOT_STRIDE * ROOT_STRIDE)
            .ok_or(ContextError::DimensionOverflow)?;
        let state_categories = total
            .checked_mul(ROOT_STRIDE * CATEGORY_STRIDE)
            .ok_or(ContextError::DimensionOverflow)?;
        for (family, values) in [
            (ContextFamily::TokenTransition, data.token_transition),
            (ContextFamily::TokenRoot, data.token_root),
        ] {
            validate_table(family, values, token_roots, ROOT_STRIDE, ROOT_COUNT, false)?;
        }
        for (family, values) in [
            (ContextFamily::SelfTransition, data.self_transition),
            (ContextFamily::SelfRoot, data.self_root),
        ] {
            validate_table(family, values, state_roots, ROOT_STRIDE, ROOT_COUNT, true)?;
        }
        for (family, values) in [
            (ContextFamily::NeighborTransition, data.neighbor_transition),
            (ContextFamily::NeighborRoot, data.neighbor_root),
        ] {
            validate_neighbor(
                family,
                values,
                state_roots,
                ROOT_STRIDE,
                ROOT_COUNT,
                lanes_per_head,
            )?;
        }
        validate_table(
            ContextFamily::TokenCategory,
            data.token_category,
            token_categories,
            CATEGORY_STRIDE,
            CATEGORY_COUNT,
            false,
        )?;
        validate_table(
            ContextFamily::SelfCategory,
            data.self_category,
            state_categories,
            CATEGORY_STRIDE,
            CATEGORY_COUNT,
            true,
        )?;
        validate_neighbor(
            ContextFamily::NeighborCategory,
            data.neighbor_category,
            state_categories,
            CATEGORY_STRIDE,
            CATEGORY_COUNT,
            lanes_per_head,
        )?;
        let mut tokens = Vec::with_capacity(vocab);
        for ((transition, root), category) in data
            .token_transition
            .chunks_exact(token_root_width)
            .zip(data.token_root.chunks_exact(token_root_width))
            .zip(data.token_category.chunks_exact(token_category_width))
        {
            tokens.push(TokenTables {
                transition: rows(transition),
                root: rows(root),
                category: rows(category),
            });
        }
        let mut lanes = Vec::with_capacity(total);
        for lane in 0..total {
            let within_head = lane % lanes_per_head;
            let neighbor_index = if within_head + 1 == lanes_per_head {
                lane - within_head
            } else {
                lane + 1
            };
            lanes.push(LaneTables {
                transition: Factors::new(
                    data.self_transition,
                    data.neighbor_transition,
                    lane,
                    ROOT_STRIDE,
                    lanes_per_head,
                ),
                root: Factors::new(
                    data.self_root,
                    data.neighbor_root,
                    lane,
                    ROOT_STRIDE,
                    lanes_per_head,
                ),
                category: Factors::new(
                    data.self_category,
                    data.neighbor_category,
                    lane,
                    CATEGORY_STRIDE,
                    lanes_per_head,
                ),
                neighbor_index,
            });
        }
        let mut codes = [H4Code::IDENTITY; ROOT_COUNT];
        for (i, code) in codes.iter_mut().enumerate() {
            *code = H4Code::try_from(i as u8).map_err(|_| ContextError::InvalidCode(i as u8))?;
        }
        let neighbor_families = usize::from(lanes_per_head > 1);
        let stored_entries = token_roots
            .checked_mul(2)
            .and_then(|n| n.checked_add(token_categories))
            .and_then(|n| {
                n.checked_add((state_roots * 2 + state_categories) * (1 + neighbor_families))
            })
            .ok_or(ContextError::DimensionOverflow)?;
        let stored_bytes = stored_entries
            .checked_mul(std::mem::size_of::<i32>())
            .ok_or(ContextError::DimensionOverflow)?;
        let transition_reads = total * ROOT_COUNT * (2 + neighbor_families);
        let root_readout_reads = transition_reads;
        let category_readout_reads = total * CATEGORY_COUNT * (2 + neighbor_families);
        Ok(Self {
            tokens: tokens.into_boxed_slice(),
            lanes: lanes.into_boxed_slice(),
            heads,
            lanes_per_head,
            codes,
            absent: AddressLane::new(1, 0, false).map_err(ContextError::Address)?,
            stats: ContextTableStats {
                stored_entries,
                stored_bytes,
                transition_reads,
                root_readout_reads,
                category_readout_reads,
                coefficient_reads: transition_reads + root_readout_reads + category_readout_reads,
            },
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.tokens.len()
    }
    pub fn heads(&self) -> usize {
        self.heads
    }
    pub fn lanes_per_head(&self) -> usize {
        self.lanes_per_head
    }
    pub fn stats(&self) -> ContextTableStats {
        self.stats
    }
}

/// Active prefixes are head-major and have length heads*lanes_per_head.
/// Inactive codes are identity and inactive outputs canonical absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextStep {
    pub heads: usize,
    pub lanes_per_head: usize,
    pub states: [H4Code; MAX_LANES],
    pub actions: [H4Code; MAX_LANES],
    pub readout_roots: [H4Code; MAX_LANES],
    pub categories: [u8; MAX_LANES],
    pub output: [AddressLane; MAX_LANES],
    pub coefficient_reads: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeContextState {
    states: [H4Code; MAX_LANES],
    heads: usize,
    lanes_per_head: usize,
    total: usize,
}

impl NativeContextState {
    pub fn new(heads: usize, lanes_per_head: usize) -> ContextResult<Self> {
        let total = dimensions(heads, lanes_per_head)?;
        Ok(Self {
            states: [H4Code::IDENTITY; MAX_LANES],
            heads,
            lanes_per_head,
            total,
        })
    }

    pub fn states(&self) -> &[H4Code] {
        &self.states[..self.total]
    }
    pub fn reset(&mut self) {
        self.states.fill(H4Code::IDENTITY);
    }

    /// OLD states drive synchronous transitions. NEW states drive both
    /// noninjective readouts. Absence is emitted only and cannot erase memory.
    /// All errors are returned before committing any internal state.
    pub fn step(
        &mut self,
        token_id: usize,
        tables: &NativeContextTables,
        geometry: &HistoricalH4Tables,
    ) -> ContextResult<ContextStep> {
        if self.heads != tables.heads || self.lanes_per_head != tables.lanes_per_head {
            return Err(ContextError::ShapeMismatch {
                state_heads: self.heads,
                state_lanes: self.lanes_per_head,
                table_heads: tables.heads,
                table_lanes: tables.lanes_per_head,
            });
        }
        let token = tables
            .tokens
            .get(token_id)
            .ok_or(ContextError::TokenOutOfRange {
                token: token_id,
                vocabulary: tables.tokens.len(),
            })?;
        let old = self.states;
        let mut next = old;
        let mut actions = [H4Code::IDENTITY; MAX_LANES];
        for (index, (token, lane)) in token.transition.iter().zip(tables.lanes.iter()).enumerate() {
            let selected =
                lane.transition
                    .select(token, old[index], old[lane.neighbor_index], 7, ROOT_COUNT);
            actions[index] = tables.codes[selected];
            next[index] = geometry.compose(old[index], actions[index]);
        }
        let mut readout_roots = [H4Code::IDENTITY; MAX_LANES];
        let mut categories = [0; MAX_LANES];
        let mut output = [tables.absent; MAX_LANES];
        for (index, ((root, category), lane)) in token
            .root
            .iter()
            .zip(token.category.iter())
            .zip(tables.lanes.iter())
            .enumerate()
        {
            let selected =
                lane.root
                    .select(root, next[index], next[lane.neighbor_index], 7, ROOT_COUNT);
            readout_roots[index] = tables.codes[selected];
            let category = lane.category.select(
                category,
                next[index],
                next[lane.neighbor_index],
                6,
                CATEGORY_COUNT,
            );
            categories[index] = category as u8;
            if category != 0 {
                output[index] =
                    AddressLane::new(readout_roots[index].index(), (category - 1) as u8, true)
                        .map_err(ContextError::Address)?;
            }
        }
        self.states = next;
        Ok(ContextStep {
            heads: self.heads,
            lanes_per_head: self.lanes_per_head,
            states: next,
            actions,
            readout_roots,
            categories,
            output,
            coefficient_reads: tables.stats.coefficient_reads,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn native_geometric_context_descriptor_strides_are_power_of_two() {
        use std::mem::{align_of, size_of};
        assert!(size_of::<TokenTables>().is_power_of_two());
        assert!(size_of::<LaneTables>().is_power_of_two());
        assert_eq!(align_of::<TokenTables>(), 64);
        assert_eq!(align_of::<LaneTables>(), 128);
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(size_of::<TokenTables>(), 64);
            assert_eq!(size_of::<LaneTables>(), 128);
            assert_eq!(
                (size_of::<TokenTables>() - 48) * MAX_VOCAB
                    + (size_of::<LaneTables>() - 104) * MAX_LANES,
                65_728
            );
        }
    }

    fn geometry() -> Result<HistoricalH4Tables, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))?;
        Ok(HistoricalH4Tables::from_bytes(&bytes)?)
    }

    struct Fixture {
        vocab: usize,
        heads: usize,
        lanes: usize,
        data: [Vec<i32>; 9],
    }
    impl Fixture {
        fn new(vocab: usize, heads: usize, lanes: usize) -> Self {
            let total = heads * lanes;
            Self {
                vocab,
                heads,
                lanes,
                data: std::array::from_fn(|family| {
                    let stride = if family < 6 {
                        ROOT_STRIDE
                    } else {
                        CATEGORY_STRIDE
                    };
                    vec![0; total * stride * if family % 3 == 0 { vocab } else { ROOT_STRIDE }]
                }),
            }
        }
        fn slices(&self) -> ContextTableSlices<'_> {
            ContextTableSlices {
                token_transition: &self.data[0],
                self_transition: &self.data[1],
                neighbor_transition: (self.lanes > 1).then_some(self.data[2].as_slice()),
                token_root: &self.data[3],
                self_root: &self.data[4],
                neighbor_root: (self.lanes > 1).then_some(self.data[5].as_slice()),
                token_category: &self.data[6],
                self_category: &self.data[7],
                neighbor_category: (self.lanes > 1).then_some(self.data[8].as_slice()),
            }
        }
        fn tables(&self) -> ContextResult<NativeContextTables> {
            NativeContextTables::new(self.vocab, self.heads, self.lanes, self.slices())
        }
    }

    #[test]
    fn native_geometric_context_noninjective_output_preserves_different_continuations() -> TestResult
    {
        let geometry = geometry()?;
        let mut f = Fixture::new(3, 1, 1);
        f.data[0][3] = 100;
        f.data[0][ROOT_STRIDE + 5] = 100;
        f.data[0][2 * ROOT_STRIDE + 1] = 100;
        f.data[3][1] = 100;
        f.data[3][ROOT_STRIDE + 1] = 100;
        f.data[4][3 * ROOT_STRIDE + 7] = 20;
        f.data[4][5 * ROOT_STRIDE + 6] = 20;
        for token in 0..3 {
            f.data[6][token * CATEGORY_STRIDE + 17] = 100;
        }
        let tables = f.tables()?;
        let mut a = NativeContextState::new(1, 1)?;
        let mut b = NativeContextState::new(1, 1)?;
        let first_a = a.step(0, &tables, &geometry)?;
        let first_b = b.step(1, &tables, &geometry)?;
        assert_eq!(first_a.output[0], first_b.output[0]);
        assert_eq!(first_a.output[0], AddressLane::new(1, 16, true)?);
        assert_ne!(first_a.states[0], first_b.states[0]);
        let next_a = a.step(2, &tables, &geometry)?;
        let next_b = b.step(2, &tables, &geometry)?;
        assert_eq!((next_a.output[0].root(), next_b.output[0].root()), (7, 6));
        assert_ne!(next_a.output[0], next_b.output[0]);
        assert_eq!(next_a.coefficient_reads, 546);
        Ok(())
    }

    #[test]
    fn native_geometric_context_head_local_synchronous_updates_and_absence() -> TestResult {
        let geometry = geometry()?;
        let mut f = Fixture::new(3, 2, 2);
        for (lane, action) in [3, 5, 5, 3].into_iter().enumerate() {
            f.data[0][lane * ROOT_STRIDE + action] = 100;
        }
        for (lane, (neighbor, action)) in [(5, 5), (3, 3), (3, 3), (5, 5)].into_iter().enumerate() {
            f.data[2][lane * ROOT_STRIDE * ROOT_STRIDE + neighbor * ROOT_STRIDE + action] = 10;
        }
        // Lane1 must use OLD lane0 (i), not NEW lane0 (k), and must not
        // cross into head1's old lane2 (j).
        for wrong in [7, 5] {
            f.data[2][ROOT_STRIDE * ROOT_STRIDE + wrong * ROOT_STRIDE + 1] = 20;
        }
        // Readouts use the completed NEW tuple, unlike transition factors.
        for (lane, (neighbor, root)) in [(6, 3), (7, 5), (7, 7), (6, 1)].into_iter().enumerate() {
            f.data[5][lane * ROOT_STRIDE * ROOT_STRIDE + neighbor * ROOT_STRIDE + root] = 10;
        }
        for lane in 0..4 {
            for neighbor in [3, 5] {
                f.data[8][lane * ROOT_STRIDE * CATEGORY_STRIDE + neighbor * CATEGORY_STRIDE + 17] =
                    10;
            }
            f.data[0][(2 * 4 + lane) * ROOT_STRIDE + 5] = 100;
            f.data[6][(2 * 4 + lane) * CATEGORY_STRIDE + 32] = 100;
        }
        let tables = f.tables()?;
        let mut state = NativeContextState::new(2, 2)?;
        let first = state.step(0, &tables, &geometry)?;
        assert_eq!(
            first.states[..4]
                .iter()
                .map(|x| x.index())
                .collect::<Vec<_>>(),
            [3, 5, 5, 3]
        );
        assert_eq!(&first.categories[..4], &[17; 4]);
        let absent = state.step(1, &tables, &geometry)?;
        assert_eq!(
            absent.states[..4]
                .iter()
                .map(|x| x.index())
                .collect::<Vec<_>>(),
            [7, 6, 6, 7]
        );
        assert_eq!(
            absent.readout_roots[..4]
                .iter()
                .map(|x| x.index())
                .collect::<Vec<_>>(),
            [3, 5, 7, 1]
        );
        assert!(absent.output[..4]
            .iter()
            .all(|x| !x.present() && x.root() == 1 && x.radius_bin() == 0));
        assert_eq!(state.states(), &absent.states[..4]);
        let next = state.step(2, &tables, &geometry)?;
        assert_eq!(
            next.states[..4]
                .iter()
                .map(|x| x.index())
                .collect::<Vec<_>>(),
            [2, 3, 3, 2]
        );
        assert!(next.output[..4]
            .iter()
            .all(|x| x.present() && x.radius_bin() == 31));
        assert_eq!(next.coefficient_reads, 3276);
        Ok(())
    }

    #[test]
    fn native_geometric_context_admission_and_atomic_errors() -> TestResult {
        let geometry = geometry()?;
        let mut f = Fixture::new(1, 1, 2);
        for vocab in [0, MAX_VOCAB + 1, usize::MAX] {
            assert!(matches!(
                NativeContextTables::new(vocab, 1, 2, f.slices()),
                Err(ContextError::InvalidVocabulary(_))
            ));
        }
        for (heads, lanes) in [(0, 1), (3, 1), (1, 0), (1, 5), (usize::MAX, usize::MAX)] {
            assert!(NativeContextState::new(heads, lanes).is_err());
        }
        for (family, offset) in [
            (0, 120),
            (1, 120 * ROOT_STRIDE),
            (2, 120),
            (3, 120),
            (4, 120 * ROOT_STRIDE),
            (5, 120),
            (6, 33),
            (7, 120 * CATEGORY_STRIDE),
            (8, 33),
        ] {
            f.data[family][offset] = 1;
            assert!(matches!(
                f.tables(),
                Err(ContextError::NonzeroPadding { .. })
            ));
            f.data[family][offset] = 0;
        }
        let mut missing = f.slices();
        missing.neighbor_root = None;
        assert!(matches!(
            NativeContextTables::new(1, 1, 2, missing),
            Err(ContextError::MissingNeighbor(ContextFamily::NeighborRoot))
        ));
        let mut short = f.slices();
        short.token_category = &f.data[6][..63];
        assert!(matches!(
            NativeContextTables::new(1, 1, 2, short),
            Err(ContextError::TableLength { .. })
        ));
        let tables = f.tables()?;
        let mut state = NativeContextState::new(1, 2)?;
        state.step(0, &tables, &geometry)?;
        let before = state;
        assert!(matches!(
            state.step(usize::MAX, &tables, &geometry),
            Err(ContextError::TokenOutOfRange { .. })
        ));
        assert_eq!(state, before);
        let mut wrong = NativeContextState::new(2, 1)?;
        let before = wrong;
        assert!(matches!(
            wrong.step(0, &tables, &geometry),
            Err(ContextError::ShapeMismatch { .. })
        ));
        assert_eq!(wrong, before);
        let mut single = Fixture::new(1, 1, 1);
        single.data[5][0] = 1;
        let mut data = single.slices();
        data.neighbor_root = Some(&single.data[5]);
        assert!(matches!(
            NativeContextTables::new(1, 1, 1, data),
            Err(ContextError::NonzeroSingleLaneNeighbor { .. })
        ));
        Ok(())
    }

    #[test]
    fn native_geometric_context_i64_extrema_ties_stats_and_fixed_state() -> TestResult {
        let geometry = geometry()?;
        for positive in [false, true] {
            let mut f = Fixture::new(1, 2, 4);
            for family in [0, 3, 6] {
                let stride = if family == 6 {
                    CATEGORY_STRIDE
                } else {
                    ROOT_STRIDE
                };
                let choices = if family == 6 {
                    CATEGORY_COUNT
                } else {
                    ROOT_COUNT
                };
                for row in f.data[family].chunks_exact_mut(stride) {
                    row[..choices].fill(if positive { 0 } else { -2 });
                    row[if positive && family == 6 { 32 } else { 0 }] =
                        if positive { i32::MAX } else { i32::MIN };
                    if !positive {
                        row[1] = -1;
                    }
                }
                for lane in f.data[family + 1].chunks_exact_mut(ROOT_STRIDE * stride) {
                    for row in lane.chunks_exact_mut(stride).take(ROOT_COUNT) {
                        row[if positive && family == 6 { 32 } else { 0 }] =
                            if positive { i32::MAX } else { i32::MIN };
                    }
                }
            }
            let tables = f.tables()?;
            let mut state = NativeContextState::new(2, 4)?;
            let step = state.step(0, &tables, &geometry)?;
            let expected = if positive { 0 } else { 1 };
            assert!(step
                .actions
                .iter()
                .chain(step.states.iter())
                .chain(step.readout_roots.iter())
                .all(|x| x.index() == expected));
            assert!(step
                .output
                .iter()
                .all(|x| x.present() && x.radius_bin() == if positive { 31 } else { 0 }));
            let stats = tables.stats();
            assert_eq!(stats.coefficient_reads, 6552);
            assert_eq!(step.coefficient_reads, 6552);
            assert_eq!(
                stats.stored_entries,
                f.data.iter().map(Vec::len).sum::<usize>()
            );
            assert_eq!(stats.stored_bytes, stats.stored_entries * 4);
            state.reset();
            assert_eq!(state.states(), &[H4Code::IDENTITY; 8]);
        }
        let zeros = Fixture::new(1, 1, 1).tables()?;
        let mut state = NativeContextState::new(1, 1)?;
        let tied = state.step(0, &zeros, &geometry)?;
        assert_eq!(tied.actions[0].index(), 0);
        assert_eq!(tied.readout_roots[0].index(), 0);
        assert_eq!(tied.categories[0], 0);
        assert!(!tied.output[0].present());
        assert_eq!(state.states()[0].index(), 0);
        fn assert_copy<T: Copy>() {}
        assert_copy::<NativeContextState>();
        assert!(!std::mem::needs_drop::<NativeContextState>());
        Ok(())
    }
}
