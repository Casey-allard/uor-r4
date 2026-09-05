//! Bounded decimal representation for typed integer values.
//!
//! This is a lexical codec, not an expression, assignment or answer parser.
//! ASCII signed decimal fragments are recognized across token boundaries;
//! identifiers, decimal fractions, exponent forms and numeric suffixes are
//! excluded. A trailing sentence period is resolved by one-byte lookahead.
//! A minus immediately before digits is a sign under this lexical law; source
//! language unary/binary-minus interpretation belongs to a different operator.
//! Every complete fragment retains its inclusive token-sequence interval.

use crate::prime_route_attention::ZPhi;
use serde::{Deserialize, Serialize};

pub(super) const NUMERAL_CODEC: &str = "ascii-signed-i64-fragment-and-byte-emission/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Numeral {
    pub tokens: [u32; 20],
    pub len: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Literal {
    pub value: i64,
    /// Inclusive token sequence containing the sign or first digit.
    pub start: u64,
    /// Inclusive token sequence containing the final digit, excluding a period.
    pub end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum ScanState {
    #[default]
    Idle,
    Sign,
    Digits,
    TrailingPoint,
    Word,
    Rejected,
}

/// Fixed scanner state. Invalid or oversized fragments remain rejected until
/// a lexical delimiter, so an overflowing prefix cannot leak a valid suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct Scanner {
    state: ScanState,
    negative: bool,
    // Negative accumulation admits i64::MIN without an intermediate positive
    // magnitude outside the signed range.
    accumulated: i64,
    digits: u8,
    start: u64,
    end: u64,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
impl Numeral {
    /// Emit the integer subdomain of exact Z[phi] into the existing raw-byte
    /// vocabulary. No whitespace, punctuation or answer template is appended.
    /// Nineteen fixed decimal places each require at most nine subtractions.
    pub(super) fn from_zphi(value: ZPhi) -> Option<Self> {
        if value.b != 0 {
            return None;
        }
        let mut result = Self {
            tokens: [0; 20],
            len: 0,
        };
        if value.a < 0 {
            result.tokens[0] = u32::from(b'-') + 2;
            result.len = 1;
        }
        let mut remaining = value.a.unsigned_abs();
        let mut started = false;
        for place in [
            1_000_000_000_000_000_000_u64,
            100_000_000_000_000_000,
            10_000_000_000_000_000,
            1_000_000_000_000_000,
            100_000_000_000_000,
            10_000_000_000_000,
            1_000_000_000_000,
            100_000_000_000,
            10_000_000_000,
            1_000_000_000,
            100_000_000,
            10_000_000,
            1_000_000,
            100_000,
            10_000,
            1_000,
            100,
            10,
            1,
        ] {
            let mut digit = 0_u8;
            while remaining >= place {
                remaining -= place;
                digit += 1;
            }
            if digit != 0 || started || place == 1 {
                result.tokens[usize::from(result.len)] = u32::from(b'0' + digit) + 2;
                result.len += 1;
                started = true;
            }
        }
        Some(result)
    }

    pub(super) fn as_tokens(&self) -> &[u32] {
        &self.tokens[..usize::from(self.len)]
    }
}

fn word_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || !byte.is_ascii()
}

impl Scanner {
    /// Host checkpoint validation; old source bytes outside retained metadata
    /// are user-provided state, not authenticated observations.
    pub(super) fn snapshot_valid(&self, seen: u64) -> bool {
        if seen == 0 {
            return *self == Self::default();
        }
        let interval = self.start <= self.end && self.end < seen;
        match self.state {
            ScanState::Idle => *self == Self::default(),
            ScanState::Word => {
                !self.negative
                    && self.accumulated == 0
                    && self.digits == 0
                    && self.start == 0
                    && self.end == 0
            }
            ScanState::Sign => {
                interval && self.start == self.end && self.accumulated == 0 && self.digits == 0
            }
            ScanState::Digits | ScanState::TrailingPoint => {
                interval && (1..=19).contains(&self.digits) && self.accumulated <= 0
            }
            ScanState::Rejected => {
                interval
                    && self.digits <= 20
                    && self.accumulated <= 0
                    && (self.digits != 0 || self.accumulated == 0)
            }
        }
    }

