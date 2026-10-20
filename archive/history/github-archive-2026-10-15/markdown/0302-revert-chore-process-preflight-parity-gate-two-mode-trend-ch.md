# #302 — Revert "chore(process): preflight parity gate, two-mode trend check, build-isolation conventions"

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T06:22:46Z
- **Updated:** 2026-10-09T14:17:27Z
- **Closed:** 2026-07-30T06:36:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/302
- **Merged:** 2026-07-30T06:36:36Z
- **Merge commit:** 30229096ba8482731ca3997052546d99a1efdf5f
- **Base / Head:** main / UOR-Foundation:revert-process-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Reverts d38c740 (#297) pending team review of the process/CI changes it introduced: removes scripts/preflight.sh, restores the single-mode Gate C regression check and the original ci.yml trend step, and restores AGENTS.md. Measurement pins (docs/transformerless/gate_c_pinned.json) are untouched — they are recorded data, and the restored check passes against the current pin (measured row == pin). The reverted content is re-proposed for review as PR #301.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

