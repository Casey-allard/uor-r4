//! Borrowed view of the optional FMM translation-table section.
//!
//! The compiler folds the fixed-point low-rank factors into one signed
//! coefficient for every signature bit and candidate token. A deployed
//! reader therefore selects a coefficient by the input bit, adds or
//! subtracts it, and never evaluates a projection/token-factor product.

use crate::error::FormatError;
use crate::header::{read_i32_le, read_u16_le, read_u32_le};
use crate::types::{ScoreQ, TokenId};

/// Four-byte section magic for the first packed FMM translation-table form.
pub const FMM_MAGIC: [u8; 4] = *b"FMM1";
/// Wire version emitted by the first table form.
pub const FMM_VERSION: u16 = 1;
/// Fixed header length in bytes.
pub const FMM_HEADER_LEN: usize = 20;
/// One little-endian u32 token identifier.
pub const FMM_TOKEN_LEN: usize = 4;
/// One little-endian i32 root or translation score.
pub const FMM_SCORE_LEN: usize = 4;

/// A zero-copy, validated FMM translation table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FmmTranslationTable<'a> {
    dimension: u16,
    rank: u16,
    token_count: u32,
    factor_fraction_bits: u8,
    tokens: &'a [u8],
    root_scores: &'a [u8],
    coefficients: &'a [u8],
    coefficient_row_len: usize,
}

impl<'a> FmmTranslationTable<'a> {
    /// Parse the fixed header and establish all table slices using checked
    /// arithmetic. No allocation or floating-point conversion occurs here.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < FMM_HEADER_LEN {
            return Err((FormatError::FmmSectionTooShort {
                actual: bytes.len() as u64,
            })
            .into());
        }
        if bytes[0..4] != FMM_MAGIC {
            return Err((FormatError::FmmSectionBadMagic).into());
        }
        let version = read_u16_le(bytes, 4);
        if version != FMM_VERSION {
            return Err((FormatError::FmmSectionUnsupportedVersion(version)).into());
        }
        let dimension = read_u16_le(bytes, 6);
        let rank = read_u16_le(bytes, 8);
        let token_count = read_u32_le(bytes, 12);
        let factor_fraction_bits = bytes[16];
        if dimension == 0 || rank == 0 || token_count == 0 || factor_fraction_bits > 31 {
            return Err((FormatError::FmmSectionInvalidDimensions {
                dimension,
                rank,
                token_count,
                factor_fraction_bits,
            })
            .into());
        }

        let token_bytes = usize::try_from(token_count)
            .ok()
            .and_then(|count| count.checked_mul(FMM_TOKEN_LEN))
            .ok_or(FormatError::FmmSectionLengthOverflow)?;
        let coefficient_count = usize::from(dimension)
            .checked_mul(
                usize::try_from(token_count).map_err(|_| FormatError::FmmSectionLengthOverflow)?,
            )
            .ok_or(FormatError::FmmSectionLengthOverflow)?;
        let score_bytes = token_bytes;
        let coefficient_bytes = coefficient_count
            .checked_mul(FMM_SCORE_LEN)
            .ok_or(FormatError::FmmSectionLengthOverflow)?;
        let expected = FMM_HEADER_LEN
            .checked_add(token_bytes)
            .and_then(|value| value.checked_add(score_bytes))
            .and_then(|value| value.checked_add(coefficient_bytes))
            .ok_or(FormatError::FmmSectionLengthOverflow)?;
        if bytes.len() != expected {
            return Err((FormatError::FmmSectionLengthMismatch {
                expected: expected as u64,
                actual: bytes.len() as u64,
            })
            .into());
        }

        let tokens_start = FMM_HEADER_LEN;
        let root_start = tokens_start + token_bytes;
        let coefficients_start = root_start + score_bytes;
        Ok(Self {
            dimension,
            rank,
            token_count,
            factor_fraction_bits,
            tokens: &bytes[tokens_start..root_start],
            root_scores: &bytes[root_start..coefficients_start],
            coefficients: &bytes[coefficients_start..],
            coefficient_row_len: coefficient_bytes / usize::from(dimension),
        })
    }

    pub fn dimension(&self) -> u16 {
        self.dimension
    }

    pub fn rank(&self) -> u16 {
        self.rank
    }

    pub fn token_count(&self) -> u32 {
        self.token_count
    }

    pub fn factor_fraction_bits(&self) -> u8 {
        self.factor_fraction_bits
    }

    pub fn token(&self, index: u32) -> Option<TokenId> {
        self.tokens().nth(index as usize)
    }

    pub fn root_score(&self, index: u32) -> Option<ScoreQ> {
        self.root_scores().nth(index as usize)
    }

    /// Read the compiler-folded contribution for one signature bit and
    /// candidate token. The runtime selects its sign from the query bit and
    /// adds this value; it never multiplies basis and factor values.
    pub fn tokens(&self) -> FmmTokenIter<'a> {
        FmmTokenIter {
            chunks: self.tokens.chunks_exact(FMM_TOKEN_LEN),
        }
    }

    pub fn root_scores(&self) -> FmmScoreIter<'a> {
        FmmScoreIter {
            chunks: self.root_scores.chunks_exact(FMM_SCORE_LEN),
        }
    }

    /// Iterate coefficient rows in signature-coordinate order.
    pub fn coefficient_rows(&self) -> FmmRows<'a> {
        FmmRows {
            rows: self.coefficients.chunks_exact(self.coefficient_row_len),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FmmTokenIter<'a> {
    chunks: core::slice::ChunksExact<'a, u8>,
}

