//! Native finite geometric context and event controller.
//!
//! For each token, every lane scores 120 signed H4 actions from a token row,
//! its OLD state row, and (when there is more than one lane) the next OLD lane's
//! row. Earliest argmax selects an action; exact right-composition updates the
//! lane. All lanes update synchronously. Four event scores then combine a
//! lexical row with rows selected by the NEW states. Earliest event argmax uses
//! HOLD, OPEN, APPEND, COMMIT order. No teacher events are inputs.
//!
//! This is a per-lane factorization with one neighboring lane, not an arbitrary
//! lookup over the complete product state. Q24 i32 tables are four-byte score
//! entries, not four-bit linear weights. Source/model/tokenizer identity belongs
//! to the enclosing artifact loader; geometry is the admitted historical H4
//! table. Admission partitions and validates every table and padding cell.
//!
//! State is a fixed four-code array. Numerical step/reset allocate nothing and
//! use no floating point, multiplication or division in source. Transition sums
//! contain at most three i32 coefficients and event sums at most five, all
//! widened to i64. Compiled instruction and complete-serving claims are separate.

use std::fmt;

use crate::geometric_span::SpanAction;
use crate::h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT};

pub const FRACTIONAL_BITS: u32 = 24;
pub const ROOT_STRIDE: usize = 128;
pub const EVENT_COUNT: usize = 4;
pub const MAX_LANES: usize = 4;
pub const MAX_VOCAB: usize = 4096;
pub const TRANSITION_ENTRIES_PER_LANE: usize = ROOT_STRIDE * ROOT_STRIDE;
pub const STATE_EVENT_ENTRIES_PER_LANE: usize = ROOT_STRIDE * EVENT_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTable {
    TokenTransition,
    SelfTransition,
    NeighborTransition,
    TokenEvent,
    StateEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventError {
    InvalidVocabulary(usize),
    InvalidLanes(usize),
    DimensionOverflow,
    TableLength {
        table: EventTable,
        expected: usize,
        actual: usize,
    },
    NonzeroPadding {
        table: EventTable,
        offset: usize,
    },
    MissingNeighbor,
    NonzeroSingleLaneNeighbor {
        offset: usize,
    },
    InvalidCode(u8),
    TokenOutOfRange {
        token: usize,
        vocabulary: usize,
    },
    LaneMismatch {
        state: usize,
        tables: usize,
    },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVocabulary(v) => write!(
                f,
                "geometric event vocabulary {v} is outside 1..={MAX_VOCAB}"
            ),
            Self::InvalidLanes(l) => {
                write!(f, "geometric event lanes {l} are outside 1..={MAX_LANES}")
            }
            Self::DimensionOverflow => f.write_str("geometric event dimensions overflow"),
            Self::TableLength {
                table,
                expected,
                actual,
            } => write!(
                f,
                "geometric event {table:?} needs {expected} entries, got {actual}"
            ),
            Self::NonzeroPadding { table, offset } => write!(
                f,
                "geometric event {table:?} padding at {offset} is nonzero"
            ),
            Self::MissingNeighbor => {
                f.write_str("multiple geometric event lanes require neighbor tables")
            }
            Self::NonzeroSingleLaneNeighbor { offset } => write!(
                f,
                "single-lane geometric event neighbor entry {offset} is nonzero"
            ),
            Self::InvalidCode(code) => write!(f, "geometric event action code {code} is invalid"),
            Self::TokenOutOfRange { token, vocabulary } => write!(
                f,
                "geometric event token {token} is outside 0..{vocabulary}"
            ),
            Self::LaneMismatch { state, tables } => write!(
                f,
                "geometric event state has {state} lanes but tables have {tables}"
            ),
        }
    }
}

impl std::error::Error for EventError {}
pub type EventResult<T> = Result<T, EventError>;

