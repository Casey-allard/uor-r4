//! Selected-record occurrence weights from a bounded H4 context recurrence.
//!
//! This reader preserves exact source identity beside geometry. Only original
//! frame occurrences are candidates; query and generated-prefix tokens update
//! the causal context but are never copy candidates. Context-relative potential
//! is used with the content channel explicitly absent (no span/value codec).
//! The returned NoRead weight is reserved for the caller's complete parent
//! distribution, not a zero-valued semantic result. Construction allocates;
//! `read` performs no allocation or floating arithmetic. This is a component,
//! not a complete language-model serving or compiled-instruction claim.

use std::fmt;

use crate::geometric_context::{
    ContextError, ContextStep, NativeContextState, NativeContextTables, MAX_LANES,
};
use crate::geometric_no_read::{NativeGeometricNoRead, NoReadError};
use crate::geometric_potential::{AddressLane, NativePotentialTables, PotentialError};
use crate::geometric_read::{NativeGeometricRead, ReadError, EXP_TABLE_LEN};
use crate::h4_tables::{H4Code, HistoricalH4Tables};

pub const MAX_SEQUENCE: usize = 128;
pub const MAX_HEADS: usize = 2;
pub const POLICY: &str = "selected-original-BPE-occurrences;frame-query-prefix-causal-H4;context-relative-potential;content-absent;zero-age;NoRead-parent-fallback;128-admission/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceIdentity {
    /// Exact store record ordinal, interpreted in caller-retained scope/entity.
    pub record: u64,
    pub commit: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceOccurrence {
    pub source: SourceIdentity,
    pub token_offset: u32,
    pub token_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameStatus {
    Found,
    Absent,
    Unresolved,
    Conflict,
}

/// Exact caller-owned metadata is not encoded as a geometric distance. The
/// caller retains/binds these bytes in its report/artifact; this numerical
/// reader validates status but does not use relation/view to score candidates.
#[derive(Clone, Copy, Debug)]
pub struct FrameMetadata<'a> {
    pub scope: &'a [u8],
    pub entity: &'a [u32],
    pub relation: u32,
    pub view: u32,
    pub status: FrameStatus,
}

#[derive(Clone, Copy, Debug)]
pub struct SelectedRecordFrame<'a> {
    pub identity: SourceIdentity,
    pub metadata: FrameMetadata<'a>,
    pub token_ids: &'a [u32],
}

