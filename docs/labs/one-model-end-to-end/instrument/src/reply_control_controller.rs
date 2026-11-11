//! Active product rule picked by the V3 bake-off: keep every served token,
//! stop on served EOS or the existing terminal short-cycle rule.

use crate::{apply_flow, Decision, TraceRow};

pub fn control_reply(rows: &[TraceRow]) -> Vec<u32> {
    apply_flow(rows, true, |_, _| Decision::Keep)
}