impl Iterator for FmmTokenIter<'_> {
    type Item = TokenId;

    fn next(&mut self) -> Option<Self::Item> {
        Some(TokenId(read_u32_le(self.chunks.next()?, 0)))
    }
}

impl ExactSizeIterator for FmmTokenIter<'_> {
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

#[derive(Debug, Clone)]
pub struct FmmScoreIter<'a> {
    chunks: core::slice::ChunksExact<'a, u8>,
}

impl Iterator for FmmScoreIter<'_> {
    type Item = ScoreQ;

    fn next(&mut self) -> Option<Self::Item> {
        Some(ScoreQ::from_raw(read_i32_le(self.chunks.next()?, 0)))
    }
}

impl ExactSizeIterator for FmmScoreIter<'_> {
    fn len(&self) -> usize {
        self.chunks.len()
    }
}

#[derive(Debug, Clone)]
pub struct FmmRows<'a> {
    rows: core::slice::ChunksExact<'a, u8>,
}

impl<'a> Iterator for FmmRows<'a> {
    type Item = FmmCoefficientRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(FmmCoefficientRow {
            bytes: self.rows.next()?,
        })
    }
}

impl ExactSizeIterator for FmmRows<'_> {
    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FmmCoefficientRow<'a> {
    bytes: &'a [u8],
}

impl<'a> FmmCoefficientRow<'a> {
    pub fn values(self) -> FmmScoreIter<'a> {
        FmmScoreIter {
            chunks: self.bytes.chunks_exact(FMM_SCORE_LEN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let dimension = 2u16;
        let rank = 1u16;
        let token_count = 2u32;
        let mut bytes = Vec::from(FMM_MAGIC);
        bytes.extend_from_slice(&FMM_VERSION.to_le_bytes());
        bytes.extend_from_slice(&dimension.to_le_bytes());
        bytes.extend_from_slice(&rank.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&token_count.to_le_bytes());
        bytes.push(29);
        bytes.extend_from_slice(&[0, 0, 0]);
        bytes.extend_from_slice(&11u32.to_le_bytes());
        bytes.extend_from_slice(&22u32.to_le_bytes());
        bytes.extend_from_slice(&7i32.to_le_bytes());
        bytes.extend_from_slice(&8i32.to_le_bytes());
        bytes.extend_from_slice(&101i32.to_le_bytes());
        bytes.extend_from_slice(&202i32.to_le_bytes());
        bytes.extend_from_slice(&(-303i32).to_le_bytes());
        bytes.extend_from_slice(&404i32.to_le_bytes());
        bytes
    }

    #[test]
    fn parses_borrowed_table_and_reads_rows() {
        let bytes = sample();
        let table = FmmTranslationTable::parse(&bytes).expect("valid table");
        assert_eq!(table.dimension(), 2);
        assert_eq!(table.rank(), 1);
        assert_eq!(table.factor_fraction_bits(), 29);
        assert_eq!(table.token(0), Some(TokenId(11)));
        assert_eq!(table.token(1), Some(TokenId(22)));
        assert_eq!(table.root_score(1), Some(ScoreQ::from_raw(8)));
        let mut rows = table.coefficient_rows();
        assert_eq!(
            rows.next().unwrap().values().collect::<Vec<_>>(),
            vec![ScoreQ::from_raw(101), ScoreQ::from_raw(202),]
        );
        assert_eq!(
            rows.next().unwrap().values().collect::<Vec<_>>(),
            vec![ScoreQ::from_raw(-303), ScoreQ::from_raw(404),]
        );
        assert!(rows.next().is_none());
        assert_eq!(table.token(2), None);
    }

    #[test]
    fn rejects_bad_magic_and_length() {
        let mut bytes = sample();
        bytes[0] = b'X';
        assert_eq!(
            FmmTranslationTable::parse(&bytes),
            Err((FormatError::FmmSectionBadMagic).into())
        );

        let mut bytes = sample();
        bytes.pop();
        assert_eq!(
            FmmTranslationTable::parse(&bytes),
            Err((FormatError::FmmSectionLengthMismatch {
                expected: 52,
                actual: 51,
            })
            .into())
        );
    }
}
