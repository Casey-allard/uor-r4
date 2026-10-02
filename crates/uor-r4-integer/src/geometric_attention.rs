//! Persistent integer geometric attention over an exact bounded occurrence prefix.
//!
//! Admission allocates all history and reducer scratch. Each push updates the
//! existing event/context engines once, observes the OLD committed span, produces
//! the current geometric payload, and reads every occurrence including itself.
//! Span actions become visible on the next push. No candidate is removed for
//! absent addresses or zero payloads, and heads retain separate denominators.
//!
//! This module does not change coefficient codecs: event and age tables retain
//! their admitted numerical meaning. The outer loader binds their provenance.
//! The output is i64 Q16 in the residual frame, not a complete language model.
//! Source hot paths use no float, multiplication, division or allocation;
//! compiled instruction/callee evidence is a separate measured obligation.

use std::fmt;

use crate::geometric_composed_read::{ComposedReadError, NativeGeometricComposedRead};
use crate::geometric_composition::{sum_heads, CompositionError, NativeGeometricComposition};
use crate::geometric_context::{
    ContextError, ContextStep, NativeContextState, NativeContextTables,
};
use crate::geometric_event::{EventError, EventStep, NativeEventState, NativeEventTables};
use crate::geometric_no_read::{NativeGeometricNoRead, NoReadError};
use crate::geometric_potential::{AddressLane, NativePotentialTables, PotentialError};
use crate::geometric_span::{SpanError, SpanRegister, TokenActionDictionary};
use crate::geometric_value_producer::{NativeValueProducer, ProducedValues, ValueProducerError};
use crate::h4_tables::{H4Code, HistoricalH4Tables};

pub const SCHEMA: &str = "uor-r4.geometric-attention-session/1";
pub const MAX_CONTEXT: usize = 128;
pub const HEADS: usize = 2;
pub const LANES_PER_HEAD: usize = 4;
pub const LANES: usize = 8;
pub const OUTPUT_WIDTH: usize = 32;
pub const CONTENT_RADIUS_BIN: u8 = 16;

/// Immutable, already admitted numerical components. Artifact/source identity
/// checks belong to the enclosing loader and happen before session creation.
#[derive(Clone, Copy)]
pub struct NativeAttentionComponents<'a> {
    pub events: &'a NativeEventTables,
    pub context: &'a NativeContextTables,
    pub span: &'a TokenActionDictionary,
    pub potential: &'a NativePotentialTables,
    pub values: &'a NativeValueProducer,
    pub no_read: &'a NativeGeometricNoRead,
    pub composition: &'a NativeGeometricComposition,
    pub geometry: &'a HistoricalH4Tables,
    /// Head-major lag table. Lag zero applies to the current occurrence.
    pub age_q24: &'a [i64],
    pub age_context: usize,
    pub exp_q31: &'a [u32],
}

#[derive(Debug)]
pub enum AttentionError {
    Capacity(usize),
    ComponentShape,
    AgeShape,
    Token { token: usize, vocabulary: usize },
    Full { capacity: usize },
    Event(EventError),
    Context(ContextError),
    Span(SpanError),
    Potential(PotentialError),
    Value(ValueProducerError),
    NoRead(NoReadError),
    Composition(CompositionError),
    Reduction(ComposedReadError),
}
impl fmt::Display for AttentionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native geometric attention: {self:?}")
    }
}
impl std::error::Error for AttentionError {}
pub type AttentionResult<T> = Result<T, AttentionError>;

/// Power-of-two private stride avoids dynamic multiply for occurrence access.
#[repr(align(64))]
#[derive(Clone, Copy)]
struct AddressRecord {
    content: [AddressLane; LANES],
    context: [AddressLane; LANES],
}

