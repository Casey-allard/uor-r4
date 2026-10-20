# #1661 — Diagnose final native geometric Copy ranking with saved-artifact audit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T21:54:47Z
- **Updated:** 2026-10-09T13:17:24Z
- **Closed:** 2026-10-03T21:56:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1661
- **Merged:** 2026-10-03T21:56:57Z
- **Merge commit:** 0e38833eff7ae0a26c5280407c487c52f1fce276
- **Base / Head:** main / UOR-Foundation:codex/geometric-final-prefix-audit-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The trained native source realizer still fails Copy ranking even at correct canonical prefixes. This adds a read-only Rust audit of the saved final checkpoint, retaining exact packed payloads, occurrence features, integer logits/masses and native token ranks. It reuses existing learning and performs zero optimizer updates.

On the exposed 20-episode development panel, 46/114 canonical targets are correct; 54/74 Copy targets lose to other Copy actions. All own-prefix outputs and integer traces exactly reproduce the saved M1 outputs on Linux. The inspected full active-feature signatures show no conflicting target collisions. This narrows the next task to diagnosing the existing q4 readout/credit path; it does not establish family expressivity, held-out language quality or general chat.

Validation: three focused Rust alias/rank/tie tests pass at audit source 67d36915; optimized Rust audit passes on CPU-only Runpod with sealed inputs unchanged. Integrated source ebd0a891 passes the locked training-example Cargo check; numerical consumer/realizer and audit sources are unchanged from the executed audit. Claim-wording, touched-example formatting and diff checks pass. Metal unavailable on Linux and not counted. Execution errors and resource receipts are retained; the Codex job card is released and final owned pod area is about 4.09 GiB.

References #1552. Shared stack_grounded_session.rs is untouched. Source-bound evidence and next proposed diagnosis are in the current state and roadmap.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