#[derive(Clone, Copy)]
pub struct OccurrenceComponents<'a> {
    pub context: &'a NativeContextTables,
    pub potential: &'a NativePotentialTables,
    pub no_read: &'a NativeGeometricNoRead,
    pub geometry: &'a HistoricalH4Tables,
    pub exp_q31: &'a [u32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OccurrenceReadError {
    ComponentShape,
    ForeignPreparedContext,
    ForeignQuerySnapshot,
    UnsupportedStatus(FrameStatus),
    EmptyQuery,
    SequenceLength { actual: usize, maximum: usize },
    Token { token: u32, vocabulary: usize },
    Context(ContextError),
    Potential(PotentialError),
    NoRead(NoReadError),
    Reduction(ReadError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context_q4::{self, ContextQ4Config, NativeContextQ4};
    use crate::geometric_no_read::{self, NoReadConfig};
    use crate::geometric_potential_q4::{self, NativePotentialQ4, PotentialQ4Config};
    use crate::geometric_read::WEIGHT_ONE;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    struct Fixture {
        context: NativeContextTables,
        potential: NativePotentialTables,
        no_read: NativeGeometricNoRead,
        geometry: HistoricalH4Tables,
        exp: Vec<u32>,
    }
    impl Fixture {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let cc = ContextQ4Config {
                vocab_size: 4,
                heads: 1,
                lanes_per_head: 1,
            };
            let context = NativeContextQ4::new(
                cc,
                &geometric_context_q4::pack_coefficients(&vec![0; cc.coefficient_count()?])?,
            )?
            .into_native()?;
            let pc = PotentialQ4Config {
                heads: 1,
                lanes_per_head: 1,
            };
            let potential = NativePotentialQ4::new(
                pc,
                &geometric_potential_q4::pack_coefficients(&vec![0; pc.coefficient_count()?])?,
            )?
            .into_native()?;
            let nc = NoReadConfig {
                vocabulary: 4,
                heads: 1,
                latent_lanes_per_head: 1,
            };
            let no_read = NativeGeometricNoRead::new(
                nc,
                &geometric_no_read::pack_coefficients(&vec![0; nc.coefficient_count()])?,
            )?;
            let geometry = HistoricalH4Tables::from_bytes(&std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/fixtures/historical-h4-tables-v1.bin"
            ))?)?;
            // Admitted deterministic table; equal zero logits exercise exact
            // occurrence mass/NoRead without a floating reference fixture.
            let mut exp = vec![0; EXP_TABLE_LEN];
            exp[0] = WEIGHT_ONE as u32;
            Ok(Self {
                context,
                potential,
                no_read,
                geometry,
                exp,
            })
        }
        fn reader(&self) -> OccurrenceReadResult<NativeOccurrenceReader<'_>> {
            NativeOccurrenceReader::new(OccurrenceComponents {
                context: &self.context,
                potential: &self.potential,
                no_read: &self.no_read,
                geometry: &self.geometry,
                exp_q31: &self.exp,
            })
        }
    }
    #[test]
    fn prepared_context_and_snapshot_reject_foreign_frames_and_components() -> TestResult {
        let f = Fixture::new()?;
        let g = Fixture::new()?;
        let mut reader = f.reader()?;
        let a = reader.prepare(frame(&[2, 1]), &[3], &[])?;
        let b = reader.prepare(frame(&[1, 2]), &[3], &[])?;
        let snapshot = a.original_snapshot()?;
        assert_eq!(
            reader.score(&b, &snapshot).err(),
            Some(OccurrenceReadError::ForeignQuerySnapshot)
        );
        let foreign = g.reader()?.prepare(frame(&[2, 1]), &[3], &[])?;
        assert_eq!(
            reader.score(&foreign, &foreign.original_snapshot()?).err(),
            Some(OccurrenceReadError::ForeignPreparedContext)
        );
        let original = reader.read(frame(&[2, 1]), &[3], &[])?;
        let old_scores = original
            .head(0)
            .ok_or("head missing")?
            .potential_q24
            .to_vec();
        let scored = reader.score(&a, &snapshot)?;
        assert_eq!(
            scored.head(0).ok_or("head missing")?.potential_q24,
            old_scores
        );
        assert_eq!(a.source_occurrences()[0].token_id, 2);
        Ok(())
    }
    fn frame(tokens: &[u32]) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: 7,
                commit: 42,
            },
            metadata: FrameMetadata {
                scope: b"scope",
                entity: &[19, 29],
                relation: 3,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: tokens,
        }
    }
    #[test]
    fn occurrence_reader_repeated_tokens_keep_exact_offsets_and_exclude_query() -> TestResult {
        let fixture = Fixture::new()?;
        let mut reader = fixture.reader()?;
        let r = reader.read(frame(&[2, 1, 2]), &[2, 3], &[1])?;
        assert_eq!(r.occurrences.len(), 3);
        assert_eq!(
            r.occurrences
                .iter()
                .map(|o| o.token_offset)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            r.occurrences.iter().map(|o| o.token_id).collect::<Vec<_>>(),
            vec![2, 1, 2]
        );
        assert!(r
            .occurrences
            .iter()
            .all(|o| o.source == frame(&[]).identity));
        let h = r.head(0).ok_or("head absent")?;
        assert_eq!(h.occurrence_weights_q31, &[WEIGHT_ONE; 3]);
        assert_eq!(h.no_read_weight_q31, WEIGHT_ONE);
        assert_eq!(h.total_weight_q31, WEIGHT_ONE << 2);
        assert_eq!(r.stats.sequence_tokens, 6);
        assert!(r.head(1).is_none());
        Ok(())
    }
    #[test]
    fn occurrence_reader_rejection_preserves_public_source_and_weights() -> TestResult {
        let fixture = Fixture::new()?;
        let mut reader = fixture.reader()?;
        reader.read(frame(&[2, 2]), &[1], &[])?;
        assert!(matches!(
            reader.read(frame(&[4]), &[1], &[]),
            Err(OccurrenceReadError::Token { .. })
        ));
        assert!(matches!(
            reader.read(frame(&[1]), &[], &[]),
            Err(OccurrenceReadError::EmptyQuery)
        ));
        assert!(matches!(
            reader.read(frame(&[1; MAX_SEQUENCE]), &[1], &[]),
            Err(OccurrenceReadError::SequenceLength { .. })
        ));
        let mut bad = frame(&[1]);
        bad.metadata.status = FrameStatus::Conflict;
        assert!(matches!(
            reader.read(bad, &[1], &[]),
            Err(OccurrenceReadError::UnsupportedStatus(
                FrameStatus::Conflict
            ))
        ));
        let r = reader.last().ok_or("prior read lost")?;
        assert_eq!(r.source.commit, 42);
        assert_eq!(r.occurrences.len(), 2);
        assert_eq!(
            r.head(0).ok_or("prior head lost")?.occurrence_weights_q31,
            &[WEIGHT_ONE; 2]
        );
        let null = reader.read(frame(&[]), &[1], &[])?;
        assert!(null.occurrences.is_empty());
        let h = null.head(0).ok_or("null head absent")?;
        assert_eq!(h.no_read_weight_q31, h.total_weight_q31);
        Ok(())
    }
}
impl fmt::Display for OccurrenceReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric occurrence read: {self:?}")
    }
}
impl std::error::Error for OccurrenceReadError {}
pub type OccurrenceReadResult<T> = Result<T, OccurrenceReadError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OccurrenceReadStats {
    pub sequence_tokens: usize,
    pub candidates: usize,
    pub context_coefficient_reads: usize,
    pub potential_table_reads: usize,
    pub no_read_table_reads: usize,
    pub geometry_relative_reads: usize,
    /// Logical owned buffers only; excludes borrowed model tables, caller
    /// payload/metadata, allocator overhead, parent distribution and training.
    pub logical_owned_bytes: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct OccurrenceHead<'a> {
    pub potential_q24: &'a [i64],
    pub occurrence_weights_q31: &'a [u64],
    pub no_read_q24: i64,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct OccurrenceRead<'a> {
    pub source: SourceIdentity,
    pub occurrences: &'a [SourceOccurrence],
    pub heads: &'a [HeadSummary],
    pub stats: OccurrenceReadStats,
    scores: &'a [[i64; MAX_SEQUENCE]; MAX_HEADS],
    weights: &'a [[u64; MAX_SEQUENCE]; MAX_HEADS],
}
impl<'a> OccurrenceRead<'a> {
    pub fn head(&self, index: usize) -> Option<OccurrenceHead<'a>> {
        let h = self.heads.get(index)?;
        Some(OccurrenceHead {
            potential_q24: &self.scores[index][..self.occurrences.len()],
            occurrence_weights_q31: &self.weights[index][..self.occurrences.len()],
            no_read_q24: h.no_read_q24,
            no_read_weight_q31: h.no_read_weight_q31,
            total_weight_q31: h.total_weight_q31,
            max_score_q24: h.max_score_q24,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HeadSummary {
    pub no_read_q24: i64,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
}

const EMPTY_ID: SourceIdentity = SourceIdentity {
    record: 0,
    commit: 0,
};
const EMPTY_OCCURRENCE: SourceOccurrence = SourceOccurrence {
    source: EMPTY_ID,
    token_offset: 0,
    token_id: 0,
};

/// Private, immutable original causal replay; no unchecked deserialization.
/// Fixed capacity retains source latents and provenance without allocating.
pub struct PreparedOccurrenceContext<'a> {
    components: OccurrenceComponents<'a>,
    source: SourceIdentity,
    occurrences: [SourceOccurrence; MAX_SEQUENCE],
    tokens: [u32; MAX_SEQUENCE],
    steps: [Option<ContextStep>; MAX_SEQUENCE],
    count: usize,
    total: usize,
    stats: OccurrenceReadStats,
    frame_binding: [u8; 32],
}

/// Only original preparation and the admitted feedback bridge construct this.
/// The reference binds one specific prepared frame, not merely its dimensions.
pub struct QuerySnapshot<'p, 'a> {
    prepared: &'p PreparedOccurrenceContext<'a>,
    step: ContextStep,
}
impl QuerySnapshot<'_, '_> {
    pub fn states(&self) -> &[H4Code] {
        &self.step.states[..self.step.heads * self.step.lanes_per_head]
    }
    pub fn codes(&self) -> &[AddressLane] {
        &self.step.output[..self.step.heads * self.step.lanes_per_head]
    }
    pub fn raw_roots(&self) -> &[H4Code] {
        &self.step.readout_roots[..self.states().len()]
    }
    pub fn categories(&self) -> &[u8] {
        &self.step.categories[..self.states().len()]
    }
    pub fn last_token(&self) -> u32 {
        self.prepared.tokens[self.prepared.total - 1]
    }
    pub fn heads(&self) -> usize {
        self.step.heads
    }
    pub fn lanes_per_head(&self) -> usize {
        self.step.lanes_per_head
    }
    pub fn frame_binding(&self) -> &[u8; 32] {
        &self.prepared.frame_binding
    }
}
impl<'a> PreparedOccurrenceContext<'a> {
    pub fn original_snapshot(&self) -> OccurrenceReadResult<QuerySnapshot<'_, 'a>> {
        let step = self.steps[self.total - 1].ok_or(OccurrenceReadError::ComponentShape)?;
        Ok(QuerySnapshot {
            prepared: self,
            step,
        })
    }
    pub fn source_occurrences(&self) -> &[SourceOccurrence] {
        &self.occurrences[..self.count]
    }
    pub fn source_states(&self, offset: usize) -> Option<&[H4Code]> {
        if offset >= self.count {
            return None;
        }
        self.steps[offset]
            .as_ref()
            .map(|s| &s.states[..s.heads * s.lanes_per_head])
    }
    pub fn source_codes(&self, offset: usize) -> Option<&[AddressLane]> {
        if offset >= self.count {
            return None;
        }
        self.steps[offset]
            .as_ref()
            .map(|s| &s.output[..s.heads * s.lanes_per_head])
    }
    pub fn tokens(&self) -> &[u32] {
        &self.tokens[..self.total]
    }
    pub fn steps(&self) -> impl Iterator<Item = &ContextStep> {
        self.steps[..self.total].iter().flatten()
    }
    pub fn context_coefficient_reads(&self) -> usize {
        self.stats.context_coefficient_reads
    }
    pub fn logical_prepared_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    pub(crate) fn observation_reads(&self) -> usize {
        self.components.context.stats().root_readout_reads
            + self.components.context.stats().category_readout_reads
    }
    pub(crate) fn apply_actions<'p>(
        &'p self,
        original: &QuerySnapshot<'p, 'a>,
        actions: &[H4Code],
    ) -> OccurrenceReadResult<QuerySnapshot<'p, 'a>> {
        if !std::ptr::eq(original.prepared, self) {
            return Err(OccurrenceReadError::ForeignQuerySnapshot);
        }
        if actions.len() != original.states().len() {
            return Err(OccurrenceReadError::ComponentShape);
        }
        let mut states = [H4Code::IDENTITY; MAX_LANES];
        for (i, (old, action)) in original.states().iter().zip(actions).enumerate() {
            states[i] = self.components.geometry.compose(*old, *action);
        }
        let step = NativeContextState::observe_states(
            original.last_token() as usize,
            &states[..actions.len()],
            self.components.context,
        )
        .map_err(OccurrenceReadError::Context)?;
        Ok(QuerySnapshot {
            prepared: self,
            step,
        })
    }
}

pub struct NativeOccurrenceReader<'a> {
    components: OccurrenceComponents<'a>,
    reducers: Vec<NativeGeometricRead>,
    #[allow(dead_code)]
    addresses: [[AddressLane; MAX_LANES]; MAX_SEQUENCE],
    scratch_scores: [[i64; MAX_SEQUENCE]; MAX_HEADS],
    scratch_weights: [[u64; MAX_SEQUENCE]; MAX_HEADS],
    scores: [[i64; MAX_SEQUENCE]; MAX_HEADS],
    weights: [[u64; MAX_SEQUENCE]; MAX_HEADS],
    occurrences: [SourceOccurrence; MAX_SEQUENCE],
    summaries: [HeadSummary; MAX_HEADS],
    last: Option<(SourceIdentity, usize, OccurrenceReadStats)>,
}