fn validate_lanes(lanes: usize) -> EventResult<()> {
    if !(1..=MAX_LANES).contains(&lanes) {
        return Err(EventError::InvalidLanes(lanes));
    }
    Ok(())
}

fn length(table: EventTable, values: &[i32], expected: usize) -> EventResult<()> {
    if values.len() != expected {
        return Err(EventError::TableLength {
            table,
            expected,
            actual: values.len(),
        });
    }
    Ok(())
}

fn transition_padding(table: EventTable, values: &[i32], root_axis: bool) -> EventResult<()> {
    for (row, values) in values.chunks_exact(ROOT_STRIDE).enumerate() {
        for (action, &value) in values.iter().enumerate() {
            if (action >= ROOT_COUNT || (root_axis && row % ROOT_STRIDE >= ROOT_COUNT))
                && value != 0
            {
                return Err(EventError::NonzeroPadding {
                    table,
                    offset: row * ROOT_STRIDE + action,
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct TokenTables {
    transition: Box<[[i32; ROOT_STRIDE]]>,
    event: [i32; EVENT_COUNT],
}

#[derive(Debug)]
struct LaneTables {
    own: Box<[i32]>,
    neighbor: Option<Box<[i32]>>,
    event: Box<[i32]>,
    neighbor_index: usize,
}

/// Table order is token/lane/action, lane/old-root/action, token/event,
/// lane/new-root/event. Root and action axes have stride128; every code>=120
/// is padding and must be zero. L1 neighbor tables are omitted or all zero.
#[derive(Debug)]
pub struct NativeEventTables {
    tokens: Box<[TokenTables]>,
    lanes: Box<[LaneTables]>,
    actions: [H4Code; ROOT_COUNT],
    coefficient_reads: usize,
}

impl NativeEventTables {
    pub fn new(
        vocab: usize,
        lanes: usize,
        token_transition: &[i32],
        self_transition: &[i32],
        neighbor_transition: Option<&[i32]>,
        token_event: &[i32],
        state_event: &[i32],
    ) -> EventResult<Self> {
        if !(1..=MAX_VOCAB).contains(&vocab) {
            return Err(EventError::InvalidVocabulary(vocab));
        }
        validate_lanes(lanes)?;
        let token_width = lanes
            .checked_mul(ROOT_STRIDE)
            .ok_or(EventError::DimensionOverflow)?;
        let token_entries = vocab
            .checked_mul(token_width)
            .ok_or(EventError::DimensionOverflow)?;
        let transition_entries = lanes
            .checked_mul(TRANSITION_ENTRIES_PER_LANE)
            .ok_or(EventError::DimensionOverflow)?;
        length(EventTable::TokenTransition, token_transition, token_entries)?;
        length(
            EventTable::SelfTransition,
            self_transition,
            transition_entries,
        )?;
        length(
            EventTable::TokenEvent,
            token_event,
            vocab
                .checked_mul(EVENT_COUNT)
                .ok_or(EventError::DimensionOverflow)?,
        )?;
        length(
            EventTable::StateEvent,
            state_event,
            lanes
                .checked_mul(STATE_EVENT_ENTRIES_PER_LANE)
                .ok_or(EventError::DimensionOverflow)?,
        )?;
        transition_padding(EventTable::TokenTransition, token_transition, false)?;
        transition_padding(EventTable::SelfTransition, self_transition, true)?;
        match neighbor_transition {
            None if lanes != 1 => return Err(EventError::MissingNeighbor),
            Some(values) => {
                length(EventTable::NeighborTransition, values, transition_entries)?;
                transition_padding(EventTable::NeighborTransition, values, true)?;
                if lanes == 1 {
                    if let Some(offset) = values.iter().position(|&value| value != 0) {
                        return Err(EventError::NonzeroSingleLaneNeighbor { offset });
                    }
                }
            }
            None => {}
        }
        for (row, values) in state_event.chunks_exact(EVENT_COUNT).enumerate() {
            if row % ROOT_STRIDE >= ROOT_COUNT {
                for (event, &value) in values.iter().enumerate() {
                    if value != 0 {
                        return Err(EventError::NonzeroPadding {
                            table: EventTable::StateEvent,
                            offset: row * EVENT_COUNT + event,
                        });
                    }
                }
            }
        }
        let mut token_tables = Vec::with_capacity(vocab);
        for (transitions, events) in token_transition
            .chunks_exact(token_width)
            .zip(token_event.chunks_exact(EVENT_COUNT))
        {
            let mut rows = Vec::with_capacity(lanes);
            for row in transitions.chunks_exact(ROOT_STRIDE) {
                let mut owned = [0; ROOT_STRIDE];
                owned.copy_from_slice(row);
                rows.push(owned);
            }
            let mut event = [0; EVENT_COUNT];
            event.copy_from_slice(events);
            token_tables.push(TokenTables {
                transition: rows.into_boxed_slice(),
                event,
            });
        }
        let mut lane_tables = Vec::with_capacity(lanes);
        for (lane, (own, events)) in self_transition
            .chunks_exact(TRANSITION_ENTRIES_PER_LANE)
            .zip(state_event.chunks_exact(STATE_EVENT_ENTRIES_PER_LANE))
            .enumerate()
        {
            let neighbor = if lanes == 1 {
                None
            } else {
                let values = neighbor_transition.ok_or(EventError::MissingNeighbor)?;
                let start = lane * TRANSITION_ENTRIES_PER_LANE;
                Some(values[start..start + TRANSITION_ENTRIES_PER_LANE].into())
            };
            lane_tables.push(LaneTables {
                own: own.into(),
                neighbor,
                event: events.into(),
                neighbor_index: if lane + 1 == lanes { 0 } else { lane + 1 },
            });
        }
        let mut actions = [H4Code::IDENTITY; ROOT_COUNT];
        for (index, code) in actions.iter_mut().enumerate() {
            *code =
                H4Code::try_from(index as u8).map_err(|_| EventError::InvalidCode(index as u8))?;
        }
        let transition_families = if lanes == 1 { 2 } else { 3 };
        Ok(Self {
            tokens: token_tables.into_boxed_slice(),
            lanes: lane_tables.into_boxed_slice(),
            actions,
            coefficient_reads: lanes * ROOT_COUNT * transition_families
                + EVENT_COUNT
                + lanes * EVENT_COUNT,
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.tokens.len()
    }
    pub fn lanes(&self) -> usize {
        self.lanes.len()
    }
}

/// Active prefixes have length `lanes`; inactive array slots remain identity.
/// The read count covers learned score coefficients, excluding fixed group
/// lookups, metadata and data-structure access. It is not an energy measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventStep {
    pub lanes: usize,
    pub states: [H4Code; MAX_LANES],
    pub actions: [H4Code; MAX_LANES],
    pub event: SpanAction,
    pub event_scores: [i64; EVENT_COUNT],
    pub coefficient_reads: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeEventState {
    states: [H4Code; MAX_LANES],
    lanes: usize,
}

impl NativeEventState {
    pub fn new(lanes: usize) -> EventResult<Self> {
        validate_lanes(lanes)?;
        Ok(Self {
            states: [H4Code::IDENTITY; MAX_LANES],
            lanes,
        })
    }

    pub fn states(&self) -> &[H4Code] {
        &self.states[..self.lanes]
    }

    pub fn reset(&mut self) {
        self.states.fill(H4Code::IDENTITY);
    }

    /// Advance on an actual token ID. Returned errors leave all state intact.
    /// Every transition observes old states; every event observes new states.
    pub fn step(
        &mut self,
        token_id: usize,
        tables: &NativeEventTables,
        geometry: &HistoricalH4Tables,
    ) -> EventResult<EventStep> {
        if self.lanes != tables.lanes.len() {
            return Err(EventError::LaneMismatch {
                state: self.lanes,
                tables: tables.lanes.len(),
            });
        }
        let token = tables
            .tokens
            .get(token_id)
            .ok_or(EventError::TokenOutOfRange {
                token: token_id,
                vocabulary: tables.tokens.len(),
            })?;
        let old = self.states;
        let mut next = old;
        let mut actions = [H4Code::IDENTITY; MAX_LANES];
        for (lane, (row, local)) in token.transition.iter().zip(tables.lanes.iter()).enumerate() {
            let own_start = usize::from(old[lane].index()) << 7;
            let own = &local.own[own_start..own_start + ROOT_COUNT];
            let neighbor_start = usize::from(old[local.neighbor_index].index()) << 7;
            let neighbor = local
                .neighbor
                .as_ref()
                .map(|values| &values[neighbor_start..neighbor_start + ROOT_COUNT]);
            let mut winner = 0;
            let mut best = i64::MIN;
            for action in 0..ROOT_COUNT {
                let mut score = i64::from(row[action]) + i64::from(own[action]);
                if let Some(neighbor) = neighbor {
                    score += i64::from(neighbor[action]);
                }
                if score > best {
                    best = score;
                    winner = action;
                }
            }
            actions[lane] = tables.actions[winner];
            next[lane] = geometry.compose(old[lane], actions[lane]);
        }
        let mut event_scores = token.event.map(i64::from);
        for (state, lane) in next.iter().zip(tables.lanes.iter()) {
            let start = usize::from(state.index()) << 2;
            for (score, &coefficient) in event_scores
                .iter_mut()
                .zip(&lane.event[start..start + EVENT_COUNT])
            {
                *score += i64::from(coefficient);
            }
        }
        let mut winner = 0;
        for event in 1..EVENT_COUNT {
            if event_scores[event] > event_scores[winner] {
                winner = event;
            }
        }
        let event = [
            SpanAction::Hold,
            SpanAction::Open,
            SpanAction::Append,
            SpanAction::Commit,
        ][winner];
        self.states = next;
        Ok(EventStep {
            lanes: self.lanes,
            states: next,
            actions,
            event,
            event_scores,
            coefficient_reads: tables.coefficient_reads,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn geometry() -> Result<HistoricalH4Tables, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))?;
        Ok(HistoricalH4Tables::from_bytes(&bytes)?)
    }

    struct Fixture {
        vocab: usize,
        lanes: usize,
        token: Vec<i32>,
        own: Vec<i32>,
        neighbor: Vec<i32>,
        event: Vec<i32>,
        state: Vec<i32>,
    }
    impl Fixture {
        fn new(vocab: usize, lanes: usize) -> Self {
            Self {
                vocab,
                lanes,
                token: vec![0; vocab * lanes * ROOT_STRIDE],
                own: vec![0; lanes * TRANSITION_ENTRIES_PER_LANE],
                neighbor: vec![0; lanes * TRANSITION_ENTRIES_PER_LANE],
                event: vec![0; vocab * EVENT_COUNT],
                state: vec![0; lanes * STATE_EVENT_ENTRIES_PER_LANE],
            }
        }
        fn tables(&self) -> EventResult<NativeEventTables> {
            NativeEventTables::new(
                self.vocab,
                self.lanes,
                &self.token,
                &self.own,
                if self.lanes == 1 {
                    None
                } else {
                    Some(&self.neighbor)
                },
                &self.event,
                &self.state,
            )
        }
    }

    #[test]
    fn native_geometric_event_ties_signed_order_and_new_state_events() -> TestResult {
        let geometry = geometry()?;
        let zeros = Fixture::new(1, 1).tables()?;
        let mut state = NativeEventState::new(1)?;
        let tie = state.step(0, &zeros, &geometry)?;
        assert_eq!(tie.actions[0].index(), 0); // earliest is -1, not identity
        assert_eq!(tie.states[0].index(), 0);
        assert_eq!(tie.event, SpanAction::Hold);
        assert_eq!(tie.event_scores, [0; 4]);
        assert_eq!(tie.coefficient_reads, 248);
        assert_eq!(
            state.step(0, &zeros, &geometry)?.states[0],
            H4Code::IDENTITY
        );
        let mut fixture = Fixture::new(2, 1);
        fixture.token[3] = 10;
        fixture.token[ROOT_STRIDE + 5] = 10; // +i then +j
        fixture.state[7 * EVENT_COUNT + 3] = 30; // +k -> COMMIT
        fixture.state[6 * EVENT_COUNT + 2] = 30; // -k -> APPEND
        let tables = fixture.tables()?;
        state.reset();
        state.step(0, &tables, &geometry)?;
        let ij = state.step(1, &tables, &geometry)?;
        assert_eq!((ij.states[0].index(), ij.event), (7, SpanAction::Commit));
        state.reset();
        state.step(1, &tables, &geometry)?;
        let ji = state.step(0, &tables, &geometry)?;
        assert_eq!((ji.states[0].index(), ji.event), (6, SpanAction::Append));
        assert!(ji.states[1..]
            .iter()
            .chain(ji.actions[1..].iter())
            .all(|&code| code == H4Code::IDENTITY));
        Ok(())
    }

    #[test]
    fn native_geometric_event_neighbor_updates_are_synchronous() -> TestResult {
        let geometry = geometry()?;
        let mut fixture = Fixture::new(2, 2);
        fixture.token[3] = 100;
        fixture.token[ROOT_STRIDE + 5] = 100;
        fixture.neighbor[5 * ROOT_STRIDE + 3] = 10; // lane0 sees OLD j, picks i
        fixture.neighbor[TRANSITION_ENTRIES_PER_LANE + 3 * ROOT_STRIDE + 5] = 10; // lane1 sees OLD i, picks j
        fixture.neighbor[TRANSITION_ENTRIES_PER_LANE + 1] = 20; // seeing new -1 would wrongly choose identity
        let tables = fixture.tables()?;
        let mut state = NativeEventState::new(2)?;
        let first = state.step(0, &tables, &geometry)?;
        assert_eq!([first.states[0].index(), first.states[1].index()], [3, 5]);
        let second = state.step(1, &tables, &geometry)?;
        assert_eq!(
            [second.actions[0].index(), second.actions[1].index()],
            [3, 5]
        );
        assert_eq!([second.states[0].index(), second.states[1].index()], [0, 0]);
        assert_eq!(second.coefficient_reads, 732);
        assert_eq!(state.states(), &second.states[..2]);
        Ok(())
    }

    #[test]
    fn native_geometric_event_admission_padding_and_atomic_errors() -> TestResult {
        let geometry = geometry()?;
        for vocab in [0, MAX_VOCAB + 1, usize::MAX] {
            assert!(matches!(
                NativeEventTables::new(vocab, 1, &[], &[], None, &[], &[]),
                Err(EventError::InvalidVocabulary(_))
            ));
        }
        for lanes in [0, MAX_LANES + 1, usize::MAX] {
            assert!(matches!(
                NativeEventState::new(lanes),
                Err(EventError::InvalidLanes(_))
            ));
        }
        let mut fixture = Fixture::new(1, 2);
        assert!(matches!(
            NativeEventTables::new(
                1,
                2,
                &fixture.token,
                &fixture.own,
                None,
                &fixture.event,
                &fixture.state
            ),
            Err(EventError::MissingNeighbor)
        ));
        for (family, offset) in [
            (EventTable::TokenTransition, 120),
            (EventTable::SelfTransition, 120 * ROOT_STRIDE),
            (EventTable::SelfTransition, 120),
            (EventTable::NeighborTransition, 120),
            (EventTable::StateEvent, 120 * EVENT_COUNT),
        ] {
            let values = match family {
                EventTable::TokenTransition => &mut fixture.token,
                EventTable::SelfTransition => &mut fixture.own,
                EventTable::NeighborTransition => &mut fixture.neighbor,
                EventTable::StateEvent => &mut fixture.state,
                EventTable::TokenEvent => &mut fixture.event,
            };
            values[offset] = 1;
            assert!(matches!(
                fixture.tables(),
                Err(EventError::NonzeroPadding { .. })
            ));
            let values = match family {
                EventTable::TokenTransition => &mut fixture.token,
                EventTable::SelfTransition => &mut fixture.own,
                EventTable::NeighborTransition => &mut fixture.neighbor,
                EventTable::StateEvent => &mut fixture.state,
                EventTable::TokenEvent => &mut fixture.event,
            };
            values[offset] = 0;
        }
        let tables = fixture.tables()?;
        let mut state = NativeEventState::new(2)?;
        state.step(0, &tables, &geometry)?;
        let before = state;
        assert!(matches!(
            state.step(usize::MAX, &tables, &geometry),
            Err(EventError::TokenOutOfRange { .. })
        ));
        assert_eq!(state, before);
        let mut wrong = NativeEventState::new(1)?;
        let before = wrong;
        assert!(matches!(
            wrong.step(0, &tables, &geometry),
            Err(EventError::LaneMismatch { .. })
        ));
        assert_eq!(wrong, before);
        let single = Fixture::new(1, 1);
        NativeEventTables::new(
            1,
            1,
            &single.token,
            &single.own,
            Some(&single.neighbor),
            &single.event,
            &single.state,
        )?;
        let mut bad = single.neighbor.clone();
        bad[0] = 1;
        assert!(matches!(
            NativeEventTables::new(
                1,
                1,
                &single.token,
                &single.own,
                Some(&bad),
                &single.event,
                &single.state
            ),
            Err(EventError::NonzeroSingleLaneNeighbor { .. })
        ));
        assert!(matches!(
            NativeEventTables::new(
                1,
                1,
                &single.token[..127],
                &single.own,
                None,
                &single.event,
                &single.state
            ),
            Err(EventError::TableLength { .. })
        ));
        Ok(())
    }

    #[test]
    fn native_geometric_event_extreme_scores_and_fixed_state_layout() -> TestResult {
        let geometry = geometry()?;
        for value in [i32::MIN, i32::MAX] {
            let mut fixture = Fixture::new(1, MAX_LANES);
            for row in fixture.token.chunks_exact_mut(ROOT_STRIDE) {
                row[..ROOT_COUNT].fill(value);
            }
            for family in [&mut fixture.own, &mut fixture.neighbor] {
                for lane in family.chunks_exact_mut(TRANSITION_ENTRIES_PER_LANE) {
                    for row in lane.chunks_exact_mut(ROOT_STRIDE).take(ROOT_COUNT) {
                        row[..ROOT_COUNT].fill(value);
                    }
                }
            }
            fixture.event.fill(value);
            for lane in fixture.state.chunks_exact_mut(STATE_EVENT_ENTRIES_PER_LANE) {
                lane[..ROOT_COUNT * EVENT_COUNT].fill(value);
            }
            let tables = fixture.tables()?;
            let mut state = NativeEventState::new(MAX_LANES)?;
            let step = state.step(0, &tables, &geometry)?;
            assert!(step.actions.iter().all(|code| code.index() == 0));
            assert_eq!(step.event_scores, [5 * i64::from(value); 4]);
            assert_eq!(step.event, SpanAction::Hold);
            assert_eq!(step.coefficient_reads, 1460);
            state.reset();
            assert_eq!(state.states(), &[H4Code::IDENTITY; MAX_LANES]);
        }
        // Copy is available because the complete state is fixed inline data,
        // with no heap owner or destructor in the numerical state object.
        fn assert_copy<T: Copy>() {}
        assert_copy::<NativeEventState>();
        assert!(!std::mem::needs_drop::<NativeEventState>());
        Ok(())
    }
}
