//! Exact finite token actions and an ordered geometric span register.
//!
//! An offline compiler supplies one historical signed H4 action per token and
//! lane. This module never classifies a floating embedding, predicts controller
//! actions, or constructs a group table. The caller supplies predicted actions
//! and an admitted [`HistoricalH4Tables`]; numerical steps use its exact
//! right-composition lookup. Antipodes and a present identity product remain
//! distinct from absence.
//!
//! Read [`SpanRegister::held`] BEFORE [`SpanRegister::step`]. A COMMIT at t is
//! visible at t+1, including the address attached to a following value source.
//! OPEN and COMMIT do not consume their token action. Width validation precedes
//! every state change, including otherwise ignored actions.
//!
//! Construction allocates; dictionary row lookup, register reads, reset and
//! steps allocate nothing. Their source uses no floating point, multiplication
//! or division. A compiled instruction audit and whole-model serving are
//! separate qualifications. The contextual controller, trunk and reader are
//! outside this component.

use std::fmt;

use crate::h4_tables::{H4Code, HistoricalH4Tables};

/// Matches the offline producer's width range of 4..=8192 coordinates.
pub const MAX_SPAN_LANES: usize = 2048;

/// Exact controller label order shared with the learned span producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SpanAction {
    Hold = 0,
    Open = 1,
    Append = 2,
    Commit = 3,
}

impl TryFrom<u8> for SpanAction {
    type Error = SpanError;

    fn try_from(value: u8) -> SpanResult<Self> {
        match value {
            0 => Ok(Self::Hold),
            1 => Ok(Self::Open),
            2 => Ok(Self::Append),
            3 => Ok(Self::Commit),
            _ => Err(SpanError::InvalidAction(value)),
        }
    }
}

/// Fixed-size errors do not allocate during row lookup or numerical steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanError {
    InvalidLanes(usize),
    InvalidAction(u8),
    WidthMismatch { expected: usize, actual: usize },
    InvalidDictionaryLength { lanes: usize, codes: usize },
    InvalidCode { offset: usize, code: u8 },
    TokenOutOfRange { token: usize, vocabulary: usize },
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLanes(lanes) => write!(f, "geometric span lanes {lanes} are outside 1..={MAX_SPAN_LANES}"),
            Self::InvalidAction(action) => write!(f, "geometric span action {action} is outside 0..4"),
            Self::WidthMismatch { expected, actual } => write!(f, "geometric span needs {expected} token lanes, got {actual}"),
            Self::InvalidDictionaryLength { lanes, codes } => write!(f, "geometric token actions need nonempty complete rows of {lanes} lanes, got {codes} codes"),
            Self::InvalidCode { offset, code } => write!(f, "geometric token action at {offset} has invalid H4 code {code}"),
            Self::TokenOutOfRange { token, vocabulary } => write!(f, "geometric token action {token} is outside vocabulary {vocabulary}"),
        }
    }
}

impl std::error::Error for SpanError {}

pub type SpanResult<T> = Result<T, SpanError>;

fn validate_lanes(lanes: usize) -> SpanResult<()> {
    if !(1..=MAX_SPAN_LANES).contains(&lanes) {
        return Err(SpanError::InvalidLanes(lanes));
    }
    Ok(())
}

/// Immutable, validated token-to-action rows. Model/tokenizer/geometry hashes
/// and complete artifact admission remain the enclosing loader's obligation.
/// No unchecked root IDs or mutable rows are exposed.
#[derive(Debug)]
pub struct TokenActionDictionary {
    lanes: usize,
    codes: Box<[H4Code]>,
    offsets: Box<[usize]>,
}

impl TokenActionDictionary {
    pub fn new(lanes: usize, flattened: &[u8]) -> SpanResult<Self> {
        validate_lanes(lanes)?;
        if flattened.is_empty() || flattened.len() % lanes != 0 {
            return Err(SpanError::InvalidDictionaryLength {
                lanes,
                codes: flattened.len(),
            });
        }
        let codes = flattened
            .iter()
            .enumerate()
            .map(|(offset, &code)| {
                H4Code::try_from(code).map_err(|_| SpanError::InvalidCode { offset, code })
            })
            .collect::<SpanResult<Vec<_>>>()?;
        // All offsets are bounded by flattened.len(); construction alone does
        // row division. The numerical lookup below needs no token*lanes.
        let mut offsets = Vec::with_capacity(flattened.len() / lanes);
        let mut offset = 0;
        for _ in flattened.chunks_exact(lanes) {
            offsets.push(offset);
            offset += lanes;
        }
        Ok(Self {
            lanes,
            codes: codes.into_boxed_slice(),
            offsets: offsets.into_boxed_slice(),
        })
    }

    pub fn lanes(&self) -> usize {
        self.lanes
    }

    pub fn vocab_size(&self) -> usize {
        self.offsets.len()
    }

