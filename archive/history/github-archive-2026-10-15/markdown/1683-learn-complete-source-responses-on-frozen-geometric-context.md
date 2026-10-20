# #1683 — Learn complete source responses on frozen geometric context

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T09:20:49Z
- **Updated:** 2026-10-09T13:17:30Z
- **Closed:** 2026-10-04T09:23:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1683
- **Merged:** 2026-10-04T09:23:46Z
- **Merge commit:** fa4c3b4f67b6098c764d4b7b9c943f72b7b59169
- **Base / Head:** main / UOR-Foundation:codex/geometric-readout-frozen-context-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Later-query geometry changed native attention scores but could not complete the eight multiword construction responses. This adds a Rust continuation that freezes that geometry and trains only the existing Copy, Stop and Period readouts using ordinary full28 alias-aware token/EOS loss on every update. Native checkpoint selection includes the unchanged parent under the same equal-episode criterion.

The completed64-update CPU run selects0064: all20 original and all8 exposed construction replies are correct with EOS; native full28 CE falls0.469654437→0.082247332. Context source bits and geometric native bins remain exact. All checkpoints and negative/intermediate loss tradeoffs are retained. This is development source consumption, not held-out attention, general chat or full-path serving qualification. The next task is prospectively frozen zero-update transfer.

Validation:39 focused Rust tests and release build pass on executed source6b9f3904; actual LinuxCPU run997.355s; independently checked11,392training,890canonical and835generated positions, admissions/freezing/native arithmetic/selection; exact193-file return. Earlier test-fixture compile failure is retained separately. No GPU/runners/laptop modelCPU or Metal test claims.

References #1552 and #1512.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

