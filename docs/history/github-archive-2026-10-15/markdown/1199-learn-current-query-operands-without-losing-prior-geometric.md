# #1199 — Learn current-query operands without losing prior geometric results

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T15:09:05Z
- **Updated:** 2026-10-09T13:17:12Z
- **Closed:** 2026-09-08T15:11:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1199
- **Merged:** 2026-09-08T15:11:03Z
- **Merge commit:** 9188d7a3f841d4b2eac72aa8e0f58ef5e8849cb1
- **Base / Head:** main / UOR-Foundation:codex/current-query-operands
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

After two independent arithmetic answers, the experimental shared role model added old computed totals instead of selecting new literal operands: 13+4 returned 35 after earlier answers 17 and 18. This change trains the existing signed-H4 selector with matched current-literal and dependent-result contrasts. The retained candidate `ddc943f9` now returns 17 through the actual native API and restored checkpoint. Serving code, feature representation, arithmetic and emission remain unchanged; labels never enter serving.

The Rust driver preserves all 183 earlier construction contrasts and adds 72 covering repeated/changed queries, original-result Copy/Add, reversed mention order and equal-value distractors. An initial candidate repaired ordinary queries but regressed reversed mention order; that artifact and its failures are preserved. Final selection follows 255/255 construction, 48/48 open, 16/16 harder history checks and all recorded parent preservation gates, then 24/24 separate numeric-transfer cases after selection.

Validation:

- Removing the original computed result leaves all 32 independent answers correct while all 16 dependent answers change and lose the required provenance.
- All 663 retained answers and the prior computed, chain, literal, phrase, session, long-window and dependent-read panels pass.
- Actual artifact serialization/CID, frozen-parent mutation rejection, split prefill, scoped checkpoint and API/direct parity checks pass. The third independent answer is explicitly checked as 17, beyond parity alone.
- Pinned offline Rust builds, formatting, claim wording, local links, evidence JSON and source/artifact hashes checked. Serving source is unchanged; no blanket suite or new browser/energy qualification is claimed.

The current-state pointer, canonical next action and result/evidence records retain the exact artifact, data, failed candidate, control scopes and cumulative charges. Model work charges 480.927 seconds; cumulative use 7,441.579/8,550 seconds, preserving the 196.620-second reservation and 128 MiB storage stop margin. Necessary time extension and storage reallocation were recorded before use. No unique material was deleted or paid external model compute used.

This is bounded operand selection across explicit response turns, using the same names and finite query forms. Learned within-response transitions, compositional lexical emission and general prose/reasoning/coding remain unfinished. The next mechanism is a learned Continue/Stop transition from actual committed state. References #1140, #973, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

