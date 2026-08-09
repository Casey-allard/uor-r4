//! ROUT decision-program bytecode (RFC §5 ROUT) — the v0 draft-line
//! opcode set — and its stage-2 validator (RFC §6 item 6).
//!
//! v0 section layout: a single decision program followed by an optional
//! trailing shortlist table. The program is a sequence of fixed-width
//! ops parsed from offset 0 and ends at the first [`OP_HALT`]; any
//! bytes after that op form the shortlist table. If the section ends
//! without HALT, the program is terminated by the section end and its
//! last op must be an [`OP_LEAF`] (and there is no table). Anything
//! else is [`FormatError::RoutingProgramUnterminated`] — this is the v0
//! reading of "at least one LEAF or HALT reachable": the last op is a
//! LEAF or HALT by construction.
//!
//! Ops (1-byte opcode + fixed little-endian operands):
//!
//! ```text
//! opcode  size  operands
//! 0x00    1     HALT
//! 0x01    12    TEST_POPCOUNT_LE { word u8, mask u64, threshold u16 }
//! 0x02    3     JMP_FWD { delta_ops u16 }
//! 0x03    7     LEAF { shortlist_start u32, shortlist_len u16 }
//! ```
//!
//! Jump encoding: the target op index is `current + 1 + delta_ops`
//! (`delta_ops` ops skipped), so jumps are forward-only by construction
//! and the program is acyclic as RFC §6 item 6 requires; the target
//! must index an existing op. Depth honesty: with forward-only jumps
//! every execution path is at most the static op count, so the v0
//! validator bounds `op_count ≤ HEAD.D`.
//!
//! LEAF shortlist ranges are byte ranges over the trailing table; when
//! no table is present (the program ran to section end, or HALT is the
//! last byte), `shortlist_len` must be 0.

use crate::error::FormatError;
use crate::head::Head;
use crate::header::{read_u16_le, read_u32_le};

/// HALT — terminate the program; any trailing bytes are the shortlist
/// table.
pub const OP_HALT: u8 = 0x00;
/// TEST_POPCOUNT_LE — `popcount(signature[word] & mask) <= threshold`.
pub const OP_TEST_POPCOUNT_LE: u8 = 0x01;
/// JMP_FWD — jump forward `1 + delta_ops` op indices.
pub const OP_JMP_FWD: u8 = 0x02;
/// LEAF — terminal op carrying a shortlist byte range.
pub const OP_LEAF: u8 = 0x03;

/// Popcount ceiling of a u64 signature word.
const MAX_POPCOUNT: u16 = 64;

/// Byte size of one op, or `None` for an opcode outside the v0 set.
fn op_size(opcode: u8) -> Option<usize> {
    Some(match opcode {
        OP_HALT => 1,
        OP_TEST_POPCOUNT_LE => 12,
        OP_JMP_FWD => 3,
        OP_LEAF => 7,
        _ => return None,
    })
}

/// Validate a ROUT section payload against the HEAD-declared bounds.
/// Two zero-allocation passes over the bytes: structure/termination,
/// then operands and jump targets. Section lengths are u32 (RFC §9.1),
/// so all op offsets and counts fit u32.
pub(crate) fn validate(bytes: &[u8], head: &Head) -> Result<(), crate::NotAProduct> {
    // Pass 1: op sizes, count, terminator, and the program/table split.
    let mut cursor: usize = 0;
    let mut op_count: u32 = 0;
    let mut last_opcode: Option<u8> = None;
    let mut halted = false;
    while cursor < bytes.len() {
        let opcode = bytes[cursor];
        let Some(size) = op_size(opcode) else {
            return Err((FormatError::UnknownRoutingOp {
                offset: cursor as u32,
                opcode,
            })
            .into());
        };
        if cursor + size > bytes.len() {
            return Err((FormatError::TruncatedRoutingOp {
                offset: cursor as u32,
                opcode,
            })
            .into());
        }
        // Cannot overflow: op_count <= bytes.len() <= u32::MAX.
        op_count += 1;
        last_opcode = Some(opcode);
        cursor += size;
        if opcode == OP_HALT {
            halted = true;
            break;
        }
    }
    if !halted && last_opcode != Some(OP_LEAF) {
        return Err((FormatError::RoutingProgramUnterminated).into());
    }
    if op_count > head.max_program_steps() {
        return Err((FormatError::RoutingProgramTooDeep {
            ops: op_count,
            max: head.max_program_steps(),
        })
        .into());
    }
    let table = &bytes[cursor..];

    // Pass 2: operand ranges and jump targets. Pass 1 has already
    // established op sizes and bounds, so re-walking cannot fail.
    let mut cursor: usize = 0;
    for index in 0..op_count {
        let opcode = bytes[cursor];
        match opcode {
            OP_TEST_POPCOUNT_LE => {
                let word = bytes[cursor + 1];
                let threshold = read_u16_le(bytes, cursor + 10);
                if u16::from(word) >= head.signature_words() || threshold > MAX_POPCOUNT {
                    return Err((FormatError::RoutingOperandOutOfBounds { op_index: index }).into());
                }
            }
            OP_JMP_FWD => {
                let delta = read_u16_le(bytes, cursor + 1);
                let target = u64::from(index) + 1 + u64::from(delta);
                if target >= u64::from(op_count) {
                    return Err((FormatError::RoutingJumpOutOfBounds {
                        op_index: index,
                        target,
                    })
                    .into());
                }
            }
            OP_LEAF => {
                let start = read_u32_le(bytes, cursor + 1);
                let len = read_u16_le(bytes, cursor + 5);
                if table.is_empty() {
                    if len != 0 {
                        return Err(
                            (FormatError::RoutingShortlistOutOfBounds { op_index: index }).into(),
                        );
                    }
                } else {
                    let end = u64::from(start) + u64::from(len);
                    if end > table.len() as u64 {
                        return Err(
                            (FormatError::RoutingShortlistOutOfBounds { op_index: index }).into(),
                        );
                    }
                }
            }
            OP_HALT => {}
            // Pass 1 rejected every unknown opcode, but stay explicit
            // rather than unreachable.
            _ => {
                return Err((FormatError::UnknownRoutingOp {
                    offset: cursor as u32,
                    opcode,
                })
                .into())
            }
        }
        // Pass 1 established the size, so this never fails.
        if let Some(size) = op_size(opcode) {
            cursor += size;
        }
    }
    Ok(())
}
