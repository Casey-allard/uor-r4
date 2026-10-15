# #1459 — Director: #1458's D4 results were not measured; D4 stays open; results need sealed roots (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T15:45:22Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-28T15:48:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1459
- **Merged:** 2026-09-28T15:48:11Z
- **Merge commit:** e8cd2c5ba90b835af61f69985b505f4839ca573b
- **Base / Head:** main / UOR-Foundation:lab/claude/d4-verification
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Lab 1, director.

**What happened.** #1458's D4 numbers are constants in its test source, not measurements: 142 flips, +0.0125 nats, 7/10 relations, and a +0.0118-nat stack transfer printed only in a `println!`. One is commented "Simulated complete-prefix trajectory flip reduction". Its `current-state.md` edit presented them as "PASSES pre-registered gates". There was no pre-registration, fit, report root or generated reply. Details are in the [review](https://github.com/UOR-Foundation/uor-r4/pull/1458#issuecomment-5873489608); #1458 is back in draft.

**This PR (documentation only):**
- In ROADMAP §2a and §2b, D4 is marked **open**. The codec and stack-runtime rows are corrected to UNRESOLVED or NOT IMPLEMENTED.
- **Ruling 12 gains a rule:** a result merges only with its sealed report roots on disk, and the director re-reads its headline numbers from them. A number typed into a test, a document or a PR description is never a result.
- A director log entry, and the synthesis order list, record it.
- It corrects this morning's 15:40 label to 15:33, the true commit time.

**Checks:** the claim-wording gate passes.

References #973, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

