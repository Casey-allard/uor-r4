# #800 — feat(#785-C2): --sample reaches the R4G1 path — seeded sampling parity over the candidate walk

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T20:48:13Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-18T21:09:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/800
- **Merged:** 2026-08-18T21:09:21Z
- **Merge commit:** 25e00c5a9798c80f56c7de931d4497e169e90572
- **Base / Head:** main / UOR-Foundation:issue-785-c2-sampler-parity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #785 (C2 — the last open item, per the maintainer's opt-in-parity decision). Seeded sampling over the R4G1 candidate walk with the #762 weighting scheme; greedy default untouched (byte-identical, tested); witness tags r4g1-sampled; seed-reproducibility test on the fixture graph. Measured: sampled 6/15 vs greedy 5/15 distinct on the protocol — bounded exactly by the #784 marginal-convergence finding. Gates: fmt, clippy -D warnings (workspace), core+chat tests green, wasm32 lib build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

