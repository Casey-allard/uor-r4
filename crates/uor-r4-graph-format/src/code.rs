//! CODE rolling-state program bytecode (RFC §5 CODE) and its stage-2 validator.
//!
//! The CODE section carries state-update programs for the semantic states.
//! Currently we define a simple accumulator-style instruction set.

use crate::error::FormatError;

/// HALT — terminate the program.
pub const OP_HALT: u8 = 0x00;
/// UPDATE_SLOT — Update a slot with specific values.
pub const OP_UPDATE_SLOT: u8 = 0x01;
/// CLEAR_SLOT — Clear a slot.
pub const OP_CLEAR_SLOT: u8 = 0x02;
/// SHIFT_SLOTS — Shift slots to make room.
pub const OP_SHIFT_SLOTS: u8 = 0x03;

/// Byte size of one op, or `None` for an opcode outside the set.
fn op_size(opcode: u8) -> Option<usize> {
    Some(match opcode {
        OP_HALT => 1,
        // UPDATE_SLOT { level: u8, region_id: u32, token: u32, score_q: i32, age: u16 }
        OP_UPDATE_SLOT => 16,
        // CLEAR_SLOT { level: u8 }
        OP_CLEAR_SLOT => 2,
        // SHIFT_SLOTS { level: u8 }
        OP_SHIFT_SLOTS => 2,
        _ => return None,
    })
}

/// Validate a CODE section payload against the HEAD-declared bounds.
pub(crate) fn validate(bytes: &[u8], max_steps: u32) -> Result<(), crate::NotAProduct> {
    let mut cursor: usize = 0;
    let mut op_count: u32 = 0;
    let mut halted = false;

    while cursor < bytes.len() {
        let opcode = bytes[cursor];
        let Some(size) = op_size(opcode) else {
            return Err((FormatError::UnknownCodeOp {
                offset: cursor as u32,
                opcode,
            })
            .into());
        };
        if cursor + size > bytes.len() {
            return Err((FormatError::TruncatedCodeOp {
                offset: cursor as u32,
                opcode,
            })
            .into());
        }

        // Cannot overflow: op_count <= bytes.len() <= u32::MAX.
        op_count += 1;

        let level = match opcode {
            OP_UPDATE_SLOT | OP_CLEAR_SLOT | OP_SHIFT_SLOTS => bytes[cursor + 1],
            _ => 0, // unused
        };

        // level 0 = local, 1 = segment, 2 = session
        if level > 2 {
            return Err((FormatError::CodeOperandOutOfBounds {
                op_index: op_count - 1,
            })
            .into());
        }

        cursor += size;
        if opcode == OP_HALT {
            halted = true;
            break;
        }
    }

    // In CODE, unlike ROUT, HALT is required: there is no implicit fall-through.
    if !halted {
        // Did not halt cleanly
        return Err((FormatError::CodeProgramUnterminated).into());
    }

    if op_count > max_steps {
        return Err((FormatError::CodeProgramTooDeep {
            ops: op_count,
            max: max_steps,
        })
        .into());
    }

    Ok(())
}