    pub fn row(&self, token_id: usize) -> SpanResult<&[H4Code]> {
        let &start = self
            .offsets
            .get(token_id)
            .ok_or(SpanError::TokenOutOfRange {
                token: token_id,
                vocabulary: self.offsets.len(),
            })?;
        // Private immutable fields establish start+lanes <= codes.len().
        Ok(&self.codes[start..start + self.lanes])
    }
}

/// Working and committed code tuples, with validity separate from identity.
/// This register consumes predicted actions; there is no teacher-event input
/// or token-content parser in the numerical implementation.
#[derive(Debug, PartialEq, Eq)]
pub struct SpanRegister {
    working: Box<[H4Code]>,
    held: Box<[H4Code]>,
    active: bool,
    nonempty: bool,
    held_valid: bool,
}

impl SpanRegister {
    pub fn new(lanes: usize) -> SpanResult<Self> {
        validate_lanes(lanes)?;
        Ok(Self {
            working: vec![H4Code::IDENTITY; lanes].into_boxed_slice(),
            held: vec![H4Code::IDENTITY; lanes].into_boxed_slice(),
            active: false,
            nonempty: false,
            held_valid: false,
        })
    }

    pub fn lanes(&self) -> usize {
        self.working.len()
    }

    /// Content for the current row, before consuming its predicted action.
    /// A committed identity tuple is Some; an uncommitted register is None.
    pub fn held(&self) -> Option<&[H4Code]> {
        self.held_valid.then_some(self.held.as_ref())
    }

    /// Advance after observing old held content. All returned errors precede
    /// mutation. Codes and the fixed canonical table were validated on entry
    /// to their respective types, so composition has no fallible partial step.
    pub fn step(
        &mut self,
        action: SpanAction,
        token: &[H4Code],
        tables: &HistoricalH4Tables,
    ) -> SpanResult<()> {
        if token.len() != self.working.len() {
            return Err(SpanError::WidthMismatch {
                expected: self.working.len(),
                actual: token.len(),
            });
        }
        match action {
            SpanAction::Hold => {}
            SpanAction::Open => {
                self.working.fill(tables.identity());
                self.active = true;
                self.nonempty = false;
            }
            SpanAction::Append if self.active => {
                for (working, &action) in self.working.iter_mut().zip(token) {
                    *working = tables.compose(*working, action);
                }
                self.nonempty = true;
            }
            SpanAction::Append => {}
            SpanAction::Commit => {
                if self.active && self.nonempty {
                    self.held.copy_from_slice(&self.working);
                    self.held_valid = true;
                }
                self.active = false;
                self.nonempty = false;
            }
        }
        Ok(())
    }