// Fixed-size publication records keep a failed second-head reduction from
// changing any previous public output. Reducer last() is never exposed here.
#[repr(align(4096))]
#[derive(Clone)]
struct HeadRecord {
    potential: [i64; MAX_CONTEXT],
    age: [i64; MAX_CONTEXT],
    weights: [u64; MAX_CONTEXT],
    output: [i64; OUTPUT_WIDTH],
    no_read_q24: i64,
    no_read_weight: u64,
    total: u64,
    max_score: i64,
}
impl HeadRecord {
    fn zero() -> Self {
        Self {
            potential: [0; MAX_CONTEXT],
            age: [0; MAX_CONTEXT],
            weights: [0; MAX_CONTEXT],
            output: [0; OUTPUT_WIDTH],
            no_read_q24: 0,
            no_read_weight: 0,
            total: 0,
            max_score: 0,
        }
    }
    fn view(&self, occurrences: usize) -> AttentionHead<'_> {
        AttentionHead {
            potential_q24: &self.potential[..occurrences],
            age_q24: &self.age[..occurrences],
            no_read_q24: self.no_read_q24,
            occurrence_weights_q31: &self.weights[..occurrences],
            no_read_weight_q31: self.no_read_weight,
            total_weight_q31: self.total,
            max_score_q24: self.max_score,
            output_q16: &self.output,
        }
    }
}

struct HeadStorage {
    payload: Box<[[i64; OUTPUT_WIDTH]]>,
    reducer: NativeGeometricComposedRead,
}
struct Committed {
    position: usize,
    event: EventStep,
    context: ContextStep,
    held: Option<[H4Code; LANES]>,
    values: ProducedValues,
    composed: [[i64; OUTPUT_WIDTH]; HEADS],
    addresses: AddressRecord,
    heads: [HeadRecord; HEADS],
    output: [i64; OUTPUT_WIDTH],
}
impl Committed {
    fn view(&self) -> AttentionStep<'_> {
        let occurrences = self.position + 1;
        AttentionStep {
            position: self.position,
            event: &self.event,
            context: &self.context,
            held_before: self.held.as_ref(),
            values: &self.values,
            composed_q16: &self.composed,
            content_codes: &self.addresses.content,
            context_codes: &self.addresses.context,
            heads: [
                self.heads[0].view(occurrences),
                self.heads[1].view(occurrences),
            ],
            output_q16: &self.output,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AttentionHead<'a> {
    pub potential_q24: &'a [i64],
    pub age_q24: &'a [i64],
    pub no_read_q24: i64,
    pub occurrence_weights_q31: &'a [u64],
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
    pub output_q16: &'a [i64; OUTPUT_WIDTH],
}

#[derive(Clone, Copy, Debug)]
pub struct AttentionStep<'a> {
    /// Zero-based exact occurrence identity, including repeated equal tokens.
    pub position: usize,
    pub event: &'a EventStep,
    pub context: &'a ContextStep,
    pub held_before: Option<&'a [H4Code; LANES]>,
    pub values: &'a ProducedValues,
    pub composed_q16: &'a [[i64; OUTPUT_WIDTH]; HEADS],
    pub content_codes: &'a [AddressLane; LANES],
    pub context_codes: &'a [AddressLane; LANES],
    pub heads: [AttentionHead<'a>; HEADS],
    pub output_q16: &'a [i64; OUTPUT_WIDTH],
}

/// Owned storage element bytes, not a process RSS or allocator measurement.
/// Includes the inline session (and its fixed publication/reducer arrays) and
/// all constructor-owned heap elements, including two copied exp tables.
/// Excludes borrowed admitted kernels, allocator bookkeeping/alignment beyond
/// element layout, and temporary stack frames during construction or push.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct AttentionStorageBytes {
    pub inline: usize,
    pub payload_heap: usize,
    pub address_heap: usize,
    pub span_heap: usize,
    pub reducer_heap: usize,
    pub total: usize,
}

