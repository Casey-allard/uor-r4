//! Strict packed-q4 coefficients for the existing native event controller.
//!
//! Source order is token transition [V,L,120], own transition [L,120,4],
//! neighbor transition [L,120,4] (omitted at L1), token event [V,4], then
//! state event [L,4,4]. Each signed coefficient is in [-7,7], with fixed
//! quarter-nat units; -8 is reserved. Tokens expand to q<<22. A root factor
//! expands to round_ties_away(sum(q_i * B_i)/8) in Q24, where B is the shared
//! canonical-F32 Q25 observation basis. That basis does not replace the exact
//! signed H4 algebra supplied to the existing controller.
//!
//! Expanded transition tables use root/action stride128, event stride4, and
//! zero padding. Transition sums are bounded by 5.25 nats at L1 and 8.75 nats
//! otherwise. Event sums are bounded by 1.75 + 3.5*L nats (15.75 at L4).
//! Wide table entries are derived score precision, not free learned weights.
//! There is no centering, selected scale, or clipping. Admission allocates;
//! all runtime state, synchronous old-neighbor timing, right composition and
//! earliest-tie behavior remain in NativeEventTables/NativeEventState.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::geometric_context_q4::ContextQ4Error;
pub use crate::geometric_context_q4::{pack_coefficients, unpack_coefficients};
use crate::geometric_event::{
    EventError, NativeEventTables, EVENT_COUNT, MAX_LANES, MAX_VOCAB, ROOT_STRIDE,
};
use crate::geometric_no_read::CANONICAL_BASIS_Q25;

