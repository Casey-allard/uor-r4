# #1477 — Salvage review nits: track the width-576 restore gap (#1476); explain the ungated WASM audit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T22:00:13Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-09-28T22:01:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1477
- **Merged:** 2026-09-28T22:01:24Z
- **Merge commit:** f85969e0ff20d6b501f2366bcf2f678b95bfd65b
- **Base / Head:** main / UOR-Foundation:lab/claude/salvage-nits
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1. Class A.** Follow-up to the non-author review of #1474, which approved it with nits.
- The ignored width-576 restore test now references tracking issue #1476.
- The packager explains why the WASM audit's exit code is recorded but not gated: the WASM helper module is documented as not D11-clean.

`cargo fmt --check` passes, and the Python still parses. The review's wording nit on the `wasm.rs` doc needed no change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