    pub(super) fn snapshot_needs_suffix(&self) -> bool {
        matches!(
            self.state,
            ScanState::Sign | ScanState::Digits | ScanState::TrailingPoint
        )
    }

    fn start_byte(&mut self, byte: u8, sequence: u64) {
        *self = Self::default();
        if byte == b'+' || byte == b'-' {
            self.state = ScanState::Sign;
            self.negative = byte == b'-';
            self.start = sequence;
            self.end = sequence;
        } else if byte.is_ascii_digit() {
            self.state = ScanState::Digits;
            self.start = sequence;
            self.digit(byte, sequence);
        } else if word_byte(byte) {
            self.state = ScanState::Word;
        } else if byte == b'.' {
            self.state = ScanState::Rejected;
        }
    }

    fn digit(&mut self, byte: u8, sequence: u64) {
        // Four checked additions form ten times the negative prefix, then
        // one checked subtraction appends the digit. No variable multiply.
        let twice = self.accumulated.checked_add(self.accumulated);
        let four = twice.and_then(|value| value.checked_add(value));
        let eight = four.and_then(|value| value.checked_add(value));
        let ten = eight.zip(twice).and_then(|(a, b)| a.checked_add(b));
        let next = ten.and_then(|value| value.checked_sub(i64::from(byte - b'0')));
        self.digits = self.digits.saturating_add(1);
        self.end = sequence;
        if self.digits > 19 || next.is_none() {
            self.state = ScanState::Rejected;
            return;
        }
        if let Some(next) = next {
            self.accumulated = next;
        }
        self.state = ScanState::Digits;
    }

    fn literal(&self) -> Option<Literal> {
        if !matches!(self.state, ScanState::Digits | ScanState::TrailingPoint) {
            return None;
        }
        let value = if self.negative {
            self.accumulated
        } else {
            self.accumulated.checked_neg()?
        };
        Some(Literal {
            value,
            start: self.start,
            end: self.end,
        })
    }

    /// Consume exactly one decoded source byte. The returned interval names
    /// its actual numeric occurrence, never a predicted answer occurrence.
    pub(super) fn feed(&mut self, byte: u8, sequence: u64) -> Option<Literal> {
        match self.state {
            ScanState::Idle => self.start_byte(byte, sequence),
            ScanState::Sign => {
                if byte.is_ascii_digit() {
                    self.digit(byte, sequence);
                } else if word_byte(byte) || matches!(byte, b'.' | b'+' | b'-') {
                    self.state = ScanState::Rejected;
                } else {
                    self.start_byte(byte, sequence);
                }
            }
            ScanState::Digits => {
                if byte.is_ascii_digit() {
                    self.digit(byte, sequence);
                } else if word_byte(byte) {
                    self.state = ScanState::Rejected;
                } else if byte == b'.' {
                    self.state = ScanState::TrailingPoint;
                } else {
                    let literal = self.literal();
                    self.start_byte(byte, sequence);
                    return literal;
                }
            }
            ScanState::TrailingPoint => {
                if byte.is_ascii_digit() || word_byte(byte) || byte == b'.' {
                    self.state = ScanState::Rejected;
                } else {
                    let literal = self.literal();
                    self.start_byte(byte, sequence);
                    return literal;
                }
            }
            ScanState::Word => {
                if byte.is_ascii_digit() || word_byte(byte) {
                    return None;
                }
                self.start_byte(byte, sequence);
            }
            ScanState::Rejected => {
                if byte.is_ascii_digit() || word_byte(byte) || matches!(byte, b'.' | b'+' | b'-') {
                    return None;
                }
                self.start_byte(byte, sequence);
            }
        }
        None
    }