impl<'a> NativeOccurrenceReader<'a> {
    pub fn new(components: OccurrenceComponents<'a>) -> OccurrenceReadResult<Self> {
        let c = components.context;
        let n = components.no_read.config();
        if c.heads() > MAX_HEADS
            || components.potential.heads() != c.heads()
            || components.potential.lanes() != c.lanes_per_head()
            || n.heads != c.heads()
            || n.latent_lanes_per_head != c.lanes_per_head()
            || n.vocabulary != c.vocab_size()
        {
            return Err(OccurrenceReadError::ComponentShape);
        }
        let absent = AddressLane::new(1, 0, false).map_err(OccurrenceReadError::Potential)?;
        let mut reducers = Vec::with_capacity(c.heads());
        for _ in 0..c.heads() {
            reducers.push(
                NativeGeometricRead::new(MAX_SEQUENCE, 1, components.exp_q31)
                    .map_err(OccurrenceReadError::Reduction)?,
            );
        }
        Ok(Self {
            components,
            reducers,
            addresses: [[absent; MAX_LANES]; MAX_SEQUENCE],
            scratch_scores: [[0; MAX_SEQUENCE]; MAX_HEADS],
            scratch_weights: [[0; MAX_SEQUENCE]; MAX_HEADS],
            scores: [[0; MAX_SEQUENCE]; MAX_HEADS],
            weights: [[0; MAX_SEQUENCE]; MAX_HEADS],
            occurrences: [EMPTY_OCCURRENCE; MAX_SEQUENCE],
            summaries: [HeadSummary::default(); MAX_HEADS],
            last: None,
        })
    }

