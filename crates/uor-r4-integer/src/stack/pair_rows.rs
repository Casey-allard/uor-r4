//! The pair rows in a served stack's artifact (claude-384).
//!
//! The dictionary, the marks, the served read and the rows file are in `crate::fbs_pair_rows`,
//! which the float trainer's snapshot tool shares. This module is the artifact's side: the
//! record's check against a stack's shape, the tables' names, and the image read from a parsed
//! container.

use serde::{Deserialize, Serialize};

use super::format::Container;
use super::{StackError, StackShape};

pub use crate::fbs_pair_rows::{PairMarks, PairRowsImage, PAIR_ROW_EXP};

/// The pair rows record of a stack's header: what the rows are keyed by and the rows file's hash.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StackPairRows {
    /// The vocabulary the pair addresses are built over: the stack's.
    pub vocab: usize,
    /// The rows' width: the stack's.
    pub width: usize,
    /// The rows' exponent: [`PAIR_ROW_EXP`].
    pub row_exp: i32,
    /// The user marker.
    pub user: Vec<u32>,
    /// The assistant header.
    pub assistant: Vec<u32>,
    /// The number of pairs, one row each.
    pub pairs: usize,
    /// SHA-256 of the image's rows file ([`PairRowsImage::sha256`]).
    pub sha256: String,
}

impl StackPairRows {
    /// The record of `image`.
    pub fn of(image: &PairRowsImage) -> Self {
        Self {
            vocab: image.dict().vocab(),
            width: image.width(),
            row_exp: PAIR_ROW_EXP,
            user: image.marks().user.clone(),
            assistant: image.marks().assistant.clone(),
            pairs: image.dict().len(),
            sha256: image.sha256(),
        }
    }

    /// Refuse a record this reader does not serve on `shape`.
    pub(crate) fn check(&self, shape: &StackShape) -> Result<(), StackError> {
        if !shape.projection_fbs() {
            return Err(StackError::Shape(
                "pair rows are served on a SpiralCore/FBS stack",
            ));
        }
        if self.width != shape.width {
            return Err(StackError::Shape("pair rows are as wide as the stack"));
        }
        if self.vocab != shape.vocab {
            return Err(StackError::Shape(
                "pair rows are keyed over the stack's vocabulary",
            ));
        }
        if self.row_exp != PAIR_ROW_EXP {
            return Err(StackError::Shape("pair rows are at 2^-12"));
        }
        Ok(())
    }
}

/// The table of pair addresses (u32).
pub(crate) const PAIR_ROWS_ADDRESSES: &str = "pair_rows.addresses";
/// The table of rows (i16, pair-major).
pub(crate) const PAIR_ROWS_VALUES: &str = "pair_rows.values";

/// The pair rows an artifact holds, or none when its header has no record.
pub(crate) fn pair_rows_image(
    artifact: &Container<'_>,
) -> Result<Option<PairRowsImage>, StackError> {
    let Some(record) = &artifact.pair_rows else {
        return Ok(None);
    };
    let addresses = artifact.table_u32(PAIR_ROWS_ADDRESSES)?;
    if addresses.len() != record.pairs {
        return Err(StackError::TableSection(PAIR_ROWS_ADDRESSES.to_owned()));
    }
    let values = artifact.table_i16(PAIR_ROWS_VALUES)?;
    if Some(values.len()) != record.pairs.checked_mul(record.width) {
        return Err(StackError::TableSection(PAIR_ROWS_VALUES.to_owned()));
    }
    let marks = PairMarks {
        user: record.user.clone(),
        assistant: record.assistant.clone(),
    };
    let image = PairRowsImage::checked(record.vocab, record.width, marks, addresses, values)
        .map_err(StackError::Numerics)?;
    if image.sha256() != record.sha256 {
        return Err(StackError::Shape(
            "pair rows are the rows file the record hashes",
        ));
    }
    Ok(Some(image))
}

/// The pair rows of the `UORLUT01` artifact `bytes`, or none when it serves none.
pub fn read_pair_rows_uorlut01(bytes: &[u8]) -> Result<Option<PairRowsImage>, StackError> {
    pair_rows_image(&Container::parse(bytes)?)
}