pub const SCHEMA: &str = "uor-r4.geometric-event-q4/1";
pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;fixed-quarter-nat;canonical-F32-Q25-observation;token-shift22/basis-div8;per-factor-Q24-nearest-ties-away;token-self-neighbor-transition/token-state-event;L1-neighbor-omitted;root-action128-zero-padding;no-centering/1";
pub const FRACTIONAL_BITS: u32 = 24;
pub const COEFFICIENT_SHIFT: u32 = 22;
pub const ROOT_COUNT: usize = 120;
pub const FAMILY_NAMES: [&str; 5] = [
    "token_transition",
    "self_transition",
    "neighbor_transition",
    "token_event",
    "state_event",
];
pub const MAX_ABS_TRANSITION_SCORE_Q24: i64 = 35i64 << 22;
pub const MAX_ABS_SINGLE_LANE_TRANSITION_SCORE_Q24: i64 = 21i64 << 22;
pub const MAX_ABS_EVENT_SCORE_Q24: i64 = 63i64 << 22;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventQ4Config {
    pub vocab_size: usize,
    pub lanes: usize,
}
impl EventQ4Config {
    pub fn validate(self) -> EventQ4Result<()> {
        if !(1..=MAX_VOCAB).contains(&self.vocab_size) || !(1..=MAX_LANES).contains(&self.lanes) {
            return Err(EventQ4Error::Configuration);
        }
        Ok(())
    }
    /// Pinned packed-source order, independent of any parameter map ordering.
    pub fn coefficient_shapes(self) -> EventQ4Result<Vec<(String, Vec<usize>)>> {
        self.validate()?;
        let mut result = vec![
            (
                FAMILY_NAMES[0].to_owned(),
                vec![self.vocab_size, self.lanes, ROOT_COUNT],
            ),
            (FAMILY_NAMES[1].to_owned(), vec![self.lanes, ROOT_COUNT, 4]),
        ];
        if self.lanes > 1 {
            result.push((FAMILY_NAMES[2].to_owned(), vec![self.lanes, ROOT_COUNT, 4]));
        }
        result.push((
            FAMILY_NAMES[3].to_owned(),
            vec![self.vocab_size, EVENT_COUNT],
        ));
        result.push((FAMILY_NAMES[4].to_owned(), vec![self.lanes, EVENT_COUNT, 4]));
        Ok(result)
    }
    pub fn coefficient_count(self) -> EventQ4Result<usize> {
        self.coefficient_shapes()?
            .iter()
            .try_fold(0usize, |total, (_, shape)| {
                let count = shape
                    .iter()
                    .try_fold(1usize, |p, &d| p.checked_mul(d))
                    .ok_or(EventQ4Error::ArithmeticOverflow)?;
                total
                    .checked_add(count)
                    .ok_or(EventQ4Error::ArithmeticOverflow)
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventQ4Error {
    Configuration,
    Codec(ContextQ4Error),
    BasisBounds,
    ArithmeticOverflow,
    TableMismatch,
    Native(EventError),
}
impl fmt::Display for EventQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric event q4: {self:?}")
    }
}
impl std::error::Error for EventQ4Error {}
impl From<ContextQ4Error> for EventQ4Error {
    fn from(error: ContextQ4Error) -> Self {
        Self::Codec(error)
    }
}
pub type EventQ4Result<T> = Result<T, EventQ4Error>;

pub fn canonical_basis_q25() -> [[i32; 4]; ROOT_COUNT] {
    CANONICAL_BASIS_Q25
}
fn validate_basis() -> EventQ4Result<()> {
    if CANONICAL_BASIS_Q25[0] != [-(1 << 25), 0, 0, 0]
        || CANONICAL_BASIS_Q25[1] != [1 << 25, 0, 0, 0]
    {
        return Err(EventQ4Error::BasisBounds);
    }
    for root in CANONICAL_BASIS_Q25 {
        let mut l1 = 0i64;
        for x in root {
            let a = i64::from(x).abs();
            if a > 1 << 25 {
                return Err(EventQ4Error::BasisBounds);
            }
            l1 = l1.checked_add(a).ok_or(EventQ4Error::ArithmeticOverflow)?;
        }
        if l1 > 2 << 25 {
            return Err(EventQ4Error::BasisBounds);
        }
    }
    Ok(())
}

fn round_div8(sum: i64) -> EventQ4Result<i32> {
    let magnitude = sum
        .unsigned_abs()
        .checked_add(4)
        .ok_or(EventQ4Error::ArithmeticOverflow)?
        >> 3;
    let value = i64::try_from(magnitude).map_err(|_| EventQ4Error::ArithmeticOverflow)?;
    let signed = if sum < 0 { -value } else { value };
    i32::try_from(signed).map_err(|_| EventQ4Error::ArithmeticOverflow)
}
/// Admission only. Each four-term integer dot is rounded once, never per axis.
fn basis_score(q: &[i8], root: &[i32; 4]) -> EventQ4Result<i32> {
    let mut sum = 0i64;
    for (&q, &b) in q.iter().zip(root) {
        let term = i64::from(q)
            .checked_mul(i64::from(b))
            .ok_or(EventQ4Error::ArithmeticOverflow)?;
        sum = sum
            .checked_add(term)
            .ok_or(EventQ4Error::ArithmeticOverflow)?;
    }
    round_div8(sum)
}

#[derive(Clone, Copy, Debug)]
pub struct EventQ4TableSlices<'a> {
    pub token_transition: &'a [i32],
    pub self_transition: &'a [i32],
    pub neighbor_transition: Option<&'a [i32]>,
    pub token_event: &'a [i32],
    pub state_event: &'a [i32],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventQ4Stats {
    pub learned_coefficients: usize,
    pub packed_bytes: usize,
    /// Serialized expanded payload only; excludes admission copies and metadata.
    pub expanded_table_entries: usize,
    pub expanded_table_bytes: usize,
    pub max_abs_transition_score_q24: i64,
    pub max_abs_event_score_q24: i64,
}
pub struct NativeEventQ4 {
    config: EventQ4Config,
    packed: Box<[u8]>,
    tables: [Vec<i32>; 5],
    native: NativeEventTables,
    coefficient_count: usize,
}
fn table_slices(t: &[Vec<i32>; 5], neighbor: bool) -> EventQ4TableSlices<'_> {
    EventQ4TableSlices {
        token_transition: &t[0],
        self_transition: &t[1],
        neighbor_transition: neighbor.then_some(t[2].as_slice()),
        token_event: &t[3],
        state_event: &t[4],
    }
}
impl NativeEventQ4 {
    pub fn new(config: EventQ4Config, packed: &[u8]) -> EventQ4Result<Self> {
        let coefficient_count = config.coefficient_count()?;
        let coefficients = unpack_coefficients(coefficient_count, packed)?;
        validate_basis()?;
        let mut tables: [Vec<i32>; 5] = std::array::from_fn(|_| Vec::new());
        let mut at = 0usize;
        tables[0] = vec![0; config.vocab_size * config.lanes * ROOT_STRIDE];
        for row in tables[0].chunks_exact_mut(ROOT_STRIDE) {
            for entry in &mut row[..ROOT_COUNT] {
                *entry = i32::from(coefficients[at]) << COEFFICIENT_SHIFT;
                at += 1;
            }
        }
        for family in [1usize, 2, 4] {
            if family == 2 && config.lanes == 1 {
                continue;
            }
            // The lexical event family precedes the state-event basis family.
            if family == 4 {
                tables[3] = coefficients[at..at + config.vocab_size * EVENT_COUNT]
                    .iter()
                    .map(|&q| i32::from(q) << COEFFICIENT_SHIFT)
                    .collect();
                at += config.vocab_size * EVENT_COUNT;
            }
            let classes = if family == 4 { EVENT_COUNT } else { ROOT_COUNT };
            let stride = if family == 4 {
                EVENT_COUNT
            } else {
                ROOT_STRIDE
            };
            tables[family] = vec![0; config.lanes * ROOT_STRIDE * stride];
            for lane in 0..config.lanes {
                for (state, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
                    for choice in 0..classes {
                        let start = at + (lane * classes + choice) * 4;
                        tables[family][(lane * ROOT_STRIDE + state) * stride + choice] =
                            basis_score(&coefficients[start..start + 4], root)?;
                    }
                }
            }
            at += config.lanes * classes * 4;
        }
        if at != coefficient_count {
            return Err(EventQ4Error::ArithmeticOverflow);
        }
        let slices = table_slices(&tables, config.lanes > 1);
        let native = NativeEventTables::new(
            config.vocab_size,
            config.lanes,
            slices.token_transition,
            slices.self_transition,
            slices.neighbor_transition,
            slices.token_event,
            slices.state_event,
        )
        .map_err(EventQ4Error::Native)?;
        Ok(Self {
            config,
            packed: packed.into(),
            tables,
            native,
            coefficient_count,
        })
    }
    /// Wide payloads are accepted only if every entry, including padding, is
    /// independently regenerated from the packed coefficients.
    pub fn from_parts(
        config: EventQ4Config,
        packed: &[u8],
        expanded: &[i32],
    ) -> EventQ4Result<Self> {
        let admitted = Self::new(config, packed)?;
        if admitted.tables.iter().map(Vec::len).sum::<usize>() != expanded.len()
            || !admitted.tables.iter().flatten().eq(expanded.iter())
        {
            return Err(EventQ4Error::TableMismatch);
        }
        Ok(admitted)
    }
    pub fn config(&self) -> EventQ4Config {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    /// Always five entries; the omitted L1 neighbor length is zero.
    pub fn expanded_lengths(&self) -> [usize; 5] {
        std::array::from_fn(|i| self.tables[i].len())
    }
    pub fn expanded_q24(&self) -> Vec<i32> {
        self.tables.iter().flatten().copied().collect()
    }
    pub fn table_bytes(&self) -> Vec<u8> {
        self.tables
            .iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect()
    }
    pub fn table_slices(&self) -> EventQ4TableSlices<'_> {
        table_slices(&self.tables, self.config.lanes > 1)
    }
    pub fn native(&self) -> &NativeEventTables {
        &self.native
    }
    pub fn into_native(self) -> EventQ4Result<NativeEventTables> {
        Ok(self.native)
    }
    pub fn stats(&self) -> EventQ4Stats {
        let entries = self.tables.iter().map(Vec::len).sum::<usize>();
        EventQ4Stats {
            learned_coefficients: self.coefficient_count,
            packed_bytes: self.packed.len(),
            expanded_table_entries: entries,
            expanded_table_bytes: entries * 4,
            max_abs_transition_score_q24: if self.config.lanes == 1 {
                MAX_ABS_SINGLE_LANE_TRANSITION_SCORE_Q24
            } else {
                MAX_ABS_TRANSITION_SCORE_Q24
            },
            max_abs_event_score_q24: (7 + 14 * self.config.lanes as i64) << COEFFICIENT_SHIFT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_event::NativeEventState;
    use crate::geometric_span::SpanAction;
    use crate::h4_tables::{H4Code, HistoricalH4Tables};
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn config(vocab_size: usize, lanes: usize) -> EventQ4Config {
        EventQ4Config { vocab_size, lanes }
    }
    fn geometry() -> Result<HistoricalH4Tables, Box<dyn std::error::Error>> {
        Ok(HistoricalH4Tables::from_bytes(include_bytes!(
            "../fixtures/historical-h4-tables-v1.bin"
        ))?)
    }
    fn oracle(q: &[i8], root: &[i32; 4]) -> i32 {
        let numerator: i128 = q
            .iter()
            .zip(root)
            .map(|(&q, &b)| i128::from(q) * i128::from(b))
            .sum();
        let magnitude = numerator.unsigned_abs();
        let rounded = magnitude / 8 + u128::from(magnitude % 8 >= 4);
        if numerator < 0 {
            -(rounded as i32)
        } else {
            rounded as i32
        }
    }
    #[test]
    fn event_q4_codec_dimensions_and_single_lane_padding() -> TestResult {
        let coefficients = [-7, -1, 0, 1, 7];
        assert_eq!(pack_coefficients(&coefficients)?, [0xf9, 0x10, 7]);
        assert_eq!(unpack_coefficients(5, &[0xf9, 0x10, 7])?, coefficients);
        assert!(pack_coefficients(&[-8]).is_err());
        assert!(pack_coefficients(&[8]).is_err());
        assert!(unpack_coefficients(2, &[8]).is_err());
        assert!(unpack_coefficients(1, &[0x10]).is_err());
        assert!(unpack_coefficients(2, &[]).is_err());
        for bad in [
            config(0, 1),
            config(4097, 1),
            config(1, 0),
            config(1, 5),
            config(usize::MAX, 1),
        ] {
            assert_eq!(bad.validate(), Err(EventQ4Error::Configuration));
            assert!(NativeEventQ4::new(bad, &[]).is_err());
        }
        assert_eq!(config(4096, 4).coefficient_count()?, 1_986_368);
        let c = config(1, 1);
        assert_eq!(c.coefficient_count()?, 620);
        assert_eq!(
            c.coefficient_shapes()?
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            [
                "token_transition",
                "self_transition",
                "token_event",
                "state_event"
            ]
        );
        let q = NativeEventQ4::new(c, &pack_coefficients(&vec![0; c.coefficient_count()?])?)?;
        assert_eq!(q.expanded_lengths(), [128, 16384, 0, 4, 512]);
        assert!(q.table_slices().neighbor_transition.is_none());
        assert_eq!(q.stats().max_abs_event_score_q24, 21i64 << 22);
        let mut state = NativeEventState::new(1)?;
        let step = state.step(0, q.native(), &geometry()?)?;
        assert_eq!(step.actions[0].index(), 0);
        assert_eq!(step.states[0].index(), 0);
        assert_eq!(step.event, SpanAction::Hold);
        assert_eq!(
            state.step(0, q.native(), &geometry()?)?.states[0],
            H4Code::IDENTITY
        );
        Ok(())
    }
    #[test]
    fn event_q4_expansion_matches_integer_oracle_and_rejects_tamper() -> TestResult {
        let c = config(2, 4);
        let coefficients: Vec<i8> = (0..c.coefficient_count()?)
            .map(|i| ((i * 11 + 3) % 15) as i8 - 7)
            .collect();
        let packed = pack_coefficients(&coefficients)?;
        let q = NativeEventQ4::new(c, &packed)?;
        let slices = q.table_slices();
        let views = [
            slices.token_transition,
            slices.self_transition,
            slices.neighbor_transition.ok_or("missing neighbor")?,
            slices.token_event,
            slices.state_event,
        ];
        let mut at = 0;
        for row in views[0].chunks_exact(ROOT_STRIDE) {
            for &entry in &row[..120] {
                assert_eq!(entry, i32::from(coefficients[at]) << 22);
                at += 1;
            }
            assert!(row[120..].iter().all(|&x| x == 0));
        }
        for family in [1usize, 2, 4] {
            if family == 4 {
                for &entry in views[3] {
                    assert_eq!(entry, i32::from(coefficients[at]) << 22);
                    at += 1;
                }
            }
            let classes = if family == 4 { 4 } else { 120 };
            let stride = if family == 4 { 4 } else { 128 };
            for lane in 0..c.lanes {
                for state in 0..128 {
                    for choice in 0..stride {
                        let actual = views[family][(lane * 128 + state) * stride + choice];
                        if state < 120 && choice < classes {
                            let start = at + (lane * classes + choice) * 4;
                            assert_eq!(
                                actual,
                                oracle(
                                    &coefficients[start..start + 4],
                                    &CANONICAL_BASIS_Q25[state]
                                )
                            );
                        } else {
                            assert_eq!(actual, 0);
                        }
                    }
                }
            }
            at += c.lanes * classes * 4;
        }
        assert_eq!(at, coefficients.len());
        assert_eq!(q.stats().max_abs_event_score_q24, MAX_ABS_EVENT_SCORE_Q24);
        let expanded = q.expanded_q24();
        assert_eq!(q.table_bytes().len(), expanded.len() * 4);
        assert_eq!(
            NativeEventQ4::from_parts(c, &packed, &expanded)?.packed_coefficients(),
            packed
        );
        let mut altered = expanded.clone();
        altered[120] = 1; // Padded action, not a live coefficient.
        assert!(matches!(
            NativeEventQ4::from_parts(c, &packed, &altered),
            Err(EventQ4Error::TableMismatch)
        ));
        altered[120] = 0;
        altered[0] += 1;
        assert!(matches!(
            NativeEventQ4::from_parts(c, &packed, &altered),
            Err(EventQ4Error::TableMismatch)
        ));
        assert!(NativeEventQ4::from_parts(c, &packed, &expanded[..expanded.len() - 1]).is_err());
        Ok(())
    }
    #[test]
    fn event_q4_basis_rounding_extremes_and_antipodes() -> TestResult {
        for (n, expected) in [
            (3, 0),
            (4, 1),
            (5, 1),
            (-3, 0),
            (-4, -1),
            (-5, -1),
            (12, 2),
            (-12, -2),
        ] {
            assert_eq!(round_div8(n)?, expected);
        }
        assert!(round_div8(i64::MIN).is_err());
        let mut largest = 0i32;
        let mut saw_half_tie = false;
        for root in CANONICAL_BASIS_Q25 {
            let q = root.map(|b| if b < 0 { -7 } else { 7 });
            let score = basis_score(&q, &root)?;
            assert_eq!(score, oracle(&q, &root));
            assert_eq!(basis_score(&q, &root.map(|b| -b))?, -score);
            largest = largest.max(score);
            let numerator: i64 = [2, 0, 0, 0]
                .iter()
                .zip(root)
                .map(|(&a, b)| i64::from(a) * i64::from(b))
                .sum();
            if numerator.unsigned_abs() % 8 == 4 {
                saw_half_tie = true;
                assert_eq!(
                    basis_score(&[2, 0, 0, 0], &root)?,
                    oracle(&[2, 0, 0, 0], &root)
                );
            }
        }
        assert_eq!(largest, 14 << 22); // Exact half-coordinate roots attain 3.5 nats.
        assert!(saw_half_tie);
        Ok(())
    }
    #[test]
    fn event_q4_synchronous_old_neighbor_and_new_state_event_match_manual_tables() -> TestResult {
        let c = config(2, 2);
        let tt_len = 2 * 2 * 120;
        let st_len = 2 * 120 * 4;
        let nt_start = tt_len + st_len;
        let te_start = nt_start + st_len;
        let se_start = te_start + 2 * 4;
        let mut coefficients = vec![0; c.coefficient_count()?];
        coefficients[3] = 7; // First token: lane0 i, lane1 j.
        coefficients[120 + 5] = 7;
        coefficients[2 * 120 + 5] = 7; // Second token: lane0 appends j.
        coefficients[tt_len + 5 * 4 + 1] = 1; // Its old i increases this score.
        coefficients[nt_start + (120 + 3) * 4 + 1] = 4; // Lane1 sees OLD lane0 i, not NEW k.
        coefficients[te_start + 4 + 2] = 1;
        coefficients[se_start + (4 + 2) * 4 + 3] = -4; // NEW lane1 -k favors Append.
        let q = NativeEventQ4::new(c, &pack_coefficients(&coefficients)?)?;
        let mut token = vec![0i32; 2 * 2 * 128];
        token[3] = 7 << 22;
        token[128 + 5] = 7 << 22;
        token[2 * 128 + 5] = 7 << 22;
        let mut own = vec![0i32; 2 * 128 * 128];
        let mut neighbor = own.clone();
        let mut event = vec![0i32; 2 * 4];
        event[4 + 2] = 1 << 22;
        let mut state_event = vec![0i32; 2 * 128 * 4];
        for (state, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
            own[state * 128 + 5] = oracle(&[0, 1, 0, 0], root);
            neighbor[(128 + state) * 128 + 3] = root[1] / 2;
            state_event[(128 + state) * 4 + 2] = -root[3] / 2;
        }
        let manual =
            NativeEventTables::new(2, 2, &token, &own, Some(&neighbor), &event, &state_event)?;
        let geometry = geometry()?;
        let mut actual = NativeEventState::new(2)?;
        let mut expected = NativeEventState::new(2)?;
        let first = actual.step(0, q.native(), &geometry)?;
        assert_eq!(first, expected.step(0, &manual, &geometry)?);
        assert_eq!(
            first.states[..2]
                .iter()
                .map(|s| s.index())
                .collect::<Vec<_>>(),
            [3, 5]
        );
        let second = actual.step(1, q.native(), &geometry)?;
        assert_eq!(second, expected.step(1, &manual, &geometry)?);
        assert_eq!(
            second.actions[..2]
                .iter()
                .map(|s| s.index())
                .collect::<Vec<_>>(),
            [5, 3]
        );
        assert_eq!(
            second.states[..2]
                .iter()
                .map(|s| s.index())
                .collect::<Vec<_>>(),
            [7, 6]
        );
        assert_eq!(second.event, SpanAction::Append);
        assert_eq!(second.event_scores, [0, 0, 5 << 22, 0]);
        let before = actual;
        assert!(actual.step(2, q.native(), &geometry).is_err());
        assert_eq!(actual, before);
        Ok(())
    }
}