    /// Logical buffer lengths, not an allocator/RSS or full-model cost claim.
    pub fn logical_owned_bytes(&self) -> usize {
        let reducer_heap = EXP_TABLE_LEN * std::mem::size_of::<u32>()
            + (MAX_SEQUENCE + 1) * std::mem::size_of::<usize>()
            + MAX_SEQUENCE * (std::mem::size_of::<i64>() + std::mem::size_of::<u64>())
            + std::mem::size_of::<i128>()
            + std::mem::size_of::<i32>();
        std::mem::size_of::<Self>()
            + self.reducers.capacity() * std::mem::size_of::<NativeGeometricRead>()
            + self.reducers.len() * reducer_heap
    }

    pub fn last(&self) -> Option<OccurrenceRead<'_>> {
        self.last.map(|(source, count, stats)| OccurrenceRead {
            source,
            occurrences: &self.occurrences[..count],
            heads: &self.summaries[..self.components.context.heads()],
            stats,
            scores: &self.scores,
            weights: &self.weights,
        })
    }

    /// Each call starts a fresh causal geometric state. All returned output is
    /// committed together after successful reduction of every head. Errors can
    /// overwrite private scratch but preserve `last()` including source/weights.
    pub fn read(
        &mut self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        response_prefix: &[u32],
    ) -> OccurrenceReadResult<OccurrenceRead<'_>> {
        let prepared = self.prepare(frame, query, response_prefix)?;
        let snapshot = prepared.original_snapshot()?;
        self.score(&prepared, &snapshot)
    }

    pub fn prepare(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        response_prefix: &[u32],
    ) -> OccurrenceReadResult<PreparedOccurrenceContext<'a>> {
        if frame.metadata.status != FrameStatus::Found {
            return Err(OccurrenceReadError::UnsupportedStatus(
                frame.metadata.status,
            ));
        }
        if query.is_empty() {
            return Err(OccurrenceReadError::EmptyQuery);
        }
        let total = frame
            .token_ids
            .len()
            .checked_add(query.len())
            .and_then(|n| n.checked_add(response_prefix.len()))
            .unwrap_or(usize::MAX);
        if total > MAX_SEQUENCE {
            return Err(OccurrenceReadError::SequenceLength {
                actual: total,
                maximum: MAX_SEQUENCE,
            });
        }
        let vocab = self.components.context.vocab_size();
        for &token in frame.token_ids.iter().chain(query).chain(response_prefix) {
            if token as usize >= vocab {
                return Err(OccurrenceReadError::Token {
                    token,
                    vocabulary: vocab,
                });
            }
        }
        let mut state = NativeContextState::new(
            self.components.context.heads(),
            self.components.context.lanes_per_head(),
        )
        .map_err(OccurrenceReadError::Context)?;
        let mut stats = OccurrenceReadStats {
            sequence_tokens: total,
            candidates: frame.token_ids.len(),
            logical_owned_bytes: self.logical_owned_bytes(),
            ..OccurrenceReadStats::default()
        };
        let mut prepared = PreparedOccurrenceContext {
            components: self.components,
            source: frame.identity,
            occurrences: [EMPTY_OCCURRENCE; MAX_SEQUENCE],
            tokens: [0; MAX_SEQUENCE],
            steps: [None; MAX_SEQUENCE],
            count: frame.token_ids.len(),
            total,
            stats,
            frame_binding: frame_digest(frame, query, response_prefix),
        };
        for (position, &token) in frame
            .token_ids
            .iter()
            .chain(query)
            .chain(response_prefix)
            .enumerate()
        {
            let step = state
                .step(
                    token as usize,
                    self.components.context,
                    self.components.geometry,
                )
                .map_err(OccurrenceReadError::Context)?;
            prepared.tokens[position] = token;
            prepared.steps[position] = Some(step);
            stats.context_coefficient_reads += step.coefficient_reads;
        }
        for (offset, &token_id) in frame.token_ids.iter().enumerate() {
            prepared.occurrences[offset] = SourceOccurrence {
                source: frame.identity,
                token_offset: offset as u32,
                token_id,
            };
        }
        prepared.stats = stats;
        Ok(prepared)
    }

    /// Scores immutable keys against one checked original or updated snapshot.
    pub fn score(
        &mut self,
        prepared: &PreparedOccurrenceContext<'a>,
        snapshot: &QuerySnapshot<'_, 'a>,
    ) -> OccurrenceReadResult<OccurrenceRead<'_>> {
        if !std::ptr::eq(prepared.components.context, self.components.context)
            || !std::ptr::eq(prepared.components.potential, self.components.potential)
            || !std::ptr::eq(prepared.components.no_read, self.components.no_read)
            || !std::ptr::eq(prepared.components.geometry, self.components.geometry)
            || !std::ptr::eq(prepared.components.exp_q31, self.components.exp_q31)
        {
            return Err(OccurrenceReadError::ForeignPreparedContext);
        }
        if !std::ptr::eq(snapshot.prepared, prepared) {
            return Err(OccurrenceReadError::ForeignQuerySnapshot);
        }
        let mut stats = prepared.stats;
        let heads = self.components.context.heads();
        let lanes = self.components.context.lanes_per_head();
        let null = self
            .components
            .no_read
            .score(
                snapshot.last_token() as usize,
                snapshot.states(),
                snapshot.codes(),
                None,
            )
            .map_err(OccurrenceReadError::NoRead)?;
        stats.no_read_table_reads = self.components.no_read.stats().without_span_reads;
        let absent =
            [AddressLane::new(1, 0, false).map_err(OccurrenceReadError::Potential)?; MAX_LANES];
        let count = prepared.count;
        let zeros = [0i32; MAX_SEQUENCE];
        let ages = [0i64; MAX_SEQUENCE];
        let mut summaries = [HeadSummary::default(); MAX_HEADS];
        let mut start = 0;
        for h in 0..heads {
            let end = start + lanes;
            let q = &snapshot.codes()[start..end];
            for j in 0..count {
                let k = &prepared
                    .source_codes(j)
                    .ok_or(OccurrenceReadError::ComponentShape)?[start..end];
                self.scratch_scores[h][j] = self
                    .components
                    .potential
                    .score(
                        h,
                        &absent[..lanes],
                        &absent[..lanes],
                        q,
                        k,
                        self.components.geometry,
                    )
                    .map_err(OccurrenceReadError::Potential)?;
                for (query_lane, source_lane) in q.iter().zip(k) {
                    stats.potential_table_reads += 2;
                    if query_lane.present() && source_lane.present() {
                        stats.potential_table_reads += 2;
                        stats.geometry_relative_reads += 1;
                    }
                }
            }
            let r = self.reducers[h]
                .reduce(
                    &self.scratch_scores[h][..count],
                    &ages[..count],
                    null[h],
                    &zeros[..count],
                )
                .map_err(OccurrenceReadError::Reduction)?;
            self.scratch_weights[h][..count].copy_from_slice(r.occurrence_weights_q31);
            summaries[h] = HeadSummary {
                no_read_q24: null[h],
                no_read_weight_q31: r.no_read_weight_q31,
                total_weight_q31: r.total_weight_q31,
                max_score_q24: r.max_score_q24,
            };
            start = end;
        }
        self.occurrences = prepared.occurrences;
        self.scores = self.scratch_scores;
        self.weights = self.scratch_weights;
        self.summaries = summaries;
        self.last = Some((prepared.source, count, stats));
        // `last` has just been committed; construct directly without an unwrap.
        Ok(OccurrenceRead {
            source: prepared.source,
            occurrences: &self.occurrences[..count],
            heads: &self.summaries[..heads],
            stats,
            scores: &self.scores,
            weights: &self.weights,
        })
    }
}

fn frame_digest(frame: SelectedRecordFrame<'_>, query: &[u32], prefix: &[u32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(frame.identity.record.to_le_bytes());
    h.update(frame.identity.commit.to_le_bytes());
    h.update((frame.metadata.scope.len() as u64).to_le_bytes());
    h.update(frame.metadata.scope);
    h.update(frame.metadata.relation.to_le_bytes());
    h.update(frame.metadata.view.to_le_bytes());
    for ids in [frame.metadata.entity, frame.token_ids, query, prefix] {
        h.update((ids.len() as u64).to_le_bytes());
        for id in ids {
            h.update(id.to_le_bytes());
        }
    }
    h.finalize().into()
}
