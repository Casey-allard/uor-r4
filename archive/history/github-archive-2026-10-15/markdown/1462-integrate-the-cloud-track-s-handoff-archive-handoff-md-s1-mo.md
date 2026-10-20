# #1462 — Integrate the cloud track's handoff (archive + HANDOFF.md); S1 moves to Lab 1 locally

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T16:28:45Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-28T16:30:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1462
- **Merged:** 2026-09-28T16:30:09Z
- **Merge commit:** df8411b17bb032a4ef4fafa0a9465bfc049df8c8
- **Base / Head:** main / UOR-Foundation:lab/claude/cloud-handoff-integration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Lab 1, director. **Owner direction:** move the cloud session's work into the main repository and continue it locally, without losing progress.

**What merges:**
- `handoff/cloud-20260928/HANDOFF.md`: the cloud track's full handoff. It lists work items and their state, the S1 plan, and every sandbox-only item with its size.
- `handoff/cloud-20260928/sandbox/`: 290 text files (reports, briefs, drafts, experiment code, run scripts). They were scanned for credentials and none were found. The largest is 135 KB.
- **Excluded:** the 131 vendored third-party Python package files (`autograd`, `mpmath`) under `sandbox/exp/math/pylib/`. They remain on `claude/cloud-handoff-20260928`.
- **ROADMAP:** the S1 rows now read "Lab 1 locally, starting", because the cloud never started it. A log entry is added.

**Already done outside git:**
- D1's four models are on the SSD at `uor-r4-models/investigations/d1-transport-attribution-20260928/`, 24/24 SHA-256 verified, with a receipt.
- `transfer/d1-models-20260928` is deleted.

**Pending:** the owner approved one more transfer of the unrebuildable data (about 148 MB): `registry.u16`, `registry.txt`, and the cycle-3 code and wiki splits. It waits on the cloud session.

**Checks:** the claim-wording gate passes. This is documentation and archive only; the workspace's explicit `members` list keeps the archive's `Cargo.toml` copies out of builds.

References #973, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

