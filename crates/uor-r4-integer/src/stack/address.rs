//! The address table in a served stack's artifact (claude-366).
//!
//! The cell rule, the served read and the rows file are in `crate::fbs_address`, which the float
//! trainer's snapshot tool shares. This module is the artifact's side: the record's check against
//! a stack's shape, the tables' names, and the image read from a parsed container.

use super::format::Container;
use super::{StackError, StackShape};
use crate::fbs_address::MAX_WIDTH;

pub use crate::fbs_address::{
    e8_cell, token_cell, AddressArm, AddressImage, StackAddress, ADDRESS_BLOCK, ADDRESS_CELLS, ADDRESS_ROW_EXP,
    ADDRESS_ROW_STEPS, ADDRESS_SHARED_BLOCKS,
};

impl StackAddress {
    /// Refuse a record this reader does not serve on `shape`.
    pub(crate) fn check(&self, shape: &StackShape) -> Result<(), StackError> {
        if !shape.projection_fbs() {
            return Err(StackError::Shape("an address table is served on a SpiralCore/FBS stack"));
        }
        if self.block != ADDRESS_BLOCK || self.cells != ADDRESS_CELLS {
            return Err(StackError::Shape("an address table has 240 cells a block of 8"));
        }
        if self.row_exp != ADDRESS_ROW_EXP {
            return Err(StackError::Shape("an address table's rows are at the residual exponent"));
        }
        if self.layers != shape.layers() {
            return Err(StackError::Shape("an address table holds rows for every layer"));
        }
        if shape.width == 0 || shape.width % ADDRESS_BLOCK != 0 || shape.width > MAX_WIDTH {
            return Err(StackError::Shape("an address table needs a width in whole blocks of 8"));
        }
        Ok(())
    }
}

/// The table's name in the artifact for `layer`.
pub(crate) fn address_table_name(layer: usize) -> String {
    format!("address.l{layer}")
}

/// The address table an artifact holds, or none when its header has no record.
pub(crate) fn address_image(artifact: &Container<'_>) -> Result<Option<AddressImage>, StackError> {
    let Some(record) = &artifact.address else {
        return Ok(None);
    };
    let width = artifact.shape.width;
    let values = width / ADDRESS_BLOCK * ADDRESS_CELLS * width;
    let mut rows = Vec::with_capacity(record.layers);
    for layer in 0..record.layers {
        let name = address_table_name(layer);
        let table = artifact.table_i16(&name)?;
        if table.len() != values {
            return Err(StackError::TableSection(name));
        }
        rows.push(table);
    }
    AddressImage::checked(record.arm, record.seed(), width, rows)
        .map(Some)
        .map_err(StackError::Numerics)
}

/// The address table of the `UORLUT01` artifact `bytes`, or none when it serves none.
pub fn read_address_uorlut01(bytes: &[u8]) -> Result<Option<AddressImage>, StackError> {
    address_image(&Container::parse(bytes)?)
}
