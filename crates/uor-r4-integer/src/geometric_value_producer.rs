//! Finite geometric K2 value production from selected token and H4 state rows.
//!
//! Four value lanes per head each choose two signed roots and zero/dyadic-radius
//! categories. Scores sum token, retained own state, next state within the same
//! head, and (only when supplied) held-span state plus span-valid bias. A value
//! lane uses latent lane `value_lane % latent_lanes_per_head`; this mapping is
//! precomputed at admission. Root IDs select exact table rows, not a distance.
//!
//! Q24 i32 table entries are four-byte scores, not four-bit linear weights.
//! Every sum has at most five terms and widens to i64. Category0 is PRESENT_ZERO;
//! categories1..31 select radius bins0..30. Invalid occurrences produce ABSENT
//! packets independently of span validity or numeric cancellation. The canonical
//! codec checks the two-atom Q16 sum before any output is returned.
//!
//! Admission validates/partitions immutable tables and allocates the canonical
//! decoder. The numerical producer allocates nothing and contains no floating,
//! multiply or divide arithmetic. Compiled opcode and full-model qualification
//! remain separate. The enclosing loader binds source/model/artifact identity.

use std::fmt;

use crate::geometric_value::{GeometricValueError, NativeGeometricValues, ValuePacket};
use crate::h4_tables::{H4Code, ROOT_COUNT};

pub const FRACTIONAL_BITS: u32 = 24;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_HEADS: usize = 2;
pub const MAX_LATENT_LANES_PER_HEAD: usize = 4;
pub const VALUE_LANES_PER_HEAD: usize = 4;
pub const ATOMS: usize = 2;
pub const MAX_VALUE_LANES: usize = 8;
pub const MAX_ATOMS: usize = 16;
pub const MAX_COORDINATES: usize = 32;
pub const ROOT_STRIDE: usize = 128;
pub const CATEGORY_COUNT: usize = 32;
pub const CATEGORY_STRIDE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueProducerTable {
    TokenRoot,
    OwnRoot,
    NeighborRoot,
    SpanRoot,
    SpanValidRoot,
    TokenCategory,
    OwnCategory,
    NeighborCategory,
    SpanCategory,
    SpanValidCategory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueProducerError {
    InvalidVocabulary(usize),
    InvalidHeads(usize),
    InvalidLatentLanes(usize),
    DimensionOverflow,
    TableLength {
        table: ValueProducerTable,
        expected: usize,
        actual: usize,
    },
    NonzeroPadding {
        table: ValueProducerTable,
        offset: usize,
    },
    MissingNeighbor(ValueProducerTable),
    NonzeroSingleLaneNeighbor {
        table: ValueProducerTable,
        offset: usize,
    },
    TokenOutOfRange {
        token: usize,
        vocabulary: usize,
    },
    LatentLength {
        expected: usize,
        actual: usize,
    },
    SpanLength {
        expected: usize,
        actual: usize,
    },
    Codec(GeometricValueError),
}
impl fmt::Display for ValueProducerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVocabulary(v) => {
                write!(f, "native value vocabulary {v} is outside 1..={MAX_VOCAB}")
            }
            Self::InvalidHeads(h) => {
                write!(f, "native value heads {h} are outside 1..={MAX_HEADS}")
            }
            Self::InvalidLatentLanes(l) => write!(
                f,
                "native value latent lanes/head {l} are outside 1..={MAX_LATENT_LANES_PER_HEAD}"
            ),
            Self::DimensionOverflow => f.write_str("native value table dimensions overflow"),
            Self::TableLength {
                table,
                expected,
                actual,
            } => write!(
                f,
                "native value {table:?} needs {expected} entries, got {actual}"
            ),
            Self::NonzeroPadding { table, offset } => {
                write!(f, "native value {table:?} padding at {offset} is nonzero")
            }
            Self::MissingNeighbor(table) => write!(
                f,
                "native value requires {table:?} for multiple latent lanes"
            ),
            Self::NonzeroSingleLaneNeighbor { table, offset } => write!(
                f,
                "single-lane native value {table:?} at {offset} is nonzero"
            ),
            Self::TokenOutOfRange { token, vocabulary } => {
                write!(f, "native value token {token} is outside 0..{vocabulary}")
            }
            Self::LatentLength { expected, actual } => write!(
                f,
                "native value needs {expected} retained roots, got {actual}"
            ),
            Self::SpanLength { expected, actual } => write!(
                f,
                "native value needs {expected} held-span roots, got {actual}"
            ),
            Self::Codec(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ValueProducerError {}
pub type ValueProducerResult<T> = Result<T, ValueProducerError>;

/// Root choices use stride128 (120 valid choices), categories use stride32.
/// Tokens: [V,H,4,2,stride]. State factors: [H,4,2,128,stride]. Validity bias:
/// [H,4,2,stride]. State rows120..127 and root columns120..127 must be zero.
/// Neighbor tables are required for L>1; L1 accepts None or an all-zero table.
#[derive(Clone, Copy)]
pub struct ValueProducerTableSlices<'a> {
    pub token_root: &'a [i32],
    pub own_root: &'a [i32],
    pub neighbor_root: Option<&'a [i32]>,
    pub span_root: &'a [i32],
    pub span_valid_root: &'a [i32],
    pub token_category: &'a [i32],
    pub own_category: &'a [i32],
    pub neighbor_category: Option<&'a [i32]>,
    pub span_category: &'a [i32],
    pub span_valid_category: &'a [i32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueProducerStats {
    pub stored_entries: usize,
    /// Learned i32 table payload only; excludes admission metadata, allocator
    /// overhead and the canonical value codec. Private atom descriptors use
    /// 256 bytes each on 64-bit targets (4096 bytes at the 16-atom maximum).
    pub stored_bytes: usize,
    /// Logical learned-coefficient inspections, not measured cache traffic.
    pub without_span_reads: usize,
    pub with_span_reads: usize,
}

/// Active prefixes are heads*4 packet pairs, heads*16 coordinates and heads*8
/// raw atom choices. Inactive tails are canonical ABSENT/zero. Root choices
/// retain argmax diagnostics even when category0 canonicalizes the packet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducedValues {
    pub heads: usize,
    pub occurrence_valid: bool,
    pub packets: [[ValuePacket; ATOMS]; MAX_VALUE_LANES],
    pub values_q16: [i32; MAX_COORDINATES],
    pub root_choices: [u8; MAX_ATOMS],
    pub categories: [u8; MAX_ATOMS],
    pub coefficient_reads: usize,
}