    /// Clear both registers and validity without reallocating their storage.
    pub fn reset(&mut self) {
        self.working.fill(H4Code::IDENTITY);
        self.held.fill(H4Code::IDENTITY);
        self.active = false;
        self.nonempty = false;
        self.held_valid = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn tables() -> Result<HistoricalH4Tables, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))?;
        Ok(HistoricalH4Tables::from_bytes(&bytes)?)
    }

    #[test]
    fn native_geometric_span_fsm_observes_old_held_and_empty_commits() -> TestResult {
        let tables = tables()?;
        let mut state = SpanRegister::new(2)?;
        let token = [H4Code::try_from(3)?, H4Code::try_from(5)?];
        let delimiter = [H4Code::try_from(0)?; 2];
        assert_eq!(state.held(), None);
        state.step(SpanAction::Append, &token, &tables)?; // closed no-op
        state.step(SpanAction::Commit, &token, &tables)?;
        assert_eq!(state.held(), None);
        state.step(SpanAction::Open, &delimiter, &tables)?;
        state.step(SpanAction::Commit, &delimiter, &tables)?; // empty no-op
        assert_eq!(state.held(), None);
        state.step(SpanAction::Open, &delimiter, &tables)?;
        state.step(SpanAction::Append, &token, &tables)?;
        assert_eq!(state.held(), None); // old content at the COMMIT row
        state.step(SpanAction::Commit, &delimiter, &tables)?;
        assert_eq!(state.held(), Some(token.as_slice())); // next row
        state.step(SpanAction::Open, &delimiter, &tables)?;
        state.step(SpanAction::Hold, &delimiter, &tables)?;
        state.step(SpanAction::Commit, &delimiter, &tables)?;
        state.step(SpanAction::Append, &delimiter, &tables)?;
        assert_eq!(state.held(), Some(token.as_slice()));
        Ok(())
    }

    #[test]
    fn native_geometric_span_right_order_and_present_cancelled_identity() -> TestResult {
        let tables = tables()?;
        let mut state = SpanRegister::new(2)?;
        let first = [H4Code::try_from(3)?, H4Code::try_from(5)?]; // i,j
        let second = [first[1], first[0]]; // j,i
        state.step(SpanAction::Open, &second, &tables)?;
        state.step(SpanAction::Append, &first, &tables)?;
        state.step(SpanAction::Append, &second, &tables)?;
        state.step(SpanAction::Commit, &first, &tables)?;
        let expected = [H4Code::try_from(7)?, H4Code::try_from(6)?]; // k,-k
        assert_eq!(state.held(), Some(expected.as_slice()));
        state.step(SpanAction::Open, &second, &tables)?;
        state.step(SpanAction::Append, &first, &tables)?;
        let inverse = [tables.inverse(first[0]), tables.inverse(first[1])];
        state.step(SpanAction::Append, &inverse, &tables)?;
        state.step(SpanAction::Commit, &first, &tables)?;
        assert_eq!(state.held(), Some([H4Code::IDENTITY; 2].as_slice()));
        Ok(())
    }

    #[test]
    fn native_geometric_span_width_errors_are_atomic_and_reset_reuses_storage() -> TestResult {
        let tables = tables()?;
        assert_eq!(SpanRegister::new(0), Err(SpanError::InvalidLanes(0)));
        assert_eq!(
            SpanRegister::new(MAX_SPAN_LANES + 1),
            Err(SpanError::InvalidLanes(MAX_SPAN_LANES + 1))
        );
        let mut state = SpanRegister::new(MAX_SPAN_LANES)?;
        let token = vec![H4Code::try_from(3)?; MAX_SPAN_LANES];
        state.step(SpanAction::Open, &token, &tables)?;
        state.step(SpanAction::Append, &token, &tables)?;
        state.step(SpanAction::Commit, &token, &tables)?;
        state.step(SpanAction::Open, &token, &tables)?;
        state.step(SpanAction::Append, &token, &tables)?;
        let before = (
            state.working.clone(),
            state.held.clone(),
            state.active,
            state.nonempty,
            state.held_valid,
        );
        let working_ptr = state.working.as_ptr();
        let held_ptr = state.held.as_ptr();
        for action in [
            SpanAction::Hold,
            SpanAction::Open,
            SpanAction::Append,
            SpanAction::Commit,
        ] {
            assert_eq!(
                state.step(action, &token[..MAX_SPAN_LANES - 1], &tables),
                Err(SpanError::WidthMismatch {
                    expected: MAX_SPAN_LANES,
                    actual: MAX_SPAN_LANES - 1
                })
            );
            assert_eq!(
                (
                    &state.working,
                    &state.held,
                    state.active,
                    state.nonempty,
                    state.held_valid
                ),
                (&before.0, &before.1, before.2, before.3, before.4)
            );
        }
        state.reset();
        assert_eq!(state.held(), None);
        assert!(!state.active && !state.nonempty);
        assert!(state
            .working
            .iter()
            .chain(state.held.iter())
            .all(|&code| code == H4Code::IDENTITY));
        assert_eq!(state.working.as_ptr(), working_ptr);
        assert_eq!(state.held.as_ptr(), held_ptr);
        Ok(())
    }

    #[test]
    fn native_geometric_span_dictionary_validates_codes_rows_and_actions() -> TestResult {
        assert!(matches!(
            TokenActionDictionary::new(0, &[1]),
            Err(SpanError::InvalidLanes(0))
        ));
        for bytes in [&[][..], &[1, 2, 3][..]] {
            assert!(matches!(
                TokenActionDictionary::new(2, bytes),
                Err(SpanError::InvalidDictionaryLength { .. })
            ));
        }
        assert!(matches!(
            TokenActionDictionary::new(2, &[1, 120]),
            Err(SpanError::InvalidCode {
                offset: 1,
                code: 120
            })
        ));
        let dictionary = TokenActionDictionary::new(3, &[1, 3, 5, 0, 6, 119])?;
        assert_eq!(dictionary.lanes(), 3);
        assert_eq!(dictionary.vocab_size(), 2);
        assert_eq!(
            dictionary
                .row(0)?
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>(),
            [1, 3, 5]
        );
        assert_eq!(
            dictionary
                .row(1)?
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>(),
            [0, 6, 119]
        );
        assert_eq!(
            dictionary.row(2),
            Err(SpanError::TokenOutOfRange {
                token: 2,
                vocabulary: 2
            })
        );
        assert_eq!(
            dictionary.row(usize::MAX),
            Err(SpanError::TokenOutOfRange {
                token: usize::MAX,
                vocabulary: 2
            })
        );
        for (code, expected) in [
            SpanAction::Hold,
            SpanAction::Open,
            SpanAction::Append,
            SpanAction::Commit,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(SpanAction::try_from(code as u8)?, expected);
        }
        for code in [4, 120, 255] {
            assert_eq!(
                SpanAction::try_from(code),
                Err(SpanError::InvalidAction(code))
            );
        }
        Ok(())
    }
}
