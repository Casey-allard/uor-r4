//! Deliberately dumb bake-off control: copy every supplied row.

use crate::TraceRow;

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    rows.iter().map(|row| row.token).collect()
}