    /// An explicit end-of-input boundary closes a final integer fragment.
    /// The caller supplies the boundary; this never detects task grammar.
    pub(super) fn finish(&mut self) -> Option<Literal> {
        let literal = self.literal();
        *self = Self::default();
        literal
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(chunks: &[&[u8]]) -> Vec<Literal> {
        let mut scanner = Scanner::default();
        let mut literals = Vec::new();
        for (sequence, chunk) in chunks.iter().enumerate() {
            for &byte in *chunk {
                if let Some(literal) = scanner.feed(byte, sequence as u64) {
                    literals.push(literal);
                }
            }
        }
        if let Some(literal) = scanner.finish() {
            literals.push(literal);
        }
        literals
    }

    #[test]
    fn native_numeral_exact_signed_extremes_and_zphi_boundary() {
        for value in [0, 1, -1, 10, -17, 1000, i64::MAX, i64::MIN] {
            let numeral = Numeral::from_zphi(ZPhi::new(value, 0)).unwrap();
            let bytes = numeral
                .as_tokens()
                .iter()
                .map(|&token| (token - 2) as u8)
                .collect::<Vec<_>>();
            assert_eq!(bytes, value.to_string().as_bytes());
            assert!(numeral.len <= 20);
            assert_eq!(scan(&[&bytes])[0].value, value);
        }
        assert!(Numeral::from_zphi(ZPhi::new(3, 1)).is_none());
        let sum = ZPhi::new(13, 0).checked_add(ZPhi::new(4, 0)).unwrap();
        assert_eq!(Numeral::from_zphi(sum).unwrap().as_tokens(), &[51, 57]);
        assert!(ZPhi::new(i64::MAX, 0).checked_add(ZPhi::new(1, 0)).is_err());
        assert!(ZPhi::new(i64::MIN, 0)
            .checked_add(ZPhi::new(-1, 0))
            .is_err());
    }

    #[test]
    fn native_numeral_scanner_preserves_cross_token_provenance_and_repeated_values() {
        assert_eq!(
            scan(&[b"x = -", b"1", b"7; y = 17", b".", b" z = +", b"0004"]),
            vec![
                Literal {
                    value: -17,
                    start: 0,
                    end: 2
                },
                Literal {
                    value: 17,
                    start: 2,
                    end: 2
                },
                Literal {
                    value: 4,
                    start: 4,
                    end: 5
                },
            ]
        );
        let split = scan(&[b"13", b".", b" ", b"13", b" "]);
        assert_eq!(
            split[0],
            Literal {
                value: 13,
                start: 0,
                end: 0
            }
        );
        assert_eq!(
            split[1],
            Literal {
                value: 13,
                start: 3,
                end: 3
            }
        );
        assert_eq!(scan(&[b"13."])[0].value, 13);
    }

    #[test]
    fn native_numeral_scanner_rejects_overflow_and_noninteger_fragments() {
        let source = b"i64 x13 13x 13_i64 1.25 2e3 4e-2 .5 --6 9223372036854775808 -9223372036854775809 00000000000000000000 7";
        let bytes = source.iter().map(std::slice::from_ref).collect::<Vec<_>>();
        let values = scan(&bytes)
            .into_iter()
            .map(|literal| literal.value)
            .collect::<Vec<_>>();
        assert_eq!(values, [7]);
        assert!(scan(&[b"-", b" "]).is_empty());
        assert!(scan(&[b"13", b".", b"25"]).is_empty());
    }

    #[test]
    fn native_numeral_scanner_serialization_keeps_unfinished_and_rejected_runs() {
        for prefix in [
            b"-922337203685477580".as_slice(),
            b"13.",
            b"92233720368547758089",
        ] {
            let mut scanner = Scanner::default();
            for &byte in prefix {
                assert!(scanner.feed(byte, 8).is_none());
            }
            let mut restored: Scanner =
                serde_json::from_slice(&serde_json::to_vec(&scanner).unwrap()).unwrap();
            for (sequence, byte) in b"8; +17".iter().copied().enumerate() {
                assert_eq!(
                    scanner.feed(byte, sequence as u64 + 9),
                    restored.feed(byte, sequence as u64 + 9)
                );
            }
            assert_eq!(scanner.finish(), restored.finish());
            assert_eq!(scanner.finish(), None);
        }
    }
}