pub struct NativeAttentionSession<'a> {
    components: NativeAttentionComponents<'a>,
    vocabulary: usize,
    capacity: usize,
    len: usize,
    event: NativeEventState,
    context: NativeContextState,
    span: SpanRegister,
    addresses: Box<[AddressRecord]>,
    heads: [HeadStorage; HEADS],
    last: Option<Committed>,
}
impl<'a> NativeAttentionSession<'a> {
    pub fn new(
        components: NativeAttentionComponents<'a>,
        capacity: usize,
    ) -> AttentionResult<Self> {
        if !(1..=MAX_CONTEXT).contains(&capacity) {
            return Err(AttentionError::Capacity(capacity));
        }
        let vocabulary = components.context.vocab_size();
        let null = components.no_read.config();
        if components.context.heads() != HEADS
            || components.context.lanes_per_head() != LANES_PER_HEAD
            || components.events.vocab_size() != vocabulary
            || components.span.vocab_size() != vocabulary
            || components.span.lanes() != LANES
            || components.potential.heads() != HEADS
            || components.potential.lanes() != LANES_PER_HEAD
            || components.values.vocab_size() != vocabulary
            || components.values.heads() != HEADS
            || components.values.latent_lanes_per_head() != LANES_PER_HEAD
            || null.vocabulary != vocabulary
            || null.heads != HEADS
            || null.latent_lanes_per_head != LANES_PER_HEAD
        {
            return Err(AttentionError::ComponentShape);
        }
        if components.age_context < capacity
            || components.age_context > MAX_CONTEXT
            || components.age_q24.len() != components.age_context * HEADS
        {
            return Err(AttentionError::AgeShape);
        }
        let absent = AddressLane::new(1, 0, false).map_err(AttentionError::Potential)?;
        let make_head = || -> AttentionResult<HeadStorage> {
            Ok(HeadStorage {
                payload: vec![[0; OUTPUT_WIDTH]; capacity].into_boxed_slice(),
                reducer: NativeGeometricComposedRead::new(capacity, components.exp_q31)
                    .map_err(AttentionError::Reduction)?,
            })
        };
        Ok(Self {
            event: NativeEventState::new(components.events.lanes())
                .map_err(AttentionError::Event)?,
            context: NativeContextState::new(HEADS, LANES_PER_HEAD)
                .map_err(AttentionError::Context)?,
            span: SpanRegister::new(LANES).map_err(AttentionError::Span)?,
            addresses: vec![
                AddressRecord {
                    content: [absent; LANES],
                    context: [absent; LANES]
                };
                capacity
            ]
            .into_boxed_slice(),
            heads: [make_head()?, make_head()?],
            components,
            vocabulary,
            capacity,
            len: 0,
            last: None,
        })
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Storage accounting outside the numerical push path. The reducer terms
    /// mirror NativeGeometricRead::new(capacity, 1, exp), which allocates the
    /// prefix-length array at capacity+1 and all other arrays at exact lengths.
    pub fn storage_bytes(&self) -> AttentionStorageBytes {
        use std::mem::size_of;
        let inline = size_of::<Self>();
        let payload_heap = HEADS * self.capacity * size_of::<[i64; OUTPUT_WIDTH]>();
        let address_heap = self.addresses.len() * size_of::<AddressRecord>();
        let span_heap = 2 * self.span.lanes() * size_of::<H4Code>();
        let reducer_heap = HEADS
            * (self.components.exp_q31.len() * size_of::<u32>()
                + (self.capacity + 1) * size_of::<usize>()
                + self.capacity * size_of::<i64>()
                + self.capacity * size_of::<u64>()
                + size_of::<i128>()
                + size_of::<i32>());
        AttentionStorageBytes {
            inline,
            payload_heap,
            address_heap,
            span_heap,
            reducer_heap,
            total: inline + payload_heap + address_heap + span_heap + reducer_heap,
        }
    }
    pub fn last(&self) -> Option<AttentionStep<'_>> {
        self.last.as_ref().map(Committed::view)
    }
    /// Reset all independent causal states and logical history together.
    /// Storage/scratch is retained and stale slots are never in read support.
    pub fn reset(&mut self) {
        self.event.reset();
        self.context.reset();
        self.span.reset();
        self.len = 0;
        self.last = None;
    }

