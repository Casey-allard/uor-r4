use uor_r4_graph_format::{GraphView, SectionId};
use uor_r4_graph_format::{OP_HALT, OP_JMP_FWD, OP_LEAF, OP_TEST_POPCOUNT_LE};

/// Evaluate the ROUT bytecode program, returning the resolved shortlist slice.
///
/// Total verdict: `Some(slice)` on a well-formed traversal (the empty slice
/// when there is no ROUT section), `None` when the program is malformed —
/// an absent head, an out-of-bounds field, or an unknown opcode.
pub fn evaluate_route<'a>(view: &GraphView<'a>, signature: &[u64]) -> Option<&'a [u8]> {
    let head = view.head()?;
    let rout_bytes = view.section(SectionId::ROUT).unwrap_or(&[]);
    if rout_bytes.is_empty() {
        return Some(&[]);
    }

    // Scan for HALT to find the shortlist table start
    let mut table_start = rout_bytes.len();
    let mut scan_pc = 0;
    while scan_pc < rout_bytes.len() {
        let op = rout_bytes[scan_pc];
        match op {
            OP_HALT => {
                table_start = scan_pc + 1;
                break;
            }
            OP_TEST_POPCOUNT_LE => scan_pc += 12,
            OP_JMP_FWD => scan_pc += 3,
            OP_LEAF => scan_pc += 7,
            // Unknown opcode at `scan_pc`: the ROUT program is malformed.
            _ => return None,
        }
    }

    let table = if table_start < rout_bytes.len() {
        &rout_bytes[table_start..]
    } else {
        &[]
    };

    let mut pc = 0;
    let mut step_count = 0;
    let max_steps = head.max_program_steps();

    while pc < rout_bytes.len() && step_count < max_steps {
        let opcode = rout_bytes[pc];
        step_count += 1;

        match opcode {
            OP_HALT => {
                return Some(table); // Return the entire table as fallback
            }
            OP_TEST_POPCOUNT_LE => {
                if pc + 12 > rout_bytes.len() {
                    return None;
                }
                let word = rout_bytes[pc + 1] as usize;
                let mask = u64::from_le_bytes(rout_bytes[pc + 2..pc + 10].try_into().ok()?);
                let threshold = u16::from_le_bytes(rout_bytes[pc + 10..pc + 12].try_into().ok()?);

                let popcount = if word < signature.len() {
                    (signature[word] & mask).count_ones() as u16
                } else {
                    0
                };

                if popcount <= threshold {
                    pc += 12; // Condition met, proceed to next op
                } else {
                    // Condition not met, skip next op
                    pc += 12;
                    if pc < rout_bytes.len() {
                        let next_op = rout_bytes[pc];
                        let skip_size = match next_op {
                            OP_HALT => 1,
                            OP_TEST_POPCOUNT_LE => 12,
                            OP_JMP_FWD => 3,
                            OP_LEAF => 7,
                            _ => return None,
                        };
                        pc += skip_size;
                    }
                }
            }
            OP_JMP_FWD => {
                if pc + 3 > rout_bytes.len() {
                    return None;
                }
                let delta_ops = u16::from_le_bytes(rout_bytes[pc + 1..pc + 3].try_into().ok()?);
                pc += 3;
                for _ in 0..delta_ops {
                    if pc >= rout_bytes.len() {
                        return None;
                    }
                    let skip_op = rout_bytes[pc];
                    let skip_size = match skip_op {
                        OP_HALT => 1,
                        OP_TEST_POPCOUNT_LE => 12,
                        OP_JMP_FWD => 3,
                        OP_LEAF => 7,
                        _ => return None,
                    };
                    pc += skip_size;
                }
            }
            OP_LEAF => {
                if pc + 7 > rout_bytes.len() {
                    return None;
                }
                let shortlist_start =
                    u32::from_le_bytes(rout_bytes[pc + 1..pc + 5].try_into().ok()?) as usize;
                let shortlist_len =
                    u16::from_le_bytes(rout_bytes[pc + 5..pc + 7].try_into().ok()?) as usize;
                if shortlist_start + shortlist_len > table.len() {
                    return None;
                }
                return Some(&table[shortlist_start..shortlist_start + shortlist_len]);
            }
            _ => return None,
        }
    }

    Some(table)
}