fn validate_table(
    table: ValueProducerTable,
    values: &[i32],
    expected: usize,
    stride: usize,
    choices: usize,
    state_rows: bool,
) -> ValueProducerResult<()> {
    if values.len() != expected {
        return Err(ValueProducerError::TableLength {
            table,
            expected,
            actual: values.len(),
        });
    }
    for (row, values) in values.chunks_exact(stride).enumerate() {
        for (column, &value) in values.iter().enumerate() {
            if (column >= choices || (state_rows && row % ROOT_STRIDE >= ROOT_COUNT)) && value != 0
            {
                return Err(ValueProducerError::NonzeroPadding {
                    table,
                    offset: row * stride + column,
                });
            }
        }
    }
    Ok(())
}

fn validate_neighbor(
    table: ValueProducerTable,
    values: Option<&[i32]>,
    expected: usize,
    stride: usize,
    choices: usize,
    lanes: usize,
) -> ValueProducerResult<()> {
    match values {
        None if lanes > 1 => Err(ValueProducerError::MissingNeighbor(table)),
        None => Ok(()),
        Some(values) => {
            validate_table(table, values, expected, stride, choices, true)?;
            if lanes == 1 {
                if let Some(offset) = values.iter().position(|&x| x != 0) {
                    return Err(ValueProducerError::NonzeroSingleLaneNeighbor { table, offset });
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
            let mut out = [0; N];
            out.copy_from_slice(row);
            out
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
struct TokenTables {
    roots: Box<[[i32; ROOT_STRIDE]]>,
    categories: Box<[[i32; CATEGORY_STRIDE]]>,
}
struct Factors {
    own: Box<[i32]>,
    neighbor: Option<Box<[i32]>>,
    span: Box<[i32]>,
    span_valid: Box<[i32]>,
}
impl Factors {
    fn select(
        &self,
        token: &[i32],
        own: H4Code,
        neighbor: H4Code,
        span: Option<H4Code>,
        shift: u32,
        choices: usize,
    ) -> u8 {
        let own_at = usize::from(own.index()) << shift;
        let neighbor_at = usize::from(neighbor.index()) << shift;
        let span_at = span.map(|root| usize::from(root.index()) << shift);
        let mut winner = 0;
        let mut best = i64::MIN;
        for (choice, &lexical) in token.iter().take(choices).enumerate() {
            let mut score = i64::from(lexical) + i64::from(self.own[own_at + choice]);
            if let Some(neighbor) = &self.neighbor {
                score += i64::from(neighbor[neighbor_at + choice]);
            }
            if let Some(span_at) = span_at {
                score += i64::from(self.span[span_at + choice]);
                score += i64::from(self.span_valid[choice]);
            }
            if score > best {
                best = score;
                winner = choice;
            }
        }
        winner as u8
    }
}
// Power-of-two array stride prevents an iterator-end pointer from requiring a
// multiply by the unpadded descriptor size. On 64-bit targets this rounds 144
// bytes to 256: at most 1792 extra bytes across the admitted 16 atoms.
#[repr(align(128))]
struct AtomTables {
    root: Factors,
    category: Factors,
    own_index: usize,
    neighbor_index: usize,
}

pub struct NativeValueProducer {
    tokens: Box<[TokenTables]>,
    atoms: Box<[AtomTables]>,
    heads: usize,
    latent_lanes_per_head: usize,
    latent_count: usize,
    codec: NativeGeometricValues,
    stats: ValueProducerStats,
}

impl NativeValueProducer {
    pub fn new(
        vocabulary: usize,
        heads: usize,
        latent_lanes_per_head: usize,
        data: ValueProducerTableSlices<'_>,
    ) -> ValueProducerResult<Self> {
        if !(1..=MAX_VOCAB).contains(&vocabulary) {
            return Err(ValueProducerError::InvalidVocabulary(vocabulary));
        }
        if !(1..=MAX_HEADS).contains(&heads) {
            return Err(ValueProducerError::InvalidHeads(heads));
        }
        if !(1..=MAX_LATENT_LANES_PER_HEAD).contains(&latent_lanes_per_head) {
            return Err(ValueProducerError::InvalidLatentLanes(
                latent_lanes_per_head,
            ));
        }
        let atom_count = heads * VALUE_LANES_PER_HEAD * ATOMS;
        let token_roots = vocabulary
            .checked_mul(atom_count * ROOT_STRIDE)
            .ok_or(ValueProducerError::DimensionOverflow)?;
        let token_categories = vocabulary
            .checked_mul(atom_count * CATEGORY_STRIDE)
            .ok_or(ValueProducerError::DimensionOverflow)?;
        let state_roots = atom_count * ROOT_STRIDE * ROOT_STRIDE;
        let state_categories = atom_count * ROOT_STRIDE * CATEGORY_STRIDE;
        for (table, values, count, state) in [
            (
                ValueProducerTable::TokenRoot,
                data.token_root,
                token_roots,
                false,
            ),
            (
                ValueProducerTable::OwnRoot,
                data.own_root,
                state_roots,
                true,
            ),
            (
                ValueProducerTable::SpanRoot,
                data.span_root,
                state_roots,
                true,
            ),
            (
                ValueProducerTable::SpanValidRoot,
                data.span_valid_root,
                atom_count * ROOT_STRIDE,
                false,
            ),
        ] {
            validate_table(table, values, count, ROOT_STRIDE, ROOT_COUNT, state)?;
        }
        for (table, values, count, state) in [
            (
                ValueProducerTable::TokenCategory,
                data.token_category,
                token_categories,
                false,
            ),
            (
                ValueProducerTable::OwnCategory,
                data.own_category,
                state_categories,
                true,
            ),
            (
                ValueProducerTable::SpanCategory,
                data.span_category,
                state_categories,
                true,
            ),
            (
                ValueProducerTable::SpanValidCategory,
                data.span_valid_category,
                atom_count * CATEGORY_STRIDE,
                false,
            ),
        ] {
            validate_table(table, values, count, CATEGORY_STRIDE, CATEGORY_COUNT, state)?;
        }
        validate_neighbor(
            ValueProducerTable::NeighborRoot,
            data.neighbor_root,
            state_roots,
            ROOT_STRIDE,
            ROOT_COUNT,
            latent_lanes_per_head,
        )?;
        validate_neighbor(
            ValueProducerTable::NeighborCategory,
            data.neighbor_category,
            state_categories,
            CATEGORY_STRIDE,
            CATEGORY_COUNT,
            latent_lanes_per_head,
        )?;
        let mut tokens = Vec::with_capacity(vocabulary);
        for (root, category) in data.token_root.chunks_exact(atom_count * ROOT_STRIDE).zip(
            data.token_category
                .chunks_exact(atom_count * CATEGORY_STRIDE),
        ) {
            tokens.push(TokenTables {
                roots: rows(root),
                categories: rows(category),
            });
        }
        let factor = |slot: usize,
                      stride: usize,
                      own: &[i32],
                      neighbor: Option<&[i32]>,
                      span: &[i32],
                      bias: &[i32]| {
            let count = ROOT_STRIDE * stride;
            let at = slot * count;
            Factors {
                own: own[at..at + count].into(),
                neighbor: if latent_lanes_per_head == 1 {
                    None
                } else {
                    neighbor.map(|table| table[at..at + count].into())
                },
                span: span[at..at + count].into(),
                span_valid: bias[slot * stride..(slot + 1) * stride].into(),
            }
        };
        let mut atoms = Vec::with_capacity(atom_count);
        for head in 0..heads {
            for lane in 0..VALUE_LANES_PER_HEAD {
                let own_lane = lane % latent_lanes_per_head;
                let own_index = head * latent_lanes_per_head + own_lane;
                let neighbor_index =
                    head * latent_lanes_per_head + (own_lane + 1) % latent_lanes_per_head;
                for atom in 0..ATOMS {
                    let slot = (head * VALUE_LANES_PER_HEAD + lane) * ATOMS + atom;
                    atoms.push(AtomTables {
                        root: factor(
                            slot,
                            ROOT_STRIDE,
                            data.own_root,
                            data.neighbor_root,
                            data.span_root,
                            data.span_valid_root,
                        ),
                        category: factor(
                            slot,
                            CATEGORY_STRIDE,
                            data.own_category,
                            data.neighbor_category,
                            data.span_category,
                            data.span_valid_category,
                        ),
                        own_index,
                        neighbor_index,
                    });
                }
            }
        }
        let neighbor_families = usize::from(latent_lanes_per_head > 1);
        let stored_entries = token_roots
            + token_categories
            + (state_roots + state_categories) * (2 + neighbor_families)
            + atom_count * (ROOT_STRIDE + CATEGORY_STRIDE);
        let choices = atom_count * (ROOT_COUNT + CATEGORY_COUNT);
        Ok(Self {
            tokens: tokens.into_boxed_slice(),
            atoms: atoms.into_boxed_slice(),
            heads,
            latent_lanes_per_head,
            latent_count: heads * latent_lanes_per_head,
            codec: NativeGeometricValues::canonical().map_err(ValueProducerError::Codec)?,
            stats: ValueProducerStats {
                stored_entries,
                stored_bytes: stored_entries * std::mem::size_of::<i32>(),
                without_span_reads: choices * (2 + neighbor_families),
                with_span_reads: choices * (4 + neighbor_families),
            },
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.tokens.len()
    }
    pub fn heads(&self) -> usize {
        self.heads
    }
    pub fn latent_lanes_per_head(&self) -> usize {
        self.latent_lanes_per_head
    }
    pub fn stats(&self) -> ValueProducerStats {
        self.stats
    }

    /// Inputs are the actual retained roots and optional prior held-span roots.
    /// All recoverable input errors precede output construction; checked pair
    /// overflow exposes no partial result. This object and inputs are immutable.
    #[inline(never)]
    pub fn produce(
        &self,
        token: usize,
        latent: &[H4Code],
        held: Option<&[H4Code]>,
        occurrence_valid: bool,
    ) -> ValueProducerResult<ProducedValues> {
        let token = self
            .tokens
            .get(token)
            .ok_or(ValueProducerError::TokenOutOfRange {
                token,
                vocabulary: self.tokens.len(),
            })?;
        if latent.len() != self.latent_count {
            return Err(ValueProducerError::LatentLength {
                expected: self.latent_count,
                actual: latent.len(),
            });
        }
        if let Some(held) = held {
            if held.len() != self.latent_count {
                return Err(ValueProducerError::SpanLength {
                    expected: self.latent_count,
                    actual: held.len(),
                });
            }
        }
        let mut output = ProducedValues {
            heads: self.heads,
            occurrence_valid,
            packets: [[ValuePacket::absent(); ATOMS]; MAX_VALUE_LANES],
            values_q16: [0; MAX_COORDINATES],
            root_choices: [0; MAX_ATOMS],
            categories: [0; MAX_ATOMS],
            coefficient_reads: 0,
        };
        if !occurrence_valid {
            return Ok(output);
        }
        // Advance through the three-byte packets rather than dynamically
        // indexing six-byte pairs, which can compile to an integer multiply.
        for (slot, (atom, packet)) in self
            .atoms
            .iter()
            .zip(output.packets.iter_mut().flatten())
            .enumerate()
        {
            let own = latent[atom.own_index];
            let neighbor = latent[atom.neighbor_index];
            let span = held.map(|held| held[atom.own_index]);
            let root = atom
                .root
                .select(&token.roots[slot], own, neighbor, span, 7, ROOT_COUNT);
            let category = atom.category.select(
                &token.categories[slot],
                own,
                neighbor,
                span,
                5,
                CATEGORY_COUNT,
            );
            output.root_choices[slot] = root;
            output.categories[slot] = category;
            *packet = if category == 0 {
                ValuePacket::present_zero()
            } else {
                ValuePacket::present_nonzero(root, category - 1)
                    .map_err(ValueProducerError::Codec)?
            };
        }
        let count = self.heads << 2;
        let mut at = 0;
        for packets in output.packets.iter().take(count) {
            let value = self
                .codec
                .decode_pair(packets[0], packets[1])
                .map_err(ValueProducerError::Codec)?;
            output.values_q16[at..at + 4].copy_from_slice(&value);
            at += 4;
        }
        output.coefficient_reads = if held.is_some() {
            self.stats.with_span_reads
        } else {
            self.stats.without_span_reads
        };
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_value::ValueState;

    struct Fixture {
        vocab: usize,
        heads: usize,
        lanes: usize,
        data: [Vec<i32>; 10],
    }
    impl Fixture {
        fn new(vocab: usize, heads: usize, lanes: usize) -> Self {
            let atoms = heads * 8;
            let lengths = [
                vocab * atoms * 128,
                atoms * 128 * 128,
                atoms * 128 * 128,
                atoms * 128 * 128,
                atoms * 128,
                vocab * atoms * 32,
                atoms * 128 * 32,
                atoms * 128 * 32,
                atoms * 128 * 32,
                atoms * 32,
            ];
            Self {
                vocab,
                heads,
                lanes,
                data: std::array::from_fn(|i| vec![0; lengths[i]]),
            }
        }
        fn slices(&self) -> ValueProducerTableSlices<'_> {
            ValueProducerTableSlices {
                token_root: &self.data[0],
                own_root: &self.data[1],
                neighbor_root: if self.lanes > 1 {
                    Some(&self.data[2])
                } else {
                    None
                },
                span_root: &self.data[3],
                span_valid_root: &self.data[4],
                token_category: &self.data[5],
                own_category: &self.data[6],
                neighbor_category: if self.lanes > 1 {
                    Some(&self.data[7])
                } else {
                    None
                },
                span_category: &self.data[8],
                span_valid_category: &self.data[9],
            }
        }
        fn build(&self) -> ValueProducerResult<NativeValueProducer> {
            NativeValueProducer::new(self.vocab, self.heads, self.lanes, self.slices())
        }
    }
    fn codes(values: &[u8]) -> Vec<H4Code> {
        values
            .iter()
            .map(|&v| H4Code::try_from(v).unwrap())
            .collect()
    }

    #[test]
    fn geometric_value_producer_selects_head_local_neighbor_and_valid_span_only() {
        let mut fixture = Fixture::new(1, 2, 2);
        // Observe value lane1 in each head: own latent lane1, neighbor lane0.
        for (slot, neighbor, desired) in [(2usize, 3usize, 5usize), (10, 7, 9)] {
            fixture.data[2][(slot * 128 + neighbor) * 128 + desired] = 4;
            fixture.data[5][slot * 32 + 17] = 2;
        }
        // Slot0 changes only when the correct held root is present. The bias
        // also must disappear when the span is missing.
        fixture.data[0][1] = 1;
        fixture.data[3][3 * 128 + 3] = 10;
        fixture.data[4][5] = 6;
        fixture.data[5][17] = 2;
        let producer = fixture.build().unwrap();
        let latent = codes(&[3, 1, 7, 1]);
        let no_span = producer.produce(0, &latent, None, true).unwrap();
        assert_eq!(no_span.root_choices[0], 1);
        assert_eq!(no_span.root_choices[2], 5);
        assert_eq!(no_span.root_choices[10], 9);
        let held = codes(&[3, 1, 1, 1]);
        let with_span = producer.produce(0, &latent, Some(&held), true).unwrap();
        assert_eq!(with_span.root_choices[0], 3);
        let other = codes(&[1, 1, 1, 1]);
        assert_eq!(
            producer
                .produce(0, &latent, Some(&other), true)
                .unwrap()
                .root_choices[0],
            5
        );
        assert_eq!(
            no_span.coefficient_reads,
            producer.stats().without_span_reads
        );
        assert_eq!(
            with_span.coefficient_reads,
            producer.stats().with_span_reads
        );
        assert_eq!(
            with_span.coefficient_reads - no_span.coefficient_reads,
            16 * 152 * 2
        );
    }

    #[test]
    fn geometric_value_producer_ties_cancellation_absence_and_pair_overflow() {
        let mut fixture = Fixture::new(2, 1, 1);
        // Token0 has exact opposing atoms in lane0; other slots tie at zero.
        fixture.data[0][1] = 1;
        fixture.data[0][128] = 1;
        fixture.data[5][17] = 1;
        fixture.data[5][32 + 17] = 1;
        // Token1 selects +1 at maximum radius in both atoms: +2^31 rejects.
        let token_root = 8 * 128;
        fixture.data[0][token_root + 1] = 1;
        fixture.data[0][token_root + 128 + 1] = 1;
        let token_category = 8 * 32;
        fixture.data[5][token_category + 31] = 1;
        fixture.data[5][token_category + 32 + 31] = 1;
        let producer = fixture.build().unwrap();
        let latent = [H4Code::IDENTITY];
        let output = producer.produce(0, &latent, None, true).unwrap();
        assert_eq!(&output.values_q16[..16], &[0; 16]);
        assert!(output.packets[0]
            .iter()
            .all(|p| p.state() == ValueState::PresentNonzero));
        assert_ne!(output.packets[0][0].root(), output.packets[0][1].root());
        assert_eq!(output.root_choices[2], 0);
        assert_eq!(output.categories[2], 0);
        assert_eq!(output.packets[1], [ValuePacket::present_zero(); 2]);
        let absent = producer.produce(1, &latent, None, false).unwrap();
        assert!(!absent.occurrence_valid);
        assert_eq!(absent.packets, [[ValuePacket::absent(); 2]; 8]);
        assert_eq!(absent.coefficient_reads, 0);
        assert!(matches!(
            producer.produce(1, &latent, None, true),
            Err(ValueProducerError::Codec(
                GeometricValueError::PairCoordinateOverflow { coordinate: 0 }
            ))
        ));
        assert_eq!(producer.produce(0, &latent, None, true).unwrap(), output);
    }

    #[test]
    fn geometric_value_producer_admission_and_inputs_fail_without_mutation() {
        let mut fixture = Fixture::new(1, 1, 2);
        fixture.data[0][120] = 1;
        assert!(matches!(
            fixture.build(),
            Err(ValueProducerError::NonzeroPadding {
                table: ValueProducerTable::TokenRoot,
                ..
            })
        ));
        fixture.data[0][120] = 0;
        fixture.data[8][120 * 32] = 1;
        assert!(matches!(
            fixture.build(),
            Err(ValueProducerError::NonzeroPadding {
                table: ValueProducerTable::SpanCategory,
                ..
            })
        ));
        fixture.data[8][120 * 32] = 0;
        let mut data = fixture.slices();
        data.neighbor_root = None;
        assert!(matches!(
            NativeValueProducer::new(1, 1, 2, data),
            Err(ValueProducerError::MissingNeighbor(
                ValueProducerTable::NeighborRoot
            ))
        ));
        let producer = fixture.build().unwrap();
        let latent = [H4Code::IDENTITY; 2];
        let before = producer.produce(0, &latent, None, true).unwrap();
        assert!(matches!(
            producer.produce(1, &latent, None, false),
            Err(ValueProducerError::TokenOutOfRange { .. })
        ));
        assert!(matches!(
            producer.produce(0, &latent[..1], None, false),
            Err(ValueProducerError::LatentLength { .. })
        ));
        assert!(matches!(
            producer.produce(0, &latent, Some(&latent[..1]), false),
            Err(ValueProducerError::SpanLength { .. })
        ));
        assert_eq!(producer.produce(0, &latent, None, true).unwrap(), before);
        let mut single = Fixture::new(1, 1, 1);
        single.data[2][0] = 1;
        let mut data = single.slices();
        data.neighbor_root = Some(&single.data[2]);
        assert!(matches!(
            NativeValueProducer::new(1, 1, 1, data),
            Err(ValueProducerError::NonzeroSingleLaneNeighbor { .. })
        ));
    }

    #[test]
    fn geometric_value_producer_maximum_dimensions_and_wide_scores() {
        assert_eq!(std::mem::align_of::<AtomTables>(), 128);
        assert!(std::mem::size_of::<AtomTables>().is_power_of_two());
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<AtomTables>(), 256);
        let mut fixture = Fixture::new(MAX_VOCAB, MAX_HEADS, MAX_LATENT_LANES_PER_HEAD);
        let slots = MAX_ATOMS;
        // Five maximal coefficients exceed i32 but are safe i64 score sums.
        let token = MAX_VOCAB - 1;
        for slot in 0..slots {
            fixture.data[0][(token * slots + slot) * 128 + 1] = i32::MAX;
            for family in [1, 2, 3] {
                fixture.data[family][(slot * 128 + 1) * 128 + 1] = i32::MAX;
            }
            fixture.data[4][slot * 128 + 1] = i32::MAX;
            // The runner-up has three maximal terms. Wrapping i32 sums would
            // reverse this ordering: 3*MAX wraps to MAX-2, 5*MAX to MAX-4.
            fixture.data[0][(token * slots + slot) * 128 + 2] = i32::MAX;
            for family in [1, 2] {
                fixture.data[family][(slot * 128 + 1) * 128 + 2] = i32::MAX;
            }
        }
        let producer = fixture.build().unwrap();
        let latent = [H4Code::IDENTITY; 8];
        let output = producer
            .produce(token, &latent, Some(&latent), true)
            .unwrap();
        assert_eq!(output.heads, 2);
        assert_eq!(output.root_choices, [1; 16]);
        assert_eq!(output.packets, [[ValuePacket::present_zero(); 2]; 8]);
        assert_eq!(output.values_q16, [0; 32]);
        assert_eq!(output.coefficient_reads, 16 * (120 + 32) * 5);
        assert!(matches!(
            NativeValueProducer::new(MAX_VOCAB + 1, 2, 4, fixture.slices()),
            Err(ValueProducerError::InvalidVocabulary(_))
        ));
        assert!(matches!(
            NativeValueProducer::new(1, 3, 4, fixture.slices()),
            Err(ValueProducerError::InvalidHeads(_))
        ));
        assert!(matches!(
            NativeValueProducer::new(1, 2, 5, fixture.slices()),
            Err(ValueProducerError::InvalidLatentLanes(_))
        ));
    }
}
