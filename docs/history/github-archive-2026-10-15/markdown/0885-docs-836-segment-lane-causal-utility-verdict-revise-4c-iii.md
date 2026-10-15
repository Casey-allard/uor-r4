# #885 — docs(#836): segment-lane causal-utility verdict — REVISE (4c-iii)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T01:31:46Z
- **Updated:** 2026-08-21T01:56:23Z
- **Closed:** 2026-08-21T01:53:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/885
- **Merged:** 2026-08-21T01:53:54Z
- **Merge commit:** 58f9df11b95767992c4a1d6b2dbb334a5a2cd1e8
- **Base / Head:** main / UOR-Foundation:issue-836-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #836 4c-iii — segment-lane causal-utility verdict: **REVISE** (dormant)

Records the causal-utility decision for the now fully-lowered segment lane, and
closes #836. Docs-only; no code, no behavior change.

### Verdict

**REVISE**, evidence-backed, no quality claim promoted. The lane is lowered
end-to-end and **safe by construction** (optional `PSTATE`, absent-section
identity on every existing artifact, top-8-only re-rank that never fabricates a
token or turns a serve into an abstention). But the whole-prompt-content effect
is **real yet sub-floor**, so it stays **dormant**.

### Why no run was launched (reachability arithmetic)

- Max whole-prompt-content signal measured anywhere — the #834 §6.2 learned
  content→argmax reference arm — is **+17.5‰, 95% CI [15.9, 19.0]**.
- Pre-registered causal floor: **20.0‰** (`CAUSAL_FLOOR_PERMILLE`).
- Optimistic ceiling **19.0‰ < 20.0‰**, and the deployed lane is strictly weaker
  than the ceiling (integer-`ScoreQ` quantization; top-8 candidate bound). The
  inequality is unconditional → per the long-run discipline, **no** hours-long
  causal run is launched for a foregone-conclusion negative.

### Mechanism gap is closed

The original 4c contract flagged that the deployed lane lowered only the
recurrence proxy while the +17.5‰ came from a *learned* table. 4c-i (#883, engine
consumes the table) and 4c-ii (fit + emit the table) close that gap — the lane
the repo ships **is** the §6.2 mechanism. The verdict rests purely on the ceiling
arithmetic, which the closed gap does not move.

### Ledger

Dormant: no `model/ids.toml` serving row, `CONFORMANCE.md` unchanged, no public
quality claim. Activation gate unchanged: *re-clears the 20‰ causal floor on the
packed path.* Open revision paths (candidate-support widening → bounded
minimal-pairs spot-check → 20‰ floor governance review) are filed as new issues
in implementation order.

Adds `docs/segment_lane_836_verdict.md` (contract + verdict) and
`docs/segment_lane_836_result.json` (machine-readable decision record).

Gate: `cargo run -p xtask -- validate` green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01JNGVP646hV5uuCZGMKfxTk


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

