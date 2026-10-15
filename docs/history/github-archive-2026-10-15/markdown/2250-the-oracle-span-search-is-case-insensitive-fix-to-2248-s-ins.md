# #2250 — The oracle span search is case-insensitive (fix to #2248's instrument) (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T12:16:44Z
- **Updated:** 2026-10-12T21:43:57Z
- **Closed:** 2026-10-12T21:36:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2250
- **Merged:** 2026-10-12T21:36:48Z
- **Merge commit:** 0fc2b35389bbca2e78350cdad0ae19776feddfed
- **Base / Head:** main / UOR-Foundation:kimi/span-oracle-fix-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline: unchanged (M1 v5 memory 20/40, reply 37/232)

**What this fixes.** #2248's span search matched the frozen checks' terms case-sensitively, but `parse_checks` lowercases all terms (`words()`, chat-grade.rs:619) while the conversations state the values capitalised — so the search found no span on the real panel (13/64 rows), which an end-to-end smoke after merging exposed. My unit test had bypassed `parse_checks` and never saw the lowercasing; the coverage check should have run before the instrument merged, and now it has.

**The fix.** `find_value_span` matches ASCII-case-insensitively on the decoded text (byte offsets stay exact), with a new test assertion for the lowercase-needle/capitalised-value case.

**Verification at exact head:** `cargo test -p uor-r4-training --bin chat-grade` 20/20, including the new case; end-to-end on the frozen v5 panel with a smoke artifact: **all 40 memory rows spanned, every answer turn included** (94/110 memory-row turns; the 16 without are exactly the turns before the value is stated — the intended 'not yet in the window' behavior).

**Review:** exact-head self-review, scope the one file.

**Decision test:** without it the pre-registered run's oracle would be a no-op on the panel — the fix changes whether the run measures anything.

References #2029.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