    /// Consume one actual token, never padding. Each push contributes exactly
    /// one candidate regardless of span/context presence or payload zero.
    /// Recoverable errors leave causal states, logical history and last() intact.
    #[inline(never)]
    pub fn push(&mut self, token: usize) -> AttentionResult<AttentionStep<'_>> {
        if token >= self.vocabulary {
            return Err(AttentionError::Token {
                token,
                vocabulary: self.vocabulary,
            });
        }
        if self.len == self.capacity {
            return Err(AttentionError::Full {
                capacity: self.capacity,
            });
        }
        let c = self.components;
        let token_actions = c.span.row(token).map_err(AttentionError::Span)?;
        // This makes the eventual span commit infallible under this module's
        // invariant; no span mutation precedes either head's reduction.
        if token_actions.len() != self.span.lanes() {
            return Err(AttentionError::ComponentShape);
        }
        let mut next_event = self.event;
        let event = next_event
            .step(token, c.events, c.geometry)
            .map_err(AttentionError::Event)?;
        let mut next_context = self.context;
        let context = next_context
            .step(token, c.context, c.geometry)
            .map_err(AttentionError::Context)?;
        let held = self.span.held().map(|row| {
            let mut codes = [H4Code::IDENTITY; LANES];
            codes.copy_from_slice(row);
            codes
        });
        let old_held = held.as_ref().map(|row| row.as_slice());
        let values = c
            .values
            .produce(token, &context.states, old_held, true)
            .map_err(AttentionError::Value)?;
        let null = c
            .no_read
            .score(token, &context.states, &context.output, old_held)
            .map_err(AttentionError::NoRead)?;
        let absent = AddressLane::new(1, 0, false).map_err(AttentionError::Potential)?;
        let mut current = AddressRecord {
            content: [absent; LANES],
            context: context.output,
        };
        if let Some(held) = &held {
            for (address, root) in current.content.iter_mut().zip(held) {
                *address = AddressLane::new(root.index(), CONTENT_RADIUS_BIN, true)
                    .map_err(AttentionError::Potential)?;
            }
        }
        let mut composed = [[0; OUTPUT_WIDTH]; HEADS];
        let mut published = [HeadRecord::zero(), HeadRecord::zero()];
        let count = self.len + 1;
        for (h, ((storage, row), payload)) in self
            .heads
            .iter_mut()
            .zip(published.iter_mut())
            .zip(composed.iter_mut())
            .enumerate()
        {
            let start = h << 2;
            let end = start + LANES_PER_HEAD;
            let packets = values.packets[start..end]
                .try_into()
                .map_err(|_| AttentionError::ComponentShape)?;
            *payload = c
                .composition
                .compose(h, packets)
                .map_err(AttentionError::Composition)?;
            // The scratch slot is outside committed history until the whole
            // push succeeds. A retry overwrites it; it is not publicly visible.
            storage.payload[self.len] = *payload;
            row.no_read_q24 = null[h];
            let age_start = if h == 0 { 0 } else { c.age_context };
            let ages = &c.age_q24[age_start..age_start + c.age_context];
            for (j, previous) in self.addresses[..self.len]
                .iter()
                .chain(std::iter::once(&current))
                .enumerate()
            {
                row.potential[j] = c
                    .potential
                    .score(
                        h,
                        &current.content[start..end],
                        &previous.content[start..end],
                        &current.context[start..end],
                        &previous.context[start..end],
                        c.geometry,
                    )
                    .map_err(AttentionError::Potential)?;
                row.age[j] = ages[self.len - j];
            }
            let reduction = storage
                .reducer
                .reduce(
                    &row.potential[..count],
                    &row.age[..count],
                    null[h],
                    storage.payload[..count].as_flattened(),
                )
                .map_err(AttentionError::Reduction)?;
            row.output.copy_from_slice(reduction.output_q16);
            row.weights[..count].copy_from_slice(reduction.occurrence_weights_q31);
            row.no_read_weight = reduction.no_read_weight_q31;
            row.total = reduction.total_weight_q31;
            row.max_score = reduction.max_score_q24;
        }
        let output = sum_heads(&published[0].output, &published[1].output)
            .map_err(AttentionError::Composition)?;
        self.span
            .step(event.event, token_actions, c.geometry)
            .map_err(AttentionError::Span)?;
        self.event = next_event;
        self.context = next_context;
        self.addresses[self.len] = current;
        let committed = self.last.insert(Committed {
            position: self.len,
            event,
            context,
            held,
            values,
            composed,
            addresses: current,
            heads: published,
            output,
        });
        self.len = count;
        Ok(committed.view())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context_q4::{ContextQ4Config, NativeContextQ4};
    use crate::geometric_no_read::{pack_coefficients, NoReadConfig};
    use crate::geometric_potential::{
        CONTENT_PRESENCE_OFFSET, CONTEXT_PRESENCE_OFFSET, ENTRIES_PER_LANE,
    };
    use crate::geometric_value::ValueState;
    use crate::geometric_value_q4::{NativeValueQ4, ValueQ4Config};

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    struct Fixture {
        events: NativeEventTables,
        context: NativeContextTables,
        span: TokenActionDictionary,
        potential: NativePotentialTables,
        values: NativeValueProducer,
        null: NativeGeometricNoRead,
        composition: NativeGeometricComposition,
        geometry: HistoricalH4Tables,
        ages: Vec<i64>,
        exp: Vec<u32>,
    }
    impl Fixture {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let payload = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/fixtures/historical-h4-tables-v1.bin"
            ))?;
            let geometry = HistoricalH4Tables::from_bytes(&payload)?;
            let mut transition = vec![0; 4 * 128];
            let mut event = vec![0; 4 * 4];
            for (token, action) in [1, 2, 3, 0].into_iter().enumerate() {
                transition[token * 128 + 1] = 1;
                event[token * 4 + action] = 1;
            }
            let events = NativeEventTables::new(
                4,
                1,
                &transition,
                &vec![0; 128 * 128],
                None,
                &event,
                &vec![0; 128 * 4],
            )?;
            let context_cfg = ContextQ4Config {
                vocab_size: 4,
                heads: 2,
                lanes_per_head: 4,
            };
            let mut coefficients = Vec::new();
            for (name, shape) in context_cfg.coefficient_shapes()? {
                let mut q = vec![0i8; shape.iter().product()];
                if name.starts_with("token_") {
                    let classes = if name == "token_category" { 33 } else { 120 };
                    for (row, scores) in q.chunks_exact_mut(classes).enumerate() {
                        let token = row / LANES;
                        let winner = match name.as_str() {
                            "token_transition" if token == 1 => 3,
                            "token_category" if token == 2 => 0,
                            "token_category" => 17,
                            _ => 1,
                        };
                        scores[winner] = 4;
                    }
                }
                coefficients.extend(q);
            }
            let context = NativeContextQ4::new(context_cfg, &pack_coefficients(&coefficients)?)?
                .into_native()?;
            let mut dictionary = vec![1; 4 * LANES];
            dictionary[LANES..2 * LANES].fill(3);
            let span = TokenActionDictionary::new(LANES, &dictionary)?;
            let mut potentials = vec![0; HEADS * LANES_PER_HEAD * ENTRIES_PER_LANE];
            potentials[CONTENT_PRESENCE_OFFSET + 2] = 1 << 24;
            potentials[LANES_PER_HEAD * ENTRIES_PER_LANE + CONTEXT_PRESENCE_OFFSET + 3] = 1 << 24;
            let potential = NativePotentialTables::new(HEADS, LANES_PER_HEAD, &potentials)?;
            let value_cfg = ValueQ4Config {
                vocab_size: 4,
                heads: 2,
                latent_lanes_per_head: 4,
            };
            let mut coefficients = Vec::new();
            for (name, shape) in value_cfg.coefficient_shapes()? {
                let mut q = vec![0i8; shape.iter().product()];
                if name == "token_root" || name == "token_category" {
                    let classes = if name == "token_root" { 120 } else { 32 };
                    for (row, scores) in q.chunks_exact_mut(classes).enumerate() {
                        let token = row / 16;
                        let atom = row % 2;
                        let winner = if name == "token_root" {
                            if token == 3 && atom == 1 {
                                0
                            } else {
                                1
                            }
                        } else if token == 2 || (atom == 1 && token != 3) {
                            0
                        } else {
                            17
                        };
                        scores[winner] = 4;
                    }
                }
                coefficients.extend(q);
            }
            let values =
                NativeValueQ4::new(value_cfg, &pack_coefficients(&coefficients)?)?.into_native()?;
            let null_cfg = NoReadConfig {
                vocabulary: 4,
                heads: 2,
                latent_lanes_per_head: 4,
            };
            let mut q = vec![0; null_cfg.coefficient_count()];
            q[null_cfg.coefficients_per_head()] = 4;
            let null = NativeGeometricNoRead::new(null_cfg, &pack_coefficients(&q)?)?;
            let mut gains = vec![0; 128];
            for h in 0..2 {
                for o in 0..8 {
                    gains[((h * 8 + o) * 4) * 2] = 4;
                }
            }
            let composition = NativeGeometricComposition::new(
                &[1; 128],
                &[1; 128],
                &crate::geometric_composition::pack_gains(&gains)?,
                &payload,
            )?;
            // Monotone integer-only fixture, not an exponential-accuracy claim.
            let exp = (0..crate::geometric_read::EXP_TABLE_LEN)
                .map(|i| ((1u64 << 31) >> (i / 256).min(32)) as u32)
                .collect();
            Ok(Self {
                events,
                context,
                span,
                potential,
                values,
                null,
                composition,
                geometry,
                ages: vec![0; HEADS * MAX_CONTEXT],
                exp,
            })
        }
        fn components(&self) -> NativeAttentionComponents<'_> {
            NativeAttentionComponents {
                events: &self.events,
                context: &self.context,
                span: &self.span,
                potential: &self.potential,
                values: &self.values,
                no_read: &self.null,
                composition: &self.composition,
                geometry: &self.geometry,
                age_q24: &self.ages,
                age_context: MAX_CONTEXT,
                exp_q31: &self.exp,
            }
        }
    }

    #[test]
    fn attention_session_old_held_current_support_separate_heads_and_cancellation() -> TestResult {
        let fixture = Fixture::new()?;
        let mut session = NativeAttentionSession::new(fixture.components(), 8)?;
        for token in [0, 1, 2] {
            let row = session.push(token)?;
            assert!(
                row.held_before.is_none(),
                "COMMIT only affects next occurrence"
            );
            assert_eq!(row.heads[0].occurrence_weights_q31.len(), token + 1);
            assert!(row.content_codes.iter().all(|x| !x.present()));
            if token == 2 {
                assert!(row.context_codes.iter().all(|x| !x.present()));
                assert!(row.context.states.iter().all(|x| x.index() == 3));
                assert!(row
                    .values
                    .packets
                    .iter()
                    .flatten()
                    .all(|x| x.state() == ValueState::PresentZero));
            }
        }
        let row = session.push(3)?;
        assert_eq!(row.position, 3);
        assert!(row
            .held_before
            .ok_or("missing committed span")?
            .iter()
            .all(|x| x.index() == 3));
        assert!(row
            .values
            .packets
            .iter()
            .flatten()
            .all(|x| x.state() == ValueState::PresentNonzero));
        assert_eq!(
            row.values.values_q16, [0; 32],
            "opposite present atoms cancel, not disappear"
        );
        assert_eq!(row.composed_q16, &[[0; 32]; 2]);
        assert_eq!(row.heads[0].potential_q24, &[1 << 24, 1 << 24, 1 << 24, 0]);
        assert_eq!(row.heads[1].potential_q24, &[1 << 24, 1 << 24, 0, 1 << 24]);
        assert_ne!(row.heads[0].total_weight_q31, row.heads[1].total_weight_q31);
        let history: Vec<i64> = [1i64 << 16, 1 << 16, 0, 0]
            .into_iter()
            .flat_map(|value| (0..8).flat_map(move |_| [value, 0, 0, 0]))
            .collect();
        let mut reference = NativeGeometricComposedRead::new(8, &fixture.exp)?;
        for head in &row.heads {
            assert_eq!(head.occurrence_weights_q31.len(), 4);
            assert!(head.occurrence_weights_q31.iter().all(|&w| w > 0));
            let expected =
                reference.reduce(head.potential_q24, &[0; 4], head.no_read_q24, &history)?;
            assert_eq!(head.output_q16.as_slice(), expected.output_q16);
            assert_eq!(head.occurrence_weights_q31, expected.occurrence_weights_q31);
            assert_eq!(head.total_weight_q31, expected.total_weight_q31);
        }
        assert_eq!(
            *row.output_q16,
            sum_heads(row.heads[0].output_q16, row.heads[1].output_q16)?
        );
        let duplicate = session.push(3)?;
        assert_eq!(duplicate.position, 4);
        assert_eq!(duplicate.heads[0].occurrence_weights_q31.len(), 5);
        Ok(())
    }

    #[test]
    fn attention_session_late_second_head_error_is_transactional() -> TestResult {
        let mut fixture = Fixture::new()?;
        fixture.ages[MAX_CONTEXT + 1] = i64::MAX;
        let mut session = NativeAttentionSession::new(fixture.components(), 8)?;
        session.push(0)?;
        let before = format!("{:?}", session.last());
        let span = format!("{:?}", session.span);
        let event = session.event;
        let context = session.context;
        assert!(matches!(session.push(1), Err(AttentionError::Reduction(_))));
        assert_eq!(session.len(), 1);
        assert_eq!(format!("{:?}", session.last()), before);
        assert_eq!(format!("{:?}", session.span), span);
        assert_eq!(session.event, event);
        assert_eq!(session.context, context);
        assert!(matches!(session.push(1), Err(AttentionError::Reduction(_))));
        session.reset();
        assert!(session.is_empty());
        assert!(session.last().is_none());
        assert!(session.push(0)?.held_before.is_none());
        Ok(())
    }

    #[test]
    fn attention_session_admission_capacity_reset_and_fixed_layout() -> TestResult {
        let fixture = Fixture::new()?;
        assert!(NativeAttentionSession::new(fixture.components(), 0).is_err());
        assert!(NativeAttentionSession::new(fixture.components(), 129).is_err());
        let mut wrong = fixture.components();
        wrong.age_context = 1;
        assert!(matches!(
            NativeAttentionSession::new(wrong, 2),
            Err(AttentionError::AgeShape)
        ));
        let mut session = NativeAttentionSession::new(fixture.components(), 2)?;
        assert!(matches!(session.push(4), Err(AttentionError::Token { .. })));
        assert!(session.is_empty());
        session.push(0)?;
        let first = format!("{:?}", session.last());
        session.push(1)?;
        let full = format!("{:?}", session.last());
        assert!(matches!(
            session.push(2),
            Err(AttentionError::Full { capacity: 2 })
        ));
        assert_eq!(format!("{:?}", session.last()), full);
        assert_eq!(session.len(), 2);
        assert!(session.span.held().is_none());
        session.reset();
        session.push(0)?;
        assert_eq!(format!("{:?}", session.last()), first);
        assert!(std::mem::size_of::<AddressRecord>().is_power_of_two());
        assert!(std::mem::size_of::<HeadRecord>().is_power_of_two());
        assert_eq!(session.addresses.len(), 2);
        assert!(session.heads.iter().all(|h| h.payload.len() == 2));
        let storage = session.storage_bytes();
        assert_eq!(storage.inline, std::mem::size_of_val(&session));
        assert_eq!(storage.payload_heap, 2 * 2 * 32 * 8);
        assert_eq!(
            storage.address_heap,
            2 * std::mem::size_of::<AddressRecord>()
        );
        assert_eq!(storage.span_heap, 16 * std::mem::size_of::<H4Code>());
        assert_eq!(
            storage.total,
            storage.inline
                + storage.payload_heap
                + storage.address_heap
                + storage.span_heap
                + storage.reducer_heap
        );
        // Exercise the actual maximum, including the reducer's current-token
        // support and its extra NoRead term at the 128th occurrence.
        let mut maximum = NativeAttentionSession::new(fixture.components(), MAX_CONTEXT)?;
        for position in 0..MAX_CONTEXT {
            let row = maximum.push(position.min(3))?;
            assert_eq!(row.position, position);
            for head in &row.heads {
                assert_eq!(head.occurrence_weights_q31.len(), position + 1);
                let denominator = head.occurrence_weights_q31.iter().copied().sum::<u64>()
                    + head.no_read_weight_q31;
                assert_eq!(head.total_weight_q31, denominator);
            }
        }
        let final_publication = format!("{:?}", maximum.last());
        assert!(matches!(
            maximum.push(3),
            Err(AttentionError::Full {
                capacity: MAX_CONTEXT
            })
        ));
        assert_eq!(maximum.len(), MAX_CONTEXT);
        assert_eq!(format!("{:?}", maximum.last()), final_publication);
        Ok(())
    }
}
